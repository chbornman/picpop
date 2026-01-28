#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log() { echo -e "${GREEN}[+]${NC} $1"; }
warn() { echo -e "${YELLOW}[!]${NC} $1"; }

echo ""
echo "========================================="
echo "  PicPop Full Deployment"
echo "========================================="
echo ""

# Build and deploy server (backend + frontend)
log "Building server..."
"$SCRIPT_DIR/build-server.sh"

log "Deploying server..."
"$SCRIPT_DIR/deploy-server.sh"

# Build and deploy kiosk
log "Building kiosk for ARM64..."
"$SCRIPT_DIR/build-kiosk-arm64.sh"

log "Deploying kiosk..."
"$SCRIPT_DIR/deploy-kiosk.sh"

echo ""
log "All done!"
