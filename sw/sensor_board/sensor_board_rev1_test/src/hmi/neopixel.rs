//! Async WS2812/NeoPixel driver using ESP32 RMT peripheral.
//!
//! # References
//! - WS2812B Datasheet: <https://cdn-shop.adafruit.com/datasheets/WS2812B.pdf>
//! - esp-hal RMT examples: <https://github.com/esp-rs/esp-hal/tree/main/examples>

use esp_hal::gpio::{Level, OutputPin};
use esp_hal::rmt::{Channel, PulseCode, TxChannelConfig, TxChannelCreator, Tx};
use log::warn;

/// Simple color enum for a single NeoPixel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Off,
    Red,
    Green,
    Blue,
    Yellow,
    Cyan,
    Magenta,
    White,
    Orange,
    Purple,
    Custom(u8, u8, u8), // (R, G, B)
}

impl Color {
    pub const fn all_colors() -> [Color; 9] {
        [
            Color::Red,
            Color::Orange,
            Color::Yellow,
            Color::Green,
            Color::Cyan,
            Color::Blue,
            Color::Purple,
            Color::Magenta,
            Color::White,
        ]
    }

    pub const fn to_rgb(&self) -> (u8, u8, u8) {
        match self {
            Color::Off => (0, 0, 0),
            Color::Red => (255, 0, 0),
            Color::Green => (0, 255, 0),
            Color::Blue => (0, 0, 255),
            Color::Yellow => (255, 255, 0),
            Color::Cyan => (0, 255, 255),
            Color::Magenta => (255, 0, 255),
            Color::White => (255, 255, 255),
            Color::Orange => (255, 165, 0),
            Color::Purple => (128, 0, 128),
            Color::Custom(r, g, b) => (*r, *g, *b),
        }
    }
}

/// Minimal NeoPixel driver built directly on esp-hal RMT async channel.
pub struct NeoPixel<'d> {
    channel: Channel<'d, esp_hal::Async, Tx>,
    buffer: [PulseCode; NeoPixel::FRAME_LEN],
}

impl<'d> NeoPixel<'d> {
    // 24 bits + reset + end marker
    const FRAME_LEN: usize = 26;

    /// Create a new NeoPixel driver from an RMT TX channel and GPIO pin.
    pub fn new<CH, P>(channel: CH, pin: P) -> Self
    where
        CH: TxChannelCreator<'d, esp_hal::Async>,
        P: OutputPin + 'd,
    {
        // Configure RMT for WS2812: clock divider of 1 gives us 80MHz ticks (12.5ns each)
        // Enable idle output so the line goes low after transmission
        let config = TxChannelConfig::default()
            .with_clk_divider(1)
            .with_idle_output_level(Level::Low)
            .with_idle_output(true);
        
        let tx = channel
            .configure_tx(&config)
            .unwrap()
            .with_pin(pin);

        Self {
            channel: tx,
            buffer: [PulseCode::end_marker(); NeoPixel::FRAME_LEN],
        }
    }

    /// Async set color (non-blocking RMT transfer).
    pub async fn set_color(&mut self, color: Color) {
        self.fill_buffer(color);
        if let Err(e) = self.channel.transmit(&self.buffer).await {
            warn!("[NeoPixel] Transmit error: {:?}", e);
        }
    }

    /// Async set color with brightness scaling (0-255).
    pub async fn set_color_with_brightness(&mut self, color: Color, brightness: u8) {
        let (r, g, b) = color.to_rgb();
        let scale = brightness as u16;
        let r_scaled = ((r as u16 * scale) / 255) as u8;
        let g_scaled = ((g as u16 * scale) / 255) as u8;
        let b_scaled = ((b as u16 * scale) / 255) as u8;
        self.set_color(Color::Custom(r_scaled, g_scaled, b_scaled)).await;
    }

    pub async fn clear(&mut self) {
        self.set_color(Color::Off).await;
    }

    fn fill_buffer(&mut self, color: Color) {
        // WS2812B timing @ 80MHz RMT clock (12.5ns per tick)
        // Reference: WS2812B Datasheet - Data Transfer Time
        //   T0H: 0.4us ±150ns (high for 0-bit)
        //   T0L: 0.85us ±150ns (low for 0-bit)  
        //   T1H: 0.8us ±150ns (high for 1-bit)
        //   T1L: 0.45us ±150ns (low for 1-bit)
        //   RES: >50us (reset/latch)
        const T0H: u16 = 28;   // 0.35us (28 * 12.5ns)
        const T0L: u16 = 64;   // 0.80us (64 * 12.5ns)
        const T1H: u16 = 56;   // 0.70us (56 * 12.5ns)
        const T1L: u16 = 48;   // 0.60us (48 * 12.5ns)
        const RESET: u16 = 4000; // 50us (4000 * 12.5ns)

        let (r, g, b) = color.to_rgb();
        // WS2812 wants GRB order
        let bits = [
            g, r, b,
        ];

        let mut idx = 0;
        for byte in bits {
            for bit in (0..8).rev() {
                let one = (byte >> bit) & 1 == 1;
                let code = if one {
                    PulseCode::new(Level::High, T1H, Level::Low, T1L)
                } else {
                    PulseCode::new(Level::High, T0H, Level::Low, T0L)
                };
                self.buffer[idx] = code;
                idx += 1;
            }
        }
        // reset/latch pulse (50us low, then a tiny low pulse to avoid length2=0 looking like end marker)
        self.buffer[idx] = PulseCode::new(Level::Low, RESET, Level::Low, 1);
        idx += 1;
        // end marker
        self.buffer[idx] = PulseCode::end_marker();
    }
}
