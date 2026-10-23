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

echo "🔐 4. Restarting ${SERVICE_NAME} service..."
ssh "$SERVER" "
    set -e
    chmod 755 '${APP_DIR}/${BINARY_NAME}'
    chown www-data:www-data '${APP_DIR}/${BINARY_NAME}'
    systemctl restart '${SERVICE_NAME}'
    sleep 2

    if ! systemctl is-active --quiet '${SERVICE_NAME}'; then
        echo '❌ ${SERVICE_NAME} failed to start! Recent logs:'
        journalctl -u '${SERVICE_NAME}' -n 30 --no-pager
        exit 1
    fi
    echo '  ✓ ${SERVICE_NAME} service restarted and active.'
"

echo "🧪 5. Verifying local endpoints on server..."
ssh "$SERVER" "
    set -e
    echo '  -> Health: '\$(curl -s http://127.0.0.1:${INTERNAL_PORT}/health/db)

    WS_CODE=\$(curl -s -o /dev/null -w \"%{http_code}\" \
      -H \"Connection: Upgrade\" \
      -H \"Upgrade: websocket\" \
      -H \"Sec-WebSocket-Key: SGVsbG8sIHdvcmxkIQ==\" \
      -H \"Sec-WebSocket-Version: 13\" \
      http://127.0.0.1:${INTERNAL_PORT}/ws)
    echo \"  -> Local WebSocket: HTTP \$WS_CODE (Expected 101)\"
"

echo "🌐 6. Checking public HTTPS endpoint..."
if curl -s --fail --max-time 5 "https://${PUBLIC_DOMAIN}/health/db" > /dev/null 2>&1; then
    echo "  ✓ https://${PUBLIC_DOMAIN} is online and healthy!"
else
    echo "  ⚠️ Public check pending."
fi

echo "=========================================================="
echo "✅ Deployed successfully in seconds!"
echo "=========================================================="
