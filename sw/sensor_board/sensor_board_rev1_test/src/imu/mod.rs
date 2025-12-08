#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use embassy_time::{Duration, Timer};

use crate::old_asm330::{AccelFs, GyroFs, Odr};
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
    let fsr_a = AccelFs::G2;
    let odr_a = Odr::Hz104;
    let _ = old_asm330::set_xl_fsr(&mut spi, &fsr_a);
    let _ = old_asm330::set_xl_odr(&mut spi, odr_a);

    let period = Duration::from_hz(1);
    loop {
        match old_asm330::read_xl_xyz(&mut spi).await {
            Ok(xl_raw_data) => {
                debug!(
                    "Got XL X: {}, Y: {}, Z: {}",
                    xl_raw_data.x, xl_raw_data.y, xl_raw_data.z
                );
                let accel_x_g = old_asm330::fs_a_to_g(xl_raw_data.x, &fsr_a);
                let accel_y_g = old_asm330::fs_a_to_g(xl_raw_data.y, &fsr_a);
                let accel_z_g = old_asm330::fs_a_to_g(xl_raw_data.z, &fsr_a);

                info!(
                    "Accel: X={:.3}g Y={:.3}g Z={:.3}g",
                    accel_x_g, accel_y_g, accel_z_g
                );
            }
            Err(e) => {
                warn!("Failed to read XL XYZ: {:?}", e);
            }
        }
        Timer::after(period).await;
    }
}
