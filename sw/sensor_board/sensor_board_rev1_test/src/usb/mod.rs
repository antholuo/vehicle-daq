/// USB Serial Interface Module
///
/// Provides a wrapper around esp-hal's USB Serial/JTAG peripheral for
/// communicating sensor data to a host (e.g., Raspberry Pi).
///
/// Uses COBS (Consistent Overhead Byte Stuffing) framing to ensure
/// reliable message delimiting over the serial connection.

pub mod protocol;

use esp_hal::usb_serial_jtag::UsbSerialJtagTx;
use esp_hal::Async;

pub use protocol::{serialize_forwarded_message, format_mac, ForwardError, MAX_USB_MESSAGE_SIZE};

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

/// Maximum size of a single message before COBS encoding
/// Header (8 bytes) + Payload (up to 64 bytes for GPS)
pub const MAX_MESSAGE_SIZE: usize = 128;

/// COBS encoding adds at most 1 byte per 254 bytes, plus 1 overhead byte and 1 delimiter (0x00)
pub const MAX_ENCODED_SIZE: usize = MAX_MESSAGE_SIZE + (MAX_MESSAGE_SIZE / 254) + 2;

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

        // Write the encoded data
        self.tx.write(&self.encode_buffer[..encoded_len]).unwrap();

        // Write the frame delimiter (0x00)
        self.tx.write(&[0x00]).unwrap();

        // Flush to ensure data is sent
        self.tx.flush_tx().unwrap();

        trace!("[USB] Sent {} bytes ({} encoded + delimiter)", data.len(), encoded_len);

        Ok(())
    }

    /// Write raw bytes without COBS framing (for debugging)
    #[allow(dead_code)]
    pub fn write_raw(&mut self, data: &[u8]) -> Result<(), UsbError> {
        self.tx.write(data).unwrap();
        self.tx.flush_tx().unwrap();
        Ok(())
    }
}

/// USB Serial errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbError {
    /// Message exceeds maximum allowed size
    MessageTooLarge,
}

