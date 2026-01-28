#!/bin/bash
# Deploy the native GTK4 kiosk on Radxa (assumes build was done on-device)
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
SERVICE_FILE="$PROJECT_DIR/deploy/picpop-native-kiosk.service"

# Configuration
RADXA_HOST="${RADXA_HOST:-kiosk@192.168.0.110}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log() { echo -e "${GREEN}[+]${NC} $1"; }
warn() { echo -e "${YELLOW}[!]${NC} $1"; }
error() { echo -e "${RED}[x]${NC} $1"; exit 1; }

log "Deploying native picpop-kiosk on $RADXA_HOST..."

# Test SSH connection
log "Testing SSH connection..."
ssh -o ConnectTimeout=5 "$RADXA_HOST" "echo 'SSH OK'" || error "Cannot connect to $RADXA_HOST"

# Check if the binary exists on the Radxa
log "Checking for built binary..."
ssh "$RADXA_HOST" "test -f ~/kiosk-native/target/release/picpop-kiosk" || error "Binary not found. Run ./scripts/build-native-kiosk.sh radxa first"

# Stop existing services
log "Stopping existing kiosk services..."
ssh "$RADXA_HOST" "sudo systemctl stop picpop-kiosk 2>/dev/null || true"
ssh "$RADXA_HOST" "sudo systemctl stop picpop-native-kiosk 2>/dev/null || true"

# Copy binary to final location
log "Installing binary..."
ssh "$RADXA_HOST" "sudo cp ~/kiosk-native/target/release/picpop-kiosk /home/kiosk/picpop-kiosk && sudo chown kiosk:kiosk /home/kiosk/picpop-kiosk && sudo chmod +x /home/kiosk/picpop-kiosk"

# Deploy service file
log "Deploying service file..."
scp "$SERVICE_FILE" "$RADXA_HOST:/tmp/picpop-native-kiosk.service"
ssh "$RADXA_HOST" "sudo cp /tmp/picpop-native-kiosk.service /etc/systemd/system/ && sudo systemctl daemon-reload"

# Disable old service, enable new one
log "Configuring services..."
ssh "$RADXA_HOST" "sudo systemctl disable picpop-kiosk 2>/dev/null || true"
ssh "$RADXA_HOST" "sudo systemctl enable picpop-native-kiosk"

# Start the service
log "Starting picpop-native-kiosk service..."
ssh "$RADXA_HOST" "sudo systemctl start picpop-native-kiosk"

# Check status
log "Checking service status..."
ssh "$RADXA_HOST" "sudo systemctl status picpop-native-kiosk --no-pager" || warn "Service may not be running"

log "Deployment complete!"
