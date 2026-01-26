pub mod state;
pub mod neopixel;

use esp_hal::gpio::Output;
/// hmi.rs
/// responsible for all human-machine interfacing (SPI display, buttons, LEDs, etc)

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use embassy_time::{Duration, Timer, Instant};
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
    let mut wheel_pos = 0;

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

        if state.is_bridge {
            if state.usb_host_connected {
                // Solid white for bridge + connected
                neopixel
                    .set_color_with_brightness(Color::White, neopixel_brightness)
                    .await;
            } else {
                // Rainbow wheel for bridge, waiting for USB
                let (r, g, b) = wheel(wheel_pos);
                neopixel
                    .set_color_with_brightness(Color::Custom(r, g, b), neopixel_brightness)
                    .await;
                wheel_pos = wheel_pos.wrapping_add(1);
            }
        } else {
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
        }
        drop(state);

        // Wait for the next heartbeat tick
        let elapsed = tick_start.elapsed();
        if elapsed < period {
            Timer::after(period - elapsed).await;
        }
    }
}

fn wheel(pos: u8) -> (u8, u8, u8) {
    if pos < 85 {
        (255 - pos * 3, pos * 3, 0)
    } else if pos < 170 {
        let pos = pos - 85;
        (0, 255 - pos * 3, pos * 3)
    } else {
        let pos = pos - 170;
        (pos * 3, 0, 255 - pos * 3)
    }
}

