/// USB Serial Interface Module
///
/// Provides a wrapper around esp-hal's USB Serial/JTAG peripheral for
/// communicating sensor data to a host (e.g., Raspberry Pi).
///
/// Uses COBS (Consistent Overhead Byte Stuffing) framing to ensure
/// reliable message delimiting over the serial connection.
pub mod protocol;

use embassy_time::{Duration, Instant, Timer};
use esp_hal::Async;
use esp_hal::usb_serial_jtag::{UsbSerialJtagRx, UsbSerialJtagTx};
use nb::Error as NbError;

pub use protocol::{
    ForwardError, MAX_USB_MESSAGE_SIZE, UsbCommand, format_mac, parse_usb_command,
    serialize_forwarded_message,
};

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

/// Maximum size of a single message before COBS encoding
/// Header (8 bytes) + Payload (up to 64 bytes for GPS)
pub const MAX_MESSAGE_SIZE: usize = 128;

/// COBS encoding adds at most 1 byte per 254 bytes, plus 1 overhead byte and 1 delimiter (0x00)
pub const MAX_ENCODED_SIZE: usize = MAX_MESSAGE_SIZE + (MAX_MESSAGE_SIZE / 254) + 2;
const USB_WRITE_TIMEOUT: Duration = Duration::from_millis(30);
const USB_WRITE_RETRY_DELAY: Duration = Duration::from_micros(250);
const USB_HOST_POLL_INTERVAL: Duration = Duration::from_millis(100);

/// USB Serial writer with COBS framing support
pub struct UsbSerial {
    tx: UsbSerialJtagTx<'static, Async>,
    /// Buffer for COBS encoding
    encode_buffer: [u8; MAX_ENCODED_SIZE],
}

impl UsbSerial {
    /// Create a new USB serial interface from the TX half
    pub fn new(tx: UsbSerialJtagTx<'static, Async>) -> Self {
        Self {
            tx,
            encode_buffer: [0u8; MAX_ENCODED_SIZE],
        }
    }

    /// Write a COBS-framed message
    ///
    /// The message is COBS-encoded and terminated with a 0x00 delimiter byte.
    /// This allows the receiver to reliably detect message boundaries.
    ///
    /// # Arguments
    /// * `data` - Raw message data to send (will be COBS encoded)
    ///
    /// # Returns
    /// * `Ok(())` on success
    /// * `Err(UsbError::MessageTooLarge)` if data exceeds MAX_MESSAGE_SIZE
    pub async fn write_framed(&mut self, data: &[u8]) -> Result<(), UsbError> {
        if data.len() > MAX_MESSAGE_SIZE {
            return Err(UsbError::MessageTooLarge);
        }

        // COBS encode the data
        let encoded_len = cobs::encode(data, &mut self.encode_buffer);

        // Write the encoded data (non-blocking with timeout to avoid hanging when USB is idle)
        for i in 0..encoded_len {
            let byte = self.encode_buffer[i];
            self.write_byte_with_timeout(byte).await?;
        }

        // Write the frame delimiter (0x00)
        self.write_byte_with_timeout(0x00).await?;

        // Flush to ensure data is sent
        self.flush_with_timeout().await?;

        trace!(
            "[USB] Sent {} bytes ({} encoded + delimiter)",
            data.len(),
            encoded_len
        );

        Ok(())
    }

    /// Write raw bytes without COBS framing (for debugging)
    #[allow(dead_code)]
    pub fn write_raw(&mut self, data: &[u8]) -> Result<(), UsbError> {
        self.tx.write(data).unwrap();
        self.tx.flush_tx().unwrap();
        Ok(())
    }

    async fn write_byte_with_timeout(&mut self, byte: u8) -> Result<(), UsbError> {
        let start = Instant::now();
        loop {
            match self.tx.write_byte_nb(byte) {
                Ok(()) => return Ok(()),
                Err(NbError::WouldBlock) => {
                    if start.elapsed() >= USB_WRITE_TIMEOUT {
                        return Err(UsbError::NotReady);
                    }
                    Timer::after(USB_WRITE_RETRY_DELAY).await;
                }
                Err(NbError::Other(_)) => return Err(UsbError::NotReady),
            }
        }
    }

    async fn flush_with_timeout(&mut self) -> Result<(), UsbError> {
        let start = Instant::now();
        loop {
            match self.tx.flush_tx_nb() {
                Ok(()) => return Ok(()),
                Err(NbError::WouldBlock) => {
                    if start.elapsed() >= USB_WRITE_TIMEOUT {
                        return Err(UsbError::NotReady);
                    }
                    Timer::after(USB_WRITE_RETRY_DELAY).await;
                }
                Err(NbError::Other(_)) => return Err(UsbError::NotReady),
            }
        }
    }
}

// =========================================================================
// USB Serial RX (RPi -> Bridge commands)
// =========================================================================

/// COBS-decoded buffer size for incoming commands (largest command is 9 bytes)
const USB_RX_DECODED_BUF_SIZE: usize = 32;
/// Max COBS-encoded frame we'll accumulate before discarding
const USB_RX_FRAME_MAX: usize = 64;

/// USB Serial reader that accumulates COBS-framed commands from the host.
pub struct UsbSerialRx {
    rx: UsbSerialJtagRx<'static, Async>,
    frame_buf: [u8; USB_RX_FRAME_MAX],
    frame_len: usize,
}

impl UsbSerialRx {
    pub fn new(rx: UsbSerialJtagRx<'static, Async>) -> Self {
        Self {
            rx,
            frame_buf: [0u8; USB_RX_FRAME_MAX],
            frame_len: 0,
        }
    }

    /// Non-blocking poll: read any available bytes and try to decode a
    /// complete COBS frame.  Returns `Some(cmd)` when a full command has
    /// been received, `None` otherwise (no data or incomplete frame).
    pub fn poll_command(&mut self) -> Option<UsbCommand> {
        loop {
            match self.rx.read_byte() {
                Ok(0x00) => {
                    // COBS delimiter -- decode the accumulated frame
                    if self.frame_len == 0 {
                        continue;
                    }
                    let mut decoded = [0u8; USB_RX_DECODED_BUF_SIZE];
                    let result =
                        cobs::decode(&self.frame_buf[..self.frame_len], &mut decoded);
                    self.frame_len = 0;
                    if let Ok(decoded_len) = result {
                        if let Ok(cmd) = parse_usb_command(&decoded[..decoded_len]) {
                            return Some(cmd);
                        }
                    }
                    // Malformed frame -- discard and keep going
                }
                Ok(byte) => {
                    if self.frame_len < USB_RX_FRAME_MAX {
                        self.frame_buf[self.frame_len] = byte;
                        self.frame_len += 1;
                    } else {
                        // Overflow -- discard frame
                        self.frame_len = 0;
                    }
                }
                Err(NbError::WouldBlock) => return None,
                Err(NbError::Other(_)) => return None,
            }
        }
    }
}

/// Wait until a USB host is detected (ESP32-C6 USB Serial/JTAG SOF flag).
pub async fn wait_for_usb_host_timeout(timeout: Duration) -> bool {
    let start = Instant::now();
    while !usb_host_connected() {
        if start.elapsed() >= timeout {
            return false;
        }
        Timer::after(USB_HOST_POLL_INTERVAL).await;
    }
    true
}

fn usb_host_connected() -> bool {
    // USB_DEVICE_INT_RAW register for ESP32-C6. SOF bit indicates host traffic.
    const USB_DEVICE_INT_RAW: *const u32 = 0x6000_f008 as *const u32;
    const SOF_INT_MASK: u32 = 0b10;
    unsafe { (USB_DEVICE_INT_RAW.read_volatile() & SOF_INT_MASK) != 0 }
}

/// USB Serial errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbError {
    /// Message exceeds maximum allowed size
    MessageTooLarge,
    /// USB host not ready (avoid blocking forever when not connected)
    NotReady,
}
