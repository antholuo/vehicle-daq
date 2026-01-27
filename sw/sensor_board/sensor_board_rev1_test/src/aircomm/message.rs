//! Message definitions for sensor data communication
//!
//! This module defines the data structures and message types for various
//! sensor data that can be transmitted over ESP-NOW.

// Re-export sensor data types from types module
pub use crate::types::{GpsData, ImuData};

/// Message type discriminator (1 byte)
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    /// IMU sensor data
    Imu = 0x01,
    /// GPS sensor data
    Gps = 0x02,
    /// Heartbeat/keep-alive message
    Heartbeat = 0xFF,
}

impl MessageType {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(MessageType::Imu),
            0x02 => Some(MessageType::Gps),
            0xFF => Some(MessageType::Heartbeat),
            _ => None,
        }
    }

    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

/// Heartbeat message data structure
///
/// Contains a magic byte for validation. Timestamp is at the SensorMessage level.
#[derive(Debug, Clone, Copy)]
pub struct HeartbeatData {
    /// Magic byte for validation (typically 0x42 or custom value)
    pub magic: u8,
}

impl HeartbeatData {
    /// Size in bytes when serialized
    pub const SERIALIZED_SIZE: usize = 1; // magic only

    /// Default magic byte value
    pub const DEFAULT_MAGIC: u8 = 0x42;

    /// Create a new heartbeat with magic byte
    ///
    /// # Arguments
    /// * `magic` - Magic byte for validation
    pub fn new(magic: u8) -> Self {
        Self { magic }
    }

    /// Create a new heartbeat with default magic byte
    pub fn default() -> Self {
        Self::new(Self::DEFAULT_MAGIC)
    }
}

// Serialization sizes for aircomm protocol (payload only, excluding header)
// Header is always: message_type(1) + timestamp(8) = 9 bytes
// HeartbeatData: magic(1) = 1 byte (timestamp moved to message level)
// ImuData: accel_x(4) + accel_y(4) + accel_z(4) + gyro_x(4) + gyro_y(4) + gyro_z(4) = 24 bytes
pub const IMU_SERIALIZED_SIZE: usize = 6 * 4; // 6 f32 fields = 24 bytes
// GpsTime: year(2) + month(1) + day(1) + hours(1) + minutes(1) + seconds(1) + millis(2) = 9 bytes
pub const GPS_TIME_SERIALIZED_SIZE: usize = 2 + 1 + 1 + 1 + 1 + 1 + 2;
// GpsData: lat(8) + lon(8) + alt(4) + speed(4) + heading(2) + time(9) = 35 bytes
pub const GPS_SERIALIZED_SIZE: usize = 8 + 8 + 4 + 4 + 2 + GPS_TIME_SERIALIZED_SIZE;

/// Unified sensor message structure
///
/// Represents any type of sensor data with unified timestamp at message level
#[derive(Debug, Clone)]
pub struct SensorMessage {
    /// Source MAC address
    pub src_address: [u8; 6],
    /// Sender's timestamp in microseconds (when message was created)
    pub timestamp_us: u64,
    /// Message payload
    pub payload: SensorPayload,
}

/// Sensor message payload
#[derive(Debug, Clone)]
pub enum SensorPayload {
    Imu(ImuData),
    Gps(GpsData),
    Heartbeat(HeartbeatData),
}

impl SensorPayload {
    pub fn message_type(&self) -> MessageType {
        match self {
            SensorPayload::Imu(_) => MessageType::Imu,
            SensorPayload::Gps(_) => MessageType::Gps,
            SensorPayload::Heartbeat(_) => MessageType::Heartbeat,
        }
    }
}
