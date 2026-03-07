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
#[cfg(all(feature = "wifi", feature = "usb"))]
use crate::aircomm::TimeSyncData;
#[cfg(feature = "gps")]
use crate::gps::{init_gps, start_gps};
#[cfg(feature = "hmi")]
use crate::hmi::{neopixel, start_hmi, start_hmi_floating};
#[cfg(feature = "imu")]
use crate::imu::start_imu;
#[cfg(feature = "floating")]
use crate::types::GpsData;
#[cfg(feature = "floating")]
use core::sync::atomic::{AtomicBool, Ordering};
#[cfg(all(feature = "wifi", feature = "usb"))]
use crate::types::NodeId;
#[cfg(feature = "usb")]
use crate::usb::{
    MAX_USB_MESSAGE_SIZE, UsbCommand, UsbError, UsbSerial, UsbSerialRx, format_mac,
    serialize_forwarded_message,
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

/// Floating node: channel for collecting 5 GPS samples when RequestGpsCapture is received
#[cfg(feature = "floating")]
static CAPTURE_CHANNEL: Channel<CriticalSectionRawMutex, GpsData, 5> = Channel::new();
#[cfg(feature = "floating")]
static CAPTURE_MODE: AtomicBool = AtomicBool::new(false);

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
                                let usb_rx = board
                                    .take_usb_serial_rx()
                                    .map(UsbSerialRx::new);
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
                                        usb_rx,
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
    {
        info!("[HMI] Spawning HMI task (user LED + NeoPixel)");
        spawner
            .spawn(start_hmi_task(user_led, neopixel))
            .expect("HMI task did not spawn");
    }


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
/// Applies TimeSync when received from bridge.
///
/// Note: `_wifi_controller` must be kept alive for ESP-NOW to function properly.
#[cfg(feature = "wifi")]
#[embassy_executor::task]
async fn espnow_transceiver_task(
    mut transceiver: AirCommTransceiver<'static>,
    _wifi_controller: esp_radio::wifi::WifiController<'static>,
) {
    info!("[ESP-NOW] Transceiver task started (RX + TX heartbeats, TimeSync)");

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

        match embassy_time::with_timeout(timeout, transceiver.receive()).await {
            Ok(Ok(msg)) => {
                if let SensorPayload::TimeSync(ts_data) = &msg.payload {
                    crate::timebase::apply_sync(ts_data.session_time_us);
                    info!(
                        "[ESP-NOW RX] TimeSync applied: session={}us",
                        ts_data.session_time_us
                    );
                } else {
                    handle_received_message(&msg);
                }
            }
            Ok(Err(e)) => {
                warn!("[ESP-NOW RX] Error: {:?}", e);
            }
            Err(_) => {}
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
/// Also sends periodic heartbeats and polls for incoming TimeSync messages.
///
/// Note: `_wifi_controller` must be kept alive for ESP-NOW to function properly.
#[cfg(feature = "wifi")]
#[embassy_executor::task]
async fn espnow_sender_task(
    mut transceiver: AirCommTransceiver<'static>,
    _wifi_controller: esp_radio::wifi::WifiController<'static>,
) {
    info!("[ESP-NOW] Sender task started (with TimeSync RX)");

    const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(1);
    const TIMESYNC_RX_POLL: Duration = Duration::from_millis(1);

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
                let timestamp_us = crate::timebase::synced_timestamp_us();

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
                    SensorPayload::TimeSync(_) => {
                        // TimeSync payloads are not sent from data nodes
                        Ok(())
                    }
                    SensorPayload::RequestGpsCapture => {
                        // Only bridge sends this; data nodes ignore
                        Ok(())
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

        // Poll for incoming ESP-NOW TimeSync from bridge (very short timeout)
        match embassy_time::with_timeout(TIMESYNC_RX_POLL, transceiver.receive()).await {
            Ok(Ok(msg)) => {
                if let SensorPayload::TimeSync(ts_data) = &msg.payload {
                    crate::timebase::apply_sync(ts_data.session_time_us);
                    info!(
                        "[ESP-NOW RX] TimeSync applied: session={}us",
                        ts_data.session_time_us
                    );
                }
                // Other message types received here are from other nodes; ignore
            }
            Ok(Err(e)) => {
                warn!("[ESP-NOW RX] Error during TimeSync poll: {:?}", e);
            }
            Err(_) => {
                // No incoming message -- expected
            }
        }

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
    mut usb_rx: Option<UsbSerialRx>,
) {
    use heapless::index_map::FnvIndexMap;
    use crate::types::CarPosition;

    info!("[BRIDGE] Bridge task started (ESP-NOW <-> USB, TimeSync enabled)");
    info!("[BRIDGE] Auto-assigning instance numbers based on MAC discovery order");

    const ESPNOW_RX_TIMEOUT: Duration = Duration::from_millis(50);

    let mut msg_buffer = [0u8; MAX_USB_MESSAGE_SIZE];
    let mut mac_to_node_id: FnvIndexMap<[u8; 6], NodeId, 16> = FnvIndexMap::new();
    let mut next_instance_per_position: FnvIndexMap<CarPosition, u8, 16> = FnvIndexMap::new();

    let mut messages_forwarded: u32 = 0;
    let mut errors: u32 = 0;
    let mut timesync_count: u32 = 0;
    /// Collecting 5 GPS from floating node for RequestFloatingGps response
    let mut floating_gps_remaining: u8 = 0;
    let mut floating_gps_mac: Option<[u8; 6]> = None;

    loop {
        // --- Poll USB RX for commands from RPi (non-blocking) ---
        if let Some(ref mut rx) = usb_rx {
            while let Some(cmd) = rx.poll_command() {
                match cmd {
                    UsbCommand::TimeSync { session_time_us } => {
                        timesync_count += 1;
                        crate::timebase::apply_sync(session_time_us);

                        // Broadcast TimeSync to all data nodes
                        let ts = crate::timebase::synced_timestamp_us();
                        let data = TimeSyncData::new(session_time_us);
                        if let Err(e) = transceiver.send_timesync(ts, &data, &BROADCAST).await {
                            warn!("[BRIDGE] TimeSync broadcast failed: {:?}", e);
                        } else {
                            trace!(
                                "[BRIDGE] TimeSync #{}: session={}us, broadcast OK",
                                timesync_count, session_time_us
                            );
                        }
                    }
                    UsbCommand::RequestFloatingGps => {
                        info!("[BRIDGE] received take (RequestFloatingGps)");
                        {
                            let mut hmi = crate::hmi::state::HMI_STATE.0.lock().await;
                            hmi.capture_led_phase = crate::hmi::state::CAPTURE_PHASE_REQUEST_SENT;
                        }
                        let ts = crate::timebase::synced_timestamp_us();
                        if let Err(e) = transceiver.send_request_gps_capture(ts, &BROADCAST).await {
                            warn!("[BRIDGE] RequestGpsCapture broadcast failed: {:?}", e);
                        } else {
                            floating_gps_remaining = 5;
                            floating_gps_mac = None;
                        }
                    }
                    UsbCommand::ArmTrack => {
                        info!("[BRIDGE] received arm (track capture armed)");
                        let mut hmi = crate::hmi::state::HMI_STATE.0.lock().await;
                        hmi.track_armed = true;
                    }
                    UsbCommand::EndTrack => {
                        info!("[BRIDGE] received end (track capture ended)");
                        let mut hmi = crate::hmi::state::HMI_STATE.0.lock().await;
                        hmi.track_armed = false;
                    }
                    UsbCommand::CaptureSuccess => {
                        info!("[BRIDGE] received capture success (show green 1s)");
                        let mut hmi = crate::hmi::state::HMI_STATE.0.lock().await;
                        hmi.capture_led_phase = crate::hmi::state::CAPTURE_PHASE_SUCCESS;
                        hmi.capture_phase_deadline =
                            Some(embassy_time::Instant::now() + Duration::from_secs(1));
                    }
                    UsbCommand::CaptureTimeout => {
                        info!("[BRIDGE] received capture timeout (show rainbow 1s)");
                        let mut hmi = crate::hmi::state::HMI_STATE.0.lock().await;
                        hmi.capture_led_phase = crate::hmi::state::CAPTURE_PHASE_TIMEOUT;
                        hmi.capture_phase_deadline =
                            Some(embassy_time::Instant::now() + Duration::from_secs(1));
                    }
                }
            }
        }

        // --- Receive ESP-NOW with timeout so we regularly poll USB RX ---
        match embassy_time::with_timeout(ESPNOW_RX_TIMEOUT, transceiver.receive()).await {
            Ok(Ok(msg)) => {
                let src_mac: [u8; 6] = msg.src_address;

                // Use synced bridge-receive timestamp for forwarded messages
                let timestamp_us = crate::timebase::synced_timestamp_us();

                // Auto-assign instance numbers based on MAC discovery order
                if let SensorPayload::Heartbeat(heartbeat_data) = &msg.payload {
                    let position = heartbeat_data.node_id.position;

                    if !mac_to_node_id.contains_key(&src_mac) {
                        let instance =
                            *next_instance_per_position.get(&position).unwrap_or(&0);
                        let assigned_node_id = NodeId::new(position, instance);

                        match mac_to_node_id.insert(src_mac, assigned_node_id) {
                            Ok(_) => {
                                let _ = next_instance_per_position
                                    .insert(position, instance + 1);
                            }
                            Err(_) => {
                                warn!(
                                    "[BRIDGE] Failed to store node ID for {} (map full)",
                                    format_mac(&src_mac)
                                );
                            }
                        }
                    } else {
                        let stored_node_id = *mac_to_node_id.get(&src_mac).unwrap();
                        if stored_node_id.position != position {
                            let instance =
                                *next_instance_per_position.get(&position).unwrap_or(&0);
                            let new_node_id = NodeId::new(position, instance);

                            let old_position = stored_node_id.position;
                            let old_instance = stored_node_id.instance;

                            let _ = mac_to_node_id.insert(src_mac, new_node_id);
                            let _ =
                                next_instance_per_position.insert(position, instance + 1);

                            info!(
                                "[BRIDGE] Node position changed: {} -> {}:{} (was {}:{})",
                                format_mac(&src_mac),
                                position.as_str(),
                                instance,
                                old_position.as_str(),
                                old_instance
                            );
                        }
                    }
                }

                // Look up NodeId for this MAC, default to Custom:0 if not found
                let node_id = mac_to_node_id
                    .get(&src_mac)
                    .copied()
                    .unwrap_or_else(|| NodeId::new(CarPosition::Custom, 0));

                // When collecting floating GPS, only forward the 5 GPS from the floating node (first GPS sender)
                let should_forward = if floating_gps_remaining > 0 {
                    if let SensorPayload::Gps(_) = &msg.payload {
                        let is_floating = floating_gps_mac.map(|m| m == src_mac).unwrap_or(true);
                        if is_floating && floating_gps_mac.is_none() {
                            floating_gps_mac = Some(src_mac);
                        }
                        is_floating
                    } else {
                        false
                    }
                } else {
                    true
                };

                if should_forward {
                    match serialize_forwarded_message(
                        &src_mac,
                        &node_id,
                        timestamp_us,
                        &msg.payload,
                        &mut msg_buffer,
                    ) {
                        Ok(len) => {
                            match usb_serial.write_framed(&msg_buffer[..len]).await {
                                Ok(()) => {
                                    messages_forwarded += 1;
                                    if let SensorPayload::Gps(_) = &msg.payload {
                                        if floating_gps_remaining > 0 {
                                            floating_gps_remaining -= 1;
                                            if floating_gps_remaining == 0 {
                                                floating_gps_mac = None;
                                                info!("[BRIDGE] received responses (5 GPS forwarded to RPi)");
                                            }
                                        }
                                    }
                                    trace!(
                                        "[BRIDGE] Forwarded {:?} from {} ({}:{}) (total: {})",
                                        msg.payload.message_type(),
                                        format_mac(&src_mac),
                                        node_id.position.as_str(),
                                        node_id.instance,
                                        messages_forwarded
                                    );
                                }
                                Err(UsbError::NotReady) => {}
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
                }

                // Log periodically
                if messages_forwarded % 100 == 0 && messages_forwarded > 0 {
                    info!(
                        "[BRIDGE] Stats: {} forwarded, {} errors, {} timesyncs",
                        messages_forwarded, errors, timesync_count
                    );
                }
            }
            Ok(Err(e)) => {
                warn!("[BRIDGE] Receive error: {:?}", e);
            }
            Err(_) => {
                // Timeout -- loop back to poll USB RX
            }
        }
    }
}

// =============================================================================
// Floating (GPS-only) node: wait for RequestGpsCapture, reply with 5 GPS samples
// =============================================================================

#[cfg(all(feature = "floating", feature = "wifi"))]
#[embassy_executor::task]
async fn floating_espnow_task(
    mut transceiver: AirCommTransceiver<'static>,
    _wifi_controller: esp_radio::wifi::WifiController<'static>,
) {
    use crate::aircomm::SensorPayload;

    info!("[FLOATING] ESP-NOW task started (wait for RequestGpsCapture, reply with 5 GPS)");

    const COLLECT_TIMEOUT: Duration = Duration::from_millis(800);
    const NUM_SAMPLES: u32 = 5;

    loop {
        match transceiver.receive().await {
            Ok(msg) => {
                if let SensorPayload::RequestGpsCapture = msg.payload {
                    info!("[FLOATING] RequestGpsCapture received, collecting 5 GPS samples");
                    {
                        let mut hmi = crate::hmi::state::HMI_STATE.0.lock().await;
                        hmi.acquiring_gps_capture = true;
                    }
                    CAPTURE_MODE.store(true, Ordering::SeqCst);
                    // Give GPS task time to see CAPTURE_MODE and for next GGA to arrive (10 Hz = 100 ms period)
                    Timer::after_millis(150).await;

                    let mut sent = 0u32;
                    for _ in 0..NUM_SAMPLES {
                        match embassy_time::with_timeout(
                            COLLECT_TIMEOUT,
                            CAPTURE_CHANNEL.receive(),
                        )
                        .await
                        {
                            Ok(gps) => {
                                let ts = crate::timebase::synced_timestamp_us();
                                if transceiver.send_gps(ts, &gps, &BROADCAST).await.is_ok() {
                                    sent += 1;
                                }
                            }
                            Err(_) => break,
                        }
                    }
                    info!(
                        "[FLOATING] Sent {} GPS samples in response to capture request",
                        sent
                    );
                    if sent == 0 {
                        warn!(
                            "[FLOATING] No GPS samples: ensure the receiver has a fix (outdoor, antenna) and outputs GGA with position"
                        );
                        crate::gps::with_last_raw_gga(|opt| {
                            if let Some(raw) = opt {
                                info!("[FLOATING] Last parsed GGA (raw): {}", raw);
                            } else {
                                info!("[FLOATING] No GGA sentence parsed yet (no fix or no NMEA?)");
                            }
                        });
                    }

                    CAPTURE_MODE.store(false, Ordering::SeqCst);
                    {
                        let mut hmi = crate::hmi::state::HMI_STATE.0.lock().await;
                        hmi.acquiring_gps_capture = false;
                    }
                }
                // Ignore other message types (TimeSync, Heartbeat, etc.)
            }
            Err(e) => {
                warn!("[FLOATING] Receive error: {:?}", e);
            }
        }
    }
}

#[cfg(feature = "floating")]
pub async fn app_run_floating<B: BoardPeripherals>(spawner: embassy_executor::Spawner, mut board: B) {
    info!("[FLOATING] app starting (GPS-only, wait for RequestGpsCapture)");

    crate::timebase::set_program_start();

    let mut user_led = board.take_user_led();
    let mut neopixel = board.take_neopixel();

    // Startup delay: show rainbow cycle on NeoPixel (same as regular sensor board)
    {
        use crate::hmi::Color;
        let startup_duration = Duration::from_secs(3);
        let step = Duration::from_millis(80);
        let steps = (startup_duration.as_millis() / step.as_millis()) as u32;
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

    let _ = board.take_disp_spi_device();
    let gps2_uart = match board.take_gps2_uart_blocking() {
        Some(blocking) => crate::gps::detect_gps_baud_and_init(blocking).await,
        None => init_gps(board.take_gps2_uart()).await,
    };

    if let Some(wifi) = board.take_wifi() {
        match AirCommTransceiver::new(wifi.esp_now) {
            Ok(transceiver) => {
                spawner
                    .spawn(floating_espnow_task(transceiver, wifi.controller))
                    .expect("floating ESP-NOW task did not spawn");
            }
            Err(e) => warn!("[FLOATING] AirComm init failed: {:?}", e),
        }
    }

    spawner
        .spawn(start_gps_task_floating(gps2_uart))
        .expect("GPS task did not spawn");

    let led_rate_hz = 1u32;
    // Use higher NeoPixel brightness (50) so the single WS2812 is clearly visible; 10 is very dim
    let neopixel_brightness = 50u8;
    info!("[FLOATING] Spawning HMI floating task (user LED + NeoPixel)");
    spawner
        .spawn(start_hmi_floating_task(
            user_led,
            neopixel,
            led_rate_hz,
            neopixel_brightness,
        ))
        .expect("HMI floating task did not spawn");

    spawner
        .spawn(gps_stale_watchdog_task())
        .expect("GPS stale watchdog task did not spawn");

    loop {
        Timer::after_secs(1).await;
    }
}

/// Warns or errors if no successfully parsed GPS (GGA) in the last 1s or 5s.
#[cfg(feature = "floating")]
#[embassy_executor::task]
async fn gps_stale_watchdog_task() {
    const WARN_THRESHOLD: Duration = Duration::from_secs(1);
    const ERROR_THRESHOLD: Duration = Duration::from_secs(5);

    loop {
        Timer::after_secs(1).await;
        let state = crate::hmi::state::HMI_STATE.0.lock().await;
        let now = embassy_time::Instant::now();
        let stale = match state.last_gps_timestamp {
            Some(ts) => now.duration_since(ts),
            None => {
                drop(state);
                warn!("[FLOATING] No successfully parsed GPS (GGA) message yet");
                continue;
            }
        };
        drop(state);
        if stale >= ERROR_THRESHOLD {
            error!(
                "[FLOATING] No GPS (GGA) parsed for {}s - check antenna, fix, and NMEA output",
                stale.as_secs()
            );
        } else if stale >= WARN_THRESHOLD {
            warn!(
                "[FLOATING] No GPS (GGA) parsed in the last {}s",
                stale.as_secs()
            );
        }
    }
}

#[cfg(feature = "floating")]
#[embassy_executor::task]
async fn start_gps_task_floating(
    mut gps2_uart: esp_hal::uart::Uart<'static, esp_hal::Async>,
) {
    gps2_uart = init_gps(gps2_uart).await;
    let on_data = |data: GpsData| {
        if CAPTURE_MODE.load(Ordering::SeqCst) {
            if CAPTURE_CHANNEL.try_send(data).is_ok() {
                info!("[FLOATING] GPS sample pushed to capture channel (lat={:.6}, lon={:.6})", data.lat, data.lon);
            }
        }
    };
    start_gps(gps2_uart, on_data).await;
}

#[cfg(feature = "floating")]
#[embassy_executor::task]
async fn start_hmi_floating_task(
    user_led: esp_hal::gpio::Output<'static>,
    neopixel: neopixel::NeoPixel<'static>,
    led_rate_hz: u32,
    neopixel_brightness: u8,
) {
    start_hmi_floating(user_led, led_rate_hz, neopixel, neopixel_brightness).await;
}

// =============================================================================
// Helper Functions
// =============================================================================

#[cfg(feature = "wifi")]
async fn send_heartbeat(transceiver: &mut AirCommTransceiver<'static>) {
    let timestamp_us = crate::timebase::synced_timestamp_us();
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
                "[ESP-NOW RX] Heartbeat from {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X} | node={}:{}, magic=0x{:02X}, time={}us",
                src[0], src[1], src[2], src[3], src[4], src[5], 
                data.node_id.position.as_str(),
                data.node_id.instance,
                data.magic, 
                timestamp
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
        SensorPayload::TimeSync(data) => {
            info!(
                "[ESP-NOW RX] TimeSync from {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X} | session={}us",
                src[0], src[1], src[2], src[3], src[4], src[5], data.session_time_us
            );
        }
        SensorPayload::RequestGpsCapture => {
            // Bridge -> floating; regular nodes ignore
            trace!("[ESP-NOW RX] RequestGpsCapture (ignored)");
        }
    }
}
