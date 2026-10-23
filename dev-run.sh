#!/usr/bin/env bash
# ==============================================================================
# EZSN Quiz - Development Runner Script
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

# Set default host and port if not specified
HOST="${HOST:-0.0.0.0}"
PORT="${PORT:-8081}"
export HOST
export PORT
export CARGO_TARGET_DIR="./target"
export RUST_LOG="${RUST_LOG:-info}"

echo "=================================================================="
echo "⚡ EZSN Quiz - Real-Time Multiplayer Quiz Platform"
echo "=================================================================="
echo "🚀 Server URL : http://localhost:${PORT}"
echo "📺 Host Screen : http://localhost:${PORT}/host"
echo "✏️  Quiz Creator: http://localhost:${PORT}/edit"
echo "📜 Past History: http://localhost:${PORT}/history"
echo "💾 Database   : ${DATABASE_URL}"
echo "=================================================================="
echo "Starting Actix Web server with live WebSockets..."
echo ""

# Run the Rust server
exec cargo run
