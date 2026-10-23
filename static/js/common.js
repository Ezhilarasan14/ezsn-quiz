// Common Utilities for EZSN-Quiz

function showToast(message, isError = false) {
  let container = document.getElementById("toast-container");
  if (!container) {
    container = document.createElement("div");
    container.id = "toast-container";
    document.body.appendChild(container);
  }

  const toast = document.createElement("div");
  toast.className = `toast ${isError ? "toast-error" : ""}`;
  toast.textContent = message;
  container.appendChild(toast);

  setTimeout(() => {
    toast.style.opacity = "0";
    toast.style.transform = "translateY(20px)";
    toast.style.transition = "all 0.3s ease";
    setTimeout(() => toast.remove(), 300);
  }, 3500);
}

async function apiRequest(url, options = {}) {
  try {
    const res = await fetch(url, {
      headers: {
        "Content-Type": "application/json",
        ...options.headers,
      },
      ...options,
    });

    const data = await res.json().catch(() => null);

    if (!res.ok) {
      const errMsg = (data && data.error) || (data && data.message) || `Request failed with status ${res.status}`;
      throw new Error(errMsg);
    }

    return data;
  } catch (err) {
    console.error(`API Error on ${url}:`, err);
    throw err;
  }
}

// Global WebSocket Connection Manager
class QuizWebSocket {
  constructor() {
    this.ws = null;
    this.handlers = new Map();
    this.reconnectAttempts = 0;
    this.isConnected = false;
    this.connectParams = null;
  }

  connect(onOpenCallback) {
    const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
    const wsUrl = `${protocol}//${window.location.host}/ws`;

    this.ws = new WebSocket(wsUrl);

    this.ws.onopen = () => {
      this.isConnected = true;
      this.reconnectAttempts = 0;
      console.log("WebSocket connected to server.");
      if (onOpenCallback) onOpenCallback();
    };

    this.ws.onmessage = (event) => {
      try {
        const msg = JSON.parse(event.data);
        const type = msg.type;
        const payload = msg.payload;

        if (this.handlers.has(type)) {
          this.handlers.get(type).forEach((fn) => fn(payload));
        }
      } catch (err) {
        console.error("Failed to parse incoming WebSocket message:", err);
      }
    };

    this.ws.onclose = (event) => {
      this.isConnected = false;
      console.warn("WebSocket closed. Attempting reconnect...", event);
      if (this.reconnectAttempts < 5) {
        this.reconnectAttempts++;
        setTimeout(() => this.connect(onOpenCallback), 2000);
      } else {
        showToast("Connection to server lost. Please refresh the page.", true);
      }
    };

    this.ws.onerror = (err) => {
      console.error("WebSocket error:", err);
    };
  }

  on(type, callback) {
    if (!this.handlers.has(type)) {
      this.handlers.set(type, []);
    }
    this.handlers.get(type).push(callback);
  }

  send(type, payload = {}) {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify({ type, payload: payload || {} }));
    } else {
      console.warn("Cannot send message: WebSocket is not open.");
    }
  }
}

window.api = {
  get: (url) => apiRequest(url, { method: "GET" }),
  post: (url, body) => apiRequest(url, { method: "POST", body: JSON.stringify(body) }),
  put: (url, body) => apiRequest(url, { method: "PUT", body: JSON.stringify(body) }),
  delete: (url) => apiRequest(url, { method: "DELETE" }),
};
