//! Logging session: byte source abstraction and run loop (decode COBS, write CSV, optional AHRS).
//! Optional track capture: when collecting_for_track is set, next 5 GPS are averaged and passed to callback.

use crate::ahrs::AhrsFilter;
use crate::track_capture::{CMD_CAPTURE_SUCCESS, CMD_CAPTURE_TIMEOUT};
use crate::types::{error_decoded_message, format_mac_address, parse_message};
use cobs::decode;
use csv::Writer;
use log::*;
use std::collections::VecDeque;
use std::io;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const COBS_DECODED_BUFFER_SIZE: usize = 128;
const AHRS_SAMPLE_PERIOD_S: f64 = 1.0 / 400.0;
const AHRS_BETA: f64 = 0.1;
const TRACK_CAPTURE_NUM_GPS: usize = 5;
/// If we don't receive 5 GPS samples within this time, we clear the capture and stop waiting.
const TRACK_CAPTURE_TIMEOUT: Duration = Duration::from_secs(5);

/// Source of bytes (e.g. serial with timeout, or stdin). Returns `None` when no byte available (e.g. timeout).
pub trait ByteSource {
    fn next_byte(&mut self) -> Option<io::Result<u8>>;
}

/// Runs the decode/write loop until EOF or `should_stop()` returns true.
/// Logs raw CSV to `raw_wtr`; if `ahrs_wtr` is `Some`, also runs AHRS and writes at `ahrs_hz`.
/// If `collecting_for_track` and `on_track_capture` are both `Some`, when the flag is true the next 5 GPS
/// messages are averaged and the callback is invoked with (lat, lon, alt).
/// If `bridge_pending_cmd` is `Some`, on success the session stores CMD_CAPTURE_SUCCESS for the bridge
/// thread to send; on timeout it stores CMD_CAPTURE_TIMEOUT.
/// If `only_log_node` is `Some("Floating")`, the per-message info! log is only emitted for that node.
/// If `last_message_time_ms` is `Some`, it is updated with current time (ms since UNIX_EPOCH) on every parsed message (for heartbeat timeout).
pub fn run_session<W, W2, B>(
    byte_source: &mut B,
    raw_wtr: &mut Writer<W>,
    mut ahrs_wtr: Option<&mut Writer<W2>>,
    ahrs_hz: u32,
    should_stop: &mut dyn FnMut() -> bool,
    collecting_for_track: Option<&std::sync::atomic::AtomicBool>,
    mut on_track_capture: Option<&mut dyn FnMut(f64, f64, f32)>,
    bridge_pending_cmd: Option<Arc<std::sync::atomic::AtomicU8>>,
    only_log_node: Option<&str>,
    last_message_time_ms: Option<Arc<std::sync::atomic::AtomicU64>>,
) -> Result<(), Box<dyn std::error::Error>>
where
    W: std::io::Write,
    W2: std::io::Write,
    B: ByteSource + ?Sized,
{
    let mut cobs_encoded_buffer: VecDeque<u8> = VecDeque::new();
    let mut decoded_buffer = [0u8; COBS_DECODED_BUFFER_SIZE];

    let mut primary_node_mac: Option<[u8; 6]> = None;
    let mut ahrs_filter: Option<AhrsFilter> = None;
    let mut last_ahrs_emit_ts_us: u64 = 0;
    let ahrs_period_us: u64 = 1_000_000 / (ahrs_hz as u64);
    let with_ahrs = ahrs_wtr.is_some();

    let mut message_count: u64 = 0;
    let mut track_gps_buf: Vec<(f64, f64, f32)> = Vec::with_capacity(TRACK_CAPTURE_NUM_GPS);
    let mut track_capture_start: Option<Instant> = None;

    loop {
        let byte_opt = byte_source.next_byte();

        match byte_opt {
            Some(Ok(byte)) => {
                cobs_encoded_buffer.push_back(byte);

                if byte == 0x00 {
                    let mut frame_buffer = Vec::with_capacity(cobs_encoded_buffer.len());
                    while let Some(b) = cobs_encoded_buffer.pop_front() {
                        frame_buffer.push(b);
                    }

                    let data_to_decode = if frame_buffer.last() == Some(&0x00) {
                        &frame_buffer[..frame_buffer.len() - 1]
                    } else {
                        &frame_buffer[..]
                    };

                    match decode(data_to_decode, &mut decoded_buffer) {
                        Ok(decoded_len) => {
                            let raw_message = &decoded_buffer[..decoded_len];
                            match parse_message(raw_message) {
                                Ok(msg) => {
                                    // Only track-capture heartbeat from bridge updates heartbeat timestamp (not every message).
                                    if msg.message_type == "TrackCaptureHeartbeat" {
                                        if let Some(ref t) = last_message_time_ms {
                                            let _ = SystemTime::now()
                                                .duration_since(UNIX_EPOCH)
                                                .map(|d| t.store(d.as_millis() as u64, Ordering::Relaxed));
                                        }
                                        continue; // Skip CSV/log/track capture for heartbeat
                                    }
                                    message_count += 1;
                                    let should_log = match only_log_node {
                                        Some(node) => msg.node_position == node,
                                        None => true,
                                    };
                                    if should_log {
                                        info!(
                                            "[{}] Type={:?}, MAC={}, Node={}/{}, Timestamp={}us",
                                            message_count,
                                            msg.message_type,
                                            format_mac_address(&msg.src_mac),
                                            msg.node_position,
                                            msg.node_instance,
                                            msg.timestamp_us
                                        );
                                    }
                                    raw_wtr.serialize(&msg)?;
                                    raw_wtr.flush()?;

                                    // Track capture: collect 5 GPS when flag set, then average and callback. Time out after 5s.
                                    if let (Some(collecting), Some(on_capture)) = (
                                        collecting_for_track.as_ref(),
                                        on_track_capture.as_mut(),
                                    ) {
                                        if collecting.load(Ordering::Relaxed) && track_capture_start.is_none() {
                                            track_capture_start = Some(Instant::now());
                                        }
                                        if msg.message_type == "Gps"
                                            && collecting.load(Ordering::Relaxed)
                                            && msg.lat.is_some()
                                            && msg.lon.is_some()
                                            && msg.alt.is_some()
                                        {
                                            if track_capture_start
                                                .as_ref()
                                                .map(|t| t.elapsed() > TRACK_CAPTURE_TIMEOUT)
                                                .unwrap_or(false)
                                            {
                                                let got = track_gps_buf.len();
                                                track_gps_buf.clear();
                                                collecting.store(false, Ordering::Relaxed);
                                                track_capture_start = None;
                                                if let Some(ref a) = bridge_pending_cmd {
                                                    a.store(CMD_CAPTURE_TIMEOUT, Ordering::Relaxed);
                                                }
                                                warn!(
                                                    "Track capture timed out after 5s (got {} samples), cone not recorded",
                                                    got
                                                );
                                            } else {
                                                let lat = msg.lat.unwrap();
                                                let lon = msg.lon.unwrap();
                                                let alt = msg.alt.unwrap();
                                                track_gps_buf.push((lat, lon, alt));
                                                if track_gps_buf.len() >= TRACK_CAPTURE_NUM_GPS {
                                                    let n = track_gps_buf.len();
                                                    let (slat, slon, salt) = track_gps_buf
                                                        .drain(..)
                                                        .fold((0.0_f64, 0.0_f64, 0.0_f32),
                                                            |(a, b, c), (x, y, z)| (a + x, b + y, c + z));
                                                    collecting.store(false, Ordering::Relaxed);
                                                    track_capture_start = None;
                                                    if let Some(ref a) = bridge_pending_cmd {
                                                        a.store(CMD_CAPTURE_SUCCESS, Ordering::Relaxed);
                                                    }
                                                    on_capture(
                                                        slat / (n as f64),
                                                        slon / (n as f64),
                                                        salt / (n as f32),
                                                    );
                                                    info!("Track capture: averaged {} GPS samples", n);
                                                }
                                            }
                                        }
                                    }

                                    if with_ahrs {
                                        if msg.message_type == "Gps" {
                                            if let (Some(lat), Some(lon), Some(alt)) =
                                                (msg.lat, msg.lon, msg.alt)
                                            {
                                                if let Some(ref mut filter) = ahrs_filter {
                                                    if !filter.has_origin() {
                                                        filter.set_origin(lat, lon, alt as f64);
                                                        info!(
                                                            "AHRS origin set from GPS: {}, {}, {}",
                                                            lat, lon, alt
                                                        );
                                                    }
                                                }
                                            }
                                        }
                                        if msg.message_type == "Imu" {
                                            let ts = msg.timestamp_us;
                                            if primary_node_mac.is_none() {
                                                primary_node_mac = Some(msg.src_mac);
                                                ahrs_filter = Some(AhrsFilter::new(
                                                    AHRS_SAMPLE_PERIOD_S,
                                                    AHRS_BETA,
                                                ));
                                                last_ahrs_emit_ts_us = ts;
                                            }
                                            if primary_node_mac == Some(msg.src_mac) {
                                                if let (Some(ax), Some(ay), Some(az), Some(gx), Some(gy), Some(gz)) = (
                                                    msg.accel_x,
                                                    msg.accel_y,
                                                    msg.accel_z,
                                                    msg.gyro_x,
                                                    msg.gyro_y,
                                                    msg.gyro_z,
                                                ) {
                                                    if let Some(ref mut filter) = ahrs_filter {
                                                        filter.update_imu(ts, ax, ay, az, gx, gy, gz);
                                                        if ts.saturating_sub(last_ahrs_emit_ts_us)
                                                            >= ahrs_period_us
                                                        {
                                                            let row = filter.get_output_row(ts);
                                                            if let Some(ref mut w) = ahrs_wtr {
                                                                w.serialize(&row)?;
                                                                w.flush()?;
                                                            }
                                                            last_ahrs_emit_ts_us = ts;
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("Failed to parse message: {:?}", e);
                                    let err_msg =
                                        error_decoded_message("Error", &format!("{:?}", e));
                                    raw_wtr.serialize(&err_msg)?;
                                    raw_wtr.flush()?;
                                }
                            }
                        }
                        Err(e) => {
                            error!("COBS decoding error: {:?}", e);
                            let err_msg =
                                error_decoded_message("COBS_Error", &format!("{:?}", e));
                            raw_wtr.serialize(&err_msg)?;
                            raw_wtr.flush()?;
                        }
                    }
                    
                    // Check if we should stop after processing each complete frame
                    if should_stop() {
                        info!("Stop requested, ending session");
                        break;
                    }
                }
            }
            Some(Err(e)) => return Err(e.into()),
            None => {
                if should_stop() {
                    break;
                }
            }
        }
    }

    Ok(())
}
