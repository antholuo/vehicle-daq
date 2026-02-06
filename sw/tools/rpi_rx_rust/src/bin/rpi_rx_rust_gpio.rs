//! GPIO-controlled logging service: no CLI, for systemd.
//!
//! GPIO #11: init (start raw logging, AHRS init, assume vehicle level stationary).
//! GPIO #19:  start logging including postprocessed AHRS at default rate (10 Hz).
//! Logs stop only when **both** GPIOs go LOW. Temporary data gaps do not stop logging.
//!
//! Build on Linux with: `cargo build --release --features gpio`

use chrono::Local;
use csv::Writer;
use gpiod::{Chip, EdgeDetect, Options};
use log::*;
use rpi_rx_rust::session::{run_session, ByteSource};
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use std::time::Duration;

const GPIO_INIT: u32 = 11;   // BCM 11: init / raw
const GPIO_POST: u32 = 19;   // BCM 19: postprocessed logging
const DEFAULT_SERIAL_PORT: &str = "/dev/ttyACM0";
const DEFAULT_BAUD_RATE: u32 = 115_200;
const SERIAL_READ_TIMEOUT_MS: u64 = 200;
const DEFAULT_AHRS_HZ: u32 = 10;

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

fn serial_port() -> String {
    std::env::var("RPI_RX_SERIAL_PORT").unwrap_or_else(|_| DEFAULT_SERIAL_PORT.to_string())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("rpi_rx_rust_gpio: GPIO {} (init), GPIO {} (post); logs under ~/daq/logs/<date>/", GPIO_INIT, GPIO_POST);

    let chip = Chip::new("gpiochip0").or_else(|_| Chip::new(0))?;
    let opts = Options::input([GPIO_INIT, GPIO_POST])
        .edge(EdgeDetect::Both)
        .bias(Bias::PullDown)
        .consumer("rpi_rx_rust_gpio");
    let mut inputs = chip.request_lines(opts)?;
    info!("GPIO {} and {} requested (edge both, pull-down enabled)", GPIO_INIT, GPIO_POST);
    
    // Check initial state
    let initial_values: [bool; 2] = inputs.get_values([false, false])?;
    info!("Initial GPIO state: GPIO{}={}, GPIO{}={}", 
          GPIO_INIT, initial_values[0], GPIO_POST, initial_values[1]);

    loop {
        // Idle: wait for an edge, then check if either GPIO is high
        let _event = inputs.read_event()?;
        let values: [bool; 2] = inputs.get_values([false, false])?;
        let gpio11_high = values[0];
        let gpio19_high = values[1];
        info!("GPIO edge detected: GPIO{}={}, GPIO{}={}", GPIO_INIT, gpio11_high, GPIO_POST, gpio19_high);
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

        let port = match serialport::new(serial_port(), DEFAULT_BAUD_RATE)
            .timeout(Duration::from_millis(SERIAL_READ_TIMEOUT_MS))
            .open()
        {
            Ok(p) => p,
            Err(e) => {
                error!("Failed to open serial {}: {}", serial_port(), e);
                continue;
            }
        };

        let mut byte_source = SerialTimeoutByteSource::new(port);
        let mut should_stop = || {
            let values: [bool; 2] = match inputs.get_values([false, false]) {
                Ok(v) => v,
                Err(_) => return true,
            };
            let active = values[0] || values[1];
            !active
        };

        if let Err(e) = run_session(
            &mut byte_source,
            &mut raw_wtr,
            ahrs_wtr.as_mut(),
            DEFAULT_AHRS_HZ,
            &mut should_stop,
        ) {
            error!("Session error: {}", e);
        }

        drop(ahrs_wtr);
        drop(raw_wtr);
        info!("Session stopped (both GPIOs low): {:?}", raw_path);
    }
}
