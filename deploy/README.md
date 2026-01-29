# PicPop Deployment Guide

This directory contains configuration files for deploying PicPop on a Radxa ZERO 3W.

## Hardware Requirements

- Radxa ZERO 3W (2GB RAM)
- 128GB microSD card
- 27" LCD touchscreen with HDMI + USB-HID touch
- Sony Mirrorless camera (USB connected)

## Prerequisites

1. Flash Radxa OS (Debian-based) to microSD card
2. Complete initial setup (WiFi for updates, user account)
3. Connect to the device via SSH or console

## Quick Setup

```bash
# Clone the repository
git clone <repo-url> /tmp/picpop
cd /tmp/picpop

# Run setup script as root
sudo bash deploy/setup.sh

# Reboot
sudo reboot
```

## Manual Setup

### 1. Install Dependencies

```bash
sudo apt update && sudo apt upgrade -y
sudo apt install -y \
    python3 python3-pip python3-venv \
    hostapd dnsmasq \
    gphoto2 libgphoto2-dev \
    libgtk-4-dev libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev \
    gstreamer1.0-plugins-good gstreamer1.0-plugins-bad
```

### 2. Configure WiFi Access Point

Copy the configuration files:

```bash
sudo cp deploy/hostapd.conf /etc/hostapd/hostapd.conf
sudo cp deploy/dnsmasq.conf /etc/dnsmasq.conf
```

Set static IP for wlan0:

```bash
sudo nano /etc/network/interfaces.d/wlan0
```

Add:
```
auto wlan0
iface wlan0 inet static
    address 192.168.4.1
    netmask 255.255.255.0
```

Enable hostapd:

```bash
echo 'DAEMON_CONF="/etc/hostapd/hostapd.conf"' | sudo tee /etc/default/hostapd
sudo systemctl unmask hostapd
sudo systemctl enable hostapd dnsmasq
```

### 3. Install PicPop Server

```bash
# Create directories
sudo mkdir -p /opt/picpop
sudo useradd -m -s /bin/bash picpop
sudo chown -R picpop:picpop /opt/picpop

# Copy files
sudo cp -r backend/* /opt/picpop/

# Setup Python environment
sudo -u picpop python3 -m venv /opt/picpop/.venv
sudo -u picpop /opt/picpop/.venv/bin/pip install -e /opt/picpop
```

### 4. Install Services

```bash
sudo cp deploy/picpop.service /etc/systemd/system/
sudo cp deploy/picpop-kiosk.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable picpop picpop-kiosk
```

### 5. Build Kiosk App

On the Radxa device (or cross-compile):

```bash
cd kiosk-native
cargo build --release
sudo cp target/release/picpop-kiosk /home/kiosk/
sudo chown kiosk:kiosk /home/kiosk/picpop-kiosk
```

## Configuration Files

| File | Description | Target Location |
|------|-------------|-----------------|
| `hostapd.conf` | WiFi access point config | `/etc/hostapd/hostapd.conf` |
| `dnsmasq.conf` | DHCP & DNS server config | `/etc/dnsmasq.conf` |
| `picpop.service` | Backend server systemd unit | `/etc/systemd/system/` |
| `picpop-kiosk.service` | Kiosk app systemd unit | `/etc/systemd/system/` |
| `setup.sh` | Automated setup script | Run from repo root |

## Network Configuration

- **SSID**: PicPop
- **Password**: photobooth
- **Gateway IP**: 192.168.4.1
- **DHCP Range**: 192.168.4.2 - 192.168.4.200

To change the WiFi name/password, edit:
1. `deploy/hostapd.conf` - ssid and wpa_passphrase
2. `backend/app/core/config.py` - wifi_ssid and wifi_password

## Troubleshooting

### WiFi not starting

```bash
# Check hostapd status
sudo systemctl status hostapd
sudo journalctl -u hostapd -f

# Check if wlan0 is available
ip link show wlan0
```

### Camera not detected

```bash
# List cameras
gphoto2 --auto-detect

# Check permissions
ls -la /dev/bus/usb/*/*

# Ensure user is in plugdev group
groups picpop
```

### Server not starting

```bash
# Check server status
sudo systemctl status picpop
sudo journalctl -u picpop -f

# Test manually
cd /opt/picpop
source .venv/bin/activate
uvicorn app.main:app --host 0.0.0.0 --port 8000
```

## Updating

```bash
cd /tmp
git clone <repo-url> picpop-update
cd picpop-update

# Update server
sudo cp -r backend/* /opt/picpop/
sudo -u picpop /opt/picpop/.venv/bin/pip install -e /opt/picpop

# Rebuild kiosk (if changed)
cd kiosk-native && cargo build --release
sudo cp target/release/picpop-kiosk /home/kiosk/

# Restart services
sudo systemctl restart picpop picpop-kiosk
```
