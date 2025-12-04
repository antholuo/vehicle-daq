#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use sensor_board_rev1_test::{BoardPeripherals, app::app_run, neopixel};

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

pub struct DevkitC {
    pub user_led: Option<esp_hal::gpio::Output<'static>>,
    pub neopixel: Option<neopixel::NeoPixel<'static>>,
}

impl DevkitC {
    pub fn new() -> Self {
        let config = esp_hal::Config::default().with_cpu_clock(esp_hal::clock::CpuClock::max());
        let peripherals = esp_hal::init(config);

        let user_led = esp_hal::gpio::Output::new(
            peripherals.GPIO19,
            esp_hal::gpio::Level::High,
            esp_hal::gpio::OutputConfig::default(),
        );

        let rmt =
            esp_hal::rmt::Rmt::new(peripherals.RMT, esp_hal::time::Rate::from_mhz(80)).unwrap();
        let neopixel = neopixel::NeoPixel::new(rmt.channel0, peripherals.GPIO8);

        Self {
            user_led: Some(user_led),
            neopixel: Some(neopixel),
        }
    }
}

impl BoardPeripherals for DevkitC {
    fn take_user_led(&mut self) -> esp_hal::gpio::Output<'static> {
        self.user_led.take().expect("user LED already taken")
    }

    fn take_neopixel(&mut self) -> neopixel::NeoPixel<'static> {
        self.neopixel.take().expect("NeoPixel already taken")
    }
}

#[esp_rtos::main]
async fn main(spawner: embassy_executor::Spawner) {
    let config = esp_hal::Config::default().with_cpu_clock(esp_hal::clock::CpuClock::max());
    let peripherals = esp_hal::init(config);

    // RTOS bootstrap
    let timg0 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0);
    let software_interrupt =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, software_interrupt.software_interrupt0);

    app_run(spawner, DevkitC::new()).await
}
