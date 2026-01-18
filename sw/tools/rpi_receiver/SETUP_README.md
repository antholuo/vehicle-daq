# RPi5 Data Logger with GPIO Switch Control

This setup allows your Raspberry Pi 5 to automatically start/stop data logging when an external GPIO switch is activated.

## Hardware Requirements

- **Raspberry Pi 5** with Raspberry Pi OS (Bullseye or later)
- **External Switch** (normally open/momentary or maintained contact)
- **GPIO Cable** to connect switch to RPi GPIO pin
- **Pull-down Resistor** (10kΩ recommended) to pull GPIO pin to GND when switch is open
- **USB Cable** connecting ESP32 bridge to RPi USB port

## Hardware Wiring

```
    5V (Optional, for switch LED indicator)
     |
    [LED] --> [Resistor 220Ω] --> Switch --> GND
                                    |
                                   GPIO PIN (default: 26)
                                    |
                                  [Resistor 10kΩ to GND]
```

**Simplified (no LED):**
```
Switch pin 1 --> GPIO pin (default: 26)
Switch pin 2 --> GND
```

The 10kΩ pull-down resistor ensures the GPIO pin reads LOW when the switch is open.

## File Structure

- `main.py` - Main sensor data receiver
- `protocol.py` - Protocol definitions for message parsing
- `gpio_monitor.py` - GPIO monitoring script (auto-starts main.py on switch activation)
- `rpi-data-logger.service` - Systemd service file
- `setup.sh` - Installation script
- `requirements.txt` - Python dependencies

## Installation Steps

### 1. Prepare Your Files

Copy all files from your development machine to RPi5:

```bash
scp -r /path/to/rpi_receiver pi@<rpi-ip>:~/rpi_receiver
ssh pi@<rpi-ip>
cd ~/rpi_receiver
```

### 2. Run Setup Script (as root)

```bash
sudo bash setup.sh
```

This script will:
- Update system packages
- Install Python and system dependencies
- Install required Python packages (pyserial, cobs, gpiozero)
- Create `/opt/rpi_receiver` service directory
- Copy application files
- Configure GPIO access
- Install systemd service

### 3. Optional: Configure GPIO Pin

If you're using a different GPIO pin (not 26), edit the systemd service:

```bash
sudo nano /etc/systemd/system/rpi-data-logger.service
```

Modify the ExecStart line to include your GPIO pin:

```ini
ExecStart=/usr/bin/python3 /opt/rpi_receiver/gpio_monitor.py --gpio 27
```

Then reload the service:

```bash
sudo systemctl daemon-reload
```

### 4. Enable and Start Service

```bash
# Enable service to auto-start on boot
sudo systemctl enable rpi-data-logger

# Start the service now
sudo systemctl start rpi-data-logger

# Verify it's running
sudo systemctl status rpi-data-logger
```

## Usage

### Normal Operation

1. Connect your ESP32 bridge via USB to RPi5
2. Boot up the RPi5
3. The systemd service will start automatically on boot
4. When you flip the data logging switch ON, logging begins
5. When you flip the switch OFF, logging stops
6. CSV files are saved to `/opt/rpi_receiver/data/`

### Monitoring the Service

View current status:
```bash
sudo systemctl status rpi-data-logger
```

Follow live logs:
```bash
sudo journalctl -u rpi-data-logger -f
```

View recent logs:
```bash
sudo journalctl -u rpi-data-logger -n 100
```

### Manual Control

If you need to manually control the service:

```bash
# Stop the service
sudo systemctl stop rpi-data-logger

# Start the service
sudo systemctl start rpi-data-logger

# Restart the service
sudo systemctl restart rpi-data-logger

# Disable auto-start (service still runs if started manually)
sudo systemctl disable rpi-data-logger
```

## Data Storage

CSV files are saved with sequential numbering:
- Location: `/opt/rpi_receiver/data/`
- Naming: `sensor_data_001.csv`, `sensor_data_002.csv`, etc.
- A new file is created each time the switch is activated
- **Note**: Sequential numbering doesn't require WiFi/NTP, works even without system clock accuracy

Access logs from other machine:
```bash
scp -r pi@<rpi-ip>:/opt/rpi_receiver/data ./downloaded_data
```

## Troubleshooting

### Service won't start

Check logs:
```bash
sudo journalctl -u rpi-data-logger -n 50
```

Common issues:
- Missing Python packages: `pip3 install pyserial cobs gpiozero`
- Permission denied: Check that pi user owns `/opt/rpi_receiver`
- GPIO already in use: Another process is using the GPIO

### GPIO not responding

Verify GPIO connection:
```bash
# List GPIO status (requires gpiod)
gpioinfo
```

Test GPIO pin manually (e.g., pin 26):
```bash
# Read GPIO value
gpioget gpiochip0 26

# Watch GPIO for changes
gpiowatch gpiochip0 26
```

### Serial port not connecting

Check if ESP32 is recognized:
```bash
lsusb
ls -la /dev/ttyACM*
ls -la /dev/ttyUSB*
```

Verify permissions:
```bash
groups pi  # Should include "dialout"
```

If missing, add pi to dialout group:
```bash
sudo usermod -a -G dialout pi
```

Then restart service (user must log out and back in).

### Data not being logged

1. Check if switch is activating the GPIO:
   ```bash
   sudo gpiowatch gpiochip0 26
   ```

2. Check main.py can access the serial port:
   ```bash
   python3 /opt/rpi_receiver/main.py --port /dev/ttyACM0
   ```

3. Verify /dev/ttyACM0 is the correct port:
   ```bash
   dmesg | tail  # Look for USB device messages
   ```

## Configuration Options

### Change GPIO pin

Edit gpio_monitor.py or pass as command-line argument:

```bash
/usr/bin/python3 /opt/rpi_receiver/gpio_monitor.py --gpio 27
```

### Change output directory

Edit gpio_monitor.py or modify in systemd service

### Change serial port

If your ESP32 uses a different port (e.g., /dev/ttyUSB0):

```bash
/usr/bin/python3 /opt/rpi_receiver/gpio_monitor.py --port /dev/ttyUSB0
```

## Uninstall

To completely remove the service:

```bash
# Stop and disable service
sudo systemctl stop rpi-data-logger
sudo systemctl disable rpi-data-logger

# Remove service file
sudo rm /etc/systemd/system/rpi-data-logger.service
sudo systemctl daemon-reload

# Remove application directory (optional)
sudo rm -rf /opt/rpi_receiver
```

## Additional Resources

- [Raspberry Pi GPIO Documentation](https://www.raspberrypi.com/documentation/computers/raspberry-pi.html)
- [gpiozero Documentation](https://gpiozero.readthedocs.io/)
- [systemd Service Documentation](https://www.freedesktop.org/wiki/Software/systemd/)
