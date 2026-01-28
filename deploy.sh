#!/bin/bash
# PicPop Deploy Script
# Usage: ./deploy.sh [backend|kiosk|frontend|all]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

deploy_backend() {
    log_info "Deploying backend..."
    sudo cp -r "$SCRIPT_DIR/backend/"* /opt/picpop/
    sudo chown -R picpop:picpop /opt/picpop
    sudo systemctl restart picpop
    log_info "Backend deployed and restarted"
}

deploy_frontend() {
    log_info "Building frontend..."
    cd "$SCRIPT_DIR/frontend"
    bun run build
    
    log_info "Deploying frontend..."
    sudo cp -r dist/* /opt/mobile/dist/
    sudo systemctl restart picpop
    log_info "Frontend deployed and backend restarted"
}

deploy_kiosk() {
    log_info "Building kiosk (this may take a few minutes)..."
    cd "$SCRIPT_DIR/kiosk"
    bun run tauri build
    
    log_info "Deploying kiosk..."
    sudo cp src-tauri/target/release/picpop-kiosk /opt/picpop/kiosk/
    sudo systemctl restart picpop-kiosk
    log_info "Kiosk deployed and restarted"
}

show_status() {
    echo ""
    log_info "Service status:"
    systemctl status picpop picpop-kiosk --no-pager | head -20
}

show_usage() {
    echo "Usage: $0 [command]"
    echo ""
    echo "Commands:"
    echo "  backend   - Deploy backend changes (Python)"
    echo "  frontend  - Build and deploy frontend (phone web app)"
    echo "  kiosk     - Build and deploy kiosk (Tauri app)"
    echo "  all       - Deploy everything"
    echo "  status    - Show service status"
    echo ""
    echo "Examples:"
    echo "  $0 backend    # Quick deploy for backend changes"
    echo "  $0 frontend   # Rebuild and deploy phone UI"
    echo "  $0 kiosk      # Rebuild and deploy touchscreen app"
    echo "  $0 all        # Full rebuild and deploy"
}

case "${1:-}" in
    backend)
        deploy_backend
        ;;
    frontend)
        deploy_frontend
        ;;
    kiosk)
        deploy_kiosk
        ;;
    all)
        deploy_backend
        deploy_frontend
        deploy_kiosk
        ;;
    status)
        show_status
        ;;
    *)
        show_usage
        exit 1
        ;;
esac

show_status
