#!/bin/bash
# Rebuild Docker cross-compilation images
#
# Use this after updating Dockerfiles (e.g., adding new dependencies)
#
# Usage:
#   ./scripts/rebuild-docker.sh              # Rebuild cross-compile image
#   ./scripts/rebuild-docker.sh --qemu       # Rebuild QEMU image
#   ./scripts/rebuild-docker.sh --all        # Rebuild both images

source "$(dirname "$0")/common.sh"

REBUILD_CROSS=false
REBUILD_QEMU=false
SHOW_HELP=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --qemu)
            REBUILD_QEMU=true
            shift
            ;;
        --all)
            REBUILD_CROSS=true
            REBUILD_QEMU=true
            shift
            ;;
        -h|--help)
            SHOW_HELP=true
            shift
            ;;
        *)
            shift
            ;;
    esac
done

# Default to cross if nothing specified
if ! $REBUILD_CROSS && ! $REBUILD_QEMU && ! $SHOW_HELP; then
    REBUILD_CROSS=true
fi

if $SHOW_HELP; then
    cat << 'EOF'
Rebuild Docker Cross-Compilation Images

Usage: ./scripts/rebuild-docker.sh [options]

Options:
    --qemu          Rebuild QEMU emulation image only
    --all           Rebuild both cross and QEMU images
    -h, --help      Show this help message

By default, rebuilds the cross-compilation image (faster builds).

When to rebuild:
    - After adding new system dependencies to Dockerfiles
    - After updating Dockerfile.cross or Dockerfile.qemu
    - If builds fail due to missing libraries

Examples:
    ./scripts/rebuild-docker.sh              # Rebuild cross image
    ./scripts/rebuild-docker.sh --qemu       # Rebuild QEMU image
    ./scripts/rebuild-docker.sh --all        # Rebuild everything
EOF
    exit 0
fi

container_tool=$(get_container_tool)

if $REBUILD_CROSS; then
    header "Rebuilding Cross-Compilation Image"
    
    dockerfile="$DOCKER_DIR/Dockerfile.cross"
    [[ -f "$dockerfile" ]] || error "Dockerfile not found: $dockerfile"
    
    # Remove old image if it exists
    if $container_tool image inspect "$DOCKER_IMAGE_CROSS" &>/dev/null; then
        log "Removing old image '$DOCKER_IMAGE_CROSS'..."
        $container_tool rmi "$DOCKER_IMAGE_CROSS" || warn "Could not remove old image"
    fi
    
    log "Building new image '$DOCKER_IMAGE_CROSS'..."
    $container_tool build --no-cache -t "$DOCKER_IMAGE_CROSS" -f "$dockerfile" "$DOCKER_DIR"
    
    log "Cross-compilation image rebuilt successfully!"
fi

if $REBUILD_QEMU; then
    header "Rebuilding QEMU Emulation Image"
    
    dockerfile="$DOCKER_DIR/Dockerfile.qemu"
    [[ -f "$dockerfile" ]] || error "Dockerfile not found: $dockerfile"
    
    # Setup QEMU binfmt if needed
    if [[ ! -f /proc/sys/fs/binfmt_misc/qemu-aarch64 ]]; then
        warn "Setting up QEMU binfmt_misc..."
        $container_tool run --rm --privileged multiarch/qemu-user-static --reset -p yes || \
            error "Failed to setup QEMU"
    fi
    
    # Remove old image if it exists
    if $container_tool image inspect "$DOCKER_IMAGE_QEMU" &>/dev/null; then
        log "Removing old image '$DOCKER_IMAGE_QEMU'..."
        $container_tool rmi "$DOCKER_IMAGE_QEMU" || warn "Could not remove old image"
    fi
    
    log "Building new QEMU image '$DOCKER_IMAGE_QEMU' (this takes a while)..."
    $container_tool build --no-cache --platform linux/arm64 -t "$DOCKER_IMAGE_QEMU" -f "$dockerfile" "$DOCKER_DIR"
    
    log "QEMU emulation image rebuilt successfully!"
fi

echo ""
log "Docker image rebuild complete!"
info "You can now run: ./scripts/build-and-deploy.sh"
