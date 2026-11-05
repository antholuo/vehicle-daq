/// user_led.rs
/// Module to configure & run user_led.
/// This is kept within the hmi module as we want direct control of the LED's actions during boot / debug

#[allow(unused_imports)]
use defmt::{debug, error, info, trace, warn};

use crate::hmi::types::HmiPeripherals;

pub async fn start_hmi(peripherals: HmiPeripherals) {}

