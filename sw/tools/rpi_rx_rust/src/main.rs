use clap::Parser;
use csv::Writer;
use log::*;
use std::io::{self, Read};
use std::time::Duration;
use chrono::Local;
use std::fs;
use std::path::PathBuf;

use rpi_rx_rust::session::{run_session, ByteSource};

const DEFAULT_SERIAL_PORT: &str = "/dev/ttyACM0";
const DEFAULT_BAUD_RATE: u32 = 115_200;
/// Serial read timeout; on timeout we return None so the session keeps polling instead of exiting.
const SERIAL_READ_TIMEOUT_MS: u64 = 200;

// =============================================================================
// ByteSource implementations for main binary
// =============================================================================

/// Byte source from serial with timeout: returns None on timeout so the session keeps running;
/// only real errors are propagated. Avoids exiting when the bridge is idle (e.g. no nodes in range).
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

/// Byte source from an iterator (stdin); EOF yields None.
struct IteratorByteSource<I>(I)
where
    I: Iterator<Item = io::Result<u8>>;

impl<I> ByteSource for IteratorByteSource<I>
where
    I: Iterator<Item = io::Result<u8>>,
{
    fn next_byte(&mut self) -> Option<io::Result<u8>> {
        self.0.next()
    }
}

// =============================================================================
// CLI
// =============================================================================

#[derive(Parser, Debug)]
#[command(name = "rpi_rx_rust")]
#[command(about = "Raspberry Pi receiver for sensor_board COBS stream")]
struct Args {
    /// Enable real-time AHRS; logs both <time>_raw.csv and <time>_postprocess.csv
    #[arg(long)]
    with_ahrs: bool,

    /// AHRS output rate in Hz (e.g. 10, 50, 100). Used only with --with-ahrs. [default: 10]
    #[arg(long, default_value = "10", value_parser = clap::value_parser!(u32).range(1..=200))]
    ahrs_hz: u32,
}

// =============================================================================
// Main
// =============================================================================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let args = Args::parse();
    info!(
        "Starting RPI RX Rust decoder... (--with-ahrs: {}, --ahrs-hz: {})",
        args.with_ahrs, args.ahrs_hz
    );

    let now = Local::now();
    let date_str = now.format("%Y-%m-%d").to_string();
    let timestamp_str = now.format("%H-%M-%S").to_string();

    let mut log_dir = PathBuf::new();
    if cfg!(windows) {
        log_dir.push(std::env::var("USERPROFILE").expect("Failed to get USERPROFILE"));
    } else {
        log_dir.push(std::env::var("HOME").expect("Failed to get HOME directory"));
    }
    log_dir.push("daq/logs");
    log_dir.push(&date_str);

    fs::create_dir_all(&log_dir).expect("Failed to create log directory");

    let (log_file_path_raw, log_file_path_postprocess) = if args.with_ahrs {
        (
            log_dir.join(format!("{}_raw.csv", timestamp_str)),
            log_dir.join(format!("{}_postprocess.csv", timestamp_str)),
        )
    } else {
        (log_dir.join(format!("{}.csv", timestamp_str)), PathBuf::new())
    };

    let mut wtr = Writer::from_path(&log_file_path_raw)?;
    let mut wtr_ahrs: Option<Writer<std::fs::File>> = if args.with_ahrs {
        Some(Writer::from_path(&log_file_path_postprocess)?)
    } else {
        None
    };

    if args.with_ahrs {
        info!(
            "AHRS output rate: {} Hz; first 10 s at rest",
            args.ahrs_hz
        );
    }

    let mut serial_port: Option<Box<dyn serialport::SerialPort>> = None;
    let mut input_source = "stdin".to_string();

    match serialport::new(DEFAULT_SERIAL_PORT, DEFAULT_BAUD_RATE)
        .timeout(Duration::from_millis(SERIAL_READ_TIMEOUT_MS))
        .open()
    {
        Ok(port) => {
            info!("Successfully opened serial port: {}", DEFAULT_SERIAL_PORT);
            serial_port = Some(port);
            input_source = format!("serial port {}", DEFAULT_SERIAL_PORT);
        }
        Err(e) => {
            warn!(
                "Failed to open serial port {}: {}. Falling back to stdin.",
                DEFAULT_SERIAL_PORT, e
            );
        }
    }

    info!("Reading data from {}", input_source);

    let mut byte_source: Box<dyn ByteSource> = if let Some(port) = serial_port {
        Box::new(SerialTimeoutByteSource::new(port))
    } else {
        Box::new(IteratorByteSource(io::stdin().bytes()))
    };

    // Run until Ctrl+C (or fatal error). Session only stops when should_stop() is true.
    let mut should_stop = || false;
    run_session(
        &mut *byte_source,
        &mut wtr,
        wtr_ahrs.as_mut(),
        args.ahrs_hz,
        &mut should_stop,
        None,
        None,
        None,
        None,
        None,
    )?;

    info!("Finished processing input. Output written to {:?}", log_file_path_raw);
    Ok(())
}
