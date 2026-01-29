# PicPop - Offline Photo Booth

A fully offline-capable interactive photo booth system designed for the **Radxa ZERO 3W** with a touchscreen kiosk interface and smartphone integration via WiFi captive portal.

## Hardware

| Component   | Specification                                |
| ----------- | -------------------------------------------- |
| **SBC**     | Radxa ZERO 3W (2GB RAM, ARM64)               |
| **Storage** | 128GB microSD (A2 class)                     |
| **Display** | 27" LCD with HDMI + USB-HID touch            |
| **Camera**  | Sony Mirrorless (USB connection via gphoto2) |

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           RADXA ZERO 3W                                     │
│                                                                             │
│  ┌──────────────────┐    ┌──────────────────┐    ┌────────────────────┐    │
│  │   GTK4 Kiosk     │    │  FastAPI Server  │    │   WiFi Hotspot     │    │
│  │   (Touchscreen)  │◄──►│   (Port 8000)    │◄──►│   + Captive Portal │    │
│  │                  │ WS │                  │    │   (hostapd/dnsmasq)│    │
│  │  - QR Display    │    │  - Sessions      │    │                    │    │
│  │  - Capture Btn   │    │  - WebSockets    │    │  SSID: PicPop      │    │
│  │  - Status View   │    │  - Photos API    │    │  192.168.4.1       │    │
│  └────────┬─────────┘    │  - gphoto2       │    └────────────────────┘    │
│           │              └────────┬─────────┘                               │
│           │                       │                                         │
│           │              ┌────────▼─────────┐                               │
│           │              │   Sony Camera    │                               │
│           └──────────────┤   (USB/gphoto2)  │                               │
│                          └──────────────────┘                               │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    │ WiFi (192.168.4.x)
                                    ▼
                    ┌───────────────────────────────┐
                    │        User's Phone           │
                    │                               │
                    │  1. Scan QR → Join WiFi       │
                    │  2. Captive portal redirects  │
                    │  3. WebSocket connects        │
                    │  4. Photos stream in realtime │
                    │  5. Long-press to save        │
                    └───────────────────────────────┘
```

## User Flow

### 1. Session Start

- Touchscreen displays **QR code** (encodes WiFi credentials + session URL)
- User scans QR with phone camera
- Phone auto-connects to `PicPop` WiFi hotspot

### 2. Captive Portal Connection

- Phone triggers captive portal detection (fake Apple/Google response)
- Initial captive portal page shows "Open Photo Booth" button
- Button opens the full web app in the phone's browser
- WebSocket connection established between phone and server

### 3. Photo Capture

- User taps **"Take Photos"** button on the **touchscreen**
- For each photo (default 3):
  - 3-2-1 countdown displayed on both screens
  - Camera captures photo
  - Processing starts immediately in background
- "Processing..." shown after final capture while images finalize
- Photos stream to phone via WebSocket as they're ready

### 4. Photo Retrieval

- Photos appear in phone's web interface in real-time
- User **long-presses** to save photos to camera roll
- Note: No HTTPS in offline mode = no Share Sheet (iOS limitation)

### 5. Session Options

- **"Take More"** button on touchscreen for additional photos
- **"End Session"** clears photos and returns to QR screen

## Project Structure

```
picpop/
├── kiosk-native/               # Native GTK4 touchscreen app
│   ├── src/                    # Rust application
│   │   ├── main.rs             # Entry point
│   │   ├── ui/                 # GTK4 widgets
│   │   │   ├── window.rs       # Main window
│   │   │   ├── welcome.rs      # QR code display
│   │   │   └── session.rs      # Photo session UI
│   │   ├── api/                # HTTP/WebSocket clients
│   │   └── video/              # GStreamer video pipeline
│   └── Cargo.toml
│
├── backend/                     # FastAPI backend
│   ├── app/
│   │   ├── main.py             # FastAPI app + lifespan
│   │   ├── api/
│   │   │   └── v1/
│   │   │       ├── sessions.py # Session management
│   │   │       ├── photos.py   # Photo endpoints
│   │   │       └── ws.py       # WebSocket handlers
│   │   ├── services/
│   │   │   ├── camera.py       # gphoto2 integration
│   │   │   ├── captive.py      # Captive portal responses
│   │   │   ├── qr.py           # QR code generation
│   │   │   └── websocket.py    # WebSocket manager
│   │   ├── models/
│   │   └── core/
│   │       └── config.py
│   └── pyproject.toml
│
├── frontend/                     # Phone web app (served by FastAPI)
│   ├── src/
│   │   ├── App.tsx
│   │   ├── components/
│   │   │   ├── WaitingScreen.tsx
│   │   │   ├── PhotoStream.tsx
│   │   │   └── PhotoViewer.tsx
│   │   └── hooks/
│   │       └── useWebSocket.ts
│   ├── index.html
│   └── vite.config.ts
│
├── system/                     # System configuration
│   ├── hostapd.conf            # WiFi hotspot config
│   ├── dnsmasq.conf            # DNS + DHCP for captive portal
│   ├── picpop.service          # Systemd service
│   └── setup.sh                # Initial system setup script
│
├── packages/                   # Shared code (monorepo)
│   └── shared/
│       └── src/
│           └── types.ts        # Shared TypeScript types
│
└── README.md
```

## Key Technologies

### Captive Portal Stack

- **hostapd** - WiFi access point daemon
- **dnsmasq** - DNS server (redirects all domains to 192.168.4.1)
- **Captive portal detection** endpoints:
  - `/generate_204` (Android)
  - `/hotspot-detect.html` (Apple)
  - `/connecttest.txt` (Windows)
  - `/ncsi.txt` (Windows)

### Real-time Communication

- **WebSockets** for bidirectional communication
  - Kiosk ↔ Server: Capture triggers, status updates
  - Phone ↔ Server: Photo streaming, session sync
- Session-based rooms (multiple phones can view same session)

### Camera Integration

- **gphoto2** via `python-gphoto2` library
- Supports Sony mirrorless cameras via USB PTP protocol
- Optimized capture pipeline:
  - Countdown → Capture → Background Process (parallel) → Stream
  - Image processing runs in thread pool (4 workers) for multi-core utilization
  - Processing overlaps with next photo's countdown for faster capture sequences

### Kiosk Display

- **Native GTK4** application with GStreamer
- Fullscreen, no window decorations
- Touch-optimized UI with large buttons
- Connects to local FastAPI server via WebSocket

## WebSocket Protocol

### Messages from Server to Clients

```typescript
// Session created, QR ready
{ type: "session_ready", sessionId: string, qrUrl: string }

// Phone connected to session
{ type: "phone_connected", phoneId: string }

// Countdown tick (sent to all clients)
{ type: "countdown", value: number, photoNumber: number, totalPhotos: number }

// Photo captured and available
{ type: "photo_ready", photo: { id: string, url: string, thumbnailUrl: string } }

// Photos being processed (shown after last capture)
{ type: "processing", photoCount: number }

// Capture sequence complete
{ type: "capture_complete", photoCount: number }

// Session ended
{ type: "session_ended" }
```

### Messages from Kiosk to Server

```typescript
// Request new session
{
  type: 'new_session';
}

// Trigger photo capture (3-photo sequence)
{
  type: 'start_capture';
}

// End current session
{
  type: 'end_session';
}
```

### Messages from Phone to Server

```typescript
// Join existing session
{ type: "join_session", sessionId: string }

// Request photo download
{ type: "download_photo", photoId: string }
```

## API Endpoints

### Sessions

| Method | Endpoint                        | Description                  |
| ------ | ------------------------------- | ---------------------------- |
| POST   | `/api/v1/sessions`              | Create new session           |
| GET    | `/api/v1/sessions/{id}`         | Get session details          |
| POST   | `/api/v1/sessions/{id}/capture` | Trigger 3-photo capture      |
| POST   | `/api/v1/sessions/{id}/end`     | End session                  |
| GET    | `/api/v1/sessions/{id}/qr`      | Get WiFi+Session QR code PNG |

### Photos

| Method | Endpoint                       | Description              |
| ------ | ------------------------------ | ------------------------ |
| GET    | `/api/v1/sessions/{id}/photos` | List session photos      |
| GET    | `/api/v1/photos/{id}`          | Get photo metadata       |
| GET    | `/api/v1/photos/{id}/download` | Download full resolution |

### WebSocket

| Endpoint                | Description                  |
| ----------------------- | ---------------------------- |
| `/ws/kiosk`             | Kiosk touchscreen connection |
| `/ws/phone/{sessionId}` | Phone client connection      |

### Captive Portal

| Endpoint               | Description                  |
| ---------------------- | ---------------------------- |
| `/generate_204`        | Android captive portal check |
| `/hotspot-detect.html` | Apple captive portal check   |
| `/connecttest.txt`     | Windows captive portal check |
| `/portal`              | Captive portal landing page  |

## QR Code Content

The QR code encodes a WiFi configuration URI that works on both iOS and Android:

```
WIFI:T:WPA;S:PicPop;P:photobooth;;
```

Combined with the session URL in a special format that triggers both WiFi connection and browser opening:

```
picpop://connect?ssid=PicPop&pass=photobooth&url=http://192.168.4.1/session/{sessionId}
```

For maximum compatibility, we display a two-step QR:

1. First QR: WiFi credentials (WIFI: format)
2. After connection: Session URL auto-redirects via captive portal

## Development

### Prerequisites

- Rust (for native kiosk)
- Node.js 20+ / Bun 1.x
- Python 3.11+
- GTK4 and GStreamer development libraries
- gphoto2 libraries (`libgphoto2-dev`)

### Setup

```bash
# Install dependencies
bun install

# Backend
cd backend
uv sync
uv run uvicorn app.main:app --reload

# Kiosk (development - requires GTK4/GStreamer)
cd kiosk-native
cargo run

# Frontend (development)
cd frontend
bun run dev
```

### Build for Radxa

```bash
# Build kiosk (native GTK4 app)
cd kiosk-native
cargo build --release

# Build frontend (phone web app)
cd frontend
bun run build
```

## Deployment on Radxa ZERO 3W

### Deployed File Locations

| Component            | Source                                     | Deployed Location   |
| -------------------- | ------------------------------------------ | ------------------- |
| Backend (Python)     | `backend/`                                 | `/opt/picpop/`      |
| Kiosk (GTK4 binary)  | `kiosk-native/target/release/picpop-kiosk` | `/home/kiosk/`      |
| Frontend (phone web) | `frontend/dist/`                           | `/opt/mobile/dist/` |

### Initial Setup

#### 1. Flash Radxa OS

- Use Radxa Debian/Ubuntu image
- Enable WiFi driver

#### 2. Run Setup Script

```bash
sudo ./deploy/setup.sh
```

This script:

- Installs dependencies (hostapd, dnsmasq, gphoto2)
- Configures WiFi hotspot
- Sets up captive portal DNS
- Creates the `picpop` user
- Installs systemd services
- Enables auto-start on boot

#### 3. Install Systemd Services

```bash
# Copy service files
sudo cp deploy/picpop.service /etc/systemd/system/
sudo cp deploy/picpop-kiosk.service /etc/systemd/system/

# If running kiosk as your user (not picpop), edit the service:
sudo sed -i 's/User=picpop/User=kiosk/' /etc/systemd/system/picpop-kiosk.service
sudo sed -i 's/Group=picpop/Group=kiosk/' /etc/systemd/system/picpop-kiosk.service

# Reload and enable
sudo systemctl daemon-reload
sudo systemctl enable picpop picpop-kiosk
```

#### 4. Initial Deployment

```bash
# Copy backend
sudo cp -r backend/* /opt/picpop/
sudo chown -R picpop:picpop /opt/picpop

# Build and copy kiosk (or use scripts/build-native-kiosk.sh)
cd kiosk-native && cargo build --release
sudo cp target/release/picpop-kiosk /home/kiosk/

# Build and copy frontend
cd ../frontend && bun run build
sudo mkdir -p /opt/mobile/dist
sudo cp -r dist/* /opt/mobile/dist/

# Start services
sudo systemctl start picpop picpop-kiosk
```

#### 5. Configure Display

- HDMI output for 27" display
- USB touch input auto-detected
- Kiosk app runs fullscreen on boot

### Deploy Script

After making changes, use the deploy script to rebuild and restart services:

```bash
# Deploy just backend (fast - no build needed)
./deploy.sh backend

# Deploy frontend (builds then deploys)
./deploy.sh frontend

# Deploy kiosk (builds native GTK4 app)
./deploy.sh kiosk

# Deploy everything
./deploy.sh all

# Check service status
./deploy.sh status
```

### Service Management

```bash
# Check status
systemctl status picpop picpop-kiosk

# View logs
journalctl -u picpop -f        # Backend logs
journalctl -u picpop-kiosk -f  # Kiosk logs

# Restart services
sudo systemctl restart picpop
sudo systemctl restart picpop-kiosk

# Stop services
sudo systemctl stop picpop picpop-kiosk
```

## Offline Considerations

### No HTTPS = No Share Sheet

iOS requires HTTPS for the Web Share API. In offline mode without valid certificates:

- Users must **long-press** images to save
- "Save Image" option works without HTTPS
- Share Sheet (AirDrop, Messages, etc.) unavailable

### Potential Solutions (Future)

1. **mkcert** - Generate local CA, install on phones (complex)
2. **Native app** - Build iOS/Android app with camera roll access
3. **Email option** - If WiFi has internet, email photos

### DNS Resolution

- All DNS queries resolve to `192.168.4.1`
- Prevents phones from detecting "no internet"
- Keeps captive portal active for redirects

## Photo Storage

```
/var/lib/picpop/
├── photos/
│   └── {session_id}/
│       ├── photo_001_web.jpg      # 1920px max width
│       ├── photo_001_thumb.jpg    # 400px thumbnail
│       ├── photo_002_web.jpg
│       ├── photo_002_thumb.jpg
│       └── ...
└── picpop.db                      # SQLite database
```

- Photos auto-delete after session ends or expires (configurable)
- Default session expiry: 60 minutes
- Storage cleanup runs on server startup

## Credits

This project combines and improves upon:

- **picpop_og** - Original kiosk app architecture, offline upload queue design
- **picpop_simple** - gphoto2 camera integration, FastAPI backend, React state machine

## License

MIT
