#![no_std]

#[cfg(feature = "hmi")]
use crate::hmi::neopixel::NeoPixel;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use esp_hal::Async;
use esp_hal::gpio::Output;
use esp_hal::spi::master::Spi;
#[cfg(feature = "gps")]
use esp_hal::uart::Uart;
use esp_hal::usb_serial_jtag::UsbSerialJtagTx;

#[cfg(feature = "wifi")]
pub mod aircomm;
pub mod app;
pub mod asm330;
#[cfg(feature = "wifi")]
pub mod comms;
#[cfg(feature = "gps")]
pub mod gps;
#[cfg(feature = "hmi")]
pub mod hmi;
#[cfg(feature = "imu")]
pub mod imu;
pub mod timebase;
pub mod types;
pub mod usb;

pub type SharedSpiBus = Mutex<NoopRawMutex, Spi<'static, Async>>;

pub type SharedSpiDevice = SpiDevice<'static, NoopRawMutex, Spi<'static, Async>, Output<'static>>;

/// ESP-NOW operating mode for the board
#[cfg(feature = "wifi")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EspNowMode {
    /// Sender mode: transmits sensor data from channel + heartbeats
    /// Used by sensor nodes that collect and send data
    Sender,
    /// Transceiver mode: receives ESP-NOW messages + sends heartbeats
    /// Used for testing purposes, receives data from other nodes
    Transceiver,
    /// Bridge mode: receives ESP-NOW messages and forwards to USB
    /// Used by bridge nodes that forward data to a host (e.g., Raspberry Pi)
    Bridge,
}

/// WiFi resources bundle containing the controller and available interfaces
#[cfg(feature = "wifi")]
pub struct WifiResources {
    /// WiFi controller (manages the WiFi hardware)
    pub controller: esp_radio::wifi::WifiController<'static>,
    /// ESP-NOW interface for peer-to-peer communication
    pub esp_now: esp_radio::esp_now::EspNow<'static>,
}

pub trait BoardPeripherals {
    // HMI
    #[cfg(feature = "hmi")]
    fn take_user_led(&mut self) -> Output<'static>;
    #[cfg(feature = "hmi")]
    fn take_neopixel(&mut self) -> NeoPixel<'static>;
    #[cfg(feature = "hmi")]
    fn take_disp_spi_device(&mut self) -> SharedSpiDevice;

    // SENSORS
    #[cfg(feature = "imu")]
    fn take_imu_spi_device(&mut self) -> SharedSpiDevice;
    #[cfg(feature = "gps")]
    fn take_gps2_uart(&mut self) -> Uart<'static, Async>;

    // WIRELESS
    #[cfg(feature = "wifi")]
    fn take_wifi(&mut self) -> Option<WifiResources>;

    /// Returns the ESP-NOW operating mode for this board
    #[cfg(feature = "wifi")]
    fn espnow_mode(&self) -> EspNowMode;

    // USB
    /// Take the USB Serial TX interface for host communication
    /// Returns None if USB is not available or not configured for this board
    fn take_usb_serial_tx(&mut self) -> Option<UsbSerialJtagTx<'static, Async>> {
        None // Default: USB not available
    }
}
