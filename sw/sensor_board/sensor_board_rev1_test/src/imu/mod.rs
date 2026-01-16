#[allow(unused_imports)]
mod old_asm330;

use log::{debug, error, info, trace, warn};

use embassy_time::{Duration, Timer};

use crate::SharedSpiDevice;
use crate::imu::old_asm330::{AccelFs, Odr, fs_a_to_g, fs_g_to_dps, poll_xl_gy_combined, read_status_data, GyroFs};
use crate::types::ImuData;

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
    let fsr_a = AccelFs::G8;
    let odr_a = Odr::Hz417;
    let gyro_fs = GyroFs::DPS1000;
    let _ = old_asm330::set_xl_fsr(&mut spi, &fsr_a).await;
    let _ = old_asm330::set_xl_odr(&mut spi, odr_a).await;
    let _ = old_asm330::set_gyro_config(&mut spi, odr_a, gyro_fs).await;
    // Ensure timestamp counter is enabled in CTRL10_C
    match old_asm330::enable_timestamp(&mut spi).await {
        Ok(()) => info!("Timestamp enabled"),
        Err(e) => warn!("Failed to enable timestamp: {:?}", e),
    }

    let period = Duration::from_hz(100);
    loop {
        match old_asm330::read_status_data(&mut spi).await {
            Ok(status) => {
                if status.xlda && status.gda {
                    match old_asm330::poll_xl_gy_combined(&mut spi).await {
                        Ok(raw) => {
                            let accel_x_g = old_asm330::fs_a_to_g(raw.xl.x, &fsr_a);
                            let accel_y_g = old_asm330::fs_a_to_g(raw.xl.y, &fsr_a);
                            let accel_z_g = old_asm330::fs_a_to_g(raw.xl.z, &fsr_a);
                            let gyro_x_dps = old_asm330::fs_g_to_dps(raw.gy.x, &gyro_fs);
                            let gyro_y_dps = old_asm330::fs_g_to_dps(raw.gy.y, &gyro_fs);
                            let gyro_z_dps = old_asm330::fs_g_to_dps(raw.gy.z, &gyro_fs);
                            info!("TS: {} Accel: X={:.3}g Y={:.3}g Z={:.3}g Gyro: X={:.3}dps Y={:.3}dps Z={:.3}dps",
                                raw.ts, accel_x_g, accel_y_g, accel_z_g, gyro_x_dps, gyro_y_dps, gyro_z_dps);
                            let imu_data = ImuData {
                                accel_x: accel_x_g,
                                accel_y: accel_y_g,
                                accel_z: accel_z_g,
                                gyro_x: gyro_x_dps,
                                gyro_y: gyro_y_dps,
                                gyro_z: gyro_z_dps,
                            };
                            on_data(imu_data);
                        }
                        Err(e) => { warn!("Failed to read combined XL/GY: {:?}", e); }
                    }
                } else {
                    trace!("Data not ready xlda={} gda={} tda={}", status.xlda, status.gda, status.tda);
                }
            }
            Err(e) => {
                warn!("Failed to read status: {:?}", e);
            }
        }
        Timer::after(period).await;
    }
}
