#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use crate::BoardPeripherals;
use crate::hmi::start_hmi;
use crate::neopixel;

pub async fn app_run<B: BoardPeripherals>(spawner: embassy_executor::Spawner, mut board: B) {
    info!("app is starting execution now");

    let mut user_led = board.take_user_led();
    trace!("User Led initialized!");
    let mut neopixel = board.take_neopixel();
    trace!("NeoPixel initialized!");

    spawner.spawn(start_hmi_task(neopixel)).unwrap();

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
