#!/bin/bash
# Build the PicPop server (FastAPI backend + React frontend bundle)
#
# This script:
#   1. Builds the React frontend with Bun (outputs to frontend/dist/)
#   2. The backend (Python/FastAPI) needs no compilation
#
# The built frontend is served by FastAPI as static files.
#
# Usage: ./scripts/build-server.sh

source "$(dirname "$0")/common.sh"

parse_common_flags "$@"

if $SHOW_HELP; then
    cat << 'EOF'
Build the PicPop server (FastAPI backend + React frontend)

Usage: ./scripts/build-server.sh [options]

Options:
    -h, --help      Show this help message

This script builds the React frontend for mobile phones using Bun.
The Python backend requires no compilation.

Output:
    frontend/dist/   Built React app (served by FastAPI)
EOF
    exit 0
fi

header "Building PicPop Server"

# Check for bun
require_cmd bun

# Build frontend
log "Building React frontend..."
cd "$FRONTEND_DIR"

log "Installing dependencies..."
bun install

log "Running production build..."
bun run build

# Verify output
if [[ -d "$FRONTEND_DIR/dist" ]]; then
    log "Frontend built successfully!"
    info "Output: $FRONTEND_DIR/dist/"
    ls -lh "$FRONTEND_DIR/dist/" | head -10
else
    error "Frontend build failed - dist/ directory not found"
fi

echo ""
log "Server build complete!"
info "Backend (Python) requires no compilation - will be deployed as source"
