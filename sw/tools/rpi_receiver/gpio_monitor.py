#!/usr/bin/env python3
"""
GPIO Monitor for Data Logging Switch

Monitors a GPIO pin connected to a data logging switch. When the switch is
activated (pin goes HIGH), starts the sensor data receiver. When the switch is
deactivated (pin goes LOW), stops the receiver.

This script is intended to run as a systemd service on RPi5.

Configuration:
    GPIO_PIN: The GPIO pin number to monitor (default: 17)
    OUTPUT_DIR: Directory to store CSV files (default: /opt/rpi_receiver/data)

Usage:
    python3 gpio_monitor.py [--gpio PIN] [--output DIR]
"""

import argparse
import subprocess
import sys
import time
import logging
from pathlib import Path
from typing import Optional

try:
    from gpiozero import Button
except ImportError:
    print("Error: gpiozero not installed. Run: pip install gpiozero")
    sys.exit(1)


# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


class DataLoggerMonitor:
    """Monitors GPIO switch and manages data logger process"""

    def __init__(self, gpio_pin: int, output_dir: Path, serial_port: str = '/dev/ttyACM0'):
        self.gpio_pin = gpio_pin
        self.output_dir = Path(output_dir)
        self.serial_port = serial_port
        self.button = None
        self.logger_process: Optional[subprocess.Popen] = None
        self.script_dir = Path(__file__).parent
        self.main_script = self.script_dir / 'main.py'
        self.session_counter = 0  # Counter for log files (no NTP dependency)
        
        # Create output directory if it doesn't exist
        self.output_dir.mkdir(parents=True, exist_ok=True)
        logger.info(f"Output directory: {self.output_dir}")

    def setup_button(self):
        """Initialize GPIO button with pull-down configuration"""
        try:
            # Using pull_down=False means the pin will be pulled down by hardware
            # The button will be pressed (True) when the pin goes HIGH
            self.button = Button(self.gpio_pin, pull_up=False, hold_time=0.1)
            self.button.when_pressed = self.on_switch_pressed
            self.button.when_released = self.on_switch_released
            logger.info(f"GPIO pin {self.gpio_pin} initialized successfully")
        except Exception as e:
            logger.error(f"Failed to initialize GPIO pin {self.gpio_pin}: {e}")
            sys.exit(1)

    def on_switch_pressed(self):
        """Callback when switch is pressed (activated)"""
        logger.info("Data logging switch activated")
        self.start_logger()

    def on_switch_released(self):
        """Callback when switch is released (deactivated)"""
        logger.info("Data logging switch deactivated")
        self.stop_logger()

    def start_logger(self):
        """Start the sensor data receiver"""
        if self.logger_process is not None and self.logger_process.poll() is None:
            logger.warning("Logger already running")
            return

        if not self.main_script.exists():
            logger.error(f"main.py not found at {self.main_script}")
            return

        # Generate output file with sequence number (no system time required)
        self.session_counter += 1
        output_file = self.output_dir / f"sensor_data_{self.session_counter:03d}.csv"

        try:
            logger.info(f"Starting data logger: {self.main_script}")
            logger.info(f"Output file: {output_file}")
            
            self.logger_process = subprocess.Popen(
                [
                    'python3',
                    str(self.main_script),
                    '--port', self.serial_port,
                    '--output', str(output_file),
                    '--verbose'
                ],
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                bufsize=1,  # Line buffering
            )
            logger.info(f"Data logger started (PID: {self.logger_process.pid})")
        except Exception as e:
            logger.error(f"Failed to start data logger: {e}")

    def stop_logger(self):
        """Stop the sensor data receiver"""
        if self.logger_process is None or self.logger_process.poll() is not None:
            logger.warning("Logger not running")
            return

        try:
            logger.info(f"Stopping data logger (PID: {self.logger_process.pid})")
            self.logger_process.terminate()
            
            # Wait for graceful shutdown
            try:
                self.logger_process.wait(timeout=5)
                logger.info("Data logger stopped gracefully")
            except subprocess.TimeoutExpired:
                logger.warning("Logger did not stop gracefully, killing...")
                self.logger_process.kill()
                self.logger_process.wait()
                logger.info("Data logger killed")
                
        except Exception as e:
            logger.error(f"Error stopping data logger: {e}")

    def run(self):
        """Main loop"""
        logger.info("=" * 60)
        logger.info("GPIO Data Logger Monitor")
        logger.info("=" * 60)
        logger.info(f"GPIO Pin: {self.gpio_pin}")
        logger.info(f"Output Directory: {self.output_dir}")
        logger.info(f"Serial Port: {self.serial_port}")
        logger.info("Press Ctrl+C to exit")
        logger.info("=" * 60)
        
        self.setup_button()
        
        try:
            # Keep the script running
            while True:
                time.sleep(0.1)
        except KeyboardInterrupt:
            logger.info("\nShutting down...")
            self.stop_logger()
        finally:
            if self.button:
                self.button.close()
            logger.info("GPIO monitor stopped")


def main():
    parser = argparse.ArgumentParser(
        description="Monitor GPIO switch for data logging control"
    )
    parser.add_argument(
        '--gpio',
        type=int,
        default=26,
        help='GPIO pin number to monitor (default: 26)'
    )
    parser.add_argument(
        '--output',
        type=Path,
        default=Path('/opt/rpi_receiver/data'),
        help='Output directory for CSV files (default: /opt/rpi_receiver/data)'
    )
    parser.add_argument(
        '--port',
        default='/dev/ttyACM0',
        help='Serial port for ESP32 bridge (default: /dev/ttyACM0)'
    )
    
    args = parser.parse_args()
    
    monitor = DataLoggerMonitor(
        gpio_pin=args.gpio,
        output_dir=args.output,
        serial_port=args.port
    )
    monitor.run()


if __name__ == '__main__':
    main()
