use embassy_time::{Duration, Instant};
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;

use crate::{EspNowMode, aircomm};
#[cfg(feature = "usb")]
use crate::{types, usb};

/// Capacity of the sensor data channel
/// Allows buffering sensor samples while ESP-NOW sends
const SENSOR_CHANNEL_CAPACITY: usize = 8;

/// Sensor data channel for inter-task communication
/// Sensors push data here, ESP-NOW sender task consumes and transmits
pub(crate) static SENSOR_CHANNEL: Channel<
    CriticalSectionRawMutex,
    aircomm::SensorPayload,
    SENSOR_CHANNEL_CAPACITY,
> = Channel::new();

#[cfg(feature = "usb")]
use esp_hal::Async;
#[cfg(feature = "usb")]
use esp_hal::usb_serial_jtag::UsbSerialJtagTx;

#[embassy_executor::task]
pub async fn start_comms_task(
    spawner: embassy_executor::Spawner,
    espnow_mode: EspNowMode,
    wifi_resources: Option<crate::WifiResources>,
    #[cfg(feature = "usb")] usb_serial_tx: Option<UsbSerialJtagTx<'static, Async>>,
) {
    let mode = espnow_mode;
    if let Some(wifi) = wifi_resources {
        match aircomm::AirCommTransceiver::new(wifi.esp_now) {
            Ok(transceiver) => {
                info!("AirComm transceiver initialized");

                match mode {
                    EspNowMode::Sender => {
                        info!("ESP-NOW mode: Sender (transmit sensor data)");
                        spawner
                            .spawn(espnow_sender_task(transceiver, wifi.controller))
                            .expect("ESP-NOW sender task did not spawn");
                    }
                    EspNowMode::Transceiver => {
                        info!("ESP-NOW mode: Transceiver (receive + heartbeat)");
                        spawner
                            .spawn(espnow_transceiver_task(transceiver, wifi.controller))
                            .expect("ESP-NOW transceiver task did not spawn");
                    }
                    EspNowMode::Bridge => {
                        info!("ESP-NOW mode: Bridge (receive + forward to USB)");
                        #[cfg(feature = "usb")]
                        if let Some(usb_tx) = usb_serial_tx {
                            let usb_serial = usb::UsbSerial::new(usb_tx);
                            info!("[BRIDGE] Waiting for USB host before starting");
                            spawner
                                .spawn(espnow_bridge_task(transceiver, wifi.controller, usb_serial))
                                .expect("ESP-NOW bridge task did not spawn");
                        } else {
                            warn!("USB Serial not available - falling back to transceiver mode");
                            spawner
                                .spawn(espnow_transceiver_task(transceiver, wifi.controller))
                                .expect("ESP-NOW transceiver task did not spawn");
                        }
                        #[cfg(not(feature = "usb"))]
                        {
                            warn!(
                                "Bridge mode requires USB feature - falling back to transceiver mode"
                            );
                            spawner
                                .spawn(espnow_transceiver_task(transceiver, wifi.controller))
                                .expect("ESP-NOW transceiver task did not spawn");
                        }
                    }
                }
            }
            Err(e) => {
                warn!("Failed to create AirComm transceiver: {:?}", e);
            }
        }
    }
}

// =============================================================================
// ESP-NOW Tasks
// =============================================================================

/// ESP-NOW transceiver task
///
/// Handles both receiving ESP-NOW messages and sending periodic heartbeats.
/// Used by bridge/receiver nodes (e.g., DevKit-C) for testing or forwarding data.
///
/// Note: `_wifi_controller` must be kept alive for ESP-NOW to function properly.
#[embassy_executor::task]
async fn espnow_transceiver_task(
    mut transceiver: aircomm::AirCommTransceiver<'static>,
    _wifi_controller: esp_radio::wifi::WifiController<'static>,
) {
    info!("[ESP-NOW] Transceiver task started (RX + TX heartbeats)");

    const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(1);

    // Send first heartbeat immediately
    send_heartbeat(&mut transceiver).await;
    let mut last_heartbeat = Instant::now();

    loop {
        // Calculate how long until next heartbeat is due
        let elapsed = last_heartbeat.elapsed();
        let timeout = if elapsed >= HEARTBEAT_INTERVAL {
            Duration::from_millis(0)
        } else {
            HEARTBEAT_INTERVAL - elapsed
        };

        // Try to receive with timeout
        match embassy_time::with_timeout(timeout, transceiver.receive()).await {
            Ok(Ok(msg)) => {
                // Successfully received a message
                handle_received_message(&msg);
            }
            Ok(Err(e)) => {
                // Receive error
                warn!("[ESP-NOW RX] Error: {:?}", e);
            }
            Err(_) => {
                // Timeout - no message received, that's fine
            }
        }

        // Check if it's time to send heartbeat
        if last_heartbeat.elapsed() >= HEARTBEAT_INTERVAL {
            send_heartbeat(&mut transceiver).await;
            last_heartbeat = Instant::now();
        }
    }
}

/// ESP-NOW sender task (default)
///
/// Receives sensor data from the channel and transmits via ESP-NOW.
/// Also sends periodic heartbeats to indicate the node is alive.
///
/// Note: `_wifi_controller` must be kept alive for ESP-NOW to function properly.
#[embassy_executor::task]
async fn espnow_sender_task(
    mut transceiver: aircomm::AirCommTransceiver<'static>,
    _wifi_controller: esp_radio::wifi::WifiController<'static>,
) {
    info!("[ESP-NOW] Sender task started");

    const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(1);

    // Send first heartbeat immediately
    send_heartbeat(&mut transceiver).await;
    let mut last_heartbeat = Instant::now();

    loop {
        // Calculate how long until next heartbeat is due
        let elapsed = last_heartbeat.elapsed();
        let timeout = if elapsed >= HEARTBEAT_INTERVAL {
            Duration::from_millis(0)
        } else {
            HEARTBEAT_INTERVAL - elapsed
        };

        // Try to receive sensor data from channel with timeout
        match embassy_time::with_timeout(timeout, SENSOR_CHANNEL.receive()).await {
            Ok(payload) => {
                // Got sensor data, transmit it
                let timestamp_us = Instant::now().as_micros();

                let result = match &payload {
                    aircomm::SensorPayload::Imu(data) => {
                        info!(
                            "[ESP-NOW TX] Sending IMU data, timestamp_us={}",
                            timestamp_us
                        );
                        transceiver
                            .send_imu(timestamp_us, data, &aircomm::BROADCAST)
                            .await
                    }
                    aircomm::SensorPayload::Gps(data) => {
                        info!(
                            "[ESP-NOW TX] Sending GPS data, timestamp_us={}",
                            timestamp_us
                        );
                        transceiver
                            .send_gps(timestamp_us, data, &aircomm::BROADCAST)
                            .await
                    }
                    aircomm::SensorPayload::Heartbeat(data) => {
                        debug!("[ESP-NOW TX] Sending Heartbeat");
                        transceiver
                            .send_heartbeat(timestamp_us, data, &aircomm::BROADCAST)
                            .await
                    }
                };

                match result {
                    Ok(()) => {
                        trace!("[ESP-NOW TX] Sent {:?}", payload.message_type());
                    }
                    Err(e) => {
                        warn!("[ESP-NOW TX] Send failed: {:?}", e);
                    }
                }
            }
            Err(_) => {
                // Timeout - no sensor data, that's fine
            }
        }

        // Check if it's time to send heartbeat
        if last_heartbeat.elapsed() >= HEARTBEAT_INTERVAL {
            send_heartbeat(&mut transceiver).await;
            last_heartbeat = Instant::now();
        }
    }
}

/// ESP-NOW bridge task
///
/// Receives ESP-NOW messages from sensor nodes and forwards them to USB.
/// This is the core of the sensor bridge functionality.
///
/// Message format over USB (COBS-framed):
/// [MAC (6B)][NodeId (2B)][Timestamp (8B)][MsgType (1B)][Payload]
///
/// Note: `_wifi_controller` must be kept alive for ESP-NOW to function properly.
#[cfg(all(feature = "wifi", feature = "usb"))]
#[embassy_executor::task]
async fn espnow_bridge_task(
    mut transceiver: aircomm::AirCommTransceiver<'static>,
    _wifi_controller: esp_radio::wifi::WifiController<'static>,
    mut usb_serial: usb::UsbSerial,
) {
    info!("[BRIDGE] Bridge task started (ESP-NOW -> USB)");

    // Buffer for serializing messages
    let mut msg_buffer = [0u8; usb::MAX_USB_MESSAGE_SIZE];

    // Statistics
    let mut messages_forwarded: u32 = 0;
    let mut errors: u32 = 0;

    loop {
        // Wait for incoming ESP-NOW message
        match transceiver.receive().await {
            Ok(msg) => {
                let src_mac: [u8; 6] = msg.src_address;
                let timestamp_us = msg.timestamp_us;

                // For now, use a default NodeId - in production this would be
                // looked up from a MAC -> NodeId mapping table
                let node_id = types::NodeId::default();

                // Serialize the message
                match usb::serialize_forwarded_message(
                    &src_mac,
                    &node_id,
                    timestamp_us,
                    &msg.payload,
                    &mut msg_buffer,
                ) {
                    Ok(len) => {
                        // Send via USB with COBS framing
                        match usb_serial.write_framed(&msg_buffer[..len]).await {
                            Ok(()) => {
                                messages_forwarded += 1;
                                trace!(
                                    "[BRIDGE] Forwarded {:?} from {} (total: {})",
                                    msg.payload.message_type(),
                                    usb::format_mac(&src_mac),
                                    messages_forwarded
                                );
                            }
                            Err(usb::UsbError::NotReady) => {
                                // Drop silently when USB host is not ready.
                            }
                            Err(e) => {
                                errors += 1;
                                warn!("[BRIDGE] USB write error: {:?}", e);
                            }
                        }
                    }
                    Err(e) => {
                        errors += 1;
                        warn!("[BRIDGE] Serialize error: {:?}", e);
                    }
                }

                // Log periodically
                if messages_forwarded % 100 == 0 && messages_forwarded > 0 {
                    info!(
                        "[BRIDGE] Stats: {} forwarded, {} errors",
                        messages_forwarded, errors
                    );
                }
            }
            Err(e) => {
                warn!("[BRIDGE] Receive error: {:?}", e);
            }
        }
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

async fn send_heartbeat(transceiver: &mut aircomm::AirCommTransceiver<'static>) {
    let timestamp_us = Instant::now().as_micros();
    let heartbeat = aircomm::HeartbeatData::default();

    match transceiver
        .send_heartbeat(timestamp_us, &heartbeat, &aircomm::BROADCAST)
        .await
    {
        Ok(()) => {
            info!("[ESP-NOW TX] Heartbeat sent (time={}us)", timestamp_us);
        }
        Err(e) => {
            warn!("[ESP-NOW TX] Send failed: {:?}", e);
        }
    }
}

fn handle_received_message(msg: &aircomm::SensorMessage) {
    let src = msg.src_address;
    let timestamp = msg.timestamp_us;

    match &msg.payload {
        aircomm::SensorPayload::Heartbeat(data) => {
            info!(
                "[ESP-NOW RX] Heartbeat from {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X} | magic=0x{:02X}, time={}us",
                src[0], src[1], src[2], src[3], src[4], src[5], data.magic, timestamp
            );
        }
        aircomm::SensorPayload::Imu(data) => {
            info!(
                "[ESP-NOW RX] IMU from {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X} | time={}us, accel=[{:.3}, {:.3}, {:.3}]",
                src[0],
                src[1],
                src[2],
                src[3],
                src[4],
                src[5],
                timestamp,
                data.accel_x,
                data.accel_y,
                data.accel_z
            );
        }
        aircomm::SensorPayload::Gps(data) => {
            info!(
                "[ESP-NOW RX] GPS from {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X} | time={}us, lat={:.6}, lon={:.6}",
                src[0], src[1], src[2], src[3], src[4], src[5], timestamp, data.lat, data.lon
            );
        }
    }
}
