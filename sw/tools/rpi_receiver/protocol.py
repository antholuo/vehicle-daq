"""
Protocol definitions for parsing sensor data from the ESP32 bridge.

Message format (before COBS encoding):
    [MAC Address (6B)][NodeId (2B)][Timestamp (8B)][MessageType (1B)][Payload (variable)]

All multi-byte values are little-endian.
"""

import struct
from dataclasses import dataclass
from enum import IntEnum
from typing import Optional, Tuple
from datetime import datetime


class MessageType(IntEnum):
    """Sensor message types (matches Rust MessageType enum)"""
    IMU = 0x01
    GPS = 0x02
    HEARTBEAT = 0xFF


class CarPosition(IntEnum):
    """Car position enum (matches Rust CarPosition enum)"""
    FRONT_LEFT = 0
    FRONT_CENTER = 1
    FRONT_RIGHT = 2
    LEFT = 3
    CENTER = 4
    RIGHT = 5
    REAR_LEFT = 6
    REAR_CENTER = 7
    REAR_RIGHT = 8
    ROOF = 9
    CUSTOM = 255

    @classmethod
    def to_string(cls, value: int) -> str:
        """Convert position value to string name"""
        try:
            return cls(value).name
        except ValueError:
            return f"UNKNOWN_{value}"


@dataclass
class NodeId:
    """Node identifier combining car position and instance number"""
    position: CarPosition
    instance: int

    def __str__(self) -> str:
        return f"{CarPosition.to_string(self.position)}_{self.instance}"


@dataclass
class ImuData:
    """IMU sensor data (6-axis accelerometer + gyroscope)"""
    accel_x: float  # g
    accel_y: float  # g
    accel_z: float  # g
    gyro_x: float   # deg/s
    gyro_y: float   # deg/s
    gyro_z: float   # deg/s


@dataclass
class GpsTime:
    """GPS UTC timestamp"""
    year: int
    month: int
    day: int
    hours: int
    minutes: int
    seconds: int
    millis: int

    def to_iso_string(self) -> str:
        """Convert to ISO 8601 string"""
        return f"{self.year:04d}-{self.month:02d}-{self.day:02d}T{self.hours:02d}:{self.minutes:02d}:{self.seconds:02d}.{self.millis:03d}Z"


@dataclass
class GpsData:
    """GPS sensor data"""
    lat: float      # degrees
    lon: float      # degrees
    alt: float      # meters
    speed_kts: float  # knots
    heading: int    # degrees (0-360)
    utc_time: GpsTime


@dataclass
class HeartbeatData:
    """Heartbeat message data"""
    magic: int


@dataclass
class SensorMessage:
    """Parsed sensor message from the bridge"""
    mac_address: str        # Format: "AA:BB:CC:DD:EE:FF"
    node_id: NodeId
    timestamp_us: int       # Microseconds since boot
    message_type: MessageType
    payload: Optional[ImuData | GpsData | HeartbeatData]

    def to_csv_row(self) -> dict:
        """Convert to flat dictionary for CSV output"""
        row = {
            'timestamp_us': self.timestamp_us,
            'mac_address': self.mac_address,
            'car_position': CarPosition.to_string(self.node_id.position),
            'instance': self.node_id.instance,
            'sensor_type': self.message_type.name,
            # IMU fields
            'accel_x': '',
            'accel_y': '',
            'accel_z': '',
            'gyro_x': '',
            'gyro_y': '',
            'gyro_z': '',
            # GPS fields
            'lat': '',
            'lon': '',
            'alt': '',
            'speed_kts': '',
            'heading': '',
            'gps_time': '',
        }

        if isinstance(self.payload, ImuData):
            row['accel_x'] = f"{self.payload.accel_x:.6f}"
            row['accel_y'] = f"{self.payload.accel_y:.6f}"
            row['accel_z'] = f"{self.payload.accel_z:.6f}"
            row['gyro_x'] = f"{self.payload.gyro_x:.6f}"
            row['gyro_y'] = f"{self.payload.gyro_y:.6f}"
            row['gyro_z'] = f"{self.payload.gyro_z:.6f}"
        elif isinstance(self.payload, GpsData):
            row['lat'] = f"{self.payload.lat:.8f}"
            row['lon'] = f"{self.payload.lon:.8f}"
            row['alt'] = f"{self.payload.alt:.2f}"
            row['speed_kts'] = f"{self.payload.speed_kts:.2f}"
            row['heading'] = str(self.payload.heading)
            row['gps_time'] = self.payload.utc_time.to_iso_string()

        return row


# Header size: MAC (6) + NodeId (2) + Timestamp (8) + MsgType (1) = 17 bytes
HEADER_SIZE = 17

# Payload sizes
IMU_PAYLOAD_SIZE = 24    # 6 x f32
GPS_PAYLOAD_SIZE = 35    # lat(8) + lon(8) + alt(4) + speed(4) + heading(2) + time(9)
HEARTBEAT_PAYLOAD_SIZE = 1


def parse_mac_address(data: bytes) -> str:
    """Parse 6-byte MAC address to string format"""
    return ':'.join(f'{b:02X}' for b in data)


def parse_node_id(data: bytes) -> NodeId:
    """Parse 2-byte NodeId"""
    return NodeId(
        position=CarPosition(data[0]) if data[0] in [e.value for e in CarPosition] else CarPosition.CUSTOM,
        instance=data[1]
    )


def parse_imu_payload(data: bytes) -> ImuData:
    """Parse IMU payload (24 bytes: 6 x f32)"""
    if len(data) < IMU_PAYLOAD_SIZE:
        raise ValueError(f"IMU payload too short: {len(data)} < {IMU_PAYLOAD_SIZE}")
    
    values = struct.unpack('<6f', data[:IMU_PAYLOAD_SIZE])
    return ImuData(
        accel_x=values[0],
        accel_y=values[1],
        accel_z=values[2],
        gyro_x=values[3],
        gyro_y=values[4],
        gyro_z=values[5],
    )


def parse_gps_payload(data: bytes) -> GpsData:
    """Parse GPS payload (35 bytes)"""
    if len(data) < GPS_PAYLOAD_SIZE:
        raise ValueError(f"GPS payload too short: {len(data)} < {GPS_PAYLOAD_SIZE}")
    
    # lat(8) + lon(8) + alt(4) + speed(4) + heading(2) = 26 bytes
    lat, lon, alt, speed, heading = struct.unpack('<ddffH', data[:26])
    
    # GPS time: year(2) + month(1) + day(1) + hours(1) + minutes(1) + seconds(1) + millis(2) = 9 bytes
    year, month, day, hours, minutes, seconds, millis = struct.unpack('<HBBBBBH', data[26:35])
    
    return GpsData(
        lat=lat,
        lon=lon,
        alt=alt,
        speed_kts=speed,
        heading=heading,
        utc_time=GpsTime(
            year=year,
            month=month,
            day=day,
            hours=hours,
            minutes=minutes,
            seconds=seconds,
            millis=millis,
        )
    )


def parse_heartbeat_payload(data: bytes) -> HeartbeatData:
    """Parse heartbeat payload (1 byte)"""
    if len(data) < HEARTBEAT_PAYLOAD_SIZE:
        raise ValueError(f"Heartbeat payload too short: {len(data)} < {HEARTBEAT_PAYLOAD_SIZE}")
    
    return HeartbeatData(magic=data[0])


def parse_message(data: bytes) -> Optional[SensorMessage]:
    """
    Parse a complete sensor message from raw bytes (after COBS decoding).
    
    Returns:
        SensorMessage if successful, None if parsing fails
    """
    if len(data) < HEADER_SIZE:
        return None
    
    try:
        # Parse header
        mac_address = parse_mac_address(data[0:6])
        node_id = parse_node_id(data[6:8])
        timestamp_us = struct.unpack('<Q', data[8:16])[0]
        msg_type_byte = data[16]
        
        try:
            msg_type = MessageType(msg_type_byte)
        except ValueError:
            print(f"Unknown message type: 0x{msg_type_byte:02X}")
            return None
        
        # Parse payload based on message type
        payload_data = data[HEADER_SIZE:]
        payload = None
        
        if msg_type == MessageType.IMU:
            payload = parse_imu_payload(payload_data)
        elif msg_type == MessageType.GPS:
            payload = parse_gps_payload(payload_data)
        elif msg_type == MessageType.HEARTBEAT:
            payload = parse_heartbeat_payload(payload_data)
        
        return SensorMessage(
            mac_address=mac_address,
            node_id=node_id,
            timestamp_us=timestamp_us,
            message_type=msg_type,
            payload=payload,
        )
    
    except Exception as e:
        print(f"Error parsing message: {e}")
        return None

