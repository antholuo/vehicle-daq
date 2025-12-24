#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use embassy_time::{Duration, Timer};

use crate::old_asm330::{AccelFs, Odr};
use crate::types::ImuData;
use crate::{SharedSpiDevice, old_asm330};

/// Start IMU task with a callback for each data sample
///
/// The callback `on_data` is invoked whenever new IMU data is available.
/// This allows the caller to decide what to do with the data (log, send, etc.)
/// without the driver needing to know about channels or networking.
///
/// # Arguments
/// * `spi` - The shared SPI device for IMU communication
/// * `on_data` - Callback invoked with each IMU sample
pub async fn start_imu<F>(mut spi: SharedSpiDevice, on_data: F)
where
    F: Fn(ImuData),
{
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
    let _ = old_asm330::set_xl_fsr(&mut spi, &fsr_a).await;
    let _ = old_asm330::set_xl_odr(&mut spi, odr_a).await;

    let period = Duration::from_hz(2);
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

                // Invoke callback with IMU data
                // Note: Gyro data is zeroed until driver supports gyroscope
                let imu_data = ImuData {
                    accel_x: accel_x_g,
                    accel_y: accel_y_g,
                    accel_z: accel_z_g,
                    gyro_x: 0.0,
                    gyro_y: 0.0,
                    gyro_z: 0.0,
                };
                on_data(imu_data);
            }
            Err(e) => {
                warn!("Failed to read XL XYZ: {:?}", e);
            }
        }
        Timer::after(period).await;
    }
}
