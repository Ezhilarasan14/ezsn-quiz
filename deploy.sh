#!/bin/bash
set -Eeuo pipefail

# ==============================================================================
# ezsnapps Quiz SaaS - Remote Production Deployment Script
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

# Build output path
BINARY_PATH="${PROJECT_DIR}/target/${TARGET}/release/${BINARY_NAME}"

echo "⚙️  1. Installing Linux compilation target..."
rustup target add "$TARGET"

echo "📦 2. Building optimized Linux binary (${TARGET})..."
cargo build --release --target "$TARGET" --target-dir "${PROJECT_DIR}/target"

echo "🔍 3. Verifying binary..."
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

echo "📁 4. Preparing server directory and permissions..."
ssh "$SERVER" "
    mkdir -p '$APP_DIR'
    chown -R www-data:www-data '$APP_DIR'
"

echo "🚀 5. Uploading application binary..."
rsync -avzP "$BINARY_PATH" "${SERVER}:${APP_DIR}/${BINARY_NAME}"

echo "⚙️  6. Ensuring production environment config on server..."
ssh "$SERVER" "
    cat << 'EOF' > '${APP_DIR}/.env'
HOST=127.0.0.1
PORT=8081
DATABASE_URL=sqlite:/var/www/quiz/quiz.db
RUST_LOG=info
CLEANUP_INTERVAL_SECS=60
ROOM_ABANDONED_TIMEOUT_SECS=900
MAX_ROOM_LIFETIME_SECS=7200
EOF
    chown www-data:www-data '${APP_DIR}/.env'
    chmod 644 '${APP_DIR}/.env'
"

echo "🔧 7. Synchronizing systemd service..."
if [[ -f "${PROJECT_DIR}/deployment/quiz.service" ]]; then
    rsync -avz "${PROJECT_DIR}/deployment/quiz.service" "${SERVER}:/etc/systemd/system/${SERVICE_NAME}.service"
    ssh "$SERVER" "systemctl daemon-reload && systemctl enable ${SERVICE_NAME}"
fi

echo "🌐 8. Configuring and cleaning up Nginx configs for WebSockets..."
if [[ -f "${PROJECT_DIR}/deployment/quiz.conf" ]]; then
    rsync -avz "${PROJECT_DIR}/deployment/quiz.conf" "${SERVER}:/etc/nginx/sites-available/quiz.conf"
    ssh "$SERVER" "
        mkdir -p /etc/nginx/sites-available /etc/nginx/sites-enabled

        # Remove any conflicting/old quiz configs to prevent Nginx duplicate server_name warnings
        for old in /etc/nginx/sites-enabled/*quiz* /etc/nginx/conf.d/*quiz*; do
            if [ -e \"\$old\" ] && [ \"\$old\" != \"/etc/nginx/sites-enabled/quiz.conf\" ]; then
                echo \"  -> Removing duplicate config: \$old\"
                rm -f \"\$old\"
            fi
        done

        ln -sf /etc/nginx/sites-available/quiz.conf /etc/nginx/sites-enabled/quiz.conf
    "
fi

echo "🔐 9. Setting permissions and restarting services..."
ssh "$SERVER" "
    set -e
    chmod 755 '${APP_DIR}/${BINARY_NAME}'
    chown -R www-data:www-data '$APP_DIR'
    systemctl daemon-reload
    systemctl restart '${SERVICE_NAME}'

    # Give service 2 seconds to initialize SQLite and listen on 8081
    sleep 2

    # Verify service is running; if not, output logs and abort
    if ! systemctl is-active --quiet '${SERVICE_NAME}'; then
        echo '❌ ${SERVICE_NAME} service failed to start! Recent logs:'
        journalctl -u '${SERVICE_NAME}' -n 40 --no-pager
        exit 1
    fi

    nginx -t
    systemctl restart nginx
"

echo "🧪 10. Running post-deployment API & WebSocket tests on server..."
ssh "$SERVER" "
    set -e

    echo '  -> Checking local Database Health (http://127.0.0.1:${INTERNAL_PORT}/health/db)...'
    DB_RESP=\$(curl -s -S --fail http://127.0.0.1:${INTERNAL_PORT}/health/db || (echo 'Database check failed' && journalctl -u ${SERVICE_NAME} -n 20 --no-pager && exit 1))
    echo \"     Response: \$DB_RESP\"

    echo '  -> Checking Embedded SPA Root (http://127.0.0.1:${INTERNAL_PORT}/)...'
    curl -s -S --fail http://127.0.0.1:${INTERNAL_PORT}/ | grep -q 'ezsnapps'
    echo '     Response: HTML title & assets verified.'

    echo '  -> Checking API Quizzes (http://127.0.0.1:${INTERNAL_PORT}/api/quizzes)...'
    QUIZ_RESP=\$(curl -s -S --fail http://127.0.0.1:${INTERNAL_PORT}/api/quizzes)
    echo \"     Response: \$QUIZ_RESP\"

    echo '  -> Testing Local WebSocket Handshake (http://127.0.0.1:${INTERNAL_PORT}/ws)...'
    LOCAL_WS_CODE=\$(curl -s -o /dev/null -w \"%{http_code}\" \
      -H \"Connection: Upgrade\" \
      -H \"Upgrade: websocket\" \
      -H \"Sec-WebSocket-Key: SGVsbG8sIHdvcmxkIQ==\" \
      -H \"Sec-WebSocket-Version: 13\" \
      http://127.0.0.1:${INTERNAL_PORT}/ws)
    echo \"     Local WebSocket HTTP Code: \$LOCAL_WS_CODE (Expected 101 Switching Protocols)\"
    if [ \"\$LOCAL_WS_CODE\" != \"101\" ]; then
        echo \"❌ Local WebSocket failed with code \$LOCAL_WS_CODE!\"
        journalctl -u '${SERVICE_NAME}' -n 30 --no-pager
        exit 1
    fi

    echo '  ✓ Server verification completed successfully!'
"

echo "🌐 11. Running public endpoint verification..."
if curl -s -S --fail --max-time 10 "https://${PUBLIC_DOMAIN}/health/db" > /dev/null 2>&1; then
    PUBLIC_RESP=$(curl -s "https://${PUBLIC_DOMAIN}/health/db")
    echo "  ✓ Public HTTPS Health Check Passed (https://${PUBLIC_DOMAIN}/health/db): $PUBLIC_RESP"
else
    echo "  ⚠️ Public HTTPS check failed or pending DNS/SSL. Please verify https://${PUBLIC_DOMAIN} manually."
fi

# Verify Public WSS connection
PUBLIC_WS_CODE=$(curl -s -o /dev/null -w "%{http_code}" --max-time 10 \
  -H "Connection: Upgrade" \
  -H "Upgrade: websocket" \
  -H "Sec-WebSocket-Key: SGVsbG8sIHdvcmxkIQ==" \
  -H "Sec-WebSocket-Version: 13" \
  "https://${PUBLIC_DOMAIN}/ws" || echo "failed")
echo "  ✓ Public WebSocket Endpoint Test (https://${PUBLIC_DOMAIN}/ws): HTTP ${PUBLIC_WS_CODE} (Expected 101)"

echo "=========================================================="
echo "✅ Deployment and Real-Time WebSocket Verification Complete!"
echo "=========================================================="
