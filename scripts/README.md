# PicPop Scripts

Build and deployment scripts for the PicPop photo booth system.

## Quick Start

```bash
# Build and deploy everything (cross-compile locally)
./scripts/build-and-deploy.sh

# If cross-compilation fails, use QEMU (slower but reliable)
./scripts/build-and-deploy.sh --qemu

# Build on the Radxa device itself
./scripts/build-and-deploy.sh --on-radxa
```

## Architecture

PicPop has three components:

| Component    | Tech Stack     | Runs On               | Purpose                    |
| ------------ | -------------- | --------------------- | -------------------------- |
| **Backend**  | Python/FastAPI | Radxa                 | API server, camera control |
| **Frontend** | React/Vite     | User phones (browser) | Mobile web app for guests  |
| **Kiosk**    | Rust/GTK4      | Radxa touchscreen     | Native photo booth UI      |

## Script Hierarchy

```
build-and-deploy.sh [--on-radxa] [--qemu]
    |
    +-- build.sh [--on-radxa] [--qemu]
    |       |
    |       +-- build-server.sh      # React frontend (always local)
    |       +-- build-kiosk.sh       # Rust kiosk (cross or on-device)
    |
    +-- deploy.sh [--on-radxa]       # Deploy to Radxa
```

## Scripts

### `build-and-deploy.sh`

**Main entry point.** Builds everything and deploys to the Radxa.

```bash
./scripts/build-and-deploy.sh              # Cross-compile (default)
./scripts/build-and-deploy.sh --qemu       # QEMU fallback
./scripts/build-and-deploy.sh --on-radxa   # Build on device
```

### `build.sh`

Builds all components without deploying.

```bash
./scripts/build.sh              # Cross-compile kiosk
./scripts/build.sh --qemu       # QEMU for kiosk
./scripts/build.sh --on-radxa   # Build kiosk on device
```

### `build-server.sh`

Builds the React frontend with Bun. Always runs locally.

```bash
./scripts/build-server.sh
```

Output: `frontend/dist/`

### `build-kiosk.sh`

Builds the Rust/GTK4 kiosk application.

```bash
./scripts/build-kiosk.sh              # Docker cross-compile (fast)
./scripts/build-kiosk.sh --qemu       # QEMU emulation (slow, reliable)
./scripts/build-kiosk.sh --on-radxa   # SSH to Radxa, build there
```

Output: `kiosk-native/target/aarch64-unknown-linux-gnu/release/picpop-kiosk`

### `deploy.sh`

Deploys built artifacts to the Radxa device.

```bash
./scripts/deploy.sh              # Deploy cross-compiled binary
./scripts/deploy.sh --on-radxa   # Deploy binary built on device
```

## Build Modes

### Cross-compilation (Default)

Uses Docker with Debian multiarch to compile ARM64 binaries on your x86 machine.

- **Speed:** ~5 minutes
- **Reliability:** May fail due to package conflicts

### QEMU (`--qemu`)

Uses QEMU user-mode emulation to run an ARM64 container.

- **Speed:** ~30+ minutes
- **Reliability:** Very reliable fallback

### On-device (`--on-radxa`)

SSHs to the Radxa and builds natively using cargo.

- **Speed:** ~10-15 minutes (depends on device)
- **Reliability:** Most reliable, requires Rust on device

## Configuration

Set environment variables to customize:

```bash
# Target device (default: kiosk@192.168.0.110)
export RADXA_HOST="user@hostname"

# Then run scripts
./scripts/build-and-deploy.sh
```

## Directory Structure

```
scripts/
├── build-and-deploy.sh   # Main entry point
├── build.sh              # Build all components
├── build-server.sh       # Build React frontend
├── build-kiosk.sh        # Build Rust kiosk
├── deploy.sh             # Deploy to Radxa
├── common.sh             # Shared utilities
├── README.md             # This file
└── docker/
    ├── Dockerfile.cross  # Multiarch cross-compilation
    └── Dockerfile.qemu   # QEMU emulation
```

## Troubleshooting

### Cross-compilation fails with pkg-config errors

Use QEMU mode: `./scripts/build-kiosk.sh --qemu`

### Can't connect to Radxa

1. Check SSH: `ssh kiosk@192.168.0.110`
2. Verify IP address in `common.sh` or set `RADXA_HOST`

### Docker not found

Install Docker or Podman. Scripts auto-detect which is available.

### Rebuild Docker images

```bash
docker build --no-cache -t picpop-cross-aarch64 -f scripts/docker/Dockerfile.cross scripts/docker/
docker build --no-cache --platform linux/arm64 -t picpop-cross-aarch64-qemu -f scripts/docker/Dockerfile.qemu scripts/docker/
```
