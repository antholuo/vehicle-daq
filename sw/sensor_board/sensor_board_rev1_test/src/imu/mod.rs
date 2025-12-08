#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use crate::{SharedSpiDevice, old_asm330};

pub async fn start_imu(mut spi: SharedSpiDevice) {
    info!("In imu task!");
    match old_asm330::check_who_am_i(&mut spi).await {
        Ok(val) => {
            info!("WhoAmI value is 0x{:02X}", val);
        }
        Err(e) => {
            info!("SPI Error: {:?}", e);
        }
    }
    loop {}
}
