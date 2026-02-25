use crate::aircomm::{MessageType, SensorPayload};
/// USB Protocol for sensor data forwarding
///
/// Defines the message format for forwarding ESP-NOW received data to a host.
///
/// Message format:
/// ```text
/// [MAC Address (6B)][NodeId (2B)][Timestamp (8B)][MessageType (1B)][Payload (variable)]
/// ```
///
/// The entire message is COBS-encoded and terminated with 0x00.
use crate::types::NodeId;
use byteorder::{ByteOrder, LittleEndian};

/// Header size: MAC (6) + NodeId (2) + Timestamp (8) + MsgType (1) = 17 bytes
pub const HEADER_SIZE: usize = 17;

/// Maximum payload size (GPS is largest at ~50 bytes)
pub const MAX_PAYLOAD_SIZE: usize = 64;

/// Total maximum message size
pub const MAX_USB_MESSAGE_SIZE: usize = HEADER_SIZE + MAX_PAYLOAD_SIZE;

/// Serialize a forwarded sensor message for USB transmission
///
/// # Arguments
/// * `src_mac` - Source MAC address of the ESP-NOW sender (6 bytes)
/// * `node_id` - Node identifier (position + instance)
/// * `timestamp_us` - Timestamp in microseconds
/// * `payload` - Sensor payload to forward
/// * `buffer` - Output buffer (must be at least MAX_USB_MESSAGE_SIZE)
///
/// # Returns
/// Number of bytes written to buffer, or error
pub fn serialize_forwarded_message(
    src_mac: &[u8; 6],
    node_id: &NodeId,
    timestamp_us: u64,
    payload: &SensorPayload,
    buffer: &mut [u8],
) -> Result<usize, ForwardError> {
    if buffer.len() < MAX_USB_MESSAGE_SIZE {
        return Err(ForwardError::BufferTooSmall);
    }

    let mut offset = 0;

    // MAC address (6 bytes)
    buffer[offset..offset + 6].copy_from_slice(src_mac);
    offset += 6;

    // NodeId (2 bytes)
    let node_bytes = node_id.to_bytes();
    buffer[offset..offset + 2].copy_from_slice(&node_bytes);
    offset += 2;

    // Timestamp (8 bytes, little-endian)
    LittleEndian::write_u64(&mut buffer[offset..], timestamp_us);
    offset += 8;

    // Message type and payload
    match payload {
        SensorPayload::Imu(imu) => {
            buffer[offset] = MessageType::Imu.to_u8();
            offset += 1;

            // IMU payload: 6 x f32 = 24 bytes
            LittleEndian::write_f32(&mut buffer[offset..], imu.accel_x);
            offset += 4;
            LittleEndian::write_f32(&mut buffer[offset..], imu.accel_y);
            offset += 4;
            LittleEndian::write_f32(&mut buffer[offset..], imu.accel_z);
            offset += 4;
            LittleEndian::write_f32(&mut buffer[offset..], imu.gyro_x);
            offset += 4;
            LittleEndian::write_f32(&mut buffer[offset..], imu.gyro_y);
            offset += 4;
            LittleEndian::write_f32(&mut buffer[offset..], imu.gyro_z);
            offset += 4;
        }
        SensorPayload::Gps(gps) => {
            buffer[offset] = MessageType::Gps.to_u8();
            offset += 1;

            // GPS payload: lat(8) + lon(8) + alt(4) + speed(4) + heading(2) + time(9) = 35 bytes
            LittleEndian::write_f64(&mut buffer[offset..], gps.lat);
            offset += 8;
            LittleEndian::write_f64(&mut buffer[offset..], gps.lon);
            offset += 8;
            LittleEndian::write_f32(&mut buffer[offset..], gps.alt);
            offset += 4;
            LittleEndian::write_f32(&mut buffer[offset..], gps.speed_kts);
            offset += 4;
            LittleEndian::write_u16(&mut buffer[offset..], gps.heading);
            offset += 2;

            // GPS time
            LittleEndian::write_u16(&mut buffer[offset..], gps.utc_time.year);
            offset += 2;
            buffer[offset] = gps.utc_time.month;
            offset += 1;
            buffer[offset] = gps.utc_time.day;
            offset += 1;
            buffer[offset] = gps.utc_time.hours;
            offset += 1;
            buffer[offset] = gps.utc_time.minutes;
            offset += 1;
            buffer[offset] = gps.utc_time.seconds;
            offset += 1;
            LittleEndian::write_u16(&mut buffer[offset..], gps.utc_time.millis);
            offset += 2;
        }
        SensorPayload::Heartbeat(hb) => {
            buffer[offset] = MessageType::Heartbeat.to_u8();
            offset += 1;

            // Heartbeat payload: just magic byte
            buffer[offset] = hb.magic;
            offset += 1;
        }
        SensorPayload::TimeSync(_) => {
            // TimeSync is not forwarded to USB host; bridge handles it internally
            return Err(ForwardError::UnsupportedPayload);
        }
    }

    Ok(offset)
}

/// Format MAC address as string for logging
pub fn format_mac(mac: &[u8; 6]) -> heapless::String<18> {
    use core::fmt::Write;
    let mut s = heapless::String::new();
    let _ = write!(
        s,
        "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
        mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
    );
    s
}

/// Errors that can occur during message forwarding
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForwardError {
    /// Output buffer is too small
    BufferTooSmall,
    /// Payload type cannot be forwarded over USB
    UnsupportedPayload,
}

// =========================================================================
// USB RX command protocol  (RPi -> Bridge)
//
// Commands are COBS-framed, same as the TX direction.
// Format: [CommandType: 1B][Payload: variable]
// =========================================================================

/// USB command type byte values (RPi -> Bridge)
pub const CMD_TIME_SYNC: u8 = 0x01;

/// Parsed USB command from the RPi host
#[derive(Debug, Clone, Copy)]
pub enum UsbCommand {
    /// TimeSync: RPi sends its session-elapsed time (microseconds)
    TimeSync { session_time_us: u64 },
}

/// Parse a COBS-decoded command buffer into a `UsbCommand`.
///
/// Minimum size: 1 (command type) + 8 (payload for TimeSync) = 9 bytes.
pub fn parse_usb_command(buffer: &[u8]) -> Result<UsbCommand, ForwardError> {
    if buffer.is_empty() {
        return Err(ForwardError::BufferTooSmall);
    }

    match buffer[0] {
        CMD_TIME_SYNC => {
            if buffer.len() < 9 {
                return Err(ForwardError::BufferTooSmall);
            }
            let session_time_us = LittleEndian::read_u64(&buffer[1..]);
            Ok(UsbCommand::TimeSync { session_time_us })
        }
        _ => Err(ForwardError::UnsupportedPayload),
    }
}
