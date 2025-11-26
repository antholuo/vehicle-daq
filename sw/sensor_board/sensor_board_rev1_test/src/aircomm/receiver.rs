//! Receiver functions for ESP-NOW data reception
//!
//! This module contains the core receiving logic for receiving and parsing
//! sensor data from ESP-NOW. These functions are called by AirCommTransceiver.

use super::error::Result;
use super::message::*;
use super::protocol::*;
use esp_radio::esp_now::EspNow;

/// Receive and parse sensor message asynchronously
///
/// Waits for incoming data and returns the parsed message with timestamp.
pub(crate) async fn receive(esp_now: &mut EspNow<'_>) -> Result<SensorMessage> {
    let received_data = esp_now.receive_async().await;
    let src_address = received_data.info.src_address;
    let message = deserialize(received_data.data(), src_address)?;

    Ok(message)
}
