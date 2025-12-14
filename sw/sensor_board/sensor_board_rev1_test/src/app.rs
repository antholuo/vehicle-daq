/// app.rs
/// responsible for starting the "app" and setting any necessary configs

use embassy_time::{Duration, Instant};
use esp_hal::gpio::Output;

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use crate::aircomm::{AirCommTransceiver, HeartbeatData, SensorMessage, SensorPayload, BROADCAST};
use crate::gps::{init_gps, start_gps};
use crate::hmi::{neopixel, start_hmi};
use crate::imu::start_imu;
use crate::{BoardPeripherals, SharedSpiDevice};

pub async fn app_run<B: BoardPeripherals>(spawner: embassy_executor::Spawner, mut board: B) {
    info!("app is starting execution now");

    let user_led = board.take_user_led();
    trace!("User Led initialized!");
    let neopixel = board.take_neopixel();
    trace!("NeoPixel initialized!");
    let _disp_spi_device = board.take_disp_spi_device();
    trace!("DISPLAY_SPI device taken");
    let imu_spi_device = board.take_imu_spi_device();
    trace!("IMU_SPI device taken");
    let gps2_uart = board.take_gps2_uart();
    trace!("Gps2_Uart initialized!");

    info!("all periphs taken");

    spawner
        .spawn(start_hmi_task(user_led, neopixel))
        .expect("HMI task did not spawn");

    spawner
        .spawn(start_imu_task(imu_spi_device))
        .expect("imu task did not spawn");
    spawner
        .spawn(start_gps_task(gps2_uart))
        .expect("GPS task did not spawn");

    // ESP-NOW / AirComm task
    if let Some(wifi) = board.take_wifi() {
        match AirCommTransceiver::new(wifi.esp_now) {
            Ok(transceiver) => {
                info!("AirComm transceiver initialized");
                spawner
                    .spawn(espnow_task(transceiver, wifi.controller))
                    .expect("ESP-NOW task did not spawn");
            }
            Err(e) => {
                warn!("Failed to create AirComm transceiver: {:?}", e);
            }
        }
    } else {
        warn!("WiFi not available - skipping wireless communication");
    }

    loop {
        embassy_time::Timer::after_secs(1).await
    }
}

#[embassy_executor::task]
async fn start_hmi_task(user_led: Output<'static>, neopixel: neopixel::NeoPixel<'static>) {
    // Task configuration
    let led_rate_hz: u32 = 1;
    let neopixel_brightness: u8 = 10;

    start_hmi(user_led, led_rate_hz, neopixel, neopixel_brightness).await;
}

#[embassy_executor::task]
async fn start_imu_task(imu_spi_device: SharedSpiDevice) {
    info!("IMU TASK BEING SPAWNED");
    start_imu(imu_spi_device).await;
}

#[embassy_executor::task]
async fn start_gps_task(mut gps2_uart: esp_hal::uart::Uart<'static, esp_hal::Async>) {
    gps2_uart = init_gps(gps2_uart).await;
    start_gps(gps2_uart).await;
}

/// ESP-NOW transceiver task
///
/// Handles both sending heartbeats at regular intervals and receiving messages.
/// Uses timeout-based receive to avoid blocking heartbeat transmission.
///
/// Note: `_wifi_controller` must be kept alive for ESP-NOW to function properly.
#[embassy_executor::task]
async fn espnow_task(
    mut transceiver: AirCommTransceiver<'static>,
    _wifi_controller: esp_radio::wifi::WifiController<'static>,
) {
    info!("[ESP-NOW] Task started");

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

        // Try to receive with timeout (doesn't cancel the receive, just times out)
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

async fn send_heartbeat(transceiver: &mut AirCommTransceiver<'static>) {
    let timestamp_us = Instant::now().as_micros();
    let heartbeat = HeartbeatData::default();

    match transceiver.send_heartbeat(timestamp_us, &heartbeat, &BROADCAST).await {
        Ok(()) => {
            info!("[ESP-NOW TX] Heartbeat sent (time={}us)", timestamp_us);
        }
        Err(e) => {
            warn!("[ESP-NOW TX] Send failed: {:?}", e);
        }
    }
}

fn handle_received_message(msg: &SensorMessage) {
    let src = msg.src_address;
    let timestamp = msg.timestamp_us;
    
    match &msg.payload {
        SensorPayload::Heartbeat(data) => {
            info!(
                "[ESP-NOW RX] Heartbeat from {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X} | magic=0x{:02X}, time={}us",
                src[0], src[1], src[2], src[3], src[4], src[5],
                data.magic, timestamp
            );
        }
        SensorPayload::Imu(_data) => {
            info!(
                "[ESP-NOW RX] IMU from {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X} | time={}us, TODO: IMU fields",
                src[0], src[1], src[2], src[3], src[4], src[5],
                timestamp
            );
        }
        SensorPayload::Gps(data) => {
            info!(
                "[ESP-NOW RX] GPS from {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X} | time={}us, lat={:.6}, lon={:.6}",
                src[0], src[1], src[2], src[3], src[4], src[5],
                timestamp, data.lat, data.lon
            );
        }
    }
}
