#!/bin/bash
# PicPop - Build and Deploy Everything
#
# This is the main entry point for building and deploying PicPop.
#
# Usage:
#   ./scripts/build-and-deploy.sh              # Cross-compile locally, deploy
#   ./scripts/build-and-deploy.sh --qemu       # Use QEMU for cross-compilation
#   ./scripts/build-and-deploy.sh --on-radxa   # Build kiosk on device, deploy

source "$(dirname "$0")/common.sh"

parse_common_flags "$@"

if $SHOW_HELP; then
    cat << 'EOF'
PicPop - Build and Deploy Everything

Usage: ./scripts/build-and-deploy.sh [options]

Options:
    --on-radxa      Build kiosk natively on the Radxa device
    --qemu          Use QEMU emulation for kiosk cross-compilation
    -h, --help      Show this help message

What This Does:
    1. Builds the server (React frontend + Python backend prep)
    2. Builds the kiosk (Rust/GTK4 native app)
    3. Deploys everything to the Radxa device

Build Modes:
    Default:
        Cross-compiles the kiosk for ARM64 using Docker multiarch.
        Fast (~5 min) but may fail on some systems.

    --qemu:
        Cross-compiles using QEMU user-mode emulation.
        Slow (~30+ min) but very reliable fallback.

    --on-radxa:
        Builds the kiosk natively on the Radxa device via SSH.
        Good if cross-compilation isn't working.

Examples:
    # Standard deployment (recommended)
    ./scripts/build-and-deploy.sh

    # If cross-compilation fails, use QEMU
    ./scripts/build-and-deploy.sh --qemu

    # Build on device (requires Rust on Radxa)
    ./scripts/build-and-deploy.sh --on-radxa

Environment Variables:
    RADXA_HOST      SSH target (default: kiosk@192.168.0.110)

Individual Scripts:
    ./scripts/build.sh          Build all components
    ./scripts/build-server.sh   Build only server
    ./scripts/build-kiosk.sh    Build only kiosk
    ./scripts/deploy.sh         Deploy to Radxa
EOF
    exit 0
fi

# Build banner
echo ""
echo "========================================="
echo "  PicPop Build & Deploy"
echo "========================================="
echo ""

if $ON_RADXA; then
    info "Mode: Build kiosk on Radxa device"
elif $USE_QEMU; then
    info "Mode: Cross-compile with QEMU (slow)"
else
    info "Mode: Cross-compile with Docker (fast)"
fi
info "Target: $RADXA_HOST"
echo ""

# Build flags to pass to sub-scripts
BUILD_FLAGS=""
$ON_RADXA && BUILD_FLAGS="--on-radxa"
$USE_QEMU && BUILD_FLAGS="--qemu"

# Step 1: Build everything
"$SCRIPT_DIR/build.sh" $BUILD_FLAGS

echo ""

# Step 2: Deploy everything
"$SCRIPT_DIR/deploy.sh" $BUILD_FLAGS

echo ""
echo "========================================="
echo "  Deployment Complete!"
echo "========================================="
echo ""
log "PicPop is now running on $RADXA_HOST"
info "Mobile app: http://192.168.4.1 (connect to PicPop WiFi)"
info "Direct access: http://${RADXA_HOST#*@}:8000"
