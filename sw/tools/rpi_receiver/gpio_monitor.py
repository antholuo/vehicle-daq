#!/usr/bin/env python3
"""
GPIO Monitor for Data Logging Switch

Monitors a GPIO pin connected to a data logging switch. When the switch is
activated (pin goes HIGH), starts the sensor data receiver. When the switch is
deactivated (pin goes LOW), stops the receiver.

This script uses python3-libgpiod (libgpiod) directly to avoid gpiozero/lgpio
backend issues on Ubuntu for Raspberry Pi 5.
"""

import argparse
import logging
import subprocess
import sys
import time
from pathlib import Path
from typing import Optional, Tuple

try:
    import gpiod
except ImportError:
    print("Error: python3-libgpiod not installed. Run: sudo apt install python3-libgpiod")
    sys.exit(1)


# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(levelname)s - %(message)s",
)
logger = logging.getLogger(__name__)


def _resolve_chip_path(gpio_pin: int, chip: Optional[str]) -> str:
    """Resolve a gpiochip path from optional chip input or gpioinfo output."""
    if chip:
        if chip.startswith("/dev/"):
            return chip
        return f"/dev/gpiochip{chip}"

    try:
        result = subprocess.run(
            ["gpioinfo"],
            check=True,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        current_chip = None
        for line in result.stdout.splitlines():
            line = line.strip()
            if line.startswith("gpiochip"):
                current_chip = line.split()[0]
                continue
            if current_chip and f"GPIO{gpio_pin}" in line:
                return f"/dev/{current_chip}"
    except Exception:
        pass

    return "/dev/gpiochip0"


def _get_v2_line_settings():
    if hasattr(gpiod, "LineSettings"):
        settings_cls = gpiod.LineSettings
        direction_enum = getattr(gpiod, "LineDirection", None)
        edge_enum = getattr(gpiod, "LineEdge", None)
        bias_enum = getattr(gpiod, "LineBias", None)
        if direction_enum is None and hasattr(gpiod, "line"):
            direction_enum = getattr(gpiod.line, "Direction", None)
            edge_enum = getattr(gpiod.line, "Edge", None)
            bias_enum = getattr(gpiod.line, "Bias", None)
        return settings_cls, direction_enum, edge_enum, bias_enum
    if hasattr(gpiod, "line") and hasattr(gpiod.line, "LineSettings"):
        return (
            gpiod.line.LineSettings,
            getattr(gpiod.line, "Direction", None),
            getattr(gpiod.line, "Edge", None),
            getattr(gpiod.line, "Bias", None),
        )
    return None


def _get_line_event_enum():
    if hasattr(gpiod, "EdgeEvent") and hasattr(gpiod.EdgeEvent, "Type"):
        return gpiod.EdgeEvent.Type
    if hasattr(gpiod, "LineEvent"):
        return gpiod.LineEvent
    if hasattr(gpiod, "line") and hasattr(gpiod.line, "LineEvent"):
        return gpiod.line.LineEvent
    return None


def _resolve_bias_enum(bias_name: str):
    if bias_name == "disable":
        return "DISABLED"
    if bias_name == "pull-up":
        return "PULL_UP"
    if bias_name == "pull-down":
        return "PULL_DOWN"
    return None


def _request_line(chip_path: str, gpio_pin: int, bias: str):
    """Request a GPIO line using libgpiod v2 or v1 APIs."""
    v2_symbols = _get_v2_line_settings()
    if hasattr(gpiod, "request_lines") and v2_symbols is not None:
        try:
            settings_cls, direction_enum, edge_enum, bias_enum = v2_symbols
            if direction_enum is None or edge_enum is None:
                raise RuntimeError("Missing v2 direction/edge enums")
            settings_kwargs = {
                "direction": direction_enum.INPUT,
                "edge_detection": edge_enum.BOTH,
            }
            bias_value = _resolve_bias_enum(bias)
            if bias_enum is not None and bias_value and hasattr(bias_enum, bias_value):
                settings_kwargs["bias"] = getattr(bias_enum, bias_value)
            settings = settings_cls(**settings_kwargs)
            request = gpiod.request_lines(
                chip_path,
                consumer="rpi-data-logger",
                config={gpio_pin: settings},
            )
            return request, "v2"
        except Exception as e:
            logger.error(f"Failed to request line via v2 API: {e}")
            raise

    try:
        chip = gpiod.Chip(chip_path)
        line = chip.get_line(gpio_pin)
        line.request(
            consumer="rpi-data-logger",
            type=gpiod.LINE_REQ_EV_BOTH_EDGES,
        )
        return line, "v1"
    except Exception as e:
        logger.error(f"Failed to request line via v1 API: {e}")
        raise


def _event_is_rising(event) -> bool:
    enum = _get_line_event_enum()
    if enum is None:
        return False
    if hasattr(event, "event_type"):
        return event.event_type == enum.RISING_EDGE
    if hasattr(event, "type"):
        return event.type == enum.RISING_EDGE
    return False


def _event_is_falling(event) -> bool:
    enum = _get_line_event_enum()
    if enum is None:
        return False
    if hasattr(event, "event_type"):
        return event.event_type == enum.FALLING_EDGE
    if hasattr(event, "type"):
        return event.type == enum.FALLING_EDGE
    return False


class DataLoggerMonitor:
    """Monitors GPIO switch and manages data logger process."""

    def __init__(
        self,
        gpio_pin: int,
        output_dir: Path,
        serial_port: str = "/dev/ttyACM0",
        chip: Optional[str] = None,
        bias: str = "disable",
    ):
        self.gpio_pin = gpio_pin
        self.output_dir = Path(output_dir)
        self.serial_port = serial_port
        self.chip_path = _resolve_chip_path(gpio_pin, chip)
        self.bias = bias
        self.logger_process: Optional[subprocess.Popen] = None
        self.script_dir = Path(__file__).parent
        self.main_script = self.script_dir / "main.py"
        self.session_counter = 0  # Counter for log files (no NTP dependency)
        self.line_request = None
        self.api_version = None

        # Create output directory if it doesn't exist
        self.output_dir.mkdir(parents=True, exist_ok=True)
        logger.info(f"Output directory: {self.output_dir}")

    def setup_gpio(self):
        """Initialize GPIO edge monitoring."""
        try:
            self.line_request, self.api_version = _request_line(
                self.chip_path, self.gpio_pin, self.bias
            )
            logger.info(
                f"GPIO pin {self.gpio_pin} initialized via {self.api_version} "
                f"on {self.chip_path}"
            )
        except Exception as e:
            logger.error(f"Failed to initialize GPIO pin {self.gpio_pin}: {e}")
            sys.exit(1)

    def on_switch_pressed(self):
        """Callback when switch is pressed (activated)."""
        logger.info("Data logging switch activated")
        self.start_logger()

    def on_switch_released(self):
        """Callback when switch is released (deactivated)."""
        logger.info("Data logging switch deactivated")
        self.stop_logger()

    def start_logger(self):
        """Start the sensor data receiver."""
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
                    sys.executable,
                    str(self.main_script),
                    "--port",
                    self.serial_port,
                    "--output",
                    str(output_file),
                    "--verbose",
                ],
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                bufsize=1,
            )
            logger.info(f"Data logger started (PID: {self.logger_process.pid})")
        except Exception as e:
            logger.error(f"Failed to start data logger: {e}")

    def stop_logger(self):
        """Stop the sensor data receiver."""
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

    def _process_events_v2(self):
        while True:
            if hasattr(self.line_request, "wait_edge_events"):
                if not self.line_request.wait_edge_events(timeout=1.0):
                    continue
            events = self.line_request.read_edge_events()
            for event in events:
                if _event_is_rising(event):
                    self.on_switch_pressed()
                elif _event_is_falling(event):
                    self.on_switch_released()

    def _process_events_v1(self):
        while True:
            if self.line_request.event_wait(sec=1):
                event = self.line_request.event_read()
                if _event_is_rising(event):
                    self.on_switch_pressed()
                elif _event_is_falling(event):
                    self.on_switch_released()

    def run(self):
        """Main loop."""
        logger.info("=" * 60)
        logger.info("GPIO Data Logger Monitor")
        logger.info("=" * 60)
        logger.info(f"GPIO Pin: {self.gpio_pin}")
        logger.info(f"GPIO Chip: {self.chip_path}")
        logger.info(f"GPIO Bias: {self.bias}")
        logger.info(f"Output Directory: {self.output_dir}")
        logger.info(f"Serial Port: {self.serial_port}")
        logger.info("Press Ctrl+C to exit")
        logger.info("=" * 60)

        self.setup_gpio()

        try:
            if self.api_version == "v2":
                self._process_events_v2()
            else:
                self._process_events_v1()
        except KeyboardInterrupt:
            logger.info("\nShutting down...")
            self.stop_logger()
        finally:
            if self.line_request is not None:
                if hasattr(self.line_request, "release"):
                    try:
                        self.line_request.release()
                    except Exception:
                        pass
            logger.info("GPIO monitor stopped")


def main():
    parser = argparse.ArgumentParser(
        description="Monitor GPIO switch for data logging control"
    )
    parser.add_argument(
        "--gpio",
        type=int,
        default=11,
        help="GPIO pin number to monitor (default: 11)",
    )
    parser.add_argument(
        "--chip",
        default=None,
        help="GPIO chip path or number (default: auto-detect)",
    )
    parser.add_argument(
        "--bias",
        choices=["disable", "pull-up", "pull-down"],
        default="disable",
        help="GPIO bias to apply (default: disable)",
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=Path("/opt/rpi_receiver/data"),
        help="Output directory for CSV files (default: /opt/rpi_receiver/data)",
    )
    parser.add_argument(
        "--port",
        default="/dev/ttyACM0",
        help="Serial port for ESP32 bridge (default: /dev/ttyACM0)",
    )

    args = parser.parse_args()

    monitor = DataLoggerMonitor(
        gpio_pin=args.gpio,
        output_dir=args.output,
        serial_port=args.port,
        chip=args.chip,
        bias=args.bias,
    )
    monitor.run()


if __name__ == "__main__":
    main()
