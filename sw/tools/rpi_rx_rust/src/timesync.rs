//! TimeSync sender: periodically sends COBS-framed TimeSync commands to the bridge via USB serial.
//! Can also send RequestFloatingGps when requested (for track capture) and bridge HMI commands
//! (arm/end/success/timeout) so the bridge can drive its LED.

use log::*;
use std::io::Write;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crate::track_capture::{encode_bridge_hmi_command, encode_request_floating_gps_command, CMD_ARM_TRACK, CMD_END_TRACK};

const CMD_TIME_SYNC: u8 = 0x01;
const TIMESYNC_INTERVAL: Duration = Duration::from_millis(500);
const RAW_CMD_LEN: usize = 9; // 1 cmd + 8 u64

/// Build a COBS-framed USB TimeSync command: `COBS_ENCODE([0x01, session_time_us LE]) + [0x00]`.
pub fn encode_timesync_command(session_time_us: u64) -> Vec<u8> {
    let mut raw = [0u8; RAW_CMD_LEN];
    raw[0] = CMD_TIME_SYNC;
    raw[1..9].copy_from_slice(&session_time_us.to_le_bytes());

    let max_encoded = cobs::max_encoding_length(RAW_CMD_LEN);
    let mut encoded = vec![0u8; max_encoded + 1]; // +1 for 0x00 delimiter
    let n = cobs::encode(&raw, &mut encoded);
    encoded.truncate(n + 1);
    encoded[n] = 0x00; // COBS frame delimiter
    encoded
}

/// Pending bridge HMI command: 0 = none, else CMD_ARM_TRACK/CMD_END_TRACK/CMD_CAPTURE_SUCCESS/CMD_CAPTURE_TIMEOUT.
/// Session sets success/timeout; caller sets arm/end.
const PENDING_NONE: u8 = 0;

/// Periodic TimeSync sender running on a dedicated writer thread.
/// Optional: set request_floating_gps to true to send one RequestFloatingGps command
/// and set collecting_for_track so the session can collect 5 GPS samples.
/// Bridge HMI: set pending_bridge_cmd (or call send_arm_track/send_end_track) to send LED commands.
pub struct TimeSyncSender {
    stop_flag: Arc<AtomicBool>,
    /// Set to true to request one floating GPS capture; thread sends command and sets collecting_for_track
    pub request_floating_gps: Arc<AtomicBool>,
    /// Set by thread when it sends RequestFloatingGps; session clears when 5 GPS collected
    pub collecting_for_track: Arc<AtomicBool>,
    /// When non-zero, thread sends this 1-byte bridge HMI command (arm/end/success/timeout). Session sets success/timeout.
    pub pending_bridge_cmd: Arc<AtomicU8>,
    handle: Option<JoinHandle<()>>,
    session_start: Instant,
}

impl TimeSyncSender {
    /// Spawn the sender thread. Begins sending immediately.
    pub fn start(mut writer: Box<dyn serialport::SerialPort>, session_start: Instant) -> Self {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let request_floating_gps = Arc::new(AtomicBool::new(false));
        let collecting_for_track = Arc::new(AtomicBool::new(false));
        let pending_bridge_cmd = Arc::new(AtomicU8::new(PENDING_NONE));
        let flag = stop_flag.clone();
        let req_float = request_floating_gps.clone();
        let coll_track = collecting_for_track.clone();
        let pending_cmd = pending_bridge_cmd.clone();

        let handle = thread::Builder::new()
            .name("timesync-sender".into())
            .spawn(move || {
                trace!("TimeSync sender started (interval={}ms)", TIMESYNC_INTERVAL.as_millis());
                while !flag.load(Ordering::Relaxed) {
                    // Send any pending bridge HMI command first (arm/end/success/timeout)
                    let cmd_byte = pending_cmd.swap(PENDING_NONE, Ordering::Relaxed);
                    if cmd_byte != PENDING_NONE {
                        let frame = encode_bridge_hmi_command(cmd_byte);
                        if let Err(e) = writer.write_all(&frame) {
                            warn!("Bridge HMI command write error: {}", e);
                        }
                    }
                    if req_float.swap(false, Ordering::Relaxed) {
                        let cmd = encode_request_floating_gps_command();
                        if let Err(e) = writer.write_all(&cmd) {
                            warn!("RequestFloatingGps write error: {}", e);
                        } else {
                            coll_track.store(true, Ordering::Relaxed);
                            trace!("TimeSync thread: sent RequestFloatingGps");
                        }
                    }
                    let elapsed_us = session_start.elapsed().as_micros() as u64;
                    let frame = encode_timesync_command(elapsed_us);
                    if let Err(e) = writer.write_all(&frame) {
                        warn!("TimeSync write error: {}", e);
                    }
                    thread::sleep(TIMESYNC_INTERVAL);
                }
                trace!("TimeSync sender stopped");
            })
            .expect("failed to spawn timesync-sender thread");

        Self {
            stop_flag,
            request_floating_gps,
            collecting_for_track,
            pending_bridge_cmd,
            handle: Some(handle),
            session_start,
        }
    }

    /// Tell the bridge to show "armed" (e.g. double blink). Call when user arms track capture.
    pub fn send_arm_track(&self) {
        self.pending_bridge_cmd.store(CMD_ARM_TRACK, Ordering::Relaxed);
    }

    /// Tell the bridge to show "not armed". Call when user ends track capture.
    pub fn send_end_track(&self) {
        self.pending_bridge_cmd.store(CMD_END_TRACK, Ordering::Relaxed);
    }

    /// Tell the bridge capture succeeded (session sets this when 5 GPS received).
    pub fn send_capture_success(&self) {
        self.pending_bridge_cmd.store(crate::track_capture::CMD_CAPTURE_SUCCESS, Ordering::Relaxed);
    }

    /// Tell the bridge capture timed out (session sets this on 5s timeout).
    pub fn send_capture_timeout(&self) {
        self.pending_bridge_cmd.store(crate::track_capture::CMD_CAPTURE_TIMEOUT, Ordering::Relaxed);
    }

    /// Request one floating GPS capture (bridge will return 5 samples; session collects them).
    pub fn request_floating_gps_capture(&self) {
        self.request_floating_gps.store(true, Ordering::Relaxed);
    }

    /// Signal the thread to stop and join it.
    pub fn stop(mut self) {
        self.stop_flag.store(true, Ordering::Relaxed);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }

    /// Returns the session start `Instant` (for video timestamp correlation).
    pub fn session_start(&self) -> Instant {
        self.session_start
    }
}
