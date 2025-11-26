//! Protocol layer for serialization and deserialization
//!
//! This module handles converting sensor data structures to/from byte arrays
//! for transmission over ESP-NOW. Uses simple binary protocol with message
//! type header.

use super::message::*;
use super::error::{AirCommError, Result};
use crate::types::{GpsData, GpsTime};

/// Maximum payload size for ESP-NOW (250 bytes)
pub const MAX_PAYLOAD_SIZE: usize = 250;

/// Protocol header size (message type + timestamp)
pub const HEADER_SIZE: usize = 1 + 8; // message_type(1) + timestamp_us(8) = 9 bytes

/// Serialize heartbeat data into a byte buffer
///
/// Format: [MessageType:1][Timestamp:8][Magic:1]
///
/// # Arguments
/// * `timestamp_us` - Sender's timestamp in microseconds
/// * `data` - Heartbeat data to serialize
/// * `buffer` - Output buffer (must be at least HEADER_SIZE + HeartbeatData::SERIALIZED_SIZE)
///
/// # Returns
/// Number of bytes written
pub fn serialize_heartbeat(timestamp_us: u64, data: &HeartbeatData, buffer: &mut [u8]) -> Result<usize> {
    let required_size = HEADER_SIZE + HeartbeatData::SERIALIZED_SIZE;
    if buffer.len() < required_size {
        return Err(AirCommError::BufferTooSmall);
    }

    let mut offset = 0;
    
    // Write message type
    buffer[offset] = MessageType::Heartbeat.to_u8();
    offset += 1;
    
    // Write timestamp
    write_u64_le(buffer, offset, timestamp_us);
    offset += 8;
    
    // Write magic byte
    buffer[offset] = data.magic;
    offset += 1;
    
    Ok(offset)
}

/// Serialize IMU data into a byte buffer
///
/// Format: [MessageType:1][Timestamp:8][ImuData:TBD]
///
/// # Arguments
/// * `timestamp_us` - Sender's timestamp in microseconds
/// * `data` - IMU data to serialize
/// * `buffer` - Output buffer (must be at least HEADER_SIZE + IMU_SERIALIZED_SIZE)
///
/// # Returns
/// Number of bytes written
pub fn serialize_imu(timestamp_us: u64, data: &ImuData, buffer: &mut [u8]) -> Result<usize> {
    let required_size = HEADER_SIZE + IMU_SERIALIZED_SIZE;
    if buffer.len() < required_size {
        return Err(AirCommError::BufferTooSmall);
    }

    // TODO: Implement serialization
    // - Write message type
    // - Write timestamp
    // - Write IMU data fields in little-endian format
    
    todo!("Implement serialize_imu")
}

/// Serialize GPS data into a byte buffer
///
/// Format: [MessageType:1][Timestamp:8][lat:8][lon:8][alt:4][speed:4][heading:2][year:2][month:1][day:1][hours:1][minutes:1][seconds:1][millis:2]
///
/// # Arguments
/// * `timestamp_us` - Sender's timestamp in microseconds
/// * `data` - GPS data to serialize
/// * `buffer` - Output buffer (must be at least HEADER_SIZE + GPS_SERIALIZED_SIZE)
///
/// # Returns
/// Number of bytes written
pub fn serialize_gps(timestamp_us: u64, data: &GpsData, buffer: &mut [u8]) -> Result<usize> {
    let required_size = HEADER_SIZE + GPS_SERIALIZED_SIZE;
    if buffer.len() < required_size {
        return Err(AirCommError::BufferTooSmall);
    }

    let mut offset = 0;
    
    // Write message type
    buffer[offset] = MessageType::Gps.to_u8();
    offset += 1;
    
    // Write timestamp
    write_u64_le(buffer, offset, timestamp_us);
    offset += 8;
    
    // Write GPS fields
    write_f64_le(buffer, offset, data.lat);
    offset += 8;
    
    write_f64_le(buffer, offset, data.lon);
    offset += 8;
    
    write_f32_le(buffer, offset, data.alt);
    offset += 4;
    
    write_f32_le(buffer, offset, data.speed_kts);
    offset += 4;
    
    write_u16_le(buffer, offset, data.heading);
    offset += 2;
    
    // Write GpsTime fields
    write_u16_le(buffer, offset, data.utc_time.year);
    offset += 2;
    
    buffer[offset] = data.utc_time.month;
    offset += 1;
    
    buffer[offset] = data.utc_time.day;
    offset += 1;
    
    buffer[offset] = data.utc_time.hours;
    offset += 1;
    
    buffer[offset] = data.utc_time.minutes;
    offset += 1;
    
    buffer[offset] = data.utc_time.seconds;
    offset += 1;
    
    write_u16_le(buffer, offset, data.utc_time.millis);
    offset += 2;
    
    Ok(offset)
}

/// Deserialize received data into a sensor message
///
/// Reads the message type header, timestamp, and deserializes the appropriate payload
///
/// # Arguments
/// * `data` - Raw received data
/// * `src_address` - Source MAC address
///
/// # Returns
/// Parsed sensor message with timestamp and payload
pub fn deserialize(data: &[u8], src_address: [u8; 6]) -> Result<SensorMessage> {
    if data.len() < HEADER_SIZE {
        return Err(AirCommError::InvalidMessage);
    }

    let msg_type = MessageType::from_u8(data[0])
        .ok_or(AirCommError::UnknownMessageType)?;
    
    // Read timestamp (always at offset 1)
    let timestamp_us = read_u64_le(data, 1);
    let payload_offset = 9; // After message type (1) + timestamp (8)

    let payload = match msg_type {
        MessageType::Imu => {
            // TODO: Deserialize IMU data
            todo!("Implement IMU deserialization")
        }
        MessageType::Gps => {
            let required_size = HEADER_SIZE + GPS_SERIALIZED_SIZE;
            if data.len() < required_size {
                return Err(AirCommError::InvalidMessage);
            }
            
            let mut offset = payload_offset;
            
            // Read GPS fields
            let lat = read_f64_le(data, offset);
            offset += 8;
            
            let lon = read_f64_le(data, offset);
            offset += 8;
            
            let alt = read_f32_le(data, offset);
            offset += 4;
            
            let speed_kts = read_f32_le(data, offset);
            offset += 4;
            
            let heading = read_u16_le(data, offset);
            offset += 2;
            
            // Read GpsTime fields
            let year = read_u16_le(data, offset);
            offset += 2;
            
            let month = data[offset];
            offset += 1;
            
            let day = data[offset];
            offset += 1;
            
            let hours = data[offset];
            offset += 1;
            
            let minutes = data[offset];
            offset += 1;
            
            let seconds = data[offset];
            offset += 1;
            
            let millis = read_u16_le(data, offset);
            
            SensorPayload::Gps(GpsData {
                lat,
                lon,
                alt,
                speed_kts,
                heading,
                utc_time: GpsTime {
                    year,
                    month,
                    day,
                    hours,
                    minutes,
                    seconds,
                    millis,
                },
            })
        }
        MessageType::Heartbeat => {
            let required_size = HEADER_SIZE + HeartbeatData::SERIALIZED_SIZE;
            if data.len() < required_size {
                return Err(AirCommError::InvalidMessage);
            }
            
            let offset = payload_offset;
            
            // Read magic byte
            let magic = data[offset];
            
            SensorPayload::Heartbeat(HeartbeatData { magic })
        }
    };
    
    Ok(SensorMessage {
        src_address,
        timestamp_us,
        payload,
    })
}

/// Helper to write f32 in little-endian format
fn write_f32_le(buffer: &mut [u8], offset: usize, value: f32) {
    let bytes = value.to_le_bytes();
    buffer[offset..offset + 4].copy_from_slice(&bytes);
}

/// Helper to write f64 in little-endian format
fn write_f64_le(buffer: &mut [u8], offset: usize, value: f64) {
    let bytes = value.to_le_bytes();
    buffer[offset..offset + 8].copy_from_slice(&bytes);
}

/// Helper to write u16 in little-endian format
fn write_u16_le(buffer: &mut [u8], offset: usize, value: u16) {
    let bytes = value.to_le_bytes();
    buffer[offset..offset + 2].copy_from_slice(&bytes);
}

/// Helper to write u64 in little-endian format
fn write_u64_le(buffer: &mut [u8], offset: usize, value: u64) {
    let bytes = value.to_le_bytes();
    buffer[offset..offset + 8].copy_from_slice(&bytes);
}

/// Helper to read f32 from little-endian format
fn read_f32_le(buffer: &[u8], offset: usize) -> f32 {
    let mut bytes = [0u8; 4];
    bytes.copy_from_slice(&buffer[offset..offset + 4]);
    f32::from_le_bytes(bytes)
}

/// Helper to read f64 from little-endian format
fn read_f64_le(buffer: &[u8], offset: usize) -> f64 {
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&buffer[offset..offset + 8]);
    f64::from_le_bytes(bytes)
}

/// Helper to read u16 from little-endian format
fn read_u16_le(buffer: &[u8], offset: usize) -> u16 {
    let mut bytes = [0u8; 2];
    bytes.copy_from_slice(&buffer[offset..offset + 2]);
    u16::from_le_bytes(bytes)
}

/// Helper to read u64 from little-endian format
fn read_u64_le(buffer: &[u8], offset: usize) -> u64 {
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&buffer[offset..offset + 8]);
    u64::from_le_bytes(bytes)
}
