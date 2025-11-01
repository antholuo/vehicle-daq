#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use crate::asm330::{AccelFs, Odr};

use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::main;
use esp_hal::spi::{
    Mode,
    master::{Config, Spi},
};
use esp_hal::time::{Duration, Instant, Rate};
use log::{debug, info, warn};

use sensor_board_rev1_test::asm330;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    // generator version: 0.6.0

    esp_println::logger::init_logger_from_env();

    ///////////////////////////
    // Peripheral Configuration

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let _peripherals = esp_hal::init(config);

    let mut user_led = Output::new(_peripherals.GPIO19, Level::High, OutputConfig::default());

    let imu_spi_maybe = match Spi::new(
        _peripherals.SPI2,
        Config::default()
            .with_frequency(Rate::from_khz(100))
            .with_mode(Mode::_0),
    ) {
        Ok(base) => Some(
            base.with_sck(_peripherals.GPIO6)
                .with_mosi(_peripherals.GPIO7)
                .with_miso(_peripherals.GPIO0)
                .with_cs(_peripherals.GPIO1),
        ),
        Err(e) => {
            esp_println::println!("SPI init failed: {:?}", e);
            None
        }
    };

    let mut imu_spi = imu_spi_maybe.expect("Spi must be initialized to continue!");

    let fsr_a = AccelFs::G2;
    let odr_a = Odr::Hz104;
    let _ = asm330::set_xl_fsr(&mut imu_spi, &fsr_a); // hiding the warnings for now
    let _ = asm330::set_xl_odr(&mut imu_spi, odr_a);
    info!("Hello world!");
    user_led.toggle();
    match asm330::check_who_am_i(&mut imu_spi) {
        Ok(val) => {
            info!("WhoAmI value is 0x{:02X}", val);
        }
        Err(e) => {
            info!("SPI Error: {:?}", e);
        }
    }

    loop {
        match asm330::read_xl_xyz(&mut imu_spi) {
            Ok(xl_raw_data) => {
                debug!(
                    "Got XL X: {}, Y: {}, Z: {}",
                    xl_raw_data.x, xl_raw_data.y, xl_raw_data.z
                );
                let accel_x_g = asm330::fs_a_to_g(xl_raw_data.x, &fsr_a);
                let accel_y_g = asm330::fs_a_to_g(xl_raw_data.y, &fsr_a);
                let accel_z_g = asm330::fs_a_to_g(xl_raw_data.z, &fsr_a);

                info!(
                    "Accel: X={:.3}g Y={:.3}g Z={:.3}g",
                    accel_x_g, accel_y_g, accel_z_g
                );
            }
            Err(e) => {
                warn!("Failed to read XL XYZ: {:?}", e);
            }
        }

        // let g_z = match asm330::read_xl_z(&mut imu_spi) {
        //     Ok(raw) => {
        //         let g_val = asm330::fs_a_to_g(raw, &fsr_a);
        //         info!("Got G_Z as {}g's", g_val);
        //         Some(g_val)
        //     }
        //     Err(e) => None,
        // };
        // if let Some(val) = g_z {
        //     info!("Unpacked g_z as {}g", val);
        // } else {
        //     warn!("No g_z value available");
        // }

        // let mut buffer: [u8; 2] = [0x8F, 0x00];
        // match imu_spi.transfer(&mut buffer) {
        //     Ok(_) => {
        //         let whoami_value = buffer[1];
        //
        //         info!("whoami value is {}", whoami_value);
        //     }
        //     Err(e) => {
        //         info!("SPI Error: {:?}", e);
        //     }
        // }

        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_millis(10) {}
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0-rc.1/examples/src/bin
}
