#!/bin/bash
# Build the native GTK4 kiosk - either locally or on the Radxa
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
KIOSK_DIR="$PROJECT_DIR/kiosk-native"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log() { echo -e "${GREEN}[+]${NC} $1"; }
warn() { echo -e "${YELLOW}[!]${NC} $1"; }
error() { echo -e "${RED}[x]${NC} $1"; exit 1; }

# Configuration
RADXA_HOST="${RADXA_HOST:-picpop@192.168.0.110}"

# Check for required system dependencies (local build only)
check_deps() {
    local missing=()

    if ! pkg-config --exists gtk4 2>/dev/null; then
        missing+=("libgtk-4-dev")
    fi

    if ! pkg-config --exists gstreamer-1.0 2>/dev/null; then
        missing+=("libgstreamer1.0-dev")
    fi

    if [ ${#missing[@]} -ne 0 ]; then
        echo "Missing dependencies: ${missing[*]}"
        echo "Install with: sudo apt install ${missing[*]} gstreamer1.0-plugins-good gstreamer1.0-plugins-bad"
        exit 1
    fi
}

# Build for local architecture
build_local() {
    log "Building native kiosk for local architecture..."
    cd "$KIOSK_DIR"
    check_deps
    cargo build --release
    log "Build complete: target/release/picpop-kiosk"
}

# Build on the Radxa device (remote native build)
build_radxa() {
    log "Building native kiosk on Radxa ($RADXA_HOST)..."

    # Test SSH connection
    ssh -o ConnectTimeout=5 "$RADXA_HOST" "echo 'SSH OK'" || error "Cannot connect to $RADXA_HOST"

    # Sync source code to Radxa
    log "Syncing source code..."
    rsync -avz --delete \
        --exclude 'target' \
        --exclude '.git' \
        "$KIOSK_DIR/" "$RADXA_HOST:~/kiosk-native/"

    # Install dependencies if needed and build
    log "Building on Radxa (this may take a while on first run)..."
    ssh "$RADXA_HOST" bash << 'EOF'
set -e
cd ~/kiosk-native

# Check/install Rust if needed
if ! command -v cargo &>/dev/null; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
fi
source ~/.cargo/env

# Check/install dependencies if needed
if ! pkg-config --exists gtk4 2>/dev/null; then
    echo "Installing GTK4 and GStreamer dependencies..."
    sudo apt-get update
    sudo apt-get install -y \
        libgtk-4-dev \
        libgstreamer1.0-dev \
        libgstreamer-plugins-base1.0-dev \
        libgstreamer-plugins-bad1.0-dev \
        gstreamer1.0-plugins-good \
        gstreamer1.0-plugins-bad \
        gstreamer1.0-gtk4 \
        libsoup-3.0-dev \
        libssl-dev \
        pkg-config \
        build-essential
fi

# Build
echo "Building release..."
cargo build --release

echo "Build complete!"
ls -lh target/release/picpop-kiosk
EOF

    log "Build complete on Radxa"
}

# Run locally for testing
run_local() {
    log "Running native kiosk locally..."
    cd "$KIOSK_DIR"
    check_deps
    RUST_LOG=info cargo run --release
}

case "${1:-radxa}" in
    local)
        build_local
        ;;
    radxa|remote|arm64)
        build_radxa
        ;;
    run)
        run_local
        ;;
    *)
        echo "Usage: $0 [local|radxa|run]"
        echo "  local - Build for current architecture (x86_64)"
        echo "  radxa - Build natively on Radxa device (default)"
        echo "  run   - Build and run locally"
        exit 1
        ;;
esac
