#!/bin/bash
# PicPop Scripts - Shared Configuration and Utilities
# Source this file in other scripts: source "$(dirname "$0")/common.sh"

set -e

# =============================================================================
# Configuration
# =============================================================================

# Target device (override with environment variable)
export RADXA_HOST="${RADXA_HOST:-picpop@192.168.0.110}"
export REMOTE_PATH="/opt/picpop"

# Build targets
export KIOSK_TARGET="aarch64-unknown-linux-gnu"

# Docker images
export DOCKER_IMAGE_CROSS="picpop-cross-aarch64"
export DOCKER_IMAGE_QEMU="picpop-cross-aarch64-qemu"

# =============================================================================
# Paths (auto-detected)
# =============================================================================

# Get the directory containing this script
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export SCRIPT_DIR

# Project root is parent of scripts/
export PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Component directories
export FRONTEND_DIR="$PROJECT_ROOT/frontend"
export BACKEND_DIR="$PROJECT_ROOT/backend"
export KIOSK_DIR="$PROJECT_ROOT/kiosk-native"
export DEPLOY_DIR="$PROJECT_ROOT/deploy"
export DOCKER_DIR="$SCRIPT_DIR/docker"

# Build outputs
export KIOSK_OUTPUT_LOCAL="$KIOSK_DIR/target/release/picpop-kiosk"
export KIOSK_OUTPUT_CROSS="$KIOSK_DIR/target/$KIOSK_TARGET/release/picpop-kiosk"

# =============================================================================
# Colors and Logging
# =============================================================================

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m' # No Color

log()   { echo -e "${GREEN}[+]${NC} $1"; }
info()  { echo -e "${BLUE}[i]${NC} $1"; }
warn()  { echo -e "${YELLOW}[!]${NC} $1"; }
error() { echo -e "${RED}[x]${NC} $1"; exit 1; }
header() { echo -e "\n${BOLD}=== $1 ===${NC}\n"; }

# =============================================================================
# Utility Functions
# =============================================================================

# Check SSH connection to Radxa
check_ssh() {
    log "Testing SSH connection to $RADXA_HOST..."
    ssh -o ConnectTimeout=5 -o BatchMode=yes "$RADXA_HOST" "echo 'SSH OK'" >/dev/null 2>&1\
        || error "Cannot connect to $RADXA_HOST. Check SSH config and device availability."
}

# Get container tool (docker or podman)
get_container_tool() {
    if command -v docker &>/dev/null; then
        echo "docker"
    elif command -v podman &>/dev/null; then
        echo "podman"
    else
        error "Please install Docker or Podman for cross-compilation"
    fi
}

# Check if a command exists
require_cmd() {
    command -v "$1" &>/dev/null || error "Required command not found: $1"
}

# Parse common flags from arguments
# Sets: ON_RADXA, USE_QEMU, SHOW_HELP
parse_common_flags() {
    ON_RADXA=false
    USE_QEMU=false
    SHOW_HELP=false
    
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --on-radxa)
                ON_RADXA=true
                shift
                ;;
            --qemu)
                USE_QEMU=true
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
    
    export ON_RADXA USE_QEMU SHOW_HELP
}
