//! Protocol layer for serialization and deserialization
//!
//! This module handles converting sensor data structures to/from byte arrays
//! for transmission over ESP-NOW. Uses simple binary protocol with message
//! type header.

use byteorder::{ByteOrder, LittleEndian};

use super::error::{AirCommError, Result};
use super::message::*;
use crate::types::{GpsData, GpsTime, ImuData};

/// Maximum payload size for ESP-NOW (250 bytes)
pub const MAX_PAYLOAD_SIZE: usize = 250;

/// Protocol header size (message type + timestamp)
pub const HEADER_SIZE: usize = 1 + 8; // message_type(1) + timestamp_us(8) = 9 bytes

/// Serialize heartbeat data into a byte buffer
///
/// Format: [MessageType:1][Timestamp:8][Magic:1][Position:1][Instance:1]
///
/// # Arguments
/// * `timestamp_us` - Sender's timestamp in microseconds
/// * `data` - Heartbeat data to serialize
/// * `buffer` - Output buffer (must be at least HEADER_SIZE + HeartbeatData::SERIALIZED_SIZE)
///
/// # Returns
/// Number of bytes written
pub fn serialize_heartbeat(
    timestamp_us: u64,
    data: &HeartbeatData,
    buffer: &mut [u8],
) -> Result<usize> {
    let required_size = HEADER_SIZE + HeartbeatData::SERIALIZED_SIZE;
    if buffer.len() < required_size {
        return Err(AirCommError::BufferTooSmall);
    }

    let mut offset = 0;

    // Write message type
    buffer[offset] = MessageType::Heartbeat.to_u8();
    offset += 1;

    // Write timestamp
    LittleEndian::write_u64(&mut buffer[offset..], timestamp_us);
    offset += 8;

    // Write magic byte
    buffer[offset] = data.magic;
    offset += 1;

    // Write node ID (position + instance)
    let node_id_bytes = data.node_id.to_bytes();
    buffer[offset] = node_id_bytes[0]; // position
    offset += 1;
    buffer[offset] = node_id_bytes[1]; // instance
    offset += 1;

    Ok(offset)
}

/// Serialize IMU data into a byte buffer
///
/// Format: [MessageType:1][Timestamp:8][accel_x:4][accel_y:4][accel_z:4][gyro_x:4][gyro_y:4][gyro_z:4]
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

    let mut offset = 0;

    // Write message type
    buffer[offset] = MessageType::Imu.to_u8();
    offset += 1;

    // Write timestamp
    LittleEndian::write_u64(&mut buffer[offset..], timestamp_us);
    offset += 8;

    // Write accelerometer data
    LittleEndian::write_f32(&mut buffer[offset..], data.accel_x);
    offset += 4;

    LittleEndian::write_f32(&mut buffer[offset..], data.accel_y);
    offset += 4;

    LittleEndian::write_f32(&mut buffer[offset..], data.accel_z);
    offset += 4;

    // Write gyroscope data
    LittleEndian::write_f32(&mut buffer[offset..], data.gyro_x);
    offset += 4;

    LittleEndian::write_f32(&mut buffer[offset..], data.gyro_y);
    offset += 4;

    LittleEndian::write_f32(&mut buffer[offset..], data.gyro_z);
    offset += 4;

    Ok(offset)
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
    LittleEndian::write_u64(&mut buffer[offset..], timestamp_us);
    offset += 8;

    // Write GPS fields
    LittleEndian::write_f64(&mut buffer[offset..], data.lat);
    offset += 8;

    LittleEndian::write_f64(&mut buffer[offset..], data.lon);
    offset += 8;

    LittleEndian::write_f32(&mut buffer[offset..], data.alt);
    offset += 4;

    LittleEndian::write_f32(&mut buffer[offset..], data.speed_kts);
    offset += 4;

    LittleEndian::write_u16(&mut buffer[offset..], data.heading);
    offset += 2;

    // Write GpsTime fields
    LittleEndian::write_u16(&mut buffer[offset..], data.utc_time.year);
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

    LittleEndian::write_u16(&mut buffer[offset..], data.utc_time.millis);
    offset += 2;

    Ok(offset)
}

/// Serialize time sync data into a byte buffer
///
/// Format: [MessageType:1][Timestamp:8][session_time_us:8]
///
/// # Arguments
/// * `timestamp_us` - Sender's local timestamp in microseconds
/// * `data` - Time sync data containing the RPi session elapsed time
/// * `buffer` - Output buffer (must be at least HEADER_SIZE + TimeSyncData::SERIALIZED_SIZE)
///
/// # Returns
/// Number of bytes written
pub fn serialize_timesync(
    timestamp_us: u64,
    data: &TimeSyncData,
    buffer: &mut [u8],
) -> Result<usize> {
    let required_size = HEADER_SIZE + TimeSyncData::SERIALIZED_SIZE;
    if buffer.len() < required_size {
        return Err(AirCommError::BufferTooSmall);
    }

    let mut offset = 0;

    buffer[offset] = MessageType::TimeSync.to_u8();
    offset += 1;

    LittleEndian::write_u64(&mut buffer[offset..], timestamp_us);
    offset += 8;

    LittleEndian::write_u64(&mut buffer[offset..], data.session_time_us);
    offset += 8;

    Ok(offset)
}

/// RequestGpsCapture has no payload; header only (9 bytes)
pub const REQUEST_GPS_CAPTURE_SIZE: usize = HEADER_SIZE;

/// Serialize RequestGpsCapture (header only)
pub fn serialize_request_gps_capture(
    timestamp_us: u64,
    buffer: &mut [u8],
) -> Result<usize> {
    if buffer.len() < REQUEST_GPS_CAPTURE_SIZE {
        return Err(AirCommError::BufferTooSmall);
    }
    buffer[0] = MessageType::RequestGpsCapture.to_u8();
    LittleEndian::write_u64(&mut buffer[1..], timestamp_us);
    Ok(REQUEST_GPS_CAPTURE_SIZE)
}

/// Serialize TrackCaptureArmed (header only)
pub fn serialize_track_capture_armed(timestamp_us: u64, buffer: &mut [u8]) -> Result<usize> {
    if buffer.len() < HEADER_SIZE {
        return Err(AirCommError::BufferTooSmall);
    }
    buffer[0] = MessageType::TrackCaptureArmed.to_u8();
    LittleEndian::write_u64(&mut buffer[1..], timestamp_us);
    Ok(HEADER_SIZE)
}

/// Serialize TrackCaptureEnded (header only)
pub fn serialize_track_capture_ended(timestamp_us: u64, buffer: &mut [u8]) -> Result<usize> {
    if buffer.len() < HEADER_SIZE {
        return Err(AirCommError::BufferTooSmall);
    }
    buffer[0] = MessageType::TrackCaptureEnded.to_u8();
    LittleEndian::write_u64(&mut buffer[1..], timestamp_us);
    Ok(HEADER_SIZE)
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

    let msg_type = MessageType::from_u8(data[0]).ok_or(AirCommError::UnknownMessageType)?;

    // Read timestamp (always at offset 1)
    let timestamp_us = LittleEndian::read_u64(&data[1..]);
    let payload_offset = 9; // After message type (1) + timestamp (8)

    let payload = match msg_type {
        MessageType::Imu => deserialize_imu(data, payload_offset)?,
        MessageType::Gps => deserialize_gps(data, payload_offset)?,
        MessageType::TrackCaptureArmed => SensorPayload::TrackCaptureArmed,
        MessageType::TrackCaptureEnded => SensorPayload::TrackCaptureEnded,
        MessageType::RequestGpsCapture => SensorPayload::RequestGpsCapture,
        MessageType::TimeSync => deserialize_timesync(data, payload_offset)?,
        MessageType::Heartbeat => deserialize_heartbeat(data, payload_offset)?,
    };

    Ok(SensorMessage {
        src_address,
        timestamp_us,
        payload,
    })
}

/// Deserialize IMU payload from buffer
fn deserialize_imu(data: &[u8], payload_offset: usize) -> Result<SensorPayload> {
    let required_size = HEADER_SIZE + IMU_SERIALIZED_SIZE;
    if data.len() < required_size {
        return Err(AirCommError::InvalidMessage);
    }

    let mut offset = payload_offset;

    // Read accelerometer data
    let accel_x = LittleEndian::read_f32(&data[offset..]);
    offset += 4;

    let accel_y = LittleEndian::read_f32(&data[offset..]);
    offset += 4;

    let accel_z = LittleEndian::read_f32(&data[offset..]);
    offset += 4;

    // Read gyroscope data
    let gyro_x = LittleEndian::read_f32(&data[offset..]);
    offset += 4;

    let gyro_y = LittleEndian::read_f32(&data[offset..]);
    offset += 4;

    let gyro_z = LittleEndian::read_f32(&data[offset..]);

    Ok(SensorPayload::Imu(ImuData {
        accel_x,
        accel_y,
        accel_z,
        gyro_x,
        gyro_y,
        gyro_z,
    }))
}

/// Deserialize GPS payload from buffer
fn deserialize_gps(data: &[u8], payload_offset: usize) -> Result<SensorPayload> {
    let required_size = HEADER_SIZE + GPS_SERIALIZED_SIZE;
    if data.len() < required_size {
        return Err(AirCommError::InvalidMessage);
    }

    let mut offset = payload_offset;

    // Read GPS fields
    let lat = LittleEndian::read_f64(&data[offset..]);
    offset += 8;

    let lon = LittleEndian::read_f64(&data[offset..]);
    offset += 8;

    let alt = LittleEndian::read_f32(&data[offset..]);
    offset += 4;

    let speed_kts = LittleEndian::read_f32(&data[offset..]);
    offset += 4;

    let heading = LittleEndian::read_u16(&data[offset..]);
    offset += 2;

    // Read GpsTime fields
    let year = LittleEndian::read_u16(&data[offset..]);
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

    let millis = LittleEndian::read_u16(&data[offset..]);

    Ok(SensorPayload::Gps(GpsData {
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
    }))
}

/// Deserialize Heartbeat payload from buffer
fn deserialize_heartbeat(data: &[u8], payload_offset: usize) -> Result<SensorPayload> {
    let required_size = HEADER_SIZE + HeartbeatData::SERIALIZED_SIZE;
    if data.len() < required_size {
        return Err(AirCommError::InvalidMessage);
    }

    let magic = data[payload_offset];
    
    // Read NodeId (position + instance)
    let node_id_bytes = [
        data[payload_offset + 1], // position
        data[payload_offset + 2], // instance
    ];
    let node_id = crate::types::NodeId::from_bytes(node_id_bytes);
    
    Ok(SensorPayload::Heartbeat(HeartbeatData { magic, node_id }))
}

/// Deserialize TimeSync payload from buffer
fn deserialize_timesync(data: &[u8], payload_offset: usize) -> Result<SensorPayload> {
    let required_size = HEADER_SIZE + TimeSyncData::SERIALIZED_SIZE;
    if data.len() < required_size {
        return Err(AirCommError::InvalidMessage);
    }

    let session_time_us = LittleEndian::read_u64(&data[payload_offset..]);

    Ok(SensorPayload::TimeSync(TimeSyncData { session_time_us }))
}
