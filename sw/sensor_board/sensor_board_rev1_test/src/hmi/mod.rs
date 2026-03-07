pub mod neopixel;
pub mod state;

use esp_hal::gpio::Output;
/// hmi.rs
/// responsible for all human-machine interfacing (SPI display, buttons, LEDs, etc)

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use crate::hmi::state::HmiState;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::{Duration, Instant, Timer};
pub use neopixel::{Color, NeoPixel};

pub async fn start_hmi(
    mut user_led: Output<'static>,
    led_rate_hz: u32,
    mut neopixel: NeoPixel<'static>,
    neopixel_brightness: u8,
) {
    info!("[HMI] Task started (LED + NeoPixel, {} Hz)", led_rate_hz);
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

        if gps_fix_recent {
            // GPS fix -> solid
            neopixel
                .set_color_with_brightness(color, neopixel_brightness)
                .await;
        } else if gps_rx_recent {
            // GPS connected, no fix -> fast double pulse (0.1s on, 0.1s off, 0.1s on, 0.7s off)
            neopixel
                .set_color_with_brightness(color, neopixel_brightness)
                .await;
            Timer::after_millis(100).await;
            neopixel.clear().await;
            Timer::after_millis(100).await;
            neopixel
                .set_color_with_brightness(color, neopixel_brightness)
                .await;
            Timer::after_millis(100).await;
            neopixel.clear().await;
            Timer::after_millis(700).await;
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

/// HMI loop for floating (GPS-only) node: GREEN = GPS fix when idle; when acquiring,
/// solid orange 50ms then rapid orange blink (50 Hz) until acquisition done.
pub async fn start_hmi_floating(
    mut user_led: Output<'static>,
    led_rate_hz: u32,
    mut neopixel: NeoPixel<'static>,
    neopixel_brightness: u8,
) {
    info!("[HMI] Floating task started (LED + NeoPixel, {} Hz)", led_rate_hz);
    // Immediate test: flash NeoPixel at high brightness so we confirm it's driven (pin GPIO18)
    neopixel
        .set_color_with_brightness(Color::Green, 80)
        .await;
    Timer::after_millis(500).await;
    neopixel.clear().await;
    Timer::after_millis(200).await;

    let period = Duration::from_hz(led_rate_hz as u64);
    let mut led_on = false;
    let mut slow_blink_on = false;

    loop {
        let tick_start = Instant::now();
        trace!("[HMI floating] tick");

        if led_on {
            user_led.set_low();
        } else {
            user_led.set_high();
        }
        led_on = !led_on;

        let state = crate::hmi::state::HMI_STATE.0.lock().await;
        let now = Instant::now();
        let acquiring = state.acquiring_gps_capture;
        let gps_fix_recent = state.gps_fix
            && state
                .last_gps_timestamp
                .map(|ts| now.duration_since(ts).as_secs() <= 3)
                .unwrap_or(false);
        let gps_rx_recent = state
            .last_gps_rx_timestamp
            .map(|ts| now.duration_since(ts).as_secs() <= 3)
            .unwrap_or(false);
        drop(state);

        if acquiring {
            // Solid orange 50 ms
            neopixel
                .set_color_with_brightness(Color::Orange, neopixel_brightness)
                .await;
            Timer::after_millis(50).await;
            // Rapid blink orange (50 Hz) until acquiring clears (up to ~500 ms total for 5 samples)
            let blink_period_ms = 20u64; // 50 Hz
            let blink_start = Instant::now();
            loop {
                let still_acquiring =
                    crate::hmi::state::HMI_STATE.0.lock().await.acquiring_gps_capture;
                if !still_acquiring || blink_start.elapsed().as_millis() >= 600 {
                    break;
                }
                neopixel
                    .set_color_with_brightness(Color::Orange, neopixel_brightness)
                    .await;
                Timer::after_millis(blink_period_ms / 2).await;
                neopixel.clear().await;
                Timer::after_millis(blink_period_ms / 2).await;
            }
            continue;
        }

        // Idle: same green GPS fix indicator as main board
        if gps_fix_recent {
            neopixel
                .set_color_with_brightness(Color::Green, neopixel_brightness)
                .await;
        } else if gps_rx_recent {
            neopixel
                .set_color_with_brightness(Color::Green, neopixel_brightness)
                .await;
            Timer::after_millis(100).await;
            neopixel.clear().await;
            Timer::after_millis(100).await;
            neopixel
                .set_color_with_brightness(Color::Green, neopixel_brightness)
                .await;
            Timer::after_millis(100).await;
            neopixel.clear().await;
            Timer::after_millis(700).await;
        } else {
            slow_blink_on = !slow_blink_on;
            if slow_blink_on {
                neopixel
                    .set_color_with_brightness(Color::Green, neopixel_brightness)
                    .await;
            } else {
                neopixel.clear().await;
            }
        }

        let elapsed = tick_start.elapsed();
        if elapsed < period {
            Timer::after(period - elapsed).await;
        }
    }
}
