/// hmi/types.rs
/// Defines common types for HMI

pub struct HmiPeripherals {
    #[cfg(feature = "user_led")]
    pub user_led: Option<esp_hal::gpio::Output<'static>>,
}