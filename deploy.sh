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

echo "⚙️ 1. Installing Linux compilation target..."
rustup target add "$TARGET"

echo "📦 2. Building optimized Linux binary (${TARGET})..."
cargo build --release --target "$TARGET" --target-dir "${PROJECT_DIR}/target"

echo "🔍 3. Verifying binary..."
if [[ ! -f "$BINARY_PATH" ]]; then
    # Check alternate binary name with hyphen
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

echo "🚀 5. Uploading binary..."
rsync -avzP "$BINARY_PATH" "${SERVER}:${APP_DIR}/${BINARY_NAME}"

echo "🔧 6. Synchronizing systemd service and Nginx configuration..."
if [[ -f "${PROJECT_DIR}/deployment/quiz.service" ]]; then
    rsync -avz "${PROJECT_DIR}/deployment/quiz.service" "${SERVER}:/etc/systemd/system/${SERVICE_NAME}.service"
fi

if [[ -f "${PROJECT_DIR}/deployment/quiz.conf" ]]; then
    rsync -avz "${PROJECT_DIR}/deployment/quiz.conf" "${SERVER}:/etc/nginx/sites-available/quiz.conf"
    ssh "$SERVER" "
        ln -sf /etc/nginx/sites-available/quiz.conf /etc/nginx/sites-enabled/quiz.conf
    "
fi

echo "🔐 7. Setting permissions and restarting ${SERVICE_NAME}..."
ssh "$SERVER" "
    set -e
    chmod 755 '${APP_DIR}/${BINARY_NAME}'
    chown -R www-data:www-data '$APP_DIR'
    systemctl daemon-reload
    systemctl restart '${SERVICE_NAME}'
    systemctl is-active --quiet '${SERVICE_NAME}'
    nginx -t && systemctl reload nginx
"

echo "🧪 8. Running post-deployment API test calls on server..."
ssh "$SERVER" "
    set -e
    echo '  -> Checking local Database Health (127.0.0.1:${INTERNAL_PORT}/health/db)...'
    DB_RESP=\$(curl -s -S --fail http://127.0.0.1:${INTERNAL_PORT}/health/db)
    echo \"     Response: \$DB_RESP\"

    echo '  -> Checking Embedded SPA Root (127.0.0.1:${INTERNAL_PORT}/)...'
    curl -s -S --fail http://127.0.0.1:${INTERNAL_PORT}/ | grep -q 'ezsnapps'
    echo '     Response: HTML title & assets verified.'

    echo '  -> Checking API Reference Data (127.0.0.1:${INTERNAL_PORT}/api/quizzes)...'
    QUIZ_RESP=\$(curl -s -S --fail http://127.0.0.1:${INTERNAL_PORT}/api/quizzes)
    echo \"     Response: \$QUIZ_RESP\"

    echo '  ✓ Local service API tests passed successfully!'
"

echo "🌐 9. Running public endpoint verification..."
if curl -s -S --fail --max-time 10 "https://${PUBLIC_DOMAIN}/health/db" > /dev/null 2>&1; then
    PUBLIC_RESP=$(curl -s "https://${PUBLIC_DOMAIN}/health/db")
    echo "  ✓ Public HTTPS Health Check Passed (https://${PUBLIC_DOMAIN}/health/db): $PUBLIC_RESP"
else
    echo "  ⚠️ Public HTTPS check failed or pending DNS/SSL. Please verify https://${PUBLIC_DOMAIN} manually."
fi

echo "=========================================================="
echo "✅ Deployment and API Verification Completed Successfully!"
echo "=========================================================="
