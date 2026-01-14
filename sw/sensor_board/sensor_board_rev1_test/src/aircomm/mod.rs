//! Air Communication Module
//!
//! This module provides a high-level interface for wireless communication
//! between ESP devices using ESP-NOW protocol. It wraps esp-hal's ESP-NOW
//! functionality to provide easy-to-use methods for sending and receiving
//! sensor data (IMU, GPS, etc.).

mod receiver;
mod sender;

pub mod error;
pub mod message;
pub mod protocol;

pub use error::{AirCommError, Result};
pub use message::{GpsData, HeartbeatData, ImuData, MessageType, SensorMessage, SensorPayload};
pub use sender::BROADCAST;

use esp_radio::esp_now::{EspNow, EspNowWifiInterface, PeerInfo};

/// Main transceiver for wireless sensor data communication
///
/// This is the primary interface for ESP-NOW communication. It handles
/// both sending and receiving sensor data, as well as peer management.
pub struct AirCommTransceiver<'a> {
    esp_now: EspNow<'a>,
}

impl<'a> AirCommTransceiver<'a> {
    /// Initialize the air communication transceiver
    pub fn new(esp_now: EspNow<'a>) -> Result<Self> {
        Ok(Self { esp_now })
    }

    /// Send a heartbeat message
    pub async fn send_heartbeat(
        &mut self,
        timestamp_us: u64,
        data: &HeartbeatData,
        peer_addr: &[u8; 6],
    ) -> Result<()> {
        sender::send_heartbeat(&mut self.esp_now, timestamp_us, data, peer_addr).await
    }

    /// Send IMU data to a peer
    pub async fn send_imu(
        &mut self,
        timestamp_us: u64,
        data: &ImuData,
        peer_addr: &[u8; 6],
    ) -> Result<()> {
        sender::send_imu(&mut self.esp_now, timestamp_us, data, peer_addr).await
    }

    /// Send GPS data to a peer
    pub async fn send_gps(
        &mut self,
        timestamp_us: u64,
        data: &GpsData,
        peer_addr: &[u8; 6],
    ) -> Result<()> {
        sender::send_gps(&mut self.esp_now, timestamp_us, data, peer_addr).await
    }

    /// Receive sensor data (suspends until data arrives)
    pub async fn receive(&mut self) -> Result<SensorMessage> {
        receiver::receive(&mut self.esp_now).await
    }

    /// Add a peer to the peer list
    ///
    /// Before sending to a specific peer, you must add them to the peer list.
    /// Broadcasting does not require adding peers.
    pub fn add_peer(&mut self, peer_addr: &[u8; 6]) -> Result<()> {
        let peer_info = PeerInfo {
            peer_address: *peer_addr,
            lmk: None,
            channel: None,
            interface: EspNowWifiInterface::Station,
            encrypt: false,
        };
        self.esp_now.add_peer(peer_info)?;
        Ok(())
    }

    /// Remove a peer from the peer list
    pub fn remove_peer(&mut self, peer_addr: &[u8; 6]) -> Result<()> {
        self.esp_now.remove_peer(peer_addr)?;
        Ok(())
    }

    /// Check if a peer exists in the peer list
    pub fn peer_exists(&self, peer_addr: &[u8; 6]) -> bool {
        self.esp_now.peer_exists(peer_addr)
    }
}
