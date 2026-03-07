//! Interactive track capture test: type "arm track capture", "take location", "end track capture"
//! to test floating GPS and track CSV without GPIO. Reads from serial, writes track to ~/daq/tracks/<date>/.

use chrono::Local;
use csv::Writer;
use log::*;
use rpi_rx_rust::session::{run_session, ByteSource};
use rpi_rx_rust::TimeSyncSender;
use std::io::{self, BufRead, Read, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// Default serial port (bridge). Override with RPI_RX_SERIAL_PORT (e.g. /dev/ttyACM0 for bridge, /dev/ttyACM1 if only one device).
const DEFAULT_SERIAL_PORT: &str = "/dev/ttyACM1";
const DEFAULT_BAUD_RATE: u32 = 115_200;

fn serial_port_name() -> String {
    std::env::var("RPI_RX_SERIAL_PORT").unwrap_or_else(|_| DEFAULT_SERIAL_PORT.to_string())
}
const SERIAL_READ_TIMEOUT_MS: u64 = 200;

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

/// Byte source that reads one byte from serial with timeout; returns None on timeout.
struct SerialTimeoutByteSource {
    port: Box<dyn serialport::SerialPort>,
    buf: [u8; 1],
}

impl SerialTimeoutByteSource {
    fn new(port: Box<dyn serialport::SerialPort>) -> Self {
        Self { port, buf: [0u8; 1] }
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

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("rpi_rx_rust_track_test: interactive track capture (arm / take location / end)");
    info!("Commands: 'arm track capture' | 'take location' | 'end track capture' | 'quit'");

    let port_name = serial_port_name();
    let (read_port, write_port) = match serialport::new(&port_name, DEFAULT_BAUD_RATE)
        .timeout(Duration::from_millis(SERIAL_READ_TIMEOUT_MS))
        .open()
    {
        Ok(port) => {
            let w = port.try_clone()?;
            (port, w)
        }
        Err(e) => {
            error!("Failed to open {}: {}", port_name, e);
            return Err(e.into());
        }
    };

    let session_start = Instant::now();
    let timesync_sender = TimeSyncSender::start(write_port, session_start);
    thread::sleep(Duration::from_millis(200));
    if let Err(e) = read_port.clear(serialport::ClearBuffer::Input) {
        warn!("Failed to clear serial input: {}", e);
    }

    let track_file: Arc<Mutex<Option<Writer<std::fs::File>>>> = Arc::new(Mutex::new(None));
    let cone_number = Arc::new(AtomicU32::new(0));
    let running = Arc::new(AtomicBool::new(true));
    let run = running.clone();

    let stdin = io::stdin();
    let mut stdin_lock = stdin.lock();
    let mut line = String::new();

    let mut byte_source = SerialTimeoutByteSource::new(read_port);
    let mut raw_wtr = Writer::from_writer(vec![]);
    let run_session_stop = run.clone();
    let collecting_for_track = timesync_sender.collecting_for_track.clone();

    let bridge_pending_cmd = timesync_sender.pending_bridge_cmd.clone();
    let tf = track_file.clone();
    let cn = cone_number.clone();
    let mut on_track_capture = move |lat: f64, lon: f64, alt: f32| {
        if let Ok(mut g) = tf.lock() {
            if let Some(ref mut wtr) = *g {
                let n = cn.load(Ordering::Relaxed);
                if wtr.write_record([n.to_string(), lat.to_string(), lon.to_string(), alt.to_string()]).is_ok() {
                    let _ = wtr.flush();
                    info!("Track: cone #{} lat={:.6} lon={:.6} alt={:.1}", n, lat, lon, alt);
                    cn.store(n + 1, Ordering::Relaxed);
                }
            }
        }
    };

    let mut ahrs_none: Option<Writer<Vec<u8>>> = None;
    thread::spawn(move || {
        let mut should_stop = move || !run_session_stop.load(Ordering::Relaxed);
        let _ = run_session::<Vec<u8>, Vec<u8>, _>(
            &mut byte_source,
            &mut raw_wtr,
            ahrs_none.as_mut(),
            10,
            &mut should_stop,
            Some(collecting_for_track.as_ref()),
            Some(&mut on_track_capture),
            Some(bridge_pending_cmd),
            Some("Floating"),
        );
    });

    while running.load(std::sync::atomic::Ordering::Relaxed) {
        line.clear();
        print!("> ");
        let _ = io::stdout().flush();
        if stdin_lock.read_line(&mut line).is_err() || line.is_empty() {
            continue;
        }
        let cmd = line.trim().to_lowercase();
        if cmd.contains("quit") || cmd.contains("exit") {
            running.store(false, std::sync::atomic::Ordering::Relaxed);
            break;
        }
        if cmd.contains("arm") && cmd.contains("track") {
            let dir = track_dir_for_today();
            if std::fs::create_dir_all(&dir).is_ok() {
                let time_str = Local::now().format("%H-%M-%S").to_string();
                let path = dir.join(format!("{}.csv", time_str));
                if let Ok(wtr) = Writer::from_path(&path) {
                    if let Ok(mut buf) = track_file.lock() {
                        *buf = Some(wtr);
                        if let Some(ref mut w) = buf.as_mut() {
                            let _ = w.write_record(["Cone #", "Lat", "Lon", "Alt"]);
                            let _ = w.flush();
                        }
                    }
                    cone_number.store(0, Ordering::Relaxed);
                    timesync_sender.send_arm_track();
                    info!("Track armed: {}", path.display());
                }
            }
        } else if cmd.contains("take") && cmd.contains("location") {
            if track_file.lock().map(|g| g.is_some()).unwrap_or(false) {
                timesync_sender.request_floating_gps_capture();
                info!("Requested floating GPS (wait for 5 samples)");
            } else {
                info!("Arm track capture first");
            }
        } else if cmd.contains("end") && cmd.contains("track") {
            if let Ok(mut buf) = track_file.lock() {
                *buf = None;
            }
            timesync_sender.send_end_track();
            info!("Track capture ended");
        }
    }

    timesync_sender.stop();
    info!("Bye");
    Ok(())
}
