#!/bin/bash
# PicPop Deployment Setup Script for Radxa ZERO 3W
# Run as root: sudo bash setup.sh

set -e

echo "=== PicPop Deployment Setup ==="
echo ""

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo "Please run as root: sudo bash setup.sh"
    exit 1
fi

# Update system
echo "Updating system packages..."
apt update && apt upgrade -y

# Install dependencies
echo "Installing dependencies..."
apt install -y \
    python3 python3-pip python3-venv \
    hostapd dnsmasq \
    gphoto2 libgphoto2-dev \
    libwebkit2gtk-4.1-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    patchelf

# Create picpop user
echo "Creating picpop user..."
if ! id "picpop" &>/dev/null; then
    useradd -m -s /bin/bash picpop
    usermod -aG video,plugdev,dialout picpop
fi

# Create installation directory
echo "Creating installation directories..."
mkdir -p /opt/picpop
mkdir -p /opt/picpop/data/photos
mkdir -p /opt/picpop/kiosk

# Set up Python virtual environment
echo "Setting up Python environment..."
python3 -m venv /opt/picpop/.venv
source /opt/picpop/.venv/bin/activate
pip install --upgrade pip

# Copy server files (assumes they're in current directory)
if [ -d "./server" ]; then
    echo "Copying server files..."
    cp -r ./server/* /opt/picpop/
    pip install -e /opt/picpop
fi

# Configure network interface
echo "Configuring network interface..."
cat > /etc/network/interfaces.d/wlan0 << EOF
auto wlan0
iface wlan0 inet static
    address 192.168.4.1
    netmask 255.255.255.0
EOF

# Stop and disable default network management for wlan0
systemctl stop NetworkManager 2>/dev/null || true
nmcli device set wlan0 managed no 2>/dev/null || true

# Install configuration files
echo "Installing configuration files..."
cp ./deploy/hostapd.conf /etc/hostapd/hostapd.conf
cp ./deploy/dnsmasq.conf /etc/dnsmasq.conf
cp ./deploy/picpop.service /etc/systemd/system/picpop.service
cp ./deploy/picpop-kiosk.service /etc/systemd/system/picpop-kiosk.service

# Enable hostapd config
echo 'DAEMON_CONF="/etc/hostapd/hostapd.conf"' > /etc/default/hostapd

# Set permissions
echo "Setting permissions..."
chown -R picpop:picpop /opt/picpop

# Set up udev rules for camera
echo "Setting up camera udev rules..."
cat > /etc/udev/rules.d/99-gphoto2.rules << 'EOF'
# Allow picpop user to access cameras
SUBSYSTEM=="usb", ENV{ID_GPHOTO2}=="1", GROUP="plugdev", MODE="0660"
EOF
udevadm control --reload-rules

# Enable and start services
echo "Enabling services..."
systemctl daemon-reload
systemctl unmask hostapd
systemctl enable hostapd
systemctl enable dnsmasq
systemctl enable picpop
systemctl enable picpop-kiosk

# Start network services
echo "Starting network services..."
ifdown wlan0 2>/dev/null || true
ifup wlan0
systemctl start hostapd
systemctl start dnsmasq

echo ""
echo "=== Setup Complete ==="
echo ""
echo "To start PicPop:"
echo "  systemctl start picpop"
echo "  systemctl start picpop-kiosk"
echo ""
echo "To check status:"
echo "  systemctl status picpop"
echo "  systemctl status picpop-kiosk"
echo ""
echo "WiFi Network: PicPop"
echo "WiFi Password: photobooth"
echo "Server URL: http://192.168.4.1:8000"
echo ""
echo "Reboot recommended: sudo reboot"
