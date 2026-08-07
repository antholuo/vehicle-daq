// sensorboard_revb.rs
// Binary targeted towards generic application on sensorboard revB.

#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

#[allow(unused_imports)]
use defmt::{debug, error, fatal, info, trace, warn};
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::timer::timg::TimerGroup;
use esp_println as _;

extern crate alloc;

// Always-on includes
use sb2::{BoardPeripherals, app::app_run};

#[cfg(feature = "user_led")]
use sb2::hmi::user_led;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

pub struct SensorBoardRevB {
    #[cfg(feature = "user_led")]
    pub user_led: Option<esp_hal::gpio::Output<'static>>,
}

impl SensorBoardRevB {
    pub fn from_peripherals(peripherals: esp_hal::peripherals::Peripherals) -> Self {
        let timg0 = TimerGroup::new(peripherals.TIMG0);
        let sw_interrupt =
            esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
        esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

        info!("Embassy initialized! - You are running `sensorboard_revb` binary");

        // HMI periphs
        #[cfg(feature = "user_led")]
        let user_led = esp_hal::gpio::Output::new(
            peripherals.GPIO19,
            esp_hal::gpio::Level::High,
            esp_hal::gpio::OutputConfig::default(),
        );

        let (mut _wifi_controller, _interfaces) =
            esp_radio::wifi::new(peripherals.WIFI, Default::default())
                .expect("Failed to initialize Wi-Fi controller");

        Self {
            #[cfg(feature = "user_led")]
            user_led: Some(user_led),
        }
    }
}

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    // generator version: 1.3.0
    // generator parameters: --chip esp32c6 -o alloc -o unstable-hal -o wifi -o embassy -o defmt -o esp-backtrace -o neovim -o esp32c6-wroom-1

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // The following pins are used to bootstrap the chip. They are available
    // for use, but check the datasheet of the module for more information on them.
    // - GPIO4
    // - GPIO5
    // - GPIO8
    // - GPIO9
    // - GPIO15
    // These GPIO pins are in use by some feature of the module and should not be used.
    let _ = peripherals.GPIO24;
    let _ = peripherals.GPIO25;
    let _ = peripherals.GPIO26;
    let _ = peripherals.GPIO27;
    let _ = peripherals.GPIO28;
    let _ = peripherals.GPIO29;
    let _ = peripherals.GPIO30;

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 65536);

    info!("Configuration complete - attempting to start app execution now");
    app_run(spawner, SensorBoardRevB::from_peripherals(peripherals)).await;
}
