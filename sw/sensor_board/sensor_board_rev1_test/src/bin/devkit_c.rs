#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

#[cfg(any(feature = "hmi", feature = "imu"))]
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
#[cfg(any(feature = "hmi", feature = "imu"))]
use embassy_sync::mutex::Mutex;
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
#[cfg(any(feature = "hmi", feature = "imu"))]
use static_cell::StaticCell;

use esp_alloc as _;

#[cfg(feature = "hmi")]
use sensor_board_rev1_test::hmi::neopixel;
#[cfg(any(feature = "hmi", feature = "imu"))]
use sensor_board_rev1_test::{SharedSpiBus, SharedSpiDevice};
#[cfg(feature = "wifi")]
use sensor_board_rev1_test::{EspNowMode, WifiResources};
use sensor_board_rev1_test::{BoardPeripherals, app::app_run};

#[cfg(any(feature = "hmi", feature = "imu"))]
static SPI_BUS: StaticCell<SharedSpiBus> = StaticCell::new();

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

// Required by the ESP-IDF bootloader
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

pub struct DevkitC {
    #[cfg(feature = "hmi")]
    pub user_led: Option<esp_hal::gpio::Output<'static>>,
    #[cfg(feature = "hmi")]
    pub neopixel: Option<neopixel::NeoPixel<'static>>,
    #[cfg(feature = "hmi")]
    pub disp_spi: Option<SharedSpiDevice>,
    #[cfg(feature = "imu")]
    pub imu_spi: Option<SharedSpiDevice>,
    #[cfg(feature = "gps")]
    pub gps2_uart: Option<esp_hal::uart::Uart<'static, esp_hal::Async>>,
    #[cfg(feature = "wifi")]
    pub wifi: Option<WifiResources>,
}

impl DevkitC {
    pub fn from_peripherals(peripherals: esp_hal::peripherals::Peripherals) -> Self {
        // RTOS bootstrap
        let timg0 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0);
        let software_interrupt =
            esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);

        esp_rtos::start(timg0.timer0, software_interrupt.software_interrupt0);

        // HMI peripherals
        #[cfg(feature = "hmi")]
        let user_led = esp_hal::gpio::Output::new(
            peripherals.GPIO19,
            esp_hal::gpio::Level::High,
            esp_hal::gpio::OutputConfig::default(),
        );

        #[cfg(feature = "hmi")]
        let neopixel = {
            let rmt = esp_hal::rmt::Rmt::new(peripherals.RMT, esp_hal::time::Rate::from_mhz(80))
                .unwrap()
                .into_async();
            neopixel::NeoPixel::new(rmt.channel0, peripherals.GPIO8)
        };

        // GPS peripheral
        #[cfg(feature = "gps")]
        let gps2_uart = {
            let gps2_uart_config = esp_hal::uart::Config::default().with_baudrate(9600);
            esp_hal::uart::Uart::new(peripherals.UART1, gps2_uart_config)
                .unwrap()
                .with_rx(peripherals.GPIO23)
                .with_tx(peripherals.GPIO22)
                .into_async()
        };

        // SPI bus (shared by HMI display and IMU)
        #[cfg(any(feature = "hmi", feature = "imu"))]
        let spi_bus = {
            let spi_config = esp_hal::spi::master::Config::default()
                .with_frequency(esp_hal::time::Rate::from_khz(100))
                .with_mode(esp_hal::spi::Mode::_0);
            let spi2 = esp_hal::spi::master::Spi::new(peripherals.SPI2, spi_config)
                .unwrap()
                .with_sck(peripherals.GPIO6)
                .with_mosi(peripherals.GPIO7)
                .with_miso(peripherals.GPIO0)
                .into_async();
            SPI_BUS.init(Mutex::new(spi2))
        };

        #[cfg(feature = "imu")]
        let imu_spi_device = {
            let imu_spi2_cs = esp_hal::gpio::Output::new(
                peripherals.GPIO1,
                esp_hal::gpio::Level::High,
                esp_hal::gpio::OutputConfig::default(),
            );
            SpiDevice::new(spi_bus, imu_spi2_cs)
        };

        #[cfg(feature = "hmi")]
        let disp_spi_device = {
            let disp_spi2_cs = esp_hal::gpio::Output::new(
                peripherals.GPIO10,
                esp_hal::gpio::Level::High,
                esp_hal::gpio::OutputConfig::default(),
            );
            SpiDevice::new(spi_bus, disp_spi2_cs)
        };

        // WiFi initialization
        #[cfg(feature = "wifi")]
        let wifi = match esp_radio::wifi::new(peripherals.WIFI, esp_radio::wifi::Config::default())
        {
            Ok((mut wifi_controller, interfaces)) => {
                info!("WiFi initialized successfully");
                if let Err(e) = wifi_controller.set_mode(esp_radio::wifi::WifiMode::Station) {
                    warn!("Failed to set WiFi mode: {:?}", e);
                    None
                } else if let Err(e) = wifi_controller.start() {
                    warn!("Failed to start WiFi controller: {:?}", e);
                    None
                } else {
                    Some(WifiResources {
                        controller: wifi_controller,
                        esp_now: interfaces.esp_now,
                    })
                }
            }
            Err(e) => {
                warn!("WiFi initialization failed: {:?}", e);
                None
            }
        };

        Self {
            #[cfg(feature = "hmi")]
            user_led: Some(user_led),
            #[cfg(feature = "hmi")]
            neopixel: Some(neopixel),
            #[cfg(feature = "hmi")]
            disp_spi: Some(disp_spi_device),
            #[cfg(feature = "imu")]
            imu_spi: Some(imu_spi_device),
            #[cfg(feature = "gps")]
            gps2_uart: Some(gps2_uart),
            #[cfg(feature = "wifi")]
            wifi,
        }
    }
}

impl BoardPeripherals for DevkitC {
    #[cfg(feature = "hmi")]
    fn take_user_led(&mut self) -> esp_hal::gpio::Output<'static> {
        trace!("user led take called");
        self.user_led.take().expect("user LED already taken")
    }

    #[cfg(feature = "hmi")]
    fn take_neopixel(&mut self) -> neopixel::NeoPixel<'static> {
        trace!("neopixel take called");
        self.neopixel.take().expect("NeoPixel already taken")
    }

    #[cfg(feature = "hmi")]
    fn take_disp_spi_device(&mut self) -> SharedSpiDevice {
        trace!("disp_spi_device take called");
        self.disp_spi
            .take()
            .expect("DisplaySPI device already taken")
    }

    #[cfg(feature = "imu")]
    fn take_imu_spi_device(&mut self) -> SharedSpiDevice {
        trace!("imu_spi_device take called");
        self.imu_spi.take().expect("ImuSPI device already taken")
    }

    #[cfg(feature = "gps")]
    fn take_gps2_uart(&mut self) -> esp_hal::uart::Uart<'static, esp_hal::Async> {
        trace!("gps2_uart take called");
        self.gps2_uart.take().expect("gps2_uart already taken")
    }

    #[cfg(feature = "wifi")]
    fn take_wifi(&mut self) -> Option<WifiResources> {
        trace!("wifi take called");
        self.wifi.take()
    }

    #[cfg(feature = "wifi")]
    fn espnow_mode(&self) -> EspNowMode {
        // DevKit-C acts as a generic receiver for now
        EspNowMode::Transceiver
    }
}

#[esp_rtos::main]
async fn main(spawner: embassy_executor::Spawner) {
    esp_println::logger::init_logger_from_env();

    init_heap();

    let config = esp_hal::Config::default().with_cpu_clock(esp_hal::clock::CpuClock::max());
    let peripherals = esp_hal::init(config);

    let board = DevkitC::from_peripherals(peripherals);

    app_run(spawner, board).await;
    loop {
        embassy_time::Timer::after_secs(1).await
    }
}
