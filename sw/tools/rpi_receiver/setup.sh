#!/bin/bash

# RPi5 Data Logger Setup Script
# This script sets up the data logger as a systemd service on RPi5

set -e

echo "=========================================="
echo "RPi5 Data Logger Service Setup"
echo "=========================================="

# Check if running on RPi
if [ ! -f /boot/firmware/config.txt ] && [ ! -f /boot/config.txt ]; then
    echo "Warning: This doesn't appear to be a Raspberry Pi"
    read -p "Continue anyway? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo "Error: This script must be run as root (use: sudo bash setup.sh)"
    exit 1
fi

# Detect the actual user (not root)
if [ -n "$SUDO_USER" ]; then
    ACTUAL_USER="$SUDO_USER"
else
    ACTUAL_USER=$(logname 2>/dev/null || echo "hardy")
fi
echo "Setting up for user: $ACTUAL_USER"

# Get GPIO pin number
GPIO_PIN=${1:-11}
echo "Using GPIO pin: $GPIO_PIN"

# Step 1: Update system
echo ""
echo "Step 1: Updating system packages..."
apt-get update
apt-get upgrade -y

# Step 2: Install dependencies
echo ""
echo "Step 2: Installing dependencies..."
apt-get install -y \
    python3-pip \
    python3-dev \
    python3-venv \
    python3-full \
    gpiod \
    python3-libgpiod \
    python3-lgpio

# Step 3: Create service directory
SERVICE_DIR="/opt/rpi_receiver"
echo ""
echo "Step 3: Creating service directory: $SERVICE_DIR"
mkdir -p "$SERVICE_DIR/data"
chown "$ACTUAL_USER:$ACTUAL_USER" "$SERVICE_DIR"
chown "$ACTUAL_USER:$ACTUAL_USER" "$SERVICE_DIR/data"
chmod 755 "$SERVICE_DIR"
chmod 755 "$SERVICE_DIR/data"

# Step 4: Create virtual environment and install Python packages
# Use --system-site-packages to access system lgpio (difficult to build from source)
echo ""
echo "Step 4: Creating virtual environment and installing Python packages..."
rm -rf "$SERVICE_DIR/venv"  # Remove existing venv if any
python3 -m venv --system-site-packages "$SERVICE_DIR/venv"
"$SERVICE_DIR/venv/bin/pip" install --upgrade pip
"$SERVICE_DIR/venv/bin/pip" install pyserial cobs gpiozero
chown -R "$ACTUAL_USER:$ACTUAL_USER" "$SERVICE_DIR/venv"

# Step 5: Copy scripts to service directory
echo ""
echo "Step 5: Copying application scripts..."
if [ -f "main.py" ]; then
    cp main.py "$SERVICE_DIR/"
    chmod 755 "$SERVICE_DIR/main.py"
fi

if [ -f "protocol.py" ]; then
    cp protocol.py "$SERVICE_DIR/"
    chmod 755 "$SERVICE_DIR/protocol.py"
fi

if [ -f "gpio_monitor.py" ]; then
    cp gpio_monitor.py "$SERVICE_DIR/"
    chmod 755 "$SERVICE_DIR/gpio_monitor.py"
fi

chown -R "$ACTUAL_USER:$ACTUAL_USER" "$SERVICE_DIR"

# Step 6: Configure GPIO access
echo ""
echo "Step 6: Configuring GPIO access..."

# Create gpio group if it doesn't exist
if ! getent group gpio > /dev/null 2>&1; then
    echo "Creating gpio group..."
    groupadd gpio
fi

# Add user to gpio group
if ! groups "$ACTUAL_USER" | grep -q "\bgpio\b"; then
    usermod -a -G gpio "$ACTUAL_USER"
    echo "Added $ACTUAL_USER to gpio group"
fi

# Set up udev rules for GPIO access (needed on Ubuntu)
UDEV_RULE="/etc/udev/rules.d/99-gpio.rules"
if [ ! -f "$UDEV_RULE" ]; then
    echo "Creating udev rules for GPIO access..."
    cat > "$UDEV_RULE" << 'EOF'
# Allow gpio group to access GPIO chips
SUBSYSTEM=="gpio", KERNEL=="gpiochip*", GROUP="gpio", MODE="0660"
SUBSYSTEM=="gpio", KERNEL=="gpio*", GROUP="gpio", MODE="0660"
EOF
    udevadm control --reload-rules
    udevadm trigger
    echo "GPIO udev rules installed"
fi

# Step 7: Install systemd service
echo ""
echo "Step 7: Installing systemd service..."
if [ -f "rpi-data-logger.service" ]; then
    cp rpi-data-logger.service /etc/systemd/system/
    chmod 644 /etc/systemd/system/rpi-data-logger.service
    systemctl daemon-reload
    echo "Service installed successfully"
fi

# Step 8: User confirmation before enabling
echo ""
echo "=========================================="
echo "Setup Complete!"
echo "=========================================="
echo ""
echo "Next steps:"
echo "1. Wire your GPIO switch to GPIO pin $GPIO_PIN and GND"
echo "2. Connect your ESP32 bridge to a USB port"
echo "3. Enable the service:"
echo "   sudo systemctl enable rpi-data-logger"
echo "4. Start the service:"
echo "   sudo systemctl start rpi-data-logger"
echo ""
echo "Monitor the service:"
echo "   sudo systemctl status rpi-data-logger"
echo "   sudo journalctl -u rpi-data-logger -f  # Follow logs"
echo ""
echo "To disable the service:"
echo "   sudo systemctl stop rpi-data-logger"
echo "   sudo systemctl disable rpi-data-logger"
echo ""
echo "Data files will be saved to: $SERVICE_DIR/data/"
echo "=========================================="
