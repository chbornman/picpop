#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
KIOSK_DIR="$PROJECT_ROOT/kiosk"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log() { echo -e "${GREEN}[+]${NC} $1"; }
warn() { echo -e "${YELLOW}[!]${NC} $1"; }
error() { echo -e "${RED}[x]${NC} $1"; exit 1; }

IMAGE_NAME="picpop-cross-aarch64"

# Check if Docker image exists, build if not
if ! docker images --format "{{.Repository}}" | grep -q "^${IMAGE_NAME}$"; then
    log "Building cross-compilation Docker image (this may take a few minutes)..."
    docker build -f "$KIOSK_DIR/.cross/Dockerfile.aarch64" -t "$IMAGE_NAME" "$KIOSK_DIR/.cross/"
fi

log "Cross-compiling Tauri app for aarch64..."
docker run --rm \
    -v "$PROJECT_ROOT:/project" \
    -w /project/kiosk \
    -e CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
    -e CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc \
    -e CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++ \
    -e PKG_CONFIG_PATH=/usr/lib/aarch64-linux-gnu/pkgconfig \
    -e PKG_CONFIG_SYSROOT_DIR=/ \
    -e PKG_CONFIG_ALLOW_CROSS=1 \
    "$IMAGE_NAME" \
    bash -c "bun install && bun run tauri build --target aarch64-unknown-linux-gnu --no-bundle"

BINARY="$KIOSK_DIR/src-tauri/target/aarch64-unknown-linux-gnu/release/picpop-kiosk"

if [[ -f "$BINARY" ]]; then
    SIZE=$(du -h "$BINARY" | cut -f1)
    log "Build complete! Binary: $BINARY ($SIZE)"
else
    error "Build failed - binary not found"
fi
