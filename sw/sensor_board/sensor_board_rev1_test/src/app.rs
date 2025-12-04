use log::info;

use crate::BoardPeripherals;

pub async fn app_run<B: BoardPeripherals>(spawner: embassy_executor::Spawner, mut board: B) {
    let mut user_led = board.take_user_led();
    info!("User Led initialized!");
    let mut neopixel = board.take_neopixel();
    info!("NeoPixel initialized!");

    // spawner.spawn lighting control
}
