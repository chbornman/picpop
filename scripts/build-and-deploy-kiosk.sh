#!/bin/bash
# Build and deploy the PicPop native kiosk (GTK4)
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Colors
GREEN='\033[0;32m'
NC='\033[0m'

log() { echo -e "${GREEN}[+]${NC} $1"; }

echo ""
echo "========================================="
echo "  PicPop Native Kiosk Deployment"
echo "========================================="
echo ""

log "Building native kiosk on Radxa..."
"$SCRIPT_DIR/build-native-kiosk.sh" radxa

log "Deploying native kiosk..."
"$SCRIPT_DIR/deploy-native-kiosk.sh"

echo ""
log "Kiosk deployment complete!"
