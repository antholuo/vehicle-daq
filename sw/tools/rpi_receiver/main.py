#!/usr/bin/env python3
"""
RPi Sensor Bridge Receiver

Receives sensor data from the ESP32 bridge via USB serial, decodes COBS-framed
messages, and saves them to a CSV file.

Usage:
    python main.py [options]

Options:
    --port      Serial port (default: /dev/ttyACM0)
    --baud      Baud rate (default: 115200, not used for USB Serial/JTAG)
    --output    Output CSV file (default: sensor_data.csv)
    --verbose   Enable verbose output
"""

import argparse
import csv
import sys
import time
from datetime import datetime
from pathlib import Path
from typing import Optional

try:
    import serial
except ImportError:
    print("Error: pyserial not installed. Run: pip install pyserial")
    sys.exit(1)

try:
    from cobs import cobs
except ImportError:
    print("Error: cobs not installed. Run: pip install cobs")
    sys.exit(1)

from protocol import (
    parse_message,
    SensorMessage,
    MessageType,
)


# CSV column headers
CSV_HEADERS = [
    'timestamp_us',
    'mac_address',
    'car_position',
    'instance',
    'sensor_type',
    'accel_x',
    'accel_y',
    'accel_z',
    'gyro_x',
    'gyro_y',
    'gyro_z',
    'lat',
    'lon',
    'alt',
    'speed_kts',
    'heading',
    'gps_time',
]


class SensorReceiver:
    """Receives and processes sensor data from the ESP32 bridge"""

    def __init__(
        self,
        port: str,
        baud_rate: int,
        output_file: Path,
        verbose: bool = False,
    ):
        self.port = port
        self.baud_rate = baud_rate
        self.output_file = output_file
        self.verbose = verbose
        self.serial: Optional[serial.Serial] = None
        self.csv_file = None
        self.csv_writer = None
        
        # Statistics
        self.messages_received = 0
        self.decode_errors = 0
        self.parse_errors = 0

    def connect(self) -> bool:
        """Connect to the serial port"""
        try:
            self.serial = serial.Serial(
                port=self.port,
                baudrate=self.baud_rate,
                timeout=1.0,
            )
            print(f"Connected to {self.port}")
            return True
        except serial.SerialException as e:
            print(f"Error connecting to {self.port}: {e}")
            return False

    def open_csv(self) -> bool:
        """Open CSV file for writing"""
        try:
            # Check if file exists to determine if we need to write headers
            file_exists = self.output_file.exists()
            
            self.csv_file = open(self.output_file, 'a', newline='')
            self.csv_writer = csv.DictWriter(self.csv_file, fieldnames=CSV_HEADERS)
            
            if not file_exists:
                self.csv_writer.writeheader()
                print(f"Created new CSV file: {self.output_file}")
            else:
                print(f"Appending to existing CSV file: {self.output_file}")
            
            return True
        except IOError as e:
            print(f"Error opening CSV file: {e}")
            return False

    def read_cobs_frame(self) -> Optional[bytes]:
        """
        Read a COBS-framed message from serial.
        
        COBS frames are delimited by 0x00 bytes.
        Returns the decoded message, or None on error.
        
        Note: The ESP32 USB Serial/JTAG also outputs text debug logs which
        don't contain 0x00 bytes. We detect and skip these by looking for
        newline characters (which indicate text, not binary COBS data).
        """
        if not self.serial:
            return None
        
        # Read until we get a 0x00 delimiter
        frame_data = bytearray()
        
        while True:
            byte = self.serial.read(1)
            if not byte:
                # Timeout
                return None
            
            # Check for newline - indicates text log, not COBS data
            # Text logs from ESP32 end with \n, COBS data never contains printable newlines
            if byte[0] == ord('\n'):
                if len(frame_data) > 0:
                    # This was a text line, print it and continue
                    try:
                        text = frame_data.decode('utf-8', errors='ignore').strip()
                        if text and self.verbose:
                            print(f"[ESP32] {text}")
                    except Exception:
                        pass
                    frame_data.clear()
                continue
            
            # Check for carriage return (part of \r\n on some systems)
            if byte[0] == ord('\r'):
                continue
            
            if byte[0] == 0x00:
                # Found COBS delimiter
                if len(frame_data) == 0:
                    # Empty frame, skip
                    continue
                break
            
            frame_data.append(byte[0])
            
            # Sanity check - frames shouldn't be too large
            # Max COBS frame is ~130 bytes for our messages
            if len(frame_data) > 256:
                # Could be corrupted data, try to recover
                if self.verbose:
                    print("Warning: Frame too large, discarding and resync")
                frame_data.clear()
        
        # Decode COBS
        try:
            decoded = cobs.decode(bytes(frame_data))
            return decoded
        except cobs.DecodeError as e:
            self.decode_errors += 1
            if self.verbose:
                print(f"COBS decode error: {e}")
            return None

    def process_message(self, data: bytes) -> Optional[SensorMessage]:
        """Parse and process a decoded message"""
        msg = parse_message(data)
        if msg is None:
            self.parse_errors += 1
            return None
        
        self.messages_received += 1
        return msg

    def write_to_csv(self, msg: SensorMessage):
        """Write a message to the CSV file"""
        if self.csv_writer:
            self.csv_writer.writerow(msg.to_csv_row())
            self.csv_file.flush()  # Ensure data is written

    def print_message(self, msg: SensorMessage):
        """Print a message to console"""
        if msg.message_type == MessageType.HEARTBEAT:
            print(f"[HB] {msg.mac_address}")
        elif msg.message_type == MessageType.IMU:
            imu = msg.payload
            print(
                f"[IMU] {msg.mac_address} | "
                f"accel=[{imu.accel_x:+.3f}, {imu.accel_y:+.3f}, {imu.accel_z:+.3f}] g | "
                f"gyro=[{imu.gyro_x:+.2f}, {imu.gyro_y:+.2f}, {imu.gyro_z:+.2f}] °/s"
            )
        elif msg.message_type == MessageType.GPS:
            gps = msg.payload
            print(
                f"[GPS] {msg.mac_address} | "
                f"lat={gps.lat:.6f}, lon={gps.lon:.6f}, alt={gps.alt:.1f}m | "
                f"speed={gps.speed_kts:.1f}kts, heading={gps.heading}°"
            )

    def run(self):
        """Main receive loop"""
        print(f"\nListening for sensor data on {self.port}...")
        print("Press Ctrl+C to stop\n")
        
        last_stats_time = time.time()
        
        try:
            while True:
                # Read and decode a COBS frame
                data = self.read_cobs_frame()
                if data is None:
                    continue
                
                # Parse the message
                msg = self.process_message(data)
                if msg is None:
                    continue
                
                # Skip heartbeats in CSV (optional: remove this check to include them)
                if msg.message_type != MessageType.HEARTBEAT:
                    self.write_to_csv(msg)
                
                # Print to console
                if self.verbose or msg.message_type != MessageType.HEARTBEAT:
                    self.print_message(msg)
                
                # Print stats periodically
                if time.time() - last_stats_time > 10.0:
                    print(
                        f"\n--- Stats: {self.messages_received} received, "
                        f"{self.decode_errors} decode errors, "
                        f"{self.parse_errors} parse errors ---\n"
                    )
                    last_stats_time = time.time()

        except KeyboardInterrupt:
            print("\n\nStopping...")
        finally:
            self.cleanup()

    def cleanup(self):
        """Clean up resources"""
        if self.csv_file:
            self.csv_file.close()
            print(f"Saved data to {self.output_file}")
        
        if self.serial:
            self.serial.close()
            print("Serial port closed")
        
        print(
            f"\nFinal stats: {self.messages_received} messages received, "
            f"{self.decode_errors} decode errors, "
            f"{self.parse_errors} parse errors"
        )


def main():
    parser = argparse.ArgumentParser(
        description="Receive sensor data from ESP32 bridge via USB serial"
    )
    parser.add_argument(
        '--port', '-p',
        default='/dev/ttyACM0',
        help='Serial port (default: /dev/ttyACM0)'
    )
    parser.add_argument(
        '--baud', '-b',
        type=int,
        default=115200,
        help='Baud rate (default: 115200, not used for USB Serial/JTAG)'
    )
    parser.add_argument(
        '--output', '-o',
        type=Path,
        default=Path('sensor_data.csv'),
        help='Output CSV file (default: sensor_data.csv)'
    )
    parser.add_argument(
        '--verbose', '-v',
        action='store_true',
        help='Enable verbose output (show heartbeats)'
    )
    
    args = parser.parse_args()
    
    print("=" * 60)
    print("RPi Sensor Bridge Receiver")
    print("=" * 60)
    print(f"Port:   {args.port}")
    print(f"Output: {args.output}")
    print("=" * 60)
    
    receiver = SensorReceiver(
        port=args.port,
        baud_rate=args.baud,
        output_file=args.output,
        verbose=args.verbose,
    )
    
    if not receiver.connect():
        sys.exit(1)
    
    if not receiver.open_csv():
        sys.exit(1)
    
    receiver.run()


if __name__ == '__main__':
    main()

