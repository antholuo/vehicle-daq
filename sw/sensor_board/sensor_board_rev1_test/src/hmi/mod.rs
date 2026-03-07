pub mod neopixel;
pub mod state;

use esp_hal::gpio::Output;
/// hmi.rs
/// responsible for all human-machine interfacing (SPI display, buttons, LEDs, etc)

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use crate::hmi::state::{CAPTURE_PHASE_IDLE, CAPTURE_PHASE_REQUEST_SENT, CAPTURE_PHASE_SUCCESS, CAPTURE_PHASE_TIMEOUT};
use embassy_time::{Duration, Instant, Timer};
pub use neopixel::{Color, NeoPixel};

/// Rainbow wheel: position 0..255 -> (r, g, b)
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

pub async fn start_hmi(
    mut user_led: Output<'static>,
    led_rate_hz: u32,
    mut neopixel: NeoPixel<'static>,
    neopixel_brightness: u8,
) {
    info!("[HMI] Task started (LED + NeoPixel, {} Hz)", led_rate_hz);

    let mut led_on = false;
    let mut slow_blink_on = false;
    let led_period = Duration::from_hz(led_rate_hz as u64);
    let mut last_led_toggle = Instant::now();

    loop {
        let tick_start = Instant::now();
        trace!("[HMI] - Heartbeat OK");

        // Toggle user LED at normal rate (1 Hz) even when loop is fast for breathing
        let now_early = Instant::now();
        if now_early.duration_since(last_led_toggle) >= led_period {
            if led_on {
                user_led.set_low();
            } else {
                user_led.set_high();
            }
            led_on = !led_on;
            last_led_toggle = now_early;
        }

        // Read HMI state to decide NeoPixel behaviour
        let mut state = crate::hmi::state::HMI_STATE.0.lock().await;
        let now = Instant::now();

        // Clear capture phase when deadline passed
        if state.capture_led_phase == CAPTURE_PHASE_SUCCESS || state.capture_led_phase == CAPTURE_PHASE_TIMEOUT {
            if let Some(deadline) = state.capture_phase_deadline {
                if now >= deadline {
                    state.capture_led_phase = CAPTURE_PHASE_IDLE;
                    state.capture_phase_deadline = None;
                }
            }
        }

        // Loop period: 4x when armed (bridge); when breathing (GPS fix) use 40 Hz for smooth animation; else base rate
        let gps_fix_recent = state.gps_fix
            && state
                .last_gps_timestamp
                .map(|ts| now.duration_since(ts).as_secs() <= 3)
                .unwrap_or(false);
        let period = if state.capture_led_phase != CAPTURE_PHASE_IDLE {
            Duration::from_millis(50) // Fast tick so we clear phase when deadline passes
        } else if state.track_armed {
            Duration::from_hz((led_rate_hz * 4) as u64)
        } else if gps_fix_recent {
            Duration::from_millis(25) // 40 Hz for smooth breathing
        } else {
            Duration::from_hz(led_rate_hz as u64)
        };

        // Bridge track capture overrides: RequestSent -> orange; Success -> green 1s; Timeout -> rainbow 1s
        if state.capture_led_phase == CAPTURE_PHASE_REQUEST_SENT {
            drop(state);
            neopixel
                .set_color_with_brightness(Color::Orange, neopixel_brightness)
                .await;
        } else if state.capture_led_phase == CAPTURE_PHASE_SUCCESS {
            drop(state);
            neopixel
                .set_color_with_brightness(Color::Green, neopixel_brightness)
                .await;
        } else if state.capture_led_phase == CAPTURE_PHASE_TIMEOUT {
            let deadline = state.capture_phase_deadline.unwrap_or(now);
            let phase_start = deadline.saturating_sub(Duration::from_secs(1));
            let elapsed = now.saturating_duration_since(phase_start);
            let pos = ((elapsed.as_millis() % 1000) * 256 / 1000) as u8;
            let (r, g, b) = wheel(pos);
            drop(state);
            neopixel
                .set_color_with_brightness(Color::Custom(r, g, b), neopixel_brightness)
                .await;
        } else {
            // Normal bridge/rev1 behaviour
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
            drop(state);

            if gps_fix_recent {
                // GPS fix -> breathe (25% to max brightness) for clear "alive" indication
                let breath_period_ms: u64 = 2000;
                let elapsed_ms = (now.as_millis() as u64) % breath_period_ms;
                let phase_256 = (elapsed_ms * 256 / breath_period_ms) as u32;
                let factor = if phase_256 <= 128 {
                    64 + (phase_256 * 192) / 128
                } else {
                    64 + ((256 - phase_256) * 192) / 128
                };
                let breath_brightness = (neopixel_brightness as u32 * factor / 256).min(255) as u8;
                neopixel
                    .set_color_with_brightness(color, breath_brightness)
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

        // Wait for the next heartbeat tick
        let elapsed = tick_start.elapsed();
        if elapsed < period {
            Timer::after(period - elapsed).await;
        }
    }
}

/// HMI loop for floating (GPS-only) node: GREEN = GPS fix when idle (breathing); when acquiring,
/// 500ms solid orange (operator notice) then rapid orange blink until 5 samples done.
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

    // Fast idle tick (40 Hz) so we see acquiring_gps_capture within 25 ms and get smooth breathing
    const IDLE_TICK_MS: u64 = 25;
    let period = Duration::from_millis(IDLE_TICK_MS);
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
            // 500 ms solid orange so operator sees the capture request
            neopixel
                .set_color_with_brightness(Color::Orange, neopixel_brightness)
                .await;
            Timer::after_millis(500).await;
            // Rapid blink orange until acquiring clears (collecting 5 samples)
            let blink_period_ms = 20u64; // 50 Hz
            let blink_start = Instant::now();
            loop {
                let still_acquiring =
                    crate::hmi::state::HMI_STATE.0.lock().await.acquiring_gps_capture;
                if !still_acquiring || blink_start.elapsed().as_millis() >= 2000 {
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

        // Idle: green GPS fix with breathe (25% to max) for "alive" indication
        if gps_fix_recent {
            let breath_period_ms: u64 = 2000;
            let elapsed_ms = (now.as_millis() as u64) % breath_period_ms;
            let phase_256 = (elapsed_ms * 256 / breath_period_ms) as u32;
            let factor = if phase_256 <= 128 {
                64 + (phase_256 * 192) / 128
            } else {
                64 + ((256 - phase_256) * 192) / 128
            };
            let breath_brightness = (neopixel_brightness as u32 * factor / 256).min(255) as u8;
            neopixel
                .set_color_with_brightness(Color::Green, breath_brightness)
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
