import type { ClientMessage, ServerMessage } from '@/types/ws';

export type MessageHandler = (msg: ServerMessage) => void;
export type StatusHandler = (connected: boolean) => void;

export class WebSocketClient {
  private ws: WebSocket | null = null;
  private url: string;
  private messageHandlers: Set<MessageHandler> = new Set();
  private statusHandlers: Set<StatusHandler> = new Set();
  private pendingQueue: ClientMessage[] = [];
  private reconnectTimer: number | null = null;
  private pingInterval: number | null = null;
  private isExplicitlyClosed = false;
  private reconnectAttempts = 0;
  private lastPingSent = 0;
  private lastPongReceived = Date.now();
  private boundOnlineHandler: () => void;
  private boundVisibilityHandler: () => void;

  constructor(url?: string) {
    if (url) {
      this.url = url;
    } else {
      const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
      this.url = `${protocol}//${window.location.host}/ws`;
    }

    this.boundOnlineHandler = () => {
      console.log('[WS] Network online detected. Triggering immediate reconnect check.');
      this.reconnectImmediately();
    };

    this.boundVisibilityHandler = () => {
      if (document.visibilityState === 'visible' && !this.isExplicitlyClosed) {
        const timeSincePong = Date.now() - this.lastPongReceived;
        if (!this.isConnected() || (this.lastPingSent > 0 && timeSincePong > 30000)) {
          console.log('[WS] Tab returned to foreground and connection is stale/closed. Reconnecting immediately.');
          this.reconnectImmediately();
        }
      }
    };

    if (typeof window !== 'undefined') {
      window.addEventListener('online', this.boundOnlineHandler);
      document.addEventListener('visibilitychange', this.boundVisibilityHandler);
    }
  }

  public connect(): void {
    if (this.ws && (this.ws.readyState === WebSocket.OPEN || this.ws.readyState === WebSocket.CONNECTING)) {
      return;
    }

    this.isExplicitlyClosed = false;

    try {
      this.ws = new WebSocket(this.url);

      this.ws.onopen = () => {
        console.log('[WS] Connection established');
        this.reconnectAttempts = 0;
        this.lastPongReceived = Date.now();
        this.lastPingSent = 0;
        this.notifyStatus(true);
        this.flushPendingQueue();
        this.startHeartbeat();
      };

      this.ws.onmessage = (event: MessageEvent) => {
        this.lastPongReceived = Date.now();
        try {
          const parsed = JSON.parse(event.data) as ServerMessage;
          if (parsed.type === 'pong') {
            // Heartbeat pong acknowledged
            return;
          }
          this.notifyMessage(parsed);
        } catch (e) {
          console.warn('[WS] Failed to parse incoming message:', event.data, e);
        }
      };

      this.ws.onclose = (event: CloseEvent) => {
        console.warn(`[WS] Connection closed (code: ${event.code}, reason: ${event.reason || 'none'})`);
        this.cleanup();
        this.notifyStatus(false);
        if (!this.isExplicitlyClosed) {
          this.scheduleReconnect();
        }
      };

      this.ws.onerror = (err) => {
        console.error('[WS] WebSocket error:', err);
      };
    } catch (e) {
      console.error('[WS] Exception while connecting:', e);
      this.scheduleReconnect();
    }
  }

  public send(msg: ClientMessage): boolean {
    if (this.isConnected()) {
      return this.sendImmediate(msg);
    }

    // Queue message until socket is open
    this.pendingQueue.push(msg);

    // If socket is not connecting, initiate connection
    if (!this.ws || this.ws.readyState === WebSocket.CLOSED) {
      this.connect();
    }

    return true;
  }

  private sendImmediate(msg: ClientMessage): boolean {
    if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
      return false;
    }

    try {
      this.ws.send(JSON.stringify(msg));
      return true;
    } catch (e) {
      console.error('[WS] Send failed:', e);
      return false;
    }
  }

  private flushPendingQueue(): void {
    while (this.pendingQueue.length > 0 && this.isConnected()) {
      const msg = this.pendingQueue.shift();
      if (msg) {
        this.sendImmediate(msg);
      }
    }
  }

  public onMessage(handler: MessageHandler): () => void {
    this.messageHandlers.add(handler);
    return () => this.messageHandlers.delete(handler);
  }

  public onStatus(handler: StatusHandler): () => void {
    this.statusHandlers.add(handler);
    handler(this.isConnected());
    return () => this.statusHandlers.delete(handler);
  }

  public isConnected(): boolean {
    return !!this.ws && this.ws.readyState === WebSocket.OPEN;
  }

  public reconnectImmediately(): void {
    if (this.isExplicitlyClosed) return;

    if (this.reconnectTimer !== null) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }

    if (this.ws) {
      try {
        this.ws.close();
      } catch (e) {
        // ignore
      }
      this.ws = null;
    }

    this.reconnectAttempts = 0;
    this.connect();
  }

  public disconnect(): void {
    this.isExplicitlyClosed = true;
    this.pendingQueue = [];
    this.cleanup();
    if (this.ws) {
      try {
        this.ws.close();
      } catch (e) {
        // ignore
      }
      this.ws = null;
    }
  }

  public destroy(): void {
    this.disconnect();
    if (typeof window !== 'undefined') {
      window.removeEventListener('online', this.boundOnlineHandler);
      document.removeEventListener('visibilitychange', this.boundVisibilityHandler);
    }
  }

  private scheduleReconnect(): void {
    if (this.reconnectTimer !== null || this.isExplicitlyClosed) return;

    this.reconnectAttempts++;
    // Exponential backoff with random jitter, capped at ~8 seconds
    const backoff = Math.min(1000 * Math.pow(1.3, this.reconnectAttempts), 8000);
    const jitter = Math.floor(Math.random() * 800);
    const delay = backoff + jitter;

    console.log(`[WS] Reconnecting in ${(delay / 1000).toFixed(1)}s (attempt ${this.reconnectAttempts})...`);

    this.reconnectTimer = window.setTimeout(() => {
      this.reconnectTimer = null;
      this.connect();
    }, delay);
  }

  private startHeartbeat(): void {
    if (this.pingInterval !== null) clearInterval(this.pingInterval);
    // Send application-level ping every 15 seconds
    this.pingInterval = window.setInterval(() => {
      if (this.isConnected()) {
        const timeSincePong = Date.now() - this.lastPongReceived;
        // If we haven't received any message/pong for > 35 seconds, socket is dead (zombie TCP)
        if (this.lastPingSent > 0 && timeSincePong > 35000) {
          console.warn(`[WS] Missed heartbeats (last pong ${Math.round(timeSincePong / 1000)}s ago). Reconnecting socket.`);
          this.reconnectImmediately();
          return;
        }

        this.sendImmediate({ type: 'ping' });
        this.lastPingSent = Date.now();
      }
    }, 15000);
  }

  private cleanup(): void {
    if (this.pingInterval !== null) {
      clearInterval(this.pingInterval);
      this.pingInterval = null;
    }
    if (this.reconnectTimer !== null) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
  }

  private notifyMessage(msg: ServerMessage): void {
    this.messageHandlers.forEach((h) => h(msg));
  }

  private notifyStatus(connected: boolean): void {
    this.statusHandlers.forEach((h) => h(connected));
  }
}

export const wsClient = new WebSocketClient();
