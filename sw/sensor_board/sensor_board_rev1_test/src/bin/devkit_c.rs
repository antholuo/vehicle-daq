#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use embassy_executor::Spawner;
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use sensor_board_rev1_test::{BoardPeripherals, app::app_run, hmi::neopixel};

use esp_alloc as _;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

fn init_heap() {
    const HEAP_SIZE: usize = 32 * 1024;
    static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

    unsafe {
        esp_alloc::HEAP.add_region(esp_alloc::HeapRegion::new(
            // HEAP.as_ptr() as *mut u8,
            &HEAP[0] as *const u8 as *mut u8,
            HEAP_SIZE,
            esp_alloc::MemoryCapability::Internal.into(),
        ));
    }
}

pub struct DevkitC {
    pub user_led: Option<esp_hal::gpio::Output<'static>>,
    pub neopixel: Option<neopixel::NeoPixel<'static>>,
    pub gps2_uart: Option<esp_hal::uart::Uart<'static, esp_hal::Async>>,
}

impl DevkitC {
    pub fn from_peripherals(peripherals: esp_hal::peripherals::Peripherals) -> Self {
        // RTOS bootstrap
        let timg0 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0);
        let software_interrupt =
            esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);

        esp_rtos::start(timg0.timer0, software_interrupt.software_interrupt0);
        debug!(">>> building DevkitC");

        let user_led = esp_hal::gpio::Output::new(
            peripherals.GPIO19,
            esp_hal::gpio::Level::High,
            esp_hal::gpio::OutputConfig::default(),
        );

        let rmt =
            esp_hal::rmt::Rmt::new(peripherals.RMT, esp_hal::time::Rate::from_mhz(80)).unwrap();
        let neopixel = neopixel::NeoPixel::new(rmt.channel0, peripherals.GPIO8);

        let gps2_uart_config = esp_hal::uart::Config::default().with_baudrate(9600);
        info!("baudrate for gps2_uart_set");
        let gps2_uart = esp_hal::uart::Uart::new(peripherals.UART1, gps2_uart_config)
            .unwrap()
            .with_rx(peripherals.GPIO23)
            .with_tx(peripherals.GPIO22)
            .into_async();

        debug!(">>> devkitC returned things correctly");
        Self {
            user_led: Some(user_led),
            neopixel: Some(neopixel),
            gps2_uart: Some(gps2_uart),
        }
    }
}

impl BoardPeripherals for DevkitC {
    fn take_user_led(&mut self) -> esp_hal::gpio::Output<'static> {
        trace!("user led take called");
        self.user_led.take().expect("user LED already taken")
    }

    fn take_neopixel(&mut self) -> neopixel::NeoPixel<'static> {
        trace!("neopixel take called");
        self.neopixel.take().expect("NeoPixel already taken")
    }

    fn take_gps2_uart(&mut self) -> esp_hal::uart::Uart<'static, esp_hal::Async> {
        trace!("gps2_uart take called");
        self.gps2_uart.take().expect("gps2_uart already taken")
    }
}

#[esp_rtos::main]
async fn main(spawner: embassy_executor::Spawner) {
    esp_println::logger::init_logger_from_env();

    init_heap();

    let config = esp_hal::Config::default().with_cpu_clock(esp_hal::clock::CpuClock::max());
    let peripherals = esp_hal::init(config);

    info!("Configuration complete - running app?");
    app_run(spawner, DevkitC::from_peripherals(peripherals)).await;
    loop {
        embassy_time::Timer::after_secs(1).await
    }
}
