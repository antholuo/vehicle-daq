#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::main;
use esp_hal::spi::{
    Mode,
    master::{Config, Spi},
};
use esp_hal::time::{Duration, Instant, Rate};
use log::{debug, info, warn};

use sensor_board_rev1_test::neopixel::{Color, NeoPixel};
use sensor_board_rev1_test::old_asm330;

use asm330;
use sensor_board_rev1_test::old_asm330::{AccelFs, Odr};

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

    // Initialize RMT for NeoPixel control
    let rmt = esp_hal::rmt::Rmt::new(_peripherals.RMT, Rate::from_mhz(80)).unwrap();
    let mut neopixel = NeoPixel::new(rmt.channel0, _peripherals.GPIO18);
    info!("NeoPixel initialized on GPIO18");

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
    // let odr_a = Odr::Hz104;
    // let _ = old_asm330::set_xl_fsr(&mut imu_spi, &fsr_a); // hiding the warnings for now
    // let _ = old_asm330::set_xl_odr(&mut imu_spi, odr_a);
    info!("Hello world!");
    user_led.toggle();
    match old_asm330::check_who_am_i(&mut imu_spi) {
        Ok(val) => {
            info!("WhoAmI value is 0x{:02X}", val);
        }
        Err(e) => {
            info!("SPI Error: {:?}", e);
        }
    }

    let lpf2_en: bool = true;
    let _ = asm330::enable_xl(
        // TODO: Fix this...
        &mut imu_spi,
        asm330::Odr::Hz104,
        asm330::AccelFs::G2,
        lpf2_en,
    );

    let colors = Color::all_colors();
    let mut color_idx: usize = 0;
    let mut brightness: u8 = 0;
    let mut brightness_increasing = true;

    let mut imu_read_1hz = Instant::now();

    loop {
        if imu_read_1hz.elapsed() > Duration::from_secs(1) {
            match old_asm330::read_xl_xyz(&mut imu_spi) {
                Ok(xl_raw_data) => {
                    debug!(
                        "Got XL X: {}, Y: {}, Z: {}",
                        xl_raw_data.x, xl_raw_data.y, xl_raw_data.z
                    );
                    let accel_x_g = old_asm330::fs_a_to_g(xl_raw_data.x, &fsr_a);
                    let accel_y_g = old_asm330::fs_a_to_g(xl_raw_data.y, &fsr_a);
                    let accel_z_g = old_asm330::fs_a_to_g(xl_raw_data.z, &fsr_a);

                    info!(
                        "Old data has Accel: X={:.3}g Y={:.3}g Z={:.3}g",
                        accel_x_g, accel_y_g, accel_z_g
                    );
                }
                Err(e) => {
                    warn!("Failed to read XL XYZ: {:?}", e);
                    neopixel.set_color(Color::Red);
                }
            }

            imu_read_1hz = Instant::now();
        }

        // Cycle through colors and brightness
        // Update brightness with breathing effect
        if brightness_increasing {
            brightness = brightness.saturating_add(5);
            if brightness >= 255 {
                brightness_increasing = false;
            }
        } else {
            brightness = brightness.saturating_sub(5);
            if brightness == 0 {
                brightness_increasing = true;
                // Move to next color when brightness cycle completes
                color_idx = (color_idx + 1) % colors.len();
            }
        }

        neopixel.set_color_with_brightness(colors[color_idx], brightness);

        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_millis(10) {}
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0-rc.1/examples/src/bin
}
