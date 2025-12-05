/// app.rs
/// responsible for starting the "app" and setting any necessary configs

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use crate::BoardPeripherals;
use crate::gps::start_gps;
use crate::hmi::neopixel;
use crate::hmi::start_hmi;

pub async fn app_run<B: BoardPeripherals>(spawner: embassy_executor::Spawner, mut board: B) {
    info!("app is starting execution now");

    let mut user_led = board.take_user_led();
    trace!("User Led initialized!");
    let mut neopixel = board.take_neopixel();
    trace!("NeoPixel initialized!");
    let mut gps2_uart = board.take_gps2_uart();
    trace!("Gps2_Uart initialized!");

    spawner.spawn(start_hmi_task(neopixel)).unwrap();
    spawner.spawn(start_gps_task(gps2_uart)).unwrap();

    loop {
        embassy_time::Timer::after_secs(1).await
    }
}

#[embassy_executor::task]
async fn start_hmi_task(mut neopixel: neopixel::NeoPixel<'static>) {
    // Task configuration
    let neopixel_brightness: u8 = 15;

    start_hmi(neopixel, neopixel_brightness).await;
}

#[embassy_executor::task]
async fn start_gps_task(mut gps2_uart: esp_hal::uart::Uart<'static, esp_hal::Async>) {
    start_gps(gps2_uart).await;
}
