//! Library for rpi_rx_rust: shared AHRS logic, types, and session.

pub mod ahrs;
pub mod session;
pub mod timesync;
pub mod track_capture;
pub mod types;

pub use ahrs::{AhrsFilter, AhrsOutputRow};
pub use session::{run_session, ByteSource};
pub use timesync::{encode_timesync_command, TimeSyncSender};
pub use track_capture::{
    encode_request_floating_gps_command, request_floating_gps, FloatingGpsResult,
};
pub use types::{format_mac_address, parse_message, DecodedMessage, ParserError};
