#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Configuration
RADXA_HOST="${RADXA_HOST:-kiosk@192.168.0.110}"
REMOTE_PATH="/opt/picpop/kiosk"
LOCAL_BINARY="kiosk/src-tauri/target/aarch64-unknown-linux-gnu/release/picpop-kiosk"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log() { echo -e "${GREEN}[+]${NC} $1"; }
warn() { echo -e "${YELLOW}[!]${NC} $1"; }
error() { echo -e "${RED}[x]${NC} $1"; exit 1; }

# Check if binary exists
if [[ ! -f "$LOCAL_BINARY" ]]; then
    error "Binary not found at $LOCAL_BINARY. Did you run the cross-compile build?"
fi

log "Deploying picpop-kiosk to $RADXA_HOST..."

# Test SSH connection
log "Testing SSH connection..."
ssh -o ConnectTimeout=5 "$RADXA_HOST" "echo 'SSH OK'" || error "Cannot connect to $RADXA_HOST"

# Stop the service before deploying
log "Stopping picpop-kiosk service..."
ssh "$RADXA_HOST" "sudo systemctl stop picpop-kiosk 2>/dev/null || true"

# Create remote directory if needed
log "Ensuring remote directory exists..."
ssh "$RADXA_HOST" "sudo mkdir -p $REMOTE_PATH && sudo chown \$(whoami) $REMOTE_PATH"

# Copy the binary
log "Copying binary ($(du -h "$LOCAL_BINARY" | cut -f1))..."
scp "$LOCAL_BINARY" "$RADXA_HOST:$REMOTE_PATH/picpop-kiosk"

# Make executable
ssh "$RADXA_HOST" "chmod +x $REMOTE_PATH/picpop-kiosk"

# Deploy service file
log "Deploying service file..."
scp "$(dirname "$SCRIPT_DIR")/deploy/picpop-kiosk.service" "$RADXA_HOST:/tmp/picpop-kiosk.service"
ssh "$RADXA_HOST" "sudo cp /tmp/picpop-kiosk.service /etc/systemd/system/ && sudo systemctl daemon-reload"

# Restart the service
log "Starting picpop-kiosk service..."
ssh "$RADXA_HOST" "sudo systemctl start picpop-kiosk"

# Check status
log "Checking service status..."
ssh "$RADXA_HOST" "sudo systemctl status picpop-kiosk --no-pager" || warn "Service may not be running"

log "Deployment complete!"
