#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use esp_hal::clock::CpuClock;
use esp_hal::main;
use esp_hal::spi::{
    master::{Config, Spi},
    Mode,
};
use esp_hal::time::{Rate};
use log::{error};
use embedded_hal::spi::SpiBus;

use asm330;
use core::prelude::v1::*; 

#[panic_handler]
fn panic(msg: &core::panic::PanicInfo) -> ! {
    error!("{}", msg);
    loop {}
}

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    // generator version: 0.6.0
    esp_println::logger::init_logger_from_env();
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let _peripherals = esp_hal::init(config);

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

    // asm330::test_asm330(&mut imu_spi);
    test_asm330(&mut imu_spi);
    panic!("Should not get here")
}

fn test_asm330<S: SpiBus<u8>>(imu_spi: &mut S) {
    // let whoamireader = asm330::WhoAmIReader::new(imu_spi);
    panic!("tests passed");
}
