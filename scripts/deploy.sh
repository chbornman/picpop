#!/bin/bash
# Deploy PicPop to the Radxa device
#
# This script deploys:
#   1. Server: Python backend + React frontend bundle
#   2. Kiosk: Native GTK4 binary + systemd service
#
# Prerequisites:
#   - Server: Run ./scripts/build-server.sh first (or ./scripts/build.sh)
#   - Kiosk: Run ./scripts/build-kiosk.sh first (or use --on-radxa to skip)
#
# Usage:
#   ./scripts/deploy.sh              # Deploy cross-compiled kiosk binary
#   ./scripts/deploy.sh --on-radxa   # Deploy kiosk built on device

source "$(dirname "$0")/common.sh"

parse_common_flags "$@"

if $SHOW_HELP; then
    cat << 'EOF'
Deploy PicPop to the Radxa device

Usage: ./scripts/deploy.sh [options]

Options:
    --on-radxa      Kiosk was built on device (don't copy binary from local)
    -h, --help      Show this help message

What Gets Deployed:
    Server:
        - Python backend code -> /opt/picpop/
        - React frontend dist -> /opt/picpop/frontend/dist/
        - Systemd service: picpop.service

    Kiosk:
        - Native binary -> /home/kiosk/picpop-kiosk
        - Systemd service: picpop-kiosk.service

Prerequisites:
    Run ./scripts/build.sh first (or individual build scripts)

Environment Variables:
    RADXA_HOST      SSH target (default: kiosk@192.168.0.110)
EOF
    exit 0
fi

header "Deploying PicPop to $RADXA_HOST"

check_ssh

# =============================================================================
# Deploy Server (Backend + Frontend)
# =============================================================================
deploy_server() {
    log "Deploying server..."

    # Check if frontend is built
    if [[ ! -d "$FRONTEND_DIR/dist" ]]; then
        error "Frontend not built. Run ./scripts/build-server.sh first"
    fi

    # Stop the service before deploying
    log "Stopping picpop service..."
    ssh "$RADXA_HOST" "sudo systemctl stop picpop 2>/dev/null || true"

    # Create remote directories
    log "Ensuring remote directories exist..."
    ssh "$RADXA_HOST" "sudo mkdir -p $REMOTE_PATH/frontend/dist && sudo chown -R \$(whoami) $REMOTE_PATH"

    # Sync backend (Python source)
    # Note: --delete removes files not in source, so we must exclude frontend/
    log "Syncing backend..."
    rsync -avz --delete \
        --exclude '__pycache__' \
        --exclude '*.pyc' \
        --exclude '.venv' \
        --exclude '.pytest_cache' \
        --exclude '*.egg-info' \
        --exclude 'data/' \
        --exclude 'frontend/' \
        "$BACKEND_DIR/" "$RADXA_HOST:$REMOTE_PATH/"

    # Sync frontend dist
    log "Syncing frontend..."
    rsync -avz --delete \
        "$FRONTEND_DIR/dist/" "$RADXA_HOST:$REMOTE_PATH/frontend/dist/"

    # Install/update Python dependencies
    log "Checking Python dependencies..."
    ssh "$RADXA_HOST" "cd $REMOTE_PATH && \
        python3 -m venv .venv 2>/dev/null || true && \
        if ! cmp -s pyproject.toml .venv/.pyproject.toml.cache 2>/dev/null; then \
            echo '[+] Dependencies changed, installing...' && \
            .venv/bin/pip install -q --upgrade pip && \
            .venv/bin/pip install -q -e . && \
            cp pyproject.toml .venv/.pyproject.toml.cache; \
        else \
            echo '[i] Dependencies unchanged, skipping'; \
        fi"

    # Ensure data directory has correct permissions
    log "Fixing data directory permissions..."
    ssh "$RADXA_HOST" "sudo mkdir -p $REMOTE_PATH/data && sudo chown -R \$(whoami) $REMOTE_PATH/data"

    # Deploy service file
    log "Deploying picpop.service..."
    scp "$DEPLOY_DIR/picpop.service" "$RADXA_HOST:/tmp/picpop.service"
    ssh "$RADXA_HOST" "sudo cp /tmp/picpop.service /etc/systemd/system/ && sudo systemctl daemon-reload && sudo systemctl enable picpop"

    # Start the service
    log "Starting picpop service..."
    ssh "$RADXA_HOST" "sudo systemctl start picpop"

    info "Server deployed successfully"
}

# =============================================================================
# Deploy Kiosk
# =============================================================================
deploy_kiosk() {
    log "Deploying kiosk..."

    # Stop existing service
    log "Stopping picpop-kiosk service..."
    ssh "$RADXA_HOST" "sudo systemctl stop picpop-kiosk 2>/dev/null || true"

    if $ON_RADXA; then
        # Kiosk was built on device - just copy from build location
        log "Installing kiosk binary (built on device)..."
        ssh "$RADXA_HOST" "test -f ~/kiosk-native/target/release/picpop-kiosk" || \
            error "Kiosk binary not found on device. Run ./scripts/build-kiosk.sh --on-radxa first"
        ssh "$RADXA_HOST" "sudo cp ~/kiosk-native/target/release/picpop-kiosk /home/kiosk/picpop-kiosk && \
            sudo chown kiosk:kiosk /home/kiosk/picpop-kiosk && \
            sudo chmod +x /home/kiosk/picpop-kiosk"
    else
        # Kiosk was cross-compiled locally - copy binary to device
        if [[ ! -f "$KIOSK_OUTPUT_CROSS" ]]; then
            error "Kiosk binary not found: $KIOSK_OUTPUT_CROSS\nRun ./scripts/build-kiosk.sh first"
        fi

        log "Copying kiosk binary to device..."
        scp "$KIOSK_OUTPUT_CROSS" "$RADXA_HOST:/tmp/picpop-kiosk"
        ssh "$RADXA_HOST" "sudo cp /tmp/picpop-kiosk /home/kiosk/picpop-kiosk && \
            sudo chown kiosk:kiosk /home/kiosk/picpop-kiosk && \
            sudo chmod +x /home/kiosk/picpop-kiosk"
    fi

    # Deploy service file
    log "Deploying picpop-kiosk.service..."
    scp "$DEPLOY_DIR/picpop-kiosk.service" "$RADXA_HOST:/tmp/picpop-kiosk.service"
    ssh "$RADXA_HOST" "sudo cp /tmp/picpop-kiosk.service /etc/systemd/system/ && sudo systemctl daemon-reload && sudo systemctl enable picpop-kiosk"

    # Start the service
    log "Starting picpop-kiosk service..."
    ssh "$RADXA_HOST" "sudo systemctl start picpop-kiosk"

    info "Kiosk deployed successfully"
}

# =============================================================================
# Main
# =============================================================================

deploy_server
echo ""
deploy_kiosk

echo ""
header "Deployment Complete"

# Show service status
log "Checking service status..."
ssh "$RADXA_HOST" "sudo systemctl status picpop picpop-kiosk --no-pager -n 3" || warn "One or more services may not be running"

echo ""
log "Deployment successful!"
info "Server: http://$RADXA_HOST:8000 (or http://192.168.4.1 via WiFi AP)"
