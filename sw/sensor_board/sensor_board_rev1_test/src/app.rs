#[allow(unused_imports)]
use embassy_time::{Duration, Instant, Timer};
#[cfg(feature = "hmi")]
use esp_hal::gpio::Output;

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use crate::BoardPeripherals;
#[cfg(feature = "imu")]
use crate::SharedSpiDevice;
use crate::comms;
#[cfg(feature = "gps")]
use crate::gps::{init_gps, start_gps};
#[cfg(feature = "hmi")]
use crate::hmi::{neopixel, start_hmi};
#[cfg(feature = "imu")]
use crate::imu::start_imu;

#[allow(unused_mut, unused_variables)]
pub async fn app_run<B: BoardPeripherals>(spawner: embassy_executor::Spawner, mut board: B) {
    info!("app is starting execution now");

    // Initialize global program start time used for unified logging timestamps
    crate::timebase::set_program_start();

    #[cfg(feature = "hmi")]
    let user_led = board.take_user_led();
    #[cfg(feature = "hmi")]
    trace!("User Led initialized!");

    #[cfg(feature = "hmi")]
    let mut neopixel = board.take_neopixel();
    #[cfg(feature = "hmi")]
    trace!("NeoPixel initialized!");
    // Startup delay: show rainbow-barf cycle on NeoPixel while waiting
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
    init_hmi_state(&board).await;

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

    #[cfg(feature = "hmi")]
    spawner
        .spawn(start_hmi_task(user_led, neopixel))
        .expect("HMI task did not spawn");

    spawner
        .spawn(wait_for_board_init_task())
        .expect("wait for board init task did not start");

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
        let espnow_mode = board.espnow_mode();
        let wifi_resources = board.take_wifi();

        #[cfg(feature = "usb")]
        {
            let usb_serial_tx = board.take_usb_serial_tx();
            spawner
                .spawn(crate::comms::start_comms_task(
                    spawner,
                    espnow_mode,
                    wifi_resources,
                    usb_serial_tx,
                ))
                .expect("comms task did not start");
        }
        #[cfg(not(feature = "usb"))]
        {
            spawner
                .spawn(crate::comms::start_comms_task(
                    spawner,
                    espnow_mode,
                    wifi_resources,
                ))
                .expect("comms task did not start");
        }
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
        if comms::SENSOR_CHANNEL
            .try_send(crate::aircomm::SensorPayload::Imu(data))
            .is_err()
        {
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
        if comms::SENSOR_CHANNEL
            .try_send(crate::aircomm::SensorPayload::Gps(data))
            .is_err()
        {
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

#[cfg(feature = "hmi")]
async fn init_hmi_state<B: BoardPeripherals>(board: &B) {
    let mut state = crate::hmi::state::HMI_STATE.0.lock().await;
    #[cfg(feature = "wifi")]
    {
        state.is_bridge = board.espnow_mode() == crate::EspNowMode::Bridge;
    }
    #[cfg(not(feature = "wifi"))]
    {
        state.is_bridge = false;
    }
}

#[embassy_executor::task]
async fn wait_for_board_init_task() {
    let is_bridge = crate::hmi::state::HMI_STATE.0.lock().await.is_bridge;
    if !is_bridge {
        return;
    }

    info!("[INIT] Bridge mode, waiting for USB host...");

    #[cfg(feature = "usb")] // Only compile USB detection if USB feature is enabled
    {
        // These imports are only needed if the USB feature is enabled for this block
        use embassy_time::{Duration, Instant, Timer};

        const USB_DEVICE_INT_RAW: *const u32 = 0x6000_f008 as *const u32;
        const SOF_INT_MASK: u32 = 0b10;
        let start = Instant::now();
        let timeout = Duration::from_secs(5);

        while start.elapsed() < timeout {
            let connected = unsafe { (USB_DEVICE_INT_RAW.read_volatile() & SOF_INT_MASK) != 0 };
            if connected {
                let mut state = crate::hmi::state::HMI_STATE.0.lock().await;
                state.usb_host_connected = true;
                info!("[INIT] USB host detected!");
                return;
            }
            Timer::after_millis(100).await;
        }

        warn!("[INIT] USB host not detected within timeout, continuing...");
    }
    #[cfg(not(feature = "usb"))] // If USB is not enabled, just warn and continue
    {
        warn!("[INIT] Bridge mode but USB feature is not enabled. Skipping USB host detection.");
    }
}
