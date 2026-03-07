//! Shared types and parsing for COBS/sensor messages.

use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};
use std::io::{self, Read};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum CarPosition {
    FrontLeft = 0,
    FrontCenter = 1,
    FrontRight = 2,
    Left = 3,
    Center = 4,
    Right = 5,
    RearLeft = 6,
    RearCenter = 7,
    RearRight = 8,
    Roof = 9,
    Floating = 254,
    Custom = 255,
}

impl CarPosition {
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => CarPosition::FrontLeft,
            1 => CarPosition::FrontCenter,
            2 => CarPosition::FrontRight,
            3 => CarPosition::Left,
            4 => CarPosition::Center,
            5 => CarPosition::Right,
            6 => CarPosition::RearLeft,
            7 => CarPosition::RearCenter,
            8 => CarPosition::RearRight,
            9 => CarPosition::Roof,
            254 => CarPosition::Floating,
            _ => CarPosition::Custom,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            CarPosition::FrontLeft => "FrontLeft",
            CarPosition::FrontCenter => "FrontCenter",
            CarPosition::FrontRight => "FrontRight",
            CarPosition::Left => "Left",
            CarPosition::Center => "Center",
            CarPosition::Right => "Right",
            CarPosition::RearLeft => "RearLeft",
            CarPosition::RearCenter => "RearCenter",
            CarPosition::RearRight => "RearRight",
            CarPosition::Roof => "Roof",
            CarPosition::Floating => "Floating",
            CarPosition::Custom => "Custom",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeId {
    pub position: CarPosition,
    pub instance: u8,
}

impl NodeId {
    pub fn from_bytes(bytes: [u8; 2]) -> Self {
        Self {
            position: CarPosition::from_u8(bytes[0]),
            instance: bytes[1],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MessageType {
    Imu = 0x01,
    Gps = 0x02,
    Heartbeat = 0xFF,
    Unknown = 0x00,
}

impl MessageType {
    pub fn from_u8(value: u8) -> Self {
        match value {
            0x01 => MessageType::Imu,
            0x02 => MessageType::Gps,
            0xFF => MessageType::Heartbeat,
            _ => MessageType::Unknown,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecodedMessage {
    #[serde(with = "mac_address_serializer")]
    pub src_mac: [u8; 6],
    pub node_position: String,
    pub node_instance: u8,
    pub timestamp_us: u64,
    pub message_type: String,

    pub accel_x: Option<f32>,
    pub accel_y: Option<f32>,
    pub accel_z: Option<f32>,
    pub gyro_x: Option<f32>,
    pub gyro_y: Option<f32>,
    pub gyro_z: Option<f32>,

    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub alt: Option<f32>,
    pub speed_kts: Option<f32>,
    pub heading: Option<u16>,
    pub utc_time_year: Option<u16>,
    pub utc_time_month: Option<u8>,
    pub utc_time_day: Option<u8>,
    pub utc_time_hours: Option<u8>,
    pub utc_time_minutes: Option<u8>,
    pub utc_time_seconds: Option<u8>,
    pub utc_time_millis: Option<u16>,

    pub magic: Option<u8>,
    pub error: Option<String>,
}

pub fn format_mac_address(mac: &[u8; 6]) -> String {
    format!(
        "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
        mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
    )
}

mod mac_address_serializer {
    use serde::de;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(mac: &[u8; 6], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&super::format_mac_address(mac))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 6], D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let parts: Result<Vec<u8>, _> = s
            .split(':')
            .map(|s_part| u8::from_str_radix(s_part, 16))
            .collect();
        let parts = parts.map_err(de::Error::custom)?;
        if parts.len() == 6 {
            let mut mac = [0u8; 6];
            mac.copy_from_slice(&parts);
            Ok(mac)
        } else {
            Err(de::Error::custom("Invalid MAC address format"))
        }
    }
}

#[derive(Debug)]
pub enum ParserError {
    Io(io::Error),
    Cobs(cobs::DecodeError),
    TooShort,
    InvalidMessageType(u8),
    ParseError(String),
}

impl From<io::Error> for ParserError {
    fn from(err: io::Error) -> Self {
        ParserError::Io(err)
    }
}

impl From<cobs::DecodeError> for ParserError {
    fn from(err: cobs::DecodeError) -> Self {
        ParserError::Cobs(err)
    }
}

pub fn parse_message(buffer: &[u8]) -> Result<DecodedMessage, ParserError> {
    let mut reader = io::Cursor::new(buffer);

    if buffer.len() < 17 {
        return Err(ParserError::TooShort);
    }

    let mut src_mac = [0u8; 6];
    reader.read_exact(&mut src_mac)?;

    let mut node_id_bytes = [0u8; 2];
    reader.read_exact(&mut node_id_bytes)?;
    let node_id = NodeId::from_bytes(node_id_bytes);

    let timestamp_us = reader.read_u64::<LittleEndian>()?;
    let msg_type_byte = reader.read_u8()?;
    let message_type = MessageType::from_u8(msg_type_byte);

    let payload_data_offset = reader.position() as usize;
    let payload_data = &buffer[payload_data_offset..];

    let mut decoded_msg = DecodedMessage {
        src_mac,
        node_position: node_id.position.as_str().to_string(),
        node_instance: node_id.instance,
        timestamp_us,
        message_type: format!("{:?}", message_type),
        accel_x: None,
        accel_y: None,
        accel_z: None,
        gyro_x: None,
        gyro_y: None,
        gyro_z: None,
        lat: None,
        lon: None,
        alt: None,
        speed_kts: None,
        heading: None,
        utc_time_year: None,
        utc_time_month: None,
        utc_time_day: None,
        utc_time_hours: None,
        utc_time_minutes: None,
        utc_time_seconds: None,
        utc_time_millis: None,
        magic: None,
        error: None,
    };

    match message_type {
        MessageType::Imu => {
            if payload_data.len() < 24 {
                return Err(ParserError::ParseError("IMU payload too short".to_string()));
            }
            let mut payload_reader = io::Cursor::new(payload_data);
            decoded_msg.accel_x = Some(payload_reader.read_f32::<LittleEndian>()?);
            decoded_msg.accel_y = Some(payload_reader.read_f32::<LittleEndian>()?);
            decoded_msg.accel_z = Some(payload_reader.read_f32::<LittleEndian>()?);
            decoded_msg.gyro_x = Some(payload_reader.read_f32::<LittleEndian>()?);
            decoded_msg.gyro_y = Some(payload_reader.read_f32::<LittleEndian>()?);
            decoded_msg.gyro_z = Some(payload_reader.read_f32::<LittleEndian>()?);
        }
        MessageType::Gps => {
            if payload_data.len() < 35 {
                return Err(ParserError::ParseError("GPS payload too short".to_string()));
            }
            let mut payload_reader = io::Cursor::new(payload_data);
            decoded_msg.lat = Some(payload_reader.read_f64::<LittleEndian>()?);
            decoded_msg.lon = Some(payload_reader.read_f64::<LittleEndian>()?);
            decoded_msg.alt = Some(payload_reader.read_f32::<LittleEndian>()?);
            decoded_msg.speed_kts = Some(payload_reader.read_f32::<LittleEndian>()?);
            decoded_msg.heading = Some(payload_reader.read_u16::<LittleEndian>()?);
            decoded_msg.utc_time_year = Some(payload_reader.read_u16::<LittleEndian>()?);
            decoded_msg.utc_time_month = Some(payload_reader.read_u8()?);
            decoded_msg.utc_time_day = Some(payload_reader.read_u8()?);
            decoded_msg.utc_time_hours = Some(payload_reader.read_u8()?);
            decoded_msg.utc_time_minutes = Some(payload_reader.read_u8()?);
            decoded_msg.utc_time_seconds = Some(payload_reader.read_u8()?);
            decoded_msg.utc_time_millis = Some(payload_reader.read_u16::<LittleEndian>()?);
        }
        MessageType::Heartbeat => {
            if payload_data.len() < 1 {
                return Err(ParserError::ParseError(
                    "Heartbeat payload too short".to_string(),
                ));
            }
            let mut payload_reader = io::Cursor::new(payload_data);
            decoded_msg.magic = Some(payload_reader.read_u8()?);
        }
        MessageType::Unknown => {
            return Err(ParserError::InvalidMessageType(msg_type_byte));
        }
    }

    Ok(decoded_msg)
}

/// Build an error row for CSV when parsing or COBS fails.
pub fn error_decoded_message(kind: &str, detail: &str) -> DecodedMessage {
    DecodedMessage {
        src_mac: [0; 6],
        node_position: "Unknown".to_string(),
        node_instance: 0,
        timestamp_us: 0,
        message_type: kind.to_string(),
        accel_x: None,
        accel_y: None,
        accel_z: None,
        gyro_x: None,
        gyro_y: None,
        gyro_z: None,
        lat: None,
        lon: None,
        alt: None,
        speed_kts: None,
        heading: None,
        utc_time_year: None,
        utc_time_month: None,
        utc_time_day: None,
        utc_time_hours: None,
        utc_time_minutes: None,
        utc_time_seconds: None,
        utc_time_millis: None,
        magic: None,
        error: Some(detail.to_string()),
    }
}
