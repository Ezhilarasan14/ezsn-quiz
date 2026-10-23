#!/bin/bash
set -Eeuo pipefail

# ==============================================================================
# ezsnapps Quiz SaaS - Fast Application Deployment Script
#
# Usage:
#   ./deploy.sh          -> Fast binary deploy & service restart
#   ./deploy.sh --setup  -> Run one-time Nginx & systemd setup
# ==============================================================================

SERVER="root@69.62.74.52"
APP_DIR="/var/www/quiz"
PUBLIC_DOMAIN="quiz.ezsnapps.in"
INTERNAL_PORT="8081"

BINARY_NAME="ezsn_quiz"
TARGET="x86_64-unknown-linux-musl"
SERVICE_NAME="quiz"

# Resolve the project directory
PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$PROJECT_DIR"

# Run one-time server setup if requested
if [[ "${1:-}" == "--setup" ]]; then
    echo "⚙️ Running one-time server infrastructure setup..."
    "${PROJECT_DIR}/deployment/setup-server.sh"
    exit 0
fi

BINARY_PATH="${PROJECT_DIR}/target/${TARGET}/release/${BINARY_NAME}"

echo "📦 1. Building optimized Linux binary (${TARGET})..."
cargo build --release --target "$TARGET" --target-dir "${PROJECT_DIR}/target"

echo "🔍 2. Verifying binary..."
if [[ ! -f "$BINARY_PATH" ]]; then
    ALT_PATH="${PROJECT_DIR}/target/${TARGET}/release/ezsn-quiz"
    if [[ -f "$ALT_PATH" ]]; then
        BINARY_PATH="$ALT_PATH"
    else
        echo "❌ Binary not found: $BINARY_PATH"
        exit 1
    fi
fi
ls -lh "$BINARY_PATH"

echo "🚀 3. Uploading application binary..."
rsync -avzP "$BINARY_PATH" "${SERVER}:${APP_DIR}/${BINARY_NAME}"

echo "🔐 4. Restarting ${SERVICE_NAME} and verifying endpoints..."
ssh -o ConnectTimeout=10 "$SERVER" "bash -s -- '$SERVICE_NAME' '$APP_DIR' '$BINARY_NAME' '$INTERNAL_PORT'" << 'EOF'
set -Eeuo pipefail

SERVICE_NAME="$1"
APP_DIR="$2"
BINARY_NAME="$3"
INTERNAL_PORT="$4"

echo "  -> Stopping old service and killing any rogue instances..."
systemctl stop "$SERVICE_NAME" 2>/dev/null || true
systemctl kill -s SIGKILL "$SERVICE_NAME" 2>/dev/null || true
pkill -9 -x "$BINARY_NAME" 2>/dev/null || true
systemctl reset-failed "$SERVICE_NAME" 2>/dev/null || true

# If quiz.db is empty (0 bytes) or has orphaned lock files, clean it up
if [ -f "${APP_DIR}/quiz.db" ] && [ ! -s "${APP_DIR}/quiz.db" ]; then
    echo "  -> Cleaning up 0-byte database and stale lock files..."
    rm -f "${APP_DIR}/quiz.db"*
fi

echo "  -> Setting permissions on ${APP_DIR}..."
chmod 755 "${APP_DIR}/${BINARY_NAME}"
chown -R www-data:www-data "$APP_DIR"

echo "  -> Starting ${SERVICE_NAME}..."
if ! systemctl start "$SERVICE_NAME"; then
    echo "❌ systemctl start failed! Service status and logs:"
    systemctl status "$SERVICE_NAME" --no-pager || true
    journalctl -u "$SERVICE_NAME" -n 35 --no-pager || true
    exit 1
fi

echo "  -> Waiting for ${SERVICE_NAME} to initialize..."
HEALTH=""
for i in 1 2 3 4 5; do
    sleep 1
    HEALTH=$(curl -s --connect-timeout 1 --max-time 2 "http://127.0.0.1:${INTERNAL_PORT}/health/db" 2>/dev/null || true)
    if [ -n "$HEALTH" ]; then
        break
    fi
done

if [ -z "$HEALTH" ]; then
    echo "❌ ${SERVICE_NAME} failed to respond on port ${INTERNAL_PORT}! Recent logs:"
    journalctl -u "$SERVICE_NAME" -n 40 --no-pager
    exit 1
fi

echo "  -> Health: $HEALTH"

WS_STATUS=$( (curl -s -i --connect-timeout 2 --max-time 3 \
  -H "Connection: Upgrade" \
  -H "Upgrade: websocket" \
  -H "Sec-WebSocket-Key: SGVsbG8sIHdvcmxkIQ==" \
  -H "Sec-WebSocket-Version: 13" \
  "http://127.0.0.1:${INTERNAL_PORT}/ws" 2>/dev/null || true) | head -n 1 | tr -d '\r' )
WS_CODE=$(echo "$WS_STATUS" | awk '{print $2}')
echo "  -> Local WebSocket: HTTP ${WS_CODE:-N/A} (Expected 101)"
EOF

echo "🌐 5. Checking public HTTPS endpoint..."
if curl -s --fail --connect-timeout 3 --max-time 5 "https://${PUBLIC_DOMAIN}/health/db" > /dev/null 2>&1; then
    echo "  ✓ https://${PUBLIC_DOMAIN} is online and healthy!"
else
    echo "  ⚠️ Public check pending."
fi

echo "=========================================================="
echo "✅ Deployed successfully in seconds!"
echo "=========================================================="
