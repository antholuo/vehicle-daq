/// app.rs (sb2)
/// Responsible for actually starting our application, setting any necessary configs

// External Crates
#[allow(unused_imports)]
use defmt::{debug, error, info, trace, warn};
#[cfg(feature = "user_led")]
use esp_hal::gpio::Output;

use crate::BoardPeripherals;

pub async fn app_run<B: BoardPeripherals>(
    spawner: embassy_executor::Spawner,
    mut board_periphs: B,
) {
    info!("SB2/app is starting execution now");
}
