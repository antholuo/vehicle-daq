pub mod neopixel;

use esp_hal::gpio::Output;
/// hmi.rs
/// responsible for all human-machine interfacing (SPI display, buttons, LEDs, etc)

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use embassy_time::{Duration, Timer};
pub use neopixel::{Color, NeoPixel};

pub async fn start_hmi(
    mut user_led: Output<'static>,
    led_rate_hz: u32,
    mut neopixel: NeoPixel<'static>,
    neopixel_brightness: u8,
) {
    let colors = Color::all_colors();
    let period = Duration::from_hz(led_rate_hz as u64);

    let mut led_on = false;

    loop {
        for &color in &colors {
            trace!("[HMI] - Heartbeat OK");

            // Toggle user LED
            if led_on {
                user_led.set_low();
            } else {
                user_led.set_high();
            }
            led_on = !led_on;

            // Update NeoPixel color (async)
            neopixel
                .set_color_with_brightness(color, neopixel_brightness)
                .await;

            // Wait for the next tick based on rate
            Timer::after(period).await;
        }
    }
}
