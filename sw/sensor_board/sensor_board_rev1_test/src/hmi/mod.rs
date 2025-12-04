pub mod neopixel;

/// hmi.rs
/// responsible for all human-machine interfacing (SPI display, buttons, LEDs, etc)

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use embassy_time::{Duration, Timer};
pub use neopixel::{Color, NeoPixel};

pub async fn start_hmi(mut neopixel: NeoPixel<'static>, brightness: u8) {
    let colors = Color::all_colors();
    loop {
        for &color in &colors {
            trace!("[HMI] - Heartbeat OK");
            neopixel.set_color_with_brightness(color, brightness);
            Timer::after(Duration::from_secs(1)).await;
        }
    }
}
