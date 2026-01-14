//! Error types for the air communication module

use core::fmt;
use esp_radio::esp_now::EspNowError;

/// Result type for AirComm operations
pub type Result<T> = core::result::Result<T, AirCommError>;

/// Errors that can occur in the air communication module
#[derive(Debug)]
pub enum AirCommError {
    /// ESP-NOW underlying error
    EspNowError(EspNowError),

    /// Buffer too small for serialization
    BufferTooSmall,

    /// Invalid message format
    InvalidMessage,

    /// Unknown message type
    UnknownMessageType,
}

impl fmt::Display for AirCommError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AirCommError::EspNowError(e) => write!(f, "ESP-NOW error: {}", e),
            AirCommError::BufferTooSmall => write!(f, "Buffer too small for serialization"),
            AirCommError::InvalidMessage => write!(f, "Invalid message format"),
            AirCommError::UnknownMessageType => write!(f, "Unknown message type"),
        }
    }
}

impl From<EspNowError> for AirCommError {
    fn from(e: EspNowError) -> Self {
        AirCommError::EspNowError(e)
    }
}
