#!/bin/bash
set -Eeuo pipefail

# ==============================================================================
# ezsnapps Quiz SaaS - One-Time Server Infrastructure Setup Script
# Run once when provisioning or changing Nginx / Systemd configs:
#   ./deployment/setup-server.sh
# ==============================================================================

SERVER="root@69.62.74.52"
APP_DIR="/var/www/quiz"
SERVICE_NAME="quiz"

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "📁 1. Creating application directory on server..."
ssh -o ConnectTimeout=10 "$SERVER" "bash -s -- '$APP_DIR'" << 'EOF'
set -Eeuo pipefail
APP_DIR="$1"
mkdir -p "$APP_DIR"
chown -R www-data:www-data "$APP_DIR"
EOF

echo "⚙️  2. Creating default production .env on server if missing..."
ssh -o ConnectTimeout=10 "$SERVER" "bash -s -- '$APP_DIR'" << 'EOF'
set -Eeuo pipefail
APP_DIR="$1"
if [ ! -f "${APP_DIR}/.env" ]; then
    cat << 'ENVFILE' > "${APP_DIR}/.env"
HOST=127.0.0.1
PORT=8081
DATABASE_URL=postgres://username:password@localhost:5432/esan_quiz
RUST_LOG=info
CLEANUP_INTERVAL_SECS=60
ROOM_ABANDONED_TIMEOUT_SECS=900
MAX_ROOM_LIFETIME_SECS=7200
ENVFILE
    chown www-data:www-data "${APP_DIR}/.env"
    chmod 644 "${APP_DIR}/.env"
    echo "  -> Created ${APP_DIR}/.env"
else
    echo "  -> ${APP_DIR}/.env already exists, keeping it."
fi
EOF

echo "🔧 3. Installing systemd service..."
if [[ -f "${PROJECT_DIR}/deployment/quiz.service" ]]; then
    rsync -avz "${PROJECT_DIR}/deployment/quiz.service" "${SERVER}:/etc/systemd/system/${SERVICE_NAME}.service"
    ssh -o ConnectTimeout=10 "$SERVER" "bash -s -- '$SERVICE_NAME'" << 'EOF'
set -Eeuo pipefail
SERVICE_NAME="$1"
systemctl daemon-reload
systemctl enable "$SERVICE_NAME"
echo "  -> Systemd unit registered and enabled."
EOF
fi

echo "🌐 4. Configuring Nginx for WebSockets and HTTPS..."
if [[ -f "${PROJECT_DIR}/deployment/quiz.conf" ]]; then
    rsync -avz "${PROJECT_DIR}/deployment/quiz.conf" "${SERVER}:/etc/nginx/sites-available/quiz.conf"
    ssh -o ConnectTimeout=10 "$SERVER" 'bash -s' << 'EOF'
set -Eeuo pipefail
mkdir -p /etc/nginx/sites-available /etc/nginx/sites-enabled

# Remove duplicate/old quiz configs to avoid conflicting server_name warnings
for old in /etc/nginx/sites-enabled/*quiz* /etc/nginx/conf.d/*quiz*; do
    if [ -e "$old" ] && [ "$old" != "/etc/nginx/sites-enabled/quiz.conf" ]; then
        echo "  -> Removing old config: $old"
        rm -f "$old"
    fi
done

ln -sf /etc/nginx/sites-available/quiz.conf /etc/nginx/sites-enabled/quiz.conf
nginx -t
systemctl restart nginx
echo "  -> Nginx reloaded successfully."
EOF
fi

echo "=========================================================="
echo "✅ One-time server infrastructure setup complete!"
echo "=========================================================="
