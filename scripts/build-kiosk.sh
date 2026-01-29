#!/bin/bash
# Build the PicPop native kiosk (Rust/GTK4 touchscreen app)
#
# Build modes:
#   Default:     Cross-compile for ARM64 using Docker (multiarch)
#   --qemu:      Cross-compile using QEMU emulation (slower, more reliable)
#   --on-radxa:  Build natively on the Radxa device via SSH
#
# Usage:
#   ./scripts/build-kiosk.sh              # Cross-compile (default)
#   ./scripts/build-kiosk.sh --qemu       # Cross-compile with QEMU
#   ./scripts/build-kiosk.sh --on-radxa   # Build on device

source "$(dirname "$0")/common.sh"

parse_common_flags "$@"

if $SHOW_HELP; then
    cat << 'EOF'
Build the PicPop native kiosk (Rust/GTK4)

Usage: ./scripts/build-kiosk.sh [options]

Options:
    --on-radxa      Build natively on the Radxa device (via SSH)
    --qemu          Use QEMU emulation for cross-compilation (slower but reliable)
    -h, --help      Show this help message

Build Modes:
    Default (cross-compilation):
        Uses Docker with Debian multiarch to cross-compile for ARM64.
        Fast (~5 min) but may fail if package conflicts occur.

    QEMU (--qemu):
        Uses QEMU user-mode emulation to run an ARM64 container.
        Slow (~30+ min) but very reliable. Good fallback if cross fails.

    On-device (--on-radxa):
        SSHs to the Radxa, syncs source code, and builds natively.
        Requires Rust toolchain installed on the device.

Output:
    Cross:     kiosk-native/target/aarch64-unknown-linux-gnu/release/picpop-kiosk
    On-Radxa:  ~/kiosk-native/target/release/picpop-kiosk (on device)
EOF
    exit 0
fi

# =============================================================================
# Cross-compilation (Docker multiarch)
# =============================================================================
build_cross() {
    local container_tool
    container_tool=$(get_container_tool)
    local dockerfile="$DOCKER_DIR/Dockerfile.cross"

    header "Cross-compiling Kiosk for ARM64"
    info "Method: Docker multiarch (fast)"

    # Check dockerfile exists
    [[ -f "$dockerfile" ]] || error "Dockerfile not found: $dockerfile"

    # Build Docker image if needed
    if ! $container_tool image inspect "$DOCKER_IMAGE_CROSS" &>/dev/null; then
        log "Building Docker image '$DOCKER_IMAGE_CROSS'..."
        $container_tool build -t "$DOCKER_IMAGE_CROSS" -f "$dockerfile" "$DOCKER_DIR"
    else
        info "Using existing Docker image '$DOCKER_IMAGE_CROSS'"
        info "To rebuild: $container_tool build --no-cache -t $DOCKER_IMAGE_CROSS -f $dockerfile $DOCKER_DIR"
    fi

    # Run the build then fix ownership of created files
    log "Compiling for $KIOSK_TARGET..."
    $container_tool run --rm \
        -v "$KIOSK_DIR:/project:Z" \
        -w /project \
        -e CARGO_HOME=/project/.cargo-cross \
        "$DOCKER_IMAGE_CROSS" \
        sh -c "cargo build --target $KIOSK_TARGET --release && chown -R $(id -u):$(id -g) /project/target /project/.cargo-cross"

    verify_binary "$KIOSK_OUTPUT_CROSS"
}

# =============================================================================
# Cross-compilation (QEMU emulation)
# =============================================================================
build_qemu() {
    local container_tool
    container_tool=$(get_container_tool)
    local dockerfile="$DOCKER_DIR/Dockerfile.qemu"

    header "Cross-compiling Kiosk for ARM64"
    info "Method: QEMU emulation (slow but reliable)"

    # Check dockerfile exists
    [[ -f "$dockerfile" ]] || error "Dockerfile not found: $dockerfile"

    # Setup QEMU binfmt if needed
    if [[ ! -f /proc/sys/fs/binfmt_misc/qemu-aarch64 ]]; then
        warn "Setting up QEMU binfmt_misc..."
        $container_tool run --rm --privileged multiarch/qemu-user-static --reset -p yes || \
            error "Failed to setup QEMU. Try: docker run --rm --privileged multiarch/qemu-user-static --reset -p yes"
    fi

    # Build Docker image if needed
    if ! $container_tool image inspect "$DOCKER_IMAGE_QEMU" &>/dev/null; then
        log "Building QEMU Docker image '$DOCKER_IMAGE_QEMU' (this takes a while)..."
        $container_tool build --platform linux/arm64 -t "$DOCKER_IMAGE_QEMU" -f "$dockerfile" "$DOCKER_DIR"
    else
        info "Using existing Docker image '$DOCKER_IMAGE_QEMU'"
    fi

    # Run the build then fix ownership of created files
    log "Compiling natively in QEMU (be patient, this is slow)..."
    $container_tool run --rm --platform linux/arm64 \
        -v "$KIOSK_DIR:/project:Z" \
        -w /project \
        -e CARGO_HOME=/project/.cargo-qemu \
        "$DOCKER_IMAGE_QEMU" \
        sh -c "cargo build --release && chown -R $(id -u):$(id -g) /project/target /project/.cargo-qemu"

    # QEMU build outputs to target/release, copy to cross output location
    if [[ -f "$KIOSK_DIR/target/release/picpop-kiosk" ]]; then
        mkdir -p "$(dirname "$KIOSK_OUTPUT_CROSS")"
        cp "$KIOSK_DIR/target/release/picpop-kiosk" "$KIOSK_OUTPUT_CROSS"
        log "Copied binary to $KIOSK_OUTPUT_CROSS"
    fi

    verify_binary "$KIOSK_OUTPUT_CROSS"
}

# =============================================================================
# Build on Radxa device
# =============================================================================
build_on_radxa() {
    header "Building Kiosk on Radxa"
    info "Method: Native compilation on device"

    check_ssh

    # Sync source code to Radxa
    log "Syncing source code to $RADXA_HOST..."
    rsync -avz --delete \
        --exclude 'target' \
        --exclude '.git' \
        --exclude '.cargo*' \
        --exclude 'docker' \
        "$KIOSK_DIR/" "$RADXA_HOST:~/kiosk-native/"

    # Build on device
    log "Building on Radxa (this may take a while on first run)..."
    ssh "$RADXA_HOST" bash << 'EOF'
set -e
cd ~/kiosk-native

# Check/install Rust if needed
if ! command -v cargo &>/dev/null; then
    echo "[+] Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
fi
source ~/.cargo/env

# Check/install dependencies if needed
if ! pkg-config --exists gtk4 2>/dev/null; then
    echo "[+] Installing GTK4 and GStreamer dependencies..."
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

# Build with Cortex-A55 optimizations
# The Radxa Zero 3W has a Cortex-A55 (ARMv8.2-A) with:
#   - AES/SHA2 hardware crypto
#   - CRC32 instructions
#   - LSE atomics
echo "[+] Building release with Cortex-A55 optimizations..."
export RUSTFLAGS="-C target-cpu=cortex-a55 -C target-feature=+aes,+sha2,+crc"
cargo build --release

echo "[+] Build complete!"
ls -lh target/release/picpop-kiosk
EOF

    log "Build complete on Radxa"
    info "Binary location (on device): ~/kiosk-native/target/release/picpop-kiosk"
}

# =============================================================================
# Verify binary
# =============================================================================
verify_binary() {
    local binary="$1"

    if [[ -f "$binary" ]]; then
        local size
        size=$(du -h "$binary" | cut -f1)
        log "Build successful: $binary ($size)"

        # Verify architecture
        local arch
        arch=$(file "$binary" | grep -oE 'ARM aarch64|aarch64|x86-64' || echo "unknown")
        if [[ "$arch" == *"aarch64"* ]] || [[ "$arch" == *"ARM"* ]]; then
            info "Architecture: aarch64 (correct for Radxa)"
        elif [[ "$arch" == *"x86-64"* ]]; then
            warn "Architecture: x86-64 (won't run on Radxa!)"
        else
            warn "Architecture: $arch (unexpected)"
        fi
    else
        error "Build failed - binary not found: $binary"
    fi
}

# =============================================================================
# Main
# =============================================================================

if $ON_RADXA; then
    build_on_radxa
elif $USE_QEMU; then
    build_qemu
else
    build_cross
fi

echo ""
log "Kiosk build complete!"
