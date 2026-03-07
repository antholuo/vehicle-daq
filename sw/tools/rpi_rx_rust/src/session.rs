//! Logging session: byte source abstraction and run loop (decode COBS, write CSV, optional AHRS).
//! Optional track capture: when collecting_for_track is set, next 5 GPS are averaged and passed to callback.

use crate::ahrs::AhrsFilter;
use crate::types::{error_decoded_message, format_mac_address, parse_message};
use cobs::decode;
use csv::Writer;
use log::*;
use std::collections::VecDeque;
use std::io;
use std::sync::atomic::Ordering;

const COBS_DECODED_BUFFER_SIZE: usize = 128;
const AHRS_SAMPLE_PERIOD_S: f64 = 1.0 / 400.0;
const AHRS_BETA: f64 = 0.1;
const TRACK_CAPTURE_NUM_GPS: usize = 5;

/// Source of bytes (e.g. serial with timeout, or stdin). Returns `None` when no byte available (e.g. timeout).
pub trait ByteSource {
    fn next_byte(&mut self) -> Option<io::Result<u8>>;
}

/// Runs the decode/write loop until EOF or `should_stop()` returns true.
/// Logs raw CSV to `raw_wtr`; if `ahrs_wtr` is `Some`, also runs AHRS and writes at `ahrs_hz`.
/// If `collecting_for_track` and `on_track_capture` are both `Some`, when the flag is true the next 5 GPS
/// messages are averaged and the callback is invoked with (lat, lon, alt).
pub fn run_session<W, W2, B>(
    byte_source: &mut B,
    raw_wtr: &mut Writer<W>,
    mut ahrs_wtr: Option<&mut Writer<W2>>,
    ahrs_hz: u32,
    should_stop: &mut dyn FnMut() -> bool,
    collecting_for_track: Option<&std::sync::atomic::AtomicBool>,
    mut on_track_capture: Option<&mut dyn FnMut(f64, f64, f32)>,
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
                                    message_count += 1;
                                    info!(
                                        "[{}] Type={:?}, MAC={}, Node={}/{}, Timestamp={}us",
                                        message_count,
                                        msg.message_type,
                                        format_mac_address(&msg.src_mac),
                                        msg.node_position,
                                        msg.node_instance,
                                        msg.timestamp_us
                                    );
                                    raw_wtr.serialize(&msg)?;
                                    raw_wtr.flush()?;

                                    // Track capture: collect 5 GPS when flag set, then average and callback
                                    if let (Some(collecting), Some(on_capture)) = (
                                        collecting_for_track.as_ref(),
                                        on_track_capture.as_mut(),
                                    ) {
                                        if msg.message_type == "Gps"
                                            && collecting.load(Ordering::Relaxed)
                                            && msg.lat.is_some()
                                            && msg.lon.is_some()
                                            && msg.alt.is_some()
                                        {
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
                                                on_capture(
                                                    slat / (n as f64),
                                                    slon / (n as f64),
                                                    salt / (n as f32),
                                                );
                                                info!("Track capture: averaged {} GPS samples", n);
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
