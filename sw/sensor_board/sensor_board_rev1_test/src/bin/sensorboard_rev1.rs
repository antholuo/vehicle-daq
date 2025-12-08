#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_sync::mutex::Mutex;
use esp_alloc as _;
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use static_cell::StaticCell;

use sensor_board_rev1_test::{
    BoardPeripherals, SharedSpiBus, SharedSpiDevice, app::app_run, hmi::neopixel,
};

static SPI_BUS: StaticCell<SharedSpiBus> = StaticCell::new();

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

fn init_heap() {
    const HEAP_SIZE: usize = 128 * 1024; // arbitrary size
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

pub struct SensorBoardRev1 {
    // HMI
    pub user_led: Option<esp_hal::gpio::Output<'static>>,
    pub neopixel: Option<neopixel::NeoPixel<'static>>,
    pub disp_spi: Option<SharedSpiDevice>,

    // SENSORS
    pub imu_spi: Option<SharedSpiDevice>,
    pub gps2_uart: Option<esp_hal::uart::Uart<'static, esp_hal::Async>>,
}

impl SensorBoardRev1 {
    pub fn from_peripherals(peripherals: esp_hal::peripherals::Peripherals) -> Self {
        // RTOS bootstrap
        let timg0 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0);
        let software_interrupt =
            esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);

        esp_rtos::start(timg0.timer0, software_interrupt.software_interrupt0);
        debug!(">>> building SensorBoardRev1");

        let user_led = esp_hal::gpio::Output::new(
            peripherals.GPIO19,
            esp_hal::gpio::Level::High,
            esp_hal::gpio::OutputConfig::default(),
        );

        let rmt =
            esp_hal::rmt::Rmt::new(peripherals.RMT, esp_hal::time::Rate::from_mhz(80)).unwrap();
        let neopixel = neopixel::NeoPixel::new(rmt.channel0, peripherals.GPIO18);

        let gps2_uart_config = esp_hal::uart::Config::default().with_baudrate(9600);
        info!("baudrate for gps2_uart_set");
        let gps2_uart = esp_hal::uart::Uart::new(peripherals.UART1, gps2_uart_config)
            .unwrap()
            .with_rx(peripherals.GPIO23)
            .with_tx(peripherals.GPIO22)
            .into_async();

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
        let imu_spi2_cs = esp_hal::gpio::Output::new(
            peripherals.GPIO1,
            esp_hal::gpio::Level::High,
            esp_hal::gpio::OutputConfig::default(),
        );
        let disp_spi2_cs = esp_hal::gpio::Output::new(
            peripherals.GPIO10,
            esp_hal::gpio::Level::High,
            esp_hal::gpio::OutputConfig::default(),
        );
        let imu_spi_device = SpiDevice::new(spi_bus, imu_spi2_cs);
        let disp_spi_device = SpiDevice::new(spi_bus, disp_spi2_cs);

        debug!(">>> SensorBoardRev1 returned things correctly");
        Self {
            user_led: Some(user_led),
            neopixel: Some(neopixel),
            disp_spi: Some(disp_spi_device),
            imu_spi: Some(imu_spi_device),
            gps2_uart: Some(gps2_uart),
        }
    }
}

impl BoardPeripherals for SensorBoardRev1 {
    fn take_user_led(&mut self) -> esp_hal::gpio::Output<'static> {
        trace!("user led take called");
        self.user_led.take().expect("user LED already taken")
    }

    fn take_neopixel(&mut self) -> neopixel::NeoPixel<'static> {
        trace!("neopixel take called");
        self.neopixel.take().expect("NeoPixel already taken")
    }

    fn take_disp_spi_device(&mut self) -> SharedSpiDevice {
        trace!("disp_spi_device take called");
        self.disp_spi
            .take()
            .expect("DisplaySPI device already taken")
    }

    fn take_imu_spi_device(&mut self) -> SharedSpiDevice {
        trace!("imu_spi_device take called");
        self.disp_spi.take().expect("ImuSPI device already taken")
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
    app_run(spawner, SensorBoardRev1::from_peripherals(peripherals)).await;
    loop {
        embassy_time::Timer::after_secs(1).await
    }
}
