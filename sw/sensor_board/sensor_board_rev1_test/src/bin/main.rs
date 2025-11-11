#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use asm330::{
    AccelFs, GyroFs, Odr, enable_xl_gy_outputs, fs_a_to_g, poll_data, read_who_am_i, read_xl_xyz,
    reset_asm330, set_xl_fsr, set_xl_odr,
};

use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Level, Output};
use esp_hal::main;
use esp_hal::spi::{
    Mode,
    master::{Config, Spi},
};
use esp_hal::time::{Duration, Instant, Rate};
use log::{debug, info, warn};

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

    let mut user_led = Output::new(
        _peripherals.GPIO19,
        Level::High,
        esp_hal::gpio::OutputConfig::default(),
    );

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

    // let ocfg = asm330::OutputConfig {
    //     xl_odr: Odr::Hz12_5,
    //     xl_fsr: AccelFs::G4,
    //     xl_lpf2_en: false,
    //     gy_odr: Odr::Hz12_5,
    //     gy_fsr: GyroFs::DPS500,
    //     gy_lpf1_en: false,
    //     block_data_en: false,
    //     timestamp_en: false,
    // };

    read_who_am_i(&mut imu_spi);

    // reset_asm330(&mut imu_spi);

    read_who_am_i(&mut imu_spi);

    // enable_xl_gy_outputs(&mut imu_spi, &ocfg);

    set_xl_fsr(&mut imu_spi, &AccelFs::G2);
    set_xl_odr(&mut imu_spi, Odr::Hz12_5);

    loop {
        // match poll_data(&mut imu_spi) {
        //     Ok(data) => {
        //         let ts = data.ts.unwrap_or(0);
        //
        //         // Print header every N loops if you want (optional)
        //         // info!("--------------------------------------------------------------");
        //         // info!(
        //         //     "TS: {:>10} | ACC [mg]: x={:>6}, y={:>6}, z={:>6} | GYRO [dps]: x={:>6}, y={:>6}, z={:>6}",
        //         //     ts,
        //         //     fs_a_to_g(data.xl.x, &ocfg.xl_fsr),
        //         //     fs_a_to_g(data.xl.y, &ocfg.xl_fsr),
        //         //     fs_a_to_g(data.xl.z, &ocfg.xl_fsr),
        //         //     data.gy.x,
        //         //     data.gy.y,
        //         //     data.gy.z
        //         // );
        //     }
        //     Err(e) => {
        //         log::warn!("IMU read failed: {:?}", e);
        //     }
        // }
        match read_xl_xyz(&mut imu_spi) {
            Ok(dat) => {
                info!(
                    "x={}, y={}, z={}",
                    fs_a_to_g(dat.x, &AccelFs::G2),
                    fs_a_to_g(dat.y, &AccelFs::G2),
                    fs_a_to_g(dat.z, &AccelFs::G2),
                );
            }
            Err(e) => {
                info!("eontauhsaoeh")
            }
        }
        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_millis(1000) {}
    }
}
