#![no_std]

// crate imports
#[cfg(feature = "user_led")]
use esp_hal::gpio::Output;

// Module definitions
#[cfg(feature = "hmi")]
pub mod hmi;

// BoardPeripherals needs to always be defined, can be empty though (I think)
pub trait BoardPeripherals {
    // HMI
    #[cfg(feature = "user_led")]
    fn take_user_led(&mut self) -> Output<'static>
}
