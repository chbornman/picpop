#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Colors
GREEN='\033[0;32m'
NC='\033[0m'

log() { echo -e "${GREEN}[+]${NC} $1"; }

cd "$PROJECT_ROOT"

log "Building frontend..."
cd frontend
bun install
bun run build

log "Frontend built to frontend/dist/"
ls -lh dist/

log "Build complete! Ready to deploy with ./scripts/deploy-server.sh"
