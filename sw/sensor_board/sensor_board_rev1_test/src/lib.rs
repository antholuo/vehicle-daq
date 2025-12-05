#![no_std]

use crate::hmi::neopixel::NeoPixel;
use core::future::Future;
use esp_hal::Async;
use esp_hal::gpio::Output;
use esp_hal::uart::Uart;

pub mod app;
pub mod asm330;
pub mod gps;
pub mod hmi;
pub mod old_asm330;
pub mod types;

pub trait BoardPeripherals {
    fn take_user_led(&mut self) -> Output<'static>;
    fn take_neopixel(&mut self) -> NeoPixel<'static>;

    fn take_gps2_uart(&mut self) -> Uart<'static, Async>;
}
