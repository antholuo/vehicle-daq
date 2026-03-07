#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types"
)]

use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_sync::mutex::Mutex;
use log::info;
use sensor_board_rev1_test::hmi::neopixel;
use sensor_board_rev1_test::{app::app_run_floating, BoardPeripherals, WifiResources};
use sensor_board_rev1_test::{SharedSpiBus, SharedSpiDevice};
use static_cell::StaticCell;

use esp_alloc as _;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

esp_bootloader_esp_idf::esp_app_desc!();

fn init_heap() {
    const HEAP_SIZE: usize = 128 * 1024;
    static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];
    unsafe {
        esp_alloc::HEAP.add_region(esp_alloc::HeapRegion::new(
            &HEAP[0] as *const u8 as *mut u8,
            HEAP_SIZE,
            esp_alloc::MemoryCapability::Internal.into(),
        ));
    }
}

static SPI_BUS: StaticCell<SharedSpiBus> = StaticCell::new();

/// Board for floating (GPS-only) node: same rev1 hardware, no IMU.
pub struct FloatingBoard {
    pub user_led: Option<esp_hal::gpio::Output<'static>>,
    pub neopixel: Option<neopixel::NeoPixel<'static>>,
    pub disp_spi: Option<SharedSpiDevice>,
    pub gps2_uart: Option<esp_hal::uart::Uart<'static, esp_hal::Blocking>>,
    pub wifi: Option<WifiResources>,
}

impl FloatingBoard {
    pub fn from_peripherals(peripherals: esp_hal::peripherals::Peripherals) -> Self {
        let timg0 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0);
        let software_interrupt =
            esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
        esp_rtos::start(timg0.timer0, software_interrupt.software_interrupt0);

        let user_led = esp_hal::gpio::Output::new(
            peripherals.GPIO19,
            esp_hal::gpio::Level::High,
            esp_hal::gpio::OutputConfig::default(),
        );
        let rmt = esp_hal::rmt::Rmt::new(peripherals.RMT, esp_hal::time::Rate::from_mhz(80))
            .unwrap()
            .into_async();
        let neopixel = neopixel::NeoPixel::new(rmt.channel0, peripherals.GPIO18);

        // Start at 9600 for baud detection; do not into_async() until after detect_gps_baud_and_init
        let gps2_uart_config = esp_hal::uart::Config::default().with_baudrate(9600);
        let gps2_uart = esp_hal::uart::Uart::new(peripherals.UART1, gps2_uart_config)
            .unwrap()
            .with_rx(peripherals.GPIO23)
            .with_tx(peripherals.GPIO22);

        let spi_config = esp_hal::spi::master::Config::default()
            .with_frequency(esp_hal::time::Rate::from_khz(100))
            .with_mode(esp_hal::spi::Mode::_0);
        let spi2 = esp_hal::spi::master::Spi::new(peripherals.SPI2, spi_config)
            .unwrap()
            .with_sck(peripherals.GPIO6)
            .with_mosi(peripherals.GPIO7)
            .with_miso(peripherals.GPIO0)
            .into_async();
        let spi_bus = SPI_BUS.init(Mutex::new(spi2));
        let disp_spi2_cs = esp_hal::gpio::Output::new(
            peripherals.GPIO10,
            esp_hal::gpio::Level::High,
            esp_hal::gpio::OutputConfig::default(),
        );
        let disp_spi_device = SpiDevice::new(spi_bus, disp_spi2_cs);

        let wifi = match esp_radio::wifi::new(peripherals.WIFI, esp_radio::wifi::Config::default()) {
            Ok((mut wifi_controller, interfaces)) => {
                if wifi_controller.set_mode(esp_radio::wifi::WifiMode::Station).is_ok()
                    && wifi_controller.start().is_ok()
                {
                    Some(WifiResources {
                        controller: wifi_controller,
                        esp_now: interfaces.esp_now,
                    })
                } else {
                    None
                }
            }
            Err(_) => None,
        };

        Self {
            user_led: Some(user_led),
            neopixel: Some(neopixel),
            disp_spi: Some(disp_spi_device),
            gps2_uart: Some(gps2_uart),
            wifi,
        }
    }
}

impl BoardPeripherals for FloatingBoard {
    fn take_user_led(&mut self) -> esp_hal::gpio::Output<'static> {
        self.user_led.take().expect("user LED already taken")
    }

    fn take_neopixel(&mut self) -> neopixel::NeoPixel<'static> {
        self.neopixel.take().expect("NeoPixel already taken")
    }

    fn take_disp_spi_device(&mut self) -> SharedSpiDevice {
        self.disp_spi.take().expect("DisplaySPI device already taken")
    }

    fn take_gps2_uart(&mut self) -> esp_hal::uart::Uart<'static, esp_hal::Async> {
        panic!("floating board: use take_gps2_uart_blocking() and detect_gps_baud_and_init");
    }

    fn take_gps2_uart_blocking(&mut self) -> Option<esp_hal::uart::Uart<'static, esp_hal::Blocking>> {
        self.gps2_uart.take()
    }

    fn take_wifi(&mut self) -> Option<WifiResources> {
        self.wifi.take()
    }

    fn espnow_mode(&self) -> sensor_board_rev1_test::EspNowMode {
        sensor_board_rev1_test::EspNowMode::Transceiver
    }
}

#[esp_rtos::main]
async fn main(spawner: embassy_executor::Spawner) {
    esp_println::logger::init_logger(log::LevelFilter::Info);
    init_heap();

    let config = esp_hal::Config::default().with_cpu_clock(esp_hal::clock::CpuClock::max());
    let peripherals = esp_hal::init(config);

    info!("sensorboard_floating: GPS-only node, waiting for RequestGpsCapture");
    let board = FloatingBoard::from_peripherals(peripherals);
    app_run_floating(spawner, board).await;
    loop {
        embassy_time::Timer::after_secs(1).await
    }
}
