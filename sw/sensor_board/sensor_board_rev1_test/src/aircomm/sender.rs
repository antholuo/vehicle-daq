//! Sender functions for ESP-NOW data transmission
//!
//! This module contains the core sending logic for transmitting sensor data
//! over ESP-NOW. These functions are called by AirCommTransceiver.

use super::error::Result;
use super::message::*;
use super::protocol::{MAX_PAYLOAD_SIZE, serialize_gps, serialize_heartbeat, serialize_imu};
use esp_radio::esp_now::EspNow;

/// Broadcast MAC address for sending to all peers
pub const BROADCAST: [u8; 6] = [0xFF; 6];

/// Send heartbeat message asynchronously
pub(crate) async fn send_heartbeat(
    esp_now: &mut EspNow<'_>,
    timestamp_us: u64,
    data: &HeartbeatData,
    peer_addr: &[u8; 6],
) -> Result<()> {
    let mut buffer = [0u8; MAX_PAYLOAD_SIZE];
    let size = serialize_heartbeat(timestamp_us, data, &mut buffer)?;

    esp_now.send_async(peer_addr, &buffer[..size]).await?;

    Ok(())
}

/// Send IMU data asynchronously
pub(crate) async fn send_imu(
    esp_now: &mut EspNow<'_>,
    timestamp_us: u64,
    data: &ImuData,
    peer_addr: &[u8; 6],
) -> Result<()> {
    let mut buffer = [0u8; MAX_PAYLOAD_SIZE];
    let size = serialize_imu(timestamp_us, data, &mut buffer)?;

    esp_now.send_async(peer_addr, &buffer[..size]).await?;

    Ok(())
}

/// Send GPS data asynchronously
pub(crate) async fn send_gps(
    esp_now: &mut EspNow<'_>,
    timestamp_us: u64,
    data: &GpsData,
    peer_addr: &[u8; 6],
) -> Result<()> {
    let mut buffer = [0u8; MAX_PAYLOAD_SIZE];
    let size = serialize_gps(timestamp_us, data, &mut buffer)?;

    esp_now.send_async(peer_addr, &buffer[..size]).await?;

    Ok(())
}
