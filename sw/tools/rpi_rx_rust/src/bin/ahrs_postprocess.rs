//! Post-process a raw log CSV: run AHRS and output synthesized GPS, vehicle heading, ground track at fixed rate.
//!
//! Assumes first 10 seconds of the log are with vehicle at rest. Uses first GPS fix as origin.
//! Output: <input_stem>_ahrs_postprocess.csv

use clap::Parser;
use csv::{Reader, Writer};
use log::*;
use rpi_rx_rust::ahrs::AhrsFilter;
use std::path::Path;

const AHRS_SAMPLE_PERIOD_S: f64 = 1.0 / 400.0;
const AHRS_BETA: f64 = 0.1;

#[derive(Parser, Debug)]
#[command(name = "ahrs_postprocess")]
#[command(about = "Run AHRS on a raw log CSV; output synthesized GPS and orientation at fixed rate")]
struct Args {
    /// Input raw log CSV (from rpi_rx_rust)
    #[arg(short, long)]
    input: std::path::PathBuf,

    /// Output CSV path [default: <input_stem>_ahrs_postprocess.csv]
    #[arg(short, long)]
    output: Option<std::path::PathBuf>,

    /// AHRS output rate in Hz (e.g. 10, 50, 100)
    #[arg(long, default_value = "10", value_parser = clap::value_parser!(u32).range(1..=200))]
    ahrs_hz: u32,
}

/// One row from the raw log CSV (only fields we need).
#[derive(Debug, serde::Deserialize)]
struct RawCsvRow {
    src_mac: String,
    #[serde(rename = "timestamp_us")]
    timestamp_us: u64,
    #[serde(rename = "message_type")]
    message_type: String,
    accel_x: Option<f32>,
    accel_y: Option<f32>,
    accel_z: Option<f32>,
    gyro_x: Option<f32>,
    gyro_y: Option<f32>,
    gyro_z: Option<f32>,
    lat: Option<f64>,
    lon: Option<f64>,
    alt: Option<f32>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let args = Args::parse();

    let output_path = args.output.unwrap_or_else(|| {
        let stem = args.input.file_stem().unwrap_or_default();
        let parent = args.input.parent().unwrap_or(Path::new("."));
        parent.join(format!("{}_ahrs_postprocess.csv", stem.to_string_lossy()))
    });

    info!("Input: {:?}", args.input);
    info!("Output: {:?}", output_path);
    info!("AHRS rate: {} Hz; first 10 s at rest", args.ahrs_hz);
    let ahrs_period_us: u64 = 1_000_000 / (args.ahrs_hz as u64);

    let mut rdr = Reader::from_path(&args.input)?;
    let mut rows: Vec<RawCsvRow> = Vec::new();
    for result in rdr.deserialize() {
        let row: RawCsvRow = result?;
        if row.message_type == "Imu" || row.message_type == "Gps" {
            rows.push(row);
        }
    }
    rows.sort_by_key(|r| r.timestamp_us);
    info!("Loaded {} Imu/Gps rows", rows.len());

    let mut filter = AhrsFilter::new(AHRS_SAMPLE_PERIOD_S, AHRS_BETA);
    let primary_node: Option<String> = rows
        .iter()
        .find(|r| r.message_type == "Imu")
        .map(|r| r.src_mac.clone());
    let primary_node = primary_node.as_deref();

    let mut wtr = Writer::from_path(&output_path)?;
    let mut last_emit_ts_us: u64 = 0;
    let mut prev_ts_us: u64 = 0;

    for row in &rows {
        let ts = row.timestamp_us;
        if row.message_type == "Gps" {
            if let (Some(lat), Some(lon), Some(alt)) = (row.lat, row.lon, row.alt) {
                if !filter.has_origin() {
                    filter.set_origin(lat, lon, alt as f64);
                    info!("AHRS origin set from GPS: {}, {}, {}", lat, lon, alt);
                }
            }
        }
        if row.message_type == "Imu" && primary_node == Some(row.src_mac.as_str()) {
            if let (Some(ax), Some(ay), Some(az), Some(gx), Some(gy), Some(gz)) = (
                row.accel_x, row.accel_y, row.accel_z,
                row.gyro_x, row.gyro_y, row.gyro_z,
            ) {
                if prev_ts_us > 0 && ts <= prev_ts_us {
                    continue; // skip duplicate timestamps
                }
                prev_ts_us = ts;
                filter.update_imu(ts, ax, ay, az, gx, gy, gz);
                if ts.saturating_sub(last_emit_ts_us) >= ahrs_period_us {
                    let out = filter.get_output_row(ts);
                    wtr.serialize(&out)?;
                    last_emit_ts_us = ts;
                }
            }
        }
    }

    wtr.flush()?;
    info!("Wrote AHRS output to {:?}", output_path);
    Ok(())
}
