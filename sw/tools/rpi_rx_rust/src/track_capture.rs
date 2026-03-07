//! Track capture: request floating GPS from bridge, average 5 samples, write to CSV.
//!
//! Protocol: RPi sends CMD_REQUEST_FLOATING_GPS (0x02) COBS-framed; bridge broadcasts
//! RequestGpsCapture, collects 5 GPS from floating node, forwards as 5 normal GPS frames to RPi.

use cobs::decode;
use log::*;
use std::collections::VecDeque;
use std::io::Write;

use crate::types::parse_message;

const CMD_REQUEST_FLOATING_GPS: u8 = 0x02;
const COBS_DECODED_BUFFER_SIZE: usize = 128;
const NUM_GPS_SAMPLES: usize = 5;

/// Bridge HMI commands (RPi -> Bridge over USB). Same values as bridge firmware CMD_*.
pub const CMD_ARM_TRACK: u8 = 0x03;
pub const CMD_END_TRACK: u8 = 0x04;
pub const CMD_CAPTURE_SUCCESS: u8 = 0x05;
pub const CMD_CAPTURE_TIMEOUT: u8 = 0x06;

/// Encode and return COBS-framed 1-byte bridge HMI command (e.g. arm/end/success/timeout).
pub fn encode_bridge_hmi_command(cmd: u8) -> Vec<u8> {
    let raw = [cmd];
    let max_encoded = cobs::max_encoding_length(1);
    let mut encoded = vec![0u8; max_encoded + 1];
    let n = cobs::encode(&raw, &mut encoded);
    encoded.truncate(n + 1);
    encoded[n] = 0x00;
    encoded
}

/// Encode and return COBS-framed RequestFloatingGps command (1 byte payload + delimiter).
pub fn encode_request_floating_gps_command() -> Vec<u8> {
    let raw = [CMD_REQUEST_FLOATING_GPS];
    let max_encoded = cobs::max_encoding_length(1);
    let mut encoded = vec![0u8; max_encoded + 1];
    let n = cobs::encode(&raw, &mut encoded);
    encoded.truncate(n + 1);
    encoded[n] = 0x00;
    encoded
}

/// Result of one floating GPS capture: averaged position (and optional debug fields).
#[derive(Debug, Clone)]
pub struct FloatingGpsResult {
    pub lat: f64,
    pub lon: f64,
    pub alt: f32,
    pub num_samples: usize,
    pub hdop: Option<f32>,
    pub num_sats: Option<u32>,
}

/// Request floating GPS from bridge and read 5 GPS frames from the byte stream.
/// Caller must provide the next byte from the same serial stream (e.g. the session's byte source)
/// and a writer to send the command. Returns averaged lat/lon/alt or error.
///
/// This consumes bytes from the stream until 5 GPS messages are received or timeout.
pub fn request_floating_gps<B, W>(
    byte_source: &mut B,
    writer: &mut W,
    timeout_ms: u64,
) -> Result<FloatingGpsResult, Box<dyn std::error::Error + Send + Sync>>
where
    B: crate::session::ByteSource,
    W: Write,
{
    let cmd = encode_request_floating_gps_command();
    writer.write_all(&cmd)?;
    writer.flush()?;
    info!("Track capture: sent RequestFloatingGps, waiting for 5 GPS samples");

    let mut cobs_buf: VecDeque<u8> = VecDeque::new();
    let mut decoded_buf = [0u8; COBS_DECODED_BUFFER_SIZE];
    let mut samples: Vec<(f64, f64, f32)> = Vec::with_capacity(NUM_GPS_SAMPLES);
    let start = std::time::Instant::now();

    while samples.len() < NUM_GPS_SAMPLES && start.elapsed().as_millis() < timeout_ms as u128 {
        let byte_opt = byte_source.next_byte();
        let byte = match byte_opt {
            Some(Ok(b)) => b,
            Some(Err(e)) => return Err(e.into()),
            None => continue,
        };

        cobs_buf.push_back(byte);
        if byte != 0x00 {
            continue;
        }

        let frame: Vec<u8> = cobs_buf.drain(..).collect();
        let to_decode = if frame.last() == Some(&0x00) {
            &frame[..frame.len().saturating_sub(1)]
        } else {
            &frame[..]
        };
        if to_decode.is_empty() {
            continue;
        }

        match decode(to_decode, &mut decoded_buf) {
            Ok(decoded_len) => {
                let raw = &decoded_buf[..decoded_len];
                match parse_message(raw) {
                    Ok(msg) => {
                        if msg.message_type == "Gps" {
                            if let (Some(lat), Some(lon), Some(alt)) =
                                (msg.lat, msg.lon, msg.alt)
                            {
                                samples.push((lat, lon, alt));
                                info!(
                                    "Track capture: GPS sample {}/{} lat={:.6} lon={:.6}",
                                    samples.len(),
                                    NUM_GPS_SAMPLES,
                                    lat,
                                    lon
                                );
                            }
                        }
                    }
                    Err(_) => {}
                }
            }
            Err(_) => {}
        }
    }

    if samples.is_empty() {
        return Err("No GPS samples received within timeout".into());
    }

    let n = samples.len();
    let (sum_lat, sum_lon, sum_alt) = samples.into_iter().fold(
        (0.0_f64, 0.0_f64, 0.0_f32),
        |(slat, slon, salt), (lat, lon, alt)| (slat + lat, slon + lon, salt + alt),
    );
    Ok(FloatingGpsResult {
        lat: sum_lat / (n as f64),
        lon: sum_lon / (n as f64),
        alt: sum_alt / (n as f32),
        num_samples: n,
        hdop: None,
        num_sats: None,
    })
}
