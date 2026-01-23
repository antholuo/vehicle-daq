//! Simple no-std NeoPixel/WS2812 driver for ESP32-C6

use esp_hal::gpio::OutputPin;
use esp_hal_smartled::{buffer_size, color_order, RmtSmartLeds, Ws2812Timing};
use smart_leds::{SmartLedsWrite, RGB8};

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
    /// Get an array of all predefined colors (excluding Off and Custom)
    /// Useful for cycling through colors in animations
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

    /// Convert Color to a RGB values tuple
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

    /// Convert to RGB8 type used by smart-leds
    fn to_rgb8(&self) -> RGB8 {
        let (r, g, b) = self.to_rgb();
        RGB8 { r, g, b }
    }
}

/// NeoPixel driver - wraps RmtSmartLeds for single LED control
pub struct NeoPixel<'d> {
    led: RmtSmartLeds<
        'd,
        { buffer_size::<RGB8>(1) },
        esp_hal::Blocking,
        RGB8,
        color_order::Grb,
        Ws2812Timing,
    >,
}

impl<'d> NeoPixel<'d> {
    /// Create new NeoPixel driver using an RMT channel
    ///
    /// # Example
    /// ```ignore
    /// let rmt = Rmt::new(peripherals.RMT, 80.MHz()).unwrap();
    /// let mut neopixel = NeoPixel::new(rmt.channel0, io.pins.gpio18);
    /// ```
    pub fn new<CH, P>(channel: CH, pin: P) -> Self
    where
        CH: esp_hal::rmt::TxChannelCreator<'d, esp_hal::Blocking>,
        P: OutputPin + 'd,
    {
        let led = RmtSmartLeds::new_with_memsize(channel, pin, 2).unwrap();
        Self { led }
    }

    /// Set the NeoPixel to a specific color
    pub fn set_color(&mut self, color: Color) {
        let rgb = color.to_rgb8();
        let _ = self.led.write([rgb].iter().cloned());
    }

    /// Set color with brightness scaling
    ///
    /// # Arguments
    /// * `color` - The color to display
    /// * `brightness` - Brightness level from 0 (off) to 255 (full brightness)
    pub fn set_color_with_brightness(&mut self, color: Color, brightness: u8) {
        let (r, g, b) = color.to_rgb();
        let scale = brightness as u16;

        let r_scaled = ((r as u16 * scale) / 255) as u8;
        let g_scaled = ((g as u16 * scale) / 255) as u8;
        let b_scaled = ((b as u16 * scale) / 255) as u8;

        self.set_color(Color::Custom(r_scaled, g_scaled, b_scaled));
    }

    /// Turn off the NeoPixel
    pub fn clear(&mut self) {
        self.set_color(Color::Off);
    }
}
