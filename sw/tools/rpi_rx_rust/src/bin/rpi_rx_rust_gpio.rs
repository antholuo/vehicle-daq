//! GPIO-controlled logging service: no CLI, for systemd.
//!
//! GPIO #11: init (start raw logging, AHRS init, assume vehicle level stationary).
//! GPIO #19: start logging including postprocessed AHRS at default rate (10 Hz).
//! GPIO #26: control video recording (high = start, low = stop).
//! Logs stop only when **both** GPIOs #11 and #19 go LOW. Temporary data gaps do not stop logging.
//!
//! Build on Linux with: `cargo build --release --features gpio`

use chrono::Local;
use csv::Writer;
use gpiod::{Chip, EdgeDetect, Options, Bias};
use log::*;
use rpi_rx_rust::session::{run_session, ByteSource};
use rpi_rx_rust::TimeSyncSender;
use std::cell::{Cell, RefCell};
use std::fs;
use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};

const GPIO_INIT: u32 = 11;   // BCM 11: init / raw
const GPIO_POST: u32 = 19;   // BCM 19: postprocessed logging
const GPIO_VIDEO: u32 = 26;  // BCM 26: video recording control
const GPIO_TRACK_ARM: u32 = 10;    // BCM 10: arm track capture (high = new track file)
const GPIO_TRACK_CAPTURE: u32 = 9; // BCM 9: rising edge = take location (request floating GPS)
/// Serial port for bridge. Set RPI_RX_SERIAL_PORT if bridge is not on ACM0 (e.g. bridge=ACM0, other board=ACM1).
const DEFAULT_SERIAL_PORT: &str = "/dev/ttyACM0";
const DEFAULT_BAUD_RATE: u32 = 115_200;
const SERIAL_READ_TIMEOUT_MS: u64 = 200;
const DEFAULT_AHRS_HZ: u32 = 10;
const CAMERA_SERVER_ADDR: &str = "127.0.0.1:8888";

/// Byte source that reads one byte from serial with timeout; returns None on timeout.
struct SerialTimeoutByteSource {
    port: Box<dyn serialport::SerialPort>,
    buf: [u8; 1],
}

impl SerialTimeoutByteSource {
    fn new(port: Box<dyn serialport::SerialPort>) -> Self {
        Self {
            port,
            buf: [0u8; 1],
        }
    }
}

impl ByteSource for SerialTimeoutByteSource {
    fn next_byte(&mut self) -> Option<io::Result<u8>> {
        match self.port.read(&mut self.buf) {
            Ok(0) => None,
            Ok(1) => Some(Ok(self.buf[0])),
            Ok(_) => unreachable!(),
            Err(e) if e.kind() == io::ErrorKind::TimedOut => None,
            Err(e) => Some(Err(e)),
        }
    }
}

/// Log directory: ~/daq/logs/<date>. If RPI_RX_OUTPUT_DIR is set, use that as base then /logs/<date>.
fn log_dir_for_today() -> PathBuf {
    let date_str = Local::now().format("%Y-%m-%d").to_string();
    match std::env::var("RPI_RX_OUTPUT_DIR") {
        Ok(custom) => PathBuf::from(custom).join("logs").join(date_str),
        Err(_) => {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/opt/rpi_rx_rust".to_string());
            PathBuf::from(home).join("daq").join("logs").join(date_str)
        }
    }
}

/// Track directory: ~/daq/tracks/<date>. Same base as logs when RPI_RX_OUTPUT_DIR is set.
fn track_dir_for_today() -> PathBuf {
    let date_str = Local::now().format("%Y-%m-%d").to_string();
    match std::env::var("RPI_RX_OUTPUT_DIR") {
        Ok(custom) => PathBuf::from(custom).join("tracks").join(date_str),
        Err(_) => {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/opt/rpi_rx_rust".to_string());
            PathBuf::from(home).join("daq").join("tracks").join(date_str)
        }
    }
}

fn serial_port_name() -> String {
    std::env::var("RPI_RX_SERIAL_PORT").unwrap_or_else(|_| DEFAULT_SERIAL_PORT.to_string())
}

/// Open the serial port and clone it for bidirectional use.
/// Returns `(read_port, write_port)` or an error.
fn open_serial_bidirectional() -> Result<(Box<dyn serialport::SerialPort>, Box<dyn serialport::SerialPort>), serialport::Error> {
    let port = serialport::new(serial_port_name(), DEFAULT_BAUD_RATE)
        .timeout(Duration::from_millis(SERIAL_READ_TIMEOUT_MS))
        .open()?;
    let write_port = port.try_clone()?;
    Ok((port, write_port))
}

/// Send command to the camera server (record.py)
fn send_camera_command(command: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect(CAMERA_SERVER_ADDR)?;
    stream.set_write_timeout(Some(Duration::from_secs(2)))?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;

    stream.write_all(command.as_bytes())?;
    stream.flush()?;

    let mut response = String::new();
    stream.read_to_string(&mut response)?;

    Ok(response.trim().to_string())
}

/// Query elapsed session time from camera server while it is recording.
/// Expected response: `ELAPSED_US:<u64>` or `IDLE`.
fn query_video_elapsed_us() -> Result<Option<u64>, Box<dyn std::error::Error>> {
    let response = send_camera_command("status_us")?;
    if response.eq_ignore_ascii_case("IDLE") {
        return Ok(None);
    }

    if let Some(v) = response.strip_prefix("ELAPSED_US:") {
        let us = v.trim().parse::<u64>()?;
        return Ok(Some(us));
    }

    Err(format!("unexpected status_us response: {}", response).into())
}

/// Start video recording with a session-relative timestamp for post-processing correlation.
/// Sends `start:<elapsed_us>` to the camera server.
fn start_video_recording(session_start: &Instant) -> bool {
    let elapsed_us = session_start.elapsed().as_micros();
    let cmd = format!("start:{}", elapsed_us);
    info!("Starting video recording (session offset {}us)", elapsed_us);
    match send_camera_command(&cmd) {
        Ok(response) => {
            info!("Camera server response: {}", response);
            true
        }
        Err(e) => {
            error!("Failed to start video recording: {}", e);
            false
        }
    }
}

/// Stop video recording.
fn stop_video_recording() -> bool {
    info!("Stopping video recording");
    match send_camera_command("stop") {
        Ok(response) => {
            info!("Camera server response: {}", response);
            true
        }
        Err(e) => {
            error!("Failed to stop video recording: {}", e);
            false
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("rpi_rx_rust_gpio: GPIO {} (init), {} (post), {} (video), {} (track arm), {} (track capture); logs under ~/daq/logs/<date>/", GPIO_INIT, GPIO_POST, GPIO_VIDEO, GPIO_TRACK_ARM, GPIO_TRACK_CAPTURE);

    let chip = Chip::new("gpiochip0").or_else(|_| Chip::new(0))?;
    let opts = Options::input([GPIO_INIT, GPIO_POST, GPIO_VIDEO, GPIO_TRACK_ARM, GPIO_TRACK_CAPTURE])
        .edge(EdgeDetect::Both)
        .bias(Bias::PullDown)
        .consumer("rpi_rx_rust_gpio");
    let mut inputs = chip.request_lines(opts)?;
    info!("GPIO {} (init), {} (post), {} (video), {} (track arm), {} (track capture) requested", GPIO_INIT, GPIO_POST, GPIO_VIDEO, GPIO_TRACK_ARM, GPIO_TRACK_CAPTURE);

    let initial_values: [bool; 5] = inputs.get_values([false, false, false, false, false])?;
    info!("Initial GPIO state: init={}, post={}, video={}, track_arm={}, track_capture={}",
          initial_values[0], initial_values[1], initial_values[2], initial_values[3], initial_values[4]);

    let video_recording = Cell::new(false);

    loop {
        let _event = inputs.read_event()?;
        let values: [bool; 5] = inputs.get_values([false, false, false, false, false])?;
        let gpio11_high = values[0];
        let gpio19_high = values[1];
        let gpio26_high = values[2];
        let gpio10_high = values[3];
        let gpio9_high = values[4];
        info!("GPIO edge: init={}, post={}, video={}, track_arm={}, track_capture={}",
              gpio11_high, gpio19_high, gpio26_high, gpio10_high, gpio9_high);

        // Handle video recording control (GPIO 26) — outside session, no timestamp yet
        if gpio26_high && !video_recording.get() {
            info!("GPIO {} high: starting video recording (no active session)", GPIO_VIDEO);
            match send_camera_command("start") {
                Ok(response) => {
                    info!("Camera server response: {}", response);
                    video_recording.set(true);
                }
                Err(e) => error!("Failed to start video recording: {}", e),
            }
        } else if !gpio26_high && video_recording.get() {
            video_recording.set(!stop_video_recording());
        }

        let active = gpio11_high || gpio19_high;
        if !active {
            info!("Both GPIOs low, staying idle");
            continue;
        }

        let with_ahrs = gpio19_high;
        let log_dir = log_dir_for_today();
        fs::create_dir_all(&log_dir)?;
        let time_str = Local::now().format("%H-%M-%S").to_string();
        let raw_path = log_dir.join(format!("{}_raw.csv", time_str));
        let postprocess_path = log_dir.join(format!("{}_postprocess.csv", time_str));

        info!(
            "Starting session: raw={}, postprocess={} (GPIO11={}, GPIO19={})",
            raw_path.display(),
            with_ahrs,
            gpio11_high,
            gpio19_high
        );

        let mut raw_wtr = Writer::from_path(&raw_path)?;
        let mut ahrs_wtr: Option<Writer<std::fs::File>> = if with_ahrs {
            Some(Writer::from_path(&postprocess_path)?)
        } else {
            None
        };

        let (read_port, write_port) = match open_serial_bidirectional() {
            Ok(ports) => ports,
            Err(e) => {
                error!("Failed to open serial {}: {}", serial_port_name(), e);
                continue;
            }
        };

        // Session epoch defaults to now, but if video was already running first,
        // adopt its elapsed monotonic offset so both systems share one epoch.
        let mut session_start = Instant::now();
        if video_recording.get() {
            match query_video_elapsed_us() {
                Ok(Some(video_elapsed_us)) => {
                    let now = Instant::now();
                    if let Some(adjusted_start) = now.checked_sub(Duration::from_micros(video_elapsed_us)) {
                        session_start = adjusted_start;
                        info!(
                            "Adopted video session epoch: video_elapsed_us={} (data logger anchored to video clock)",
                            video_elapsed_us
                        );
                    } else {
                        warn!(
                            "Video elapsed too large to back-calculate session start ({}us); keeping local epoch",
                            video_elapsed_us
                        );
                    }
                }
                Ok(None) => {
                    info!("Video status_us reported IDLE at session start; keeping local epoch");
                }
                Err(e) => {
                    warn!(
                        "Failed to query video status_us at session start ({}); keeping local epoch",
                        e
                    );
                }
            }
        }

        let timesync_sender = TimeSyncSender::start(write_port, session_start);
        info!("TimeSyncSender started for session");

        // Wait for the first TimeSync to propagate through the bridge to sensor nodes,
        // then flush stale pre-sync data from the serial input buffer.
        thread::sleep(Duration::from_millis(150));
        if let Err(e) = read_port.clear(serialport::ClearBuffer::Input) {
            warn!("Failed to clear serial input buffer: {}", e);
        }
        info!("Flushed serial input buffer after TimeSync propagation delay");

        let mut byte_source = SerialTimeoutByteSource::new(read_port);
        let track_file: RefCell<Option<Writer<std::fs::File>>> = RefCell::new(None);
        let cone_number = Cell::new(0u32);
        let prev_track_arm = Cell::new(false);
        let prev_track_capture = Cell::new(false);

        let mut on_track_capture = |lat: f64, lon: f64, alt: f32| {
            if let Some(ref mut wtr) = *track_file.borrow_mut() {
                let n = cone_number.get();
                if wtr.write_record([n.to_string(), lat.to_string(), lon.to_string(), alt.to_string()]).is_ok() {
                    let _ = wtr.flush();
                    info!("Track capture: cone #{} lat={:.6} lon={:.6} alt={:.1}", n, lat, lon, alt);
                    cone_number.set(n + 1);
                }
            }
        };

        let mut should_stop = || {
            let values: [bool; 5] = match inputs.get_values([false, false, false, false, false]) {
                Ok(v) => v,
                Err(_) => return true,
            };
            let gpio11 = values[0];
            let gpio19 = values[1];
            let gpio26 = values[2];
            let gpio10 = values[3];
            let gpio9 = values[4];

            if gpio26 && !video_recording.get() {
                video_recording.set(start_video_recording(&session_start));
            } else if !gpio26 && video_recording.get() {
                video_recording.set(!stop_video_recording());
            }

            // Track capture: IO10 rising = new track file (arm); IO10 falling = end; IO9 rising = take location
            if gpio10 && !prev_track_arm.get() {
                let dir = track_dir_for_today();
                if let Ok(()) = fs::create_dir_all(&dir) {
                    let time_str = Local::now().format("%H-%M-%S").to_string();
                    let path = dir.join(format!("{}.csv", time_str));
                    if let Ok(wtr) = Writer::from_path(&path) {
                        let mut buf = track_file.borrow_mut();
                        *buf = Some(wtr);
                        if let Some(ref mut w) = buf.as_mut() {
                            let _ = w.write_record(["Cone #", "Lat", "Lon", "Alt"]);
                            let _ = w.flush();
                        }
                        cone_number.set(0);
                        timesync_sender.send_arm_track();
                        info!("Track capture armed: {}", path.display());
                    }
                }
            } else if !gpio10 && prev_track_arm.get() {
                timesync_sender.send_end_track();
            }
            prev_track_arm.set(gpio10);

            if gpio9 && !prev_track_capture.get() && track_file.borrow().is_some() {
                timesync_sender.request_floating_gps_capture();
                info!("Track capture: requested floating GPS (IO9 rising)");
            }
            prev_track_capture.set(gpio9);

            let active = gpio11 || gpio19;
            !active
        };

        if let Err(e) = run_session(
            &mut byte_source,
            &mut raw_wtr,
            ahrs_wtr.as_mut(),
            DEFAULT_AHRS_HZ,
            &mut should_stop,
            Some(&timesync_sender.collecting_for_track),
            Some(&mut on_track_capture),
            Some(timesync_sender.pending_bridge_cmd.clone()),
            None,
        ) {
            error!("Session error: {}", e);
        }

        timesync_sender.stop();
        info!("TimeSyncSender stopped");

        drop(ahrs_wtr);
        drop(raw_wtr);
        info!("Session stopped (both GPIOs low): {:?}", raw_path);
    }
}
