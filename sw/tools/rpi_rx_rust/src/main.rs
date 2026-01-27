use byteorder::{LittleEndian, ReadBytesExt};
use cobs::decode;
use csv::Writer;
use log::*;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::io::{self, Read};
use std::string::ToString; // Required for .to_string() on &str
use std::time::Duration; // For serialport timeout

/// Maximum size of the incoming COBS-encoded message (from the sender's perspective)
/// Header (17 bytes) + Max Payload (GPS is largest at 35 bytes) = 52 bytes
/// COBS encoding adds at most 1 byte per 254 bytes, plus 1 overhead byte and 1 delimiter (0x00)
const MAX_USB_MESSAGE_SIZE: usize = 52;
const COBS_DECODED_BUFFER_SIZE: usize = MAX_USB_MESSAGE_SIZE;

const DEFAULT_SERIAL_PORT: &str = "/dev/ttyACM0";
const DEFAULT_BAUD_RATE: u32 = 115_200; // Standard baud rate for USB-CDC devices

// =============================================================================
// Data Structures (mirroring sensor_board_rev1_test)
// =============================================================================

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

#[derive(Debug, Clone, Copy)]
pub struct ImuData {
    pub accel_x: f32,
    pub accel_y: f32,
    pub accel_z: f32,
    pub gyro_x: f32,
    pub gyro_y: f32,
    pub gyro_z: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct GpsTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
    pub millis: u16,
}

#[derive(Debug, Clone, Copy)]
pub struct GpsData {
    pub lat: f64,
    pub lon: f64,
    pub alt: f32,
    pub speed_kts: f32,
    pub heading: u16,
    pub utc_time: GpsTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]

pub enum MessageType {
    Imu = 0x01,
    Gps = 0x02,
    Heartbeat = 0xFF,
    Unknown = 0x00, // For deserialization errors
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

#[derive(Debug, Clone, Copy)]
pub struct HeartbeatData {
    pub magic: u8,
}

// =============================================================================
// Decoded Message for CSV
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecodedMessage {
    #[serde(with = "mac_address_serializer")]
    pub src_mac: [u8; 6],
    pub node_position: String,
    pub node_instance: u8,
    pub timestamp_us: u64,
    pub message_type: String,

    // IMU fields
    pub accel_x: Option<f32>,
    pub accel_y: Option<f32>,
    pub accel_z: Option<f32>,
    pub gyro_x: Option<f32>,
    pub gyro_y: Option<f32>,
    pub gyro_z: Option<f32>,

    // GPS fields
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

    // Heartbeat fields
    pub magic: Option<u8>,

    // For error messages in CSV
    pub error: Option<String>,
}

// Helper function to format MAC address for logging and serialization
fn format_mac_address(mac: &[u8; 6]) -> String {
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

// =============================================================================
// Parsing Logic
// =============================================================================

#[derive(Debug)]
enum ParserError {
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

// Function to parse the raw COBS decoded message
fn parse_message(buffer: &[u8]) -> Result<DecodedMessage, ParserError> {
    let mut reader = io::Cursor::new(buffer);

    // Header: MAC (6B) + NodeId (2B) + Timestamp (8B) + MsgType (1B) = 17 bytes
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

// =============================================================================
// Main function
// =============================================================================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    info!("Starting RPI RX Rust decoder...");

    let mut cobs_encoded_buffer: VecDeque<u8> = VecDeque::new();
    let mut decoded_buffer = [0u8; COBS_DECODED_BUFFER_SIZE];

    // Create a CSV writer, automatically writes header from DecodedMessage
    let mut wtr = Writer::from_path("output.csv")?;

    // Attempt to open serial port
    let mut serial_port_reader: Option<Box<dyn Read>> = None;
    let mut input_source = "stdin".to_string(); // Default to stdin

    match serialport::new(DEFAULT_SERIAL_PORT, DEFAULT_BAUD_RATE)
        .timeout(Duration::from_millis(100))
        .open()
    {
        Ok(port) => {
            info!("Successfully opened serial port: {}", DEFAULT_SERIAL_PORT);
            serial_port_reader = Some(port);
            input_source = format!("serial port {}", DEFAULT_SERIAL_PORT);
        }
        Err(e) => {
            warn!(
                "Failed to open serial port {}: {}. Falling back to stdin.",
                DEFAULT_SERIAL_PORT, e
            );
        }
    }

    info!("Reading data from {}", input_source);

    let mut input_bytes_iterator: Box<dyn Iterator<Item = io::Result<u8>>> =
        if let Some(reader) = serial_port_reader {
            Box::new(reader.bytes())
        } else {
            Box::new(io::stdin().bytes())
        };

    let mut message_count = 0; // To track processed messages

    loop {
        if let Some(Ok(byte)) = input_bytes_iterator.next() {
            cobs_encoded_buffer.push_back(byte);

            if byte == 0x00 {
                // COBS frame delimiter
                debug!("Received COBS frame delimiter from {}", input_source);

                let mut frame_buffer = Vec::with_capacity(cobs_encoded_buffer.len());
                while let Some(b) = cobs_encoded_buffer.pop_front() {
                    frame_buffer.push(b);
                }

                let data_to_decode = if frame_buffer.last() == Some(&0x00) {
                    &frame_buffer[..frame_buffer.len() - 1]
                } else {
                    &frame_buffer[..]
                };

                match decode(data_to_decode, &mut decoded_buffer) {
                    Ok(decoded_len) => {
                        debug!(
                            "Successfully COBS decoded {} bytes from {} (raw length {}).",
                            decoded_len,
                            input_source,
                            data_to_decode.len()
                        );
                        let raw_message = &decoded_buffer[..decoded_len];
                        match parse_message(raw_message) {
                            Ok(msg) => {
                                message_count += 1;
                                info!(
                                    "[{}] Parsed message from {}: Type={:?}, MAC={}, Node={}/{}, Timestamp={}us",
                                    message_count,
                                    input_source,
                                    msg.message_type,
                                    format_mac_address(&msg.src_mac), // Use the new helper function
                                    msg.node_position,
                                    msg.node_instance,
                                    msg.timestamp_us
                                );
                                wtr.serialize(&msg)?;
                                wtr.flush()?;
                            }
                            Err(e) => {
                                error!("Failed to parse message: {:?}", e);
                                let error_msg = DecodedMessage {
                                    src_mac: [0; 6],
                                    node_position: "Unknown".to_string(),
                                    node_instance: 0,
                                    timestamp_us: 0,
                                    message_type: "Error".to_string(),
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
                                    error: Some(format!("Parsing error: {:?}", e)),
                                };
                                wtr.serialize(&error_msg)?;
                                wtr.flush()?;
                            }
                        }
                    }
                    Err(e) => {
                        error!("COBS decoding error: {:?}", e);
                        let error_msg = DecodedMessage {
                            src_mac: [0; 6],
                            node_position: "Unknown".to_string(),
                            node_instance: 0,
                            timestamp_us: 0,
                            message_type: "COBS_Error".to_string(),
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
                            error: Some(format!("COBS decoding error: {:?}", e)),
                        };
                        wtr.serialize(&error_msg)?;
                        wtr.flush()?;
                    }
                }
            }
        } else {
            // End of input or error reading stdin
            break;
        }
    }

    info!("Finished processing input. Output written to output.csv");
    Ok(())
}
