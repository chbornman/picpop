#!/bin/bash
# Build and deploy the PicPop server (backend + mobile frontend)
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Colors
GREEN='\033[0;32m'
NC='\033[0m'

log() { echo -e "${GREEN}[+]${NC} $1"; }

echo ""
echo "========================================="
echo "  PicPop Server Deployment"
echo "========================================="
echo ""

log "Building server..."
"$SCRIPT_DIR/build-server.sh"

log "Deploying server..."
"$SCRIPT_DIR/deploy-server.sh"

echo ""
log "Server deployment complete!"
