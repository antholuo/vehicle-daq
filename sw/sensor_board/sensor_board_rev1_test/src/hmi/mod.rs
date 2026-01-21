pub mod state;
pub mod neopixel;

use esp_hal::gpio::Output;
/// hmi.rs
/// responsible for all human-machine interfacing (SPI display, buttons, LEDs, etc)

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use embassy_time::{Duration, Timer, Instant};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use crate::hmi::state::HmiState;
pub use neopixel::{Color, NeoPixel};

pub async fn start_hmi(
    mut user_led: Output<'static>,
    led_rate_hz: u32,
    mut neopixel: NeoPixel<'static>,
    neopixel_brightness: u8,
) {
    let period = Duration::from_hz(led_rate_hz as u64);

    let mut led_on = false;

    loop {
        trace!("[HMI] - Heartbeat OK");

        // Toggle user LED as heartbeat
        if led_on {
            user_led.set_low();
        } else {
            user_led.set_high();
        }
        led_on = !led_on;

        // Read HMI state to decide NeoPixel behaviour
        let mut state = crate::hmi::state::HMI_STATE.0.lock().await;
        let now = Instant::now();

        // Determine IMU age and color
        let color = if let Some(last_imu) = state.last_imu_timestamp {
            let age = now.duration_since(last_imu);
            if age.as_millis() <= 20 {
                Color::Green
            } else if age.as_secs() <= 10 {
                Color::Blue
            } else {
                Color::Red
            }
        } else {
            Color::Red
        };

        // Determine GPS blink vs solid
        let gps_fix = state.gps_fix;
        let gps_blink = if let Some(_last_gps) = state.last_gps_timestamp {
            // If gps_fix is false -> blink; if true -> solid
            !gps_fix
        } else {
            // No GPS data yet -> initializing -> blink
            true
        };
        drop(state);

        if gps_blink {
            // Blink: alternate between color and off each heartbeat
            neopixel.set_color_with_brightness(color, neopixel_brightness).await;
            Timer::after(period).await;
            neopixel.clear().await;
        } else {
            // Solid color
            neopixel
                .set_color_with_brightness(color, neopixel_brightness)
                .await;
        }

        // Wait for the next heartbeat tick
        Timer::after(period).await;
    }
}
