#!/bin/bash
set -e

# Configuration
RADXA_HOST="${RADXA_HOST:-kiosk@192.168.0.110}"
REMOTE_PATH="/opt/picpop"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log() { echo -e "${GREEN}[+]${NC} $1"; }
warn() { echo -e "${YELLOW}[!]${NC} $1"; }
error() { echo -e "${RED}[x]${NC} $1"; exit 1; }

cd "$PROJECT_ROOT"

# Check if frontend is built
if [[ ! -d "frontend/dist" ]]; then
    error "Frontend not built. Run ./scripts/build-server.sh first"
fi

log "Deploying PicPop server to $RADXA_HOST..."

# Test SSH connection
log "Testing SSH connection..."
ssh -o ConnectTimeout=5 "$RADXA_HOST" "echo 'SSH OK'" || error "Cannot connect to $RADXA_HOST"

# Stop the service before deploying
log "Stopping picpop service..."
ssh "$RADXA_HOST" "sudo systemctl stop picpop 2>/dev/null || true"

# Create remote directories
log "Ensuring remote directories exist..."
ssh "$RADXA_HOST" "sudo mkdir -p $REMOTE_PATH/frontend/dist && sudo chown -R \$(whoami) $REMOTE_PATH"

# Sync backend (excluding venv, cache, etc.)
log "Syncing backend..."
rsync -avz --delete \
    --exclude '__pycache__' \
    --exclude '*.pyc' \
    --exclude '.venv' \
    --exclude '.pytest_cache' \
    --exclude '*.egg-info' \
    --exclude 'data/' \
    --exclude 'frontend/' \
    --exclude 'kiosk/' \
    backend/ "$RADXA_HOST:$REMOTE_PATH/"

# Sync frontend dist
log "Syncing frontend..."
rsync -avz --delete \
    frontend/dist/ "$RADXA_HOST:$REMOTE_PATH/frontend/dist/"

# Install/update Python dependencies
log "Installing Python dependencies..."
ssh "$RADXA_HOST" "cd $REMOTE_PATH && \
    python3 -m venv .venv 2>/dev/null || true && \
    .venv/bin/pip install -q --upgrade pip && \
    .venv/bin/pip install -q -e ."

# Ensure data directory has correct permissions
log "Fixing data directory permissions..."
ssh "$RADXA_HOST" "sudo mkdir -p $REMOTE_PATH/data && sudo chown -R \$(whoami) $REMOTE_PATH/data"

# Restart the service
log "Starting picpop service..."
ssh "$RADXA_HOST" "sudo systemctl start picpop"

# Check status
log "Checking service status..."
ssh "$RADXA_HOST" "sudo systemctl status picpop --no-pager" || warn "Service may not be running"

log "Server deployment complete!"
