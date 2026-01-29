#!/bin/bash
# Build all PicPop components
#
# This script builds:
#   1. Server: FastAPI backend + React frontend (always local)
#   2. Kiosk: Rust/GTK4 native app (cross-compile or on-device)
#
# Usage:
#   ./scripts/build.sh              # Cross-compile kiosk (default)
#   ./scripts/build.sh --qemu       # Use QEMU for kiosk cross-compilation
#   ./scripts/build.sh --on-radxa   # Build kiosk on device

source "$(dirname "$0")/common.sh"

parse_common_flags "$@"

if $SHOW_HELP; then
    cat << 'EOF'
Build all PicPop components

Usage: ./scripts/build.sh [options]

Options:
    --on-radxa      Build kiosk natively on the Radxa device
    --qemu          Use QEMU emulation for kiosk cross-compilation
    -h, --help      Show this help message

Components Built:
    Server (always local):
        - React frontend compiled with Bun
        - Python backend (no compilation needed)

    Kiosk (configurable):
        - Default: Cross-compiled for ARM64 using Docker
        - --qemu: Cross-compiled using QEMU (slower, reliable)
        - --on-radxa: Built natively on the Radxa device

See also:
    ./scripts/build-server.sh   Build only the server
    ./scripts/build-kiosk.sh    Build only the kiosk
EOF
    exit 0
fi

header "Building PicPop"

# Build flags to pass to sub-scripts
BUILD_FLAGS=""
$ON_RADXA && BUILD_FLAGS="--on-radxa"
$USE_QEMU && BUILD_FLAGS="--qemu"

# Build server (always local - no flags needed)
log "Building server..."
"$SCRIPT_DIR/build-server.sh"

echo ""

# Build kiosk (pass through flags)
log "Building kiosk..."
"$SCRIPT_DIR/build-kiosk.sh" $BUILD_FLAGS

echo ""
header "Build Complete"
log "All components built successfully!"

if $ON_RADXA; then
    info "Kiosk was built on the Radxa device"
else
    info "Server output: $FRONTEND_DIR/dist/"
    info "Kiosk output:  $KIOSK_OUTPUT_CROSS"
fi
