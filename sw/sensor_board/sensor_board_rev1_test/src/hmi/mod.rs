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

    let mut slow_blink_on = false;

    loop {
        let tick_start = Instant::now();
        trace!("[HMI] - Heartbeat OK");

        // Toggle user LED as heartbeat
        if led_on {
            user_led.set_low();
        } else {
            user_led.set_high();
        }
        led_on = !led_on;

        // Read HMI state to decide NeoPixel behaviour
        let state = crate::hmi::state::HMI_STATE.0.lock().await;
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

        // Determine GPS connection vs fix status
        let gps_rx_recent = state
            .last_gps_rx_timestamp
            .map(|ts| now.duration_since(ts).as_secs() <= 3)
            .unwrap_or(false);
        let gps_fix_recent = state.gps_fix
            && state
                .last_gps_timestamp
                .map(|ts| now.duration_since(ts).as_secs() <= 3)
                .unwrap_or(false);
        drop(state);

        let period_ms = period.as_millis().max(1) as u64;
        let fast_pulse_ms = (period_ms / 4).max(1);
        let fast_pulse = Duration::from_millis(fast_pulse_ms);
        let fast_gap = fast_pulse;

        if gps_fix_recent {
            // GPS fix -> solid
            neopixel
                .set_color_with_brightness(color, neopixel_brightness)
                .await;
        } else if gps_rx_recent {
            // GPS connected, no fix -> fast blink (double pulse)
            neopixel
                .set_color_with_brightness(color, neopixel_brightness)
                .await;
            Timer::after(fast_pulse).await;
            neopixel.clear().await;
            Timer::after(fast_gap).await;
            neopixel
                .set_color_with_brightness(color, neopixel_brightness)
                .await;
            Timer::after(fast_pulse).await;
            neopixel.clear().await;
        } else {
            // No GPS data -> slow blink
            slow_blink_on = !slow_blink_on;
            if slow_blink_on {
                neopixel
                    .set_color_with_brightness(color, neopixel_brightness)
                    .await;
            } else {
                neopixel.clear().await;
            }
        }

        // Wait for the next heartbeat tick
        let elapsed = tick_start.elapsed();
        if elapsed < period {
            Timer::after(period - elapsed).await;
        }
    }
}
