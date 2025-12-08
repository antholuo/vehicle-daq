#![no_std]

use crate::hmi::neopixel::NeoPixel;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use esp_hal::Async;
use esp_hal::gpio::Output;
use esp_hal::spi::master::Spi;
use esp_hal::uart::Uart;

pub mod app;
pub mod asm330;
pub mod gps;
pub mod hmi;
pub mod imu;
pub mod old_asm330;
pub mod types;

// TODO: Create shared SPI device that allows SPI to be taken by both the HMI (display) and IMU
// eg:
//     pub type SharedSpiBus = Mutex<NoopRawMutex, Spi<'static, async>>;
//     pub type SharedSpiDevice = SpiDevice<'static, NoopRaMutex, Spi<'static, async>,
//     Output<'static>>; where Output is the CS pin
//  where we can then initialize a SPI without a CS, initialize the CS pins, and then
//  construct the two unique devices, so
//     fn take_imu_spi_device
//     fn take_display_spi_device

pub type SharedSpiBus = Mutex<NoopRawMutex, Spi<'static, Async>>;

pub type SharedSpiDevice = SpiDevice<'static, NoopRawMutex, Spi<'static, Async>, Output<'static>>;

pub trait BoardPeripherals {
    // HMI
    fn take_user_led(&mut self) -> Output<'static>;
    fn take_neopixel(&mut self) -> NeoPixel<'static>;
    fn take_disp_spi_device(&mut self) -> SharedSpiDevice;

    // SENSORS
    fn take_imu_spi_device(&mut self) -> SharedSpiDevice;
    fn take_gps2_uart(&mut self) -> Uart<'static, Async>;
}
