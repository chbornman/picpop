#!/bin/bash
set -e

RADXA_HOST="${RADXA_HOST:-kiosk@192.168.0.110}"

GREEN='\033[0;32m'
NC='\033[0m'

log() { echo -e "${GREEN}[+]${NC} $1"; }

log "Restarting services on $RADXA_HOST..."

ssh "$RADXA_HOST" "sudo systemctl restart picpop picpop-kiosk"

log "Services restarted"
ssh "$RADXA_HOST" "sudo systemctl status picpop picpop-kiosk --no-pager -n 0"
