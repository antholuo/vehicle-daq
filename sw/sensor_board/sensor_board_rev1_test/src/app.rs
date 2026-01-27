/// app.rs
/// responsible for starting the "app" and setting any necessary configs
use embassy_time::{Duration, Instant, Timer};
#[cfg(feature = "hmi")]
use esp_hal::gpio::Output;

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

#[cfg(feature = "wifi")]
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
#[cfg(feature = "wifi")]
use embassy_sync::channel::Channel;

use crate::BoardPeripherals;
#[cfg(feature = "wifi")]
use crate::EspNowMode;
#[cfg(feature = "imu")]
use crate::SharedSpiDevice;
#[cfg(feature = "wifi")]
use crate::aircomm::{AirCommTransceiver, BROADCAST, HeartbeatData, SensorMessage, SensorPayload};
#[cfg(feature = "gps")]
use crate::gps::{init_gps, start_gps};
#[cfg(feature = "hmi")]
use crate::hmi::{neopixel, start_hmi};
#[cfg(feature = "imu")]
use crate::imu::start_imu;
#[cfg(all(feature = "wifi", feature = "usb"))]
use crate::types::NodeId;
#[cfg(feature = "usb")]
use crate::usb::{
    MAX_USB_MESSAGE_SIZE, UsbError, UsbSerial, format_mac, serialize_forwarded_message,
    wait_for_usb_host_timeout,
};

/// Capacity of the sensor data channel
/// Allows buffering sensor samples while ESP-NOW sends
#[cfg(feature = "wifi")]
const SENSOR_CHANNEL_CAPACITY: usize = 8;

/// Sensor data channel for inter-task communication
/// Sensors push data here, ESP-NOW sender task consumes and transmits
#[cfg(feature = "wifi")]
static SENSOR_CHANNEL: Channel<CriticalSectionRawMutex, SensorPayload, SENSOR_CHANNEL_CAPACITY> =
    Channel::new();

pub async fn app_run<B: BoardPeripherals>(spawner: embassy_executor::Spawner, mut board: B) {
    info!("app is starting execution now");

    // Initialize global program start time used for unified logging timestamps
    crate::timebase::set_program_start();

    #[cfg(feature = "hmi")]
    let mut user_led = board.take_user_led();
    #[cfg(feature = "hmi")]
    trace!("User Led initialized!");

    #[cfg(feature = "hmi")]
    let mut neopixel = board.take_neopixel();
    #[cfg(feature = "hmi")]
    trace!("NeoPixel initialized!");

    // Startup delay: show rainbow-ish cycle on NeoPixel while waiting
    #[cfg(feature = "hmi")]
    {
        use crate::hmi::Color;
        trace!("Startup delay: showing neopixel rainbow for 3s");
        let startup_duration = Duration::from_secs(3);
        let step = Duration::from_millis(80);
        let steps = (startup_duration.as_millis() / step.as_millis()) as u32;
        // wheel function
        fn wheel(pos: u8) -> (u8, u8, u8) {
            if pos < 85 {
                (255 - pos * 3, pos * 3, 0)
            } else if pos < 170 {
                let pos = pos - 85;
                (0, 255 - pos * 3, pos * 3)
            } else {
                let pos = pos - 170;
                (pos * 3, 0, 255 - pos * 3)
            }
        }
        for i in 0..=steps {
            let pos = ((i * 256 / (steps.max(1))) % 256) as u8;
            let (r, g, b) = wheel(pos);
            neopixel
                .set_color_with_brightness(Color::Custom(r, g, b), 50)
                .await;
            Timer::after(step).await;
        }
    }

    #[cfg(feature = "hmi")]
    let _disp_spi_device = board.take_disp_spi_device();
    #[cfg(feature = "hmi")]
    trace!("DISPLAY_SPI device taken");

    #[cfg(feature = "imu")]
    let imu_spi_device = board.take_imu_spi_device();
    #[cfg(feature = "imu")]
    trace!("IMU_SPI device taken");

    #[cfg(feature = "gps")]
    let gps2_uart = board.take_gps2_uart();
    #[cfg(feature = "gps")]
    trace!("Gps2_Uart initialized!");

    info!("all periphs taken");

    // Sensor tasks with callbacks for data
    #[cfg(feature = "imu")]
    spawner
        .spawn(start_imu_task(imu_spi_device))
        .expect("imu task did not spawn");

    #[cfg(feature = "gps")]
    spawner
        .spawn(start_gps_task(gps2_uart))
        .expect("GPS task did not spawn");

    // WiFi/ESP-NOW tasks - spawn based on board's configured mode
    #[cfg(feature = "wifi")]
    {
        let mode = board.espnow_mode();
        if let Some(wifi) = board.take_wifi() {
            match AirCommTransceiver::new(wifi.esp_now) {
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
                        #[cfg(feature = "usb")]
                        EspNowMode::Bridge => {
                            info!("ESP-NOW mode: Bridge (receive + forward to USB)");
                            if let Some(usb_tx) = board.take_usb_serial_tx() {
                                let usb_serial = UsbSerial::new(usb_tx);
                                info!("[BRIDGE] Waiting for USB host before starting");
                                // Wait for USB host with neopixel animation for visual feedback
                                {
                                    const USB_DEVICE_INT_RAW: *const u32 =
                                        0x6000_f008 as *const u32;
                                    const SOF_INT_MASK: u32 = 0b10;
                                    let start = Instant::now();
                                    let timeout = Duration::from_secs(3);
                                    let step = Duration::from_millis(80);
                                    let mut i: u32 = 0;
                                    let mut host_ready = false;
                                    // simple wheel fn
                                    fn wheel(pos: u8) -> (u8, u8, u8) {
                                        if pos < 85 {
                                            (255 - pos * 3, pos * 3, 0)
                                        } else if pos < 170 {
                                            let pos = pos - 85;
                                            (0, 255 - pos * 3, pos * 3)
                                        } else {
                                            let pos = pos - 170;
                                            (pos * 3, 0, 255 - pos * 3)
                                        }
                                    }
                                    let total_steps =
                                        (timeout.as_millis() / step.as_millis()) as u32;
                                    while start.elapsed() < timeout {
                                        let connected = unsafe {
                                            (USB_DEVICE_INT_RAW.read_volatile() & SOF_INT_MASK) != 0
                                        };
                                        if connected {
                                            host_ready = true;
                                            break;
                                        }
                                        let pos = ((i * 256 / (total_steps.max(1))) % 256) as u8;
                                        let (r, g, b) = wheel(pos);
                                        neopixel
                                            .set_color_with_brightness(
                                                crate::hmi::Color::Custom(r, g, b),
                                                50,
                                            )
                                            .await;
                                        Timer::after(step).await;
                                        i = i.wrapping_add(1);
                                    }
                                    if host_ready {
                                        info!("[BRIDGE] USB host detected - starting bridge task");
                                    } else {
                                        warn!("[BRIDGE] USB host not detected - starting anyway");
                                    }
                                }
                                spawner
                                    .spawn(espnow_bridge_task(
                                        transceiver,
                                        wifi.controller,
                                        usb_serial,
                                    ))
                                    .expect("ESP-NOW bridge task did not spawn");
                            } else {
                                warn!(
                                    "USB Serial not available - falling back to transceiver mode"
                                );
                                spawner
                                    .spawn(espnow_transceiver_task(transceiver, wifi.controller))
                                    .expect("ESP-NOW transceiver task did not spawn");
                            }
                        }
                        #[cfg(not(feature = "usb"))]
                        EspNowMode::Bridge => {
                            warn!(
                                "Bridge mode requires USB feature - falling back to transceiver mode"
                            );
                            spawner
                                .spawn(espnow_transceiver_task(transceiver, wifi.controller))
                                .expect("ESP-NOW transceiver task did not spawn");
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to create AirComm transceiver: {:?}", e);
                }
            }
        } else {
            warn!("WiFi not available - skipping wireless communication");
        }
    }


    // FIXME: Moving this here so that esp-now can be handled within app.rs
    // NOTE: this is not correct behaviour, but it'll be ok ish for now
    #[cfg(feature = "hmi")]
    spawner
        .spawn(start_hmi_task(user_led, neopixel))
        .expect("HMI task did not spawn");


    loop {
        embassy_time::Timer::after_secs(1).await
    }
}

#[cfg(feature = "hmi")]
#[embassy_executor::task]
async fn start_hmi_task(user_led: Output<'static>, neopixel: neopixel::NeoPixel<'static>) {
    // Task configuration
    let led_rate_hz: u32 = 1;
    let neopixel_brightness: u8 = 10;

    start_hmi(user_led, led_rate_hz, neopixel, neopixel_brightness).await;
}

#[cfg(feature = "imu")]
#[embassy_executor::task]
async fn start_imu_task(imu_spi_device: SharedSpiDevice) {
    info!("IMU TASK BEING SPAWNED");

    // Callback: send IMU data to channel for transmission
    #[cfg(feature = "wifi")]
    let on_imu_data = |data: crate::types::ImuData| {
        if SENSOR_CHANNEL.try_send(SensorPayload::Imu(data)).is_err() {
            warn!("[IMU] Channel full, dropping sample");
        } else {
            trace!("[IMU] Sent data to channel");
        }
    };

    // No-op callback when wifi is disabled
    #[cfg(not(feature = "wifi"))]
    let on_imu_data = |_data: crate::types::ImuData| {
        // Data is logged in start_imu, nothing else to do
    };

    start_imu(imu_spi_device, on_imu_data).await;
}

#[cfg(feature = "gps")]
#[embassy_executor::task]
async fn start_gps_task(mut gps2_uart: esp_hal::uart::Uart<'static, esp_hal::Async>) {
    gps2_uart = init_gps(gps2_uart).await;

    // Callback: send GPS data to channel for transmission
    #[cfg(feature = "wifi")]
    let on_gps_data = |data: crate::types::GpsData| {
        if SENSOR_CHANNEL.try_send(SensorPayload::Gps(data)).is_err() {
            warn!("[GPS] Channel full, dropping sample");
        } else {
            trace!("[GPS] Sent data to channel");
        }
    };

    // No-op callback when wifi is disabled
    #[cfg(not(feature = "wifi"))]
    let on_gps_data = |_data: crate::types::GpsData| {
        // Data is logged in start_gps, nothing else to do
    };

    start_gps(gps2_uart, on_gps_data).await;
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
#[cfg(feature = "wifi")]
#[embassy_executor::task]
async fn espnow_transceiver_task(
    mut transceiver: AirCommTransceiver<'static>,
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
#[cfg(feature = "wifi")]
#[embassy_executor::task]
async fn espnow_sender_task(
    mut transceiver: AirCommTransceiver<'static>,
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
                    SensorPayload::Imu(data) => {
                        info!(
                            "[ESP-NOW TX] Sending IMU data, timestamp_us={}",
                            timestamp_us
                        );
                        transceiver.send_imu(timestamp_us, data, &BROADCAST).await
                    }
                    SensorPayload::Gps(data) => {
                        info!(
                            "[ESP-NOW TX] Sending GPS data, timestamp_us={}",
                            timestamp_us
                        );
                        transceiver.send_gps(timestamp_us, data, &BROADCAST).await
                    }
                    SensorPayload::Heartbeat(data) => {
                        debug!("[ESP-NOW TX] Sending Heartbeat");
                        transceiver
                            .send_heartbeat(timestamp_us, data, &BROADCAST)
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
    mut transceiver: AirCommTransceiver<'static>,
    _wifi_controller: esp_radio::wifi::WifiController<'static>,
    mut usb_serial: UsbSerial,
) {
    info!("[BRIDGE] Bridge task started (ESP-NOW -> USB)");

    // Buffer for serializing messages
    let mut msg_buffer = [0u8; MAX_USB_MESSAGE_SIZE];

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
                let node_id = NodeId::default();

                // Serialize the message
                match serialize_forwarded_message(
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
                                    format_mac(&src_mac),
                                    messages_forwarded
                                );
                            }
                            Err(UsbError::NotReady) => {
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

#[cfg(feature = "wifi")]
async fn send_heartbeat(transceiver: &mut AirCommTransceiver<'static>) {
    let timestamp_us = Instant::now().as_micros();
    let heartbeat = HeartbeatData::default();

    match transceiver
        .send_heartbeat(timestamp_us, &heartbeat, &BROADCAST)
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

#[cfg(feature = "wifi")]
fn handle_received_message(msg: &SensorMessage) {
    let src = msg.src_address;
    let timestamp = msg.timestamp_us;

    match &msg.payload {
        SensorPayload::Heartbeat(data) => {
            info!(
                "[ESP-NOW RX] Heartbeat from {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X} | magic=0x{:02X}, time={}us",
                src[0], src[1], src[2], src[3], src[4], src[5], data.magic, timestamp
            );
        }
        SensorPayload::Imu(data) => {
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
        SensorPayload::Gps(data) => {
            info!(
                "[ESP-NOW RX] GPS from {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X} | time={}us, lat={:.6}, lon={:.6}",
                src[0], src[1], src[2], src[3], src[4], src[5], timestamp, data.lat, data.lon
            );
        }
    }
}
