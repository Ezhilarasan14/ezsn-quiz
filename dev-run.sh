#!/usr/bin/env bash
# ==============================================================================
# EZSN Quiz - Local Development Runner Script
#
# Usage:
#   ./dev-run.sh          -> Build frontend if missing & start backend server
#   ./dev-run.sh --build  -> Force rebuild frontend before starting
# ==============================================================================
set -e

# Change directory to the project root
cd "$(dirname "$0")"

# Load environment variables from .env if present
if [ -f .env ]; then
  echo "📄 Loading configuration from .env..."
  set -a
  source .env
  set +a
else
  echo "⚠️  No .env file found. Copying from .env.example..."
  cp .env.example .env
  set -a
  source .env
  set +a
fi

# Ensure uploads directory exists
mkdir -p uploads

# Build frontend if missing or if explicitly requested with --build / -b
if [[ "${1:-}" == "--build" || "${1:-}" == "-b" || ! -d "frontend/dist" || ! -f "frontend/dist/index.html" ]]; then
  echo "⚡ Building Vue 3 + PrimeVue frontend bundle..."
  (cd frontend && npm run build:fast)
fi

# Set defaults
HOST="${HOST:-0.0.0.0}"
PORT="${PORT:-8081}"
DATABASE_URL="${DATABASE_URL:-postgres://postgres:postgres@localhost:5432/esan_quiz}"
export HOST
export PORT
export DATABASE_URL
export CARGO_TARGET_DIR="./target"
export RUST_LOG="${RUST_LOG:-info}"

echo "=================================================================="
echo "⚡ EZSN Quiz - Real-Time Multiplayer Quiz Platform"
echo "=================================================================="
echo "🚀 Application URL : http://localhost:${PORT}"
echo "🛡️  Admin Console   : http://localhost:${PORT}/admin"
echo "🎮 Player Join      : http://localhost:${PORT}"
echo "📺 Host Screen      : http://localhost:${PORT}/host"
echo "💾 Database (PG)    : ${DATABASE_URL}"
echo "=================================================================="
echo "Starting Actix Web server with live WebSockets..."
echo ""

# Run the Rust server
exec cargo run
