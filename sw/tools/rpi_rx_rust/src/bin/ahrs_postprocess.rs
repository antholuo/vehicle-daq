//! Post-process a raw log CSV: run AHRS and output synthesized GPS, vehicle heading, ground track.
//!
//! Assumes first 2 seconds are steady state (vehicle at rest) for board-to-vehicle calibration.
//! All sensor nodes are on the vehicle but may have different orientations.
//!
//! Outputs:
//! - <input_stem>_synthesized.csv + _synthesized_track.png: fused from all IMU nodes
//! - <input_stem>_gps_imu_only.csv + _gps_imu_only_track.png: only the node that has GPS+IMU

use clap::Parser;
use csv::{Reader, Writer};
use log::*;
use nalgebra::{Rotation3, Vector3};
use rpi_rx_rust::ahrs::AhrsFilter;
use rpi_rx_rust::board_calib::{board_to_vehicle_rotation, transform_accel, transform_gyro};
use std::collections::HashMap;
use std::path::Path;

const REST_PERIOD_US: u64 = 2 * 1_000_000; // 2 seconds steady state
const AHRS_SAMPLE_PERIOD_S: f64 = 1.0 / 400.0;
const AHRS_BETA: f64 = 0.1;

#[derive(Parser, Debug)]
#[command(name = "ahrs_postprocess")]
#[command(about = "Run AHRS on raw log; output synthesized (all nodes) and gps_imu_only tracks")]
struct Args {
    /// Input raw log CSV (from rpi_rx_rust)
    #[arg(short, long)]
    input: std::path::PathBuf,

    /// Output directory [default: same as input]
    #[arg(short, long)]
    output_dir: Option<std::path::PathBuf>,

    /// AHRS output rate in Hz
    #[arg(long, default_value = "10", value_parser = clap::value_parser!(u32).range(1..=200))]
    ahrs_hz: u32,
}

/// One row from the raw log CSV.
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
    #[serde(default)]
    speed_kts: Option<f32>,
    #[serde(default)]
    heading: Option<u16>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let args = Args::parse();

    let stem = args.input.file_stem().unwrap_or_default().to_string_lossy();
    let parent = args.output_dir
        .unwrap_or_else(|| args.input.parent().unwrap_or(Path::new(".")).to_path_buf());

    info!("Input: {:?}", args.input);
    info!("AHRS rate: {} Hz; first 2 s steady state for calibration", args.ahrs_hz);
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

    let first_ts = rows
        .iter()
        .filter(|r| r.message_type == "Imu")
        .map(|r| r.timestamp_us)
        .min()
        .unwrap_or(0);
    let rest_end_ts = first_ts + REST_PERIOD_US;

    // Calibration: per-node mean accel and gyro during first 2s
    let mut node_accel_sum: HashMap<String, (f64, f64, f64)> = HashMap::new();
    let mut node_accel_n: HashMap<String, u32> = HashMap::new();
    let mut node_gyro_sum: HashMap<String, (f64, f64, f64)> = HashMap::new();
    let mut node_gyro_n: HashMap<String, u32> = HashMap::new();

    for row in &rows {
        if row.message_type != "Imu" || row.timestamp_us >= rest_end_ts {
            continue;
        }
        if let (Some(ax), Some(ay), Some(az), Some(gx), Some(gy), Some(gz)) = (
            row.accel_x, row.accel_y, row.accel_z,
            row.gyro_x, row.gyro_y, row.gyro_z,
        ) {
            let k = row.src_mac.clone();
            *node_accel_sum.entry(k.clone()).or_insert((0.0, 0.0, 0.0)) =
                (node_accel_sum.get(&k).unwrap_or(&(0.0, 0.0, 0.0)).0 + ax as f64,
                 node_accel_sum.get(&k).unwrap_or(&(0.0, 0.0, 0.0)).1 + ay as f64,
                 node_accel_sum.get(&k).unwrap_or(&(0.0, 0.0, 0.0)).2 + az as f64);
            *node_accel_n.entry(k.clone()).or_insert(0) += 1;
            *node_gyro_sum.entry(k.clone()).or_insert((0.0, 0.0, 0.0)) =
                (node_gyro_sum.get(&k).unwrap_or(&(0.0, 0.0, 0.0)).0 + gx as f64,
                 node_gyro_sum.get(&k).unwrap_or(&(0.0, 0.0, 0.0)).1 + gy as f64,
                 node_gyro_sum.get(&k).unwrap_or(&(0.0, 0.0, 0.0)).2 + gz as f64);
            *node_gyro_n.entry(k).or_insert(0) += 1;
        }
    }

    let gps_node = rows.iter().find(|r| r.message_type == "Gps").map(|r| r.src_mac.clone());
    info!("GPS node: {:?}", gps_node);

    let mut r_b2v: HashMap<String, nalgebra::Rotation3<f64>> = HashMap::new();
    for (node, n) in &node_accel_n {
        if *n == 0 {
            continue;
        }
        let (sx, sy, sz) = node_accel_sum.get(node).copied().unwrap_or((0.0, 0.0, 0.0));
        let mean = Vector3::new(sx / *n as f64, sy / *n as f64, sz / *n as f64);
        let r = board_to_vehicle_rotation(mean);
        r_b2v.insert(node.clone(), r);
    }
    info!("Calibrated {} IMU nodes", r_b2v.len());

    // Run AHRS for gps_imu_only (single node)
    let gps_imu_only_path = parent.join(format!("{}_gps_imu_only.csv", stem));
    if let Some(ref gps_mac) = gps_node {
        if let Some(r) = r_b2v.get(gps_mac) {
            let mut filter = AhrsFilter::new(AHRS_SAMPLE_PERIOD_S, AHRS_BETA);
            let mut wtr = Writer::from_path(&gps_imu_only_path)?;
            let mut last_emit_ts: u64 = 0;
            let mut prev_ts: u64 = 0;

            for row in &rows {
                if row.message_type == "Gps" {
                    if let (Some(lat), Some(lon), Some(alt)) = (row.lat, row.lon, row.alt) {
                        if !filter.has_origin() {
                            filter.set_origin(lat, lon, alt as f64);
                        } else {
                            let speed = row.speed_kts.unwrap_or(0.0);
                            let heading = row.heading.map(|h| h as f32).unwrap_or(0.0);
                            filter.correct_from_gps(lat, lon, alt as f64, speed, heading);
                        }
                    }
                }
                if row.message_type == "Imu" && row.src_mac == *gps_mac {
                    if let (Some(ax), Some(ay), Some(az), Some(gx), Some(gy), Some(gz)) = (
                        row.accel_x, row.accel_y, row.accel_z,
                        row.gyro_x, row.gyro_y, row.gyro_z,
                    ) {
                        if prev_ts > 0 && row.timestamp_us <= prev_ts {
                            continue;
                        }
                        prev_ts = row.timestamp_us;
                        let (ax_v, ay_v, az_v) = transform_accel(r, ax as f64, ay as f64, az as f64);
                        let (gx_v, gy_v, gz_v) = transform_gyro(r, gx as f64, gy as f64, gz as f64);
                        filter.update_imu_with_frame(
                            row.timestamp_us,
                            ax_v as f32, ay_v as f32, az_v as f32,
                            gx_v as f32, gy_v as f32, gz_v as f32,
                            true,
                        );
                        if row.timestamp_us.saturating_sub(last_emit_ts) >= ahrs_period_us {
                            wtr.serialize(&filter.get_output_row(row.timestamp_us))?;
                            last_emit_ts = row.timestamp_us;
                        }
                    }
                }
            }
            wtr.flush()?;
            info!("Wrote gps_imu_only to {:?}", gps_imu_only_path);
        }
    }

    // Run AHRS for synthesized (all IMU nodes merged, transformed to vehicle frame)
    let synthesized_path = parent.join(format!("{}_synthesized.csv", stem));
    let mut filter = AhrsFilter::new(AHRS_SAMPLE_PERIOD_S, AHRS_BETA);
    let mut wtr = Writer::from_path(&synthesized_path)?;
    let mut last_emit_ts: u64 = 0;

    for row in &rows {
        if row.message_type == "Gps" {
            if let (Some(lat), Some(lon), Some(alt)) = (row.lat, row.lon, row.alt) {
                if !filter.has_origin() {
                    filter.set_origin(lat, lon, alt as f64);
                } else {
                    let speed = row.speed_kts.unwrap_or(0.0);
                    let heading = row.heading.map(|h| h as f32).unwrap_or(0.0);
                    filter.correct_from_gps(lat, lon, alt as f64, speed, heading);
                }
            }
        }
        if row.message_type == "Imu" {
            if let (Some(ax), Some(ay), Some(az), Some(gx), Some(gy), Some(gz)) = (
                row.accel_x, row.accel_y, row.accel_z,
                row.gyro_x, row.gyro_y, row.gyro_z,
            ) {
                let r = r_b2v.get(&row.src_mac).copied().unwrap_or(Rotation3::identity());
                let (ax_v, ay_v, az_v) = transform_accel(&r, ax as f64, ay as f64, az as f64);
                let (gx_v, gy_v, gz_v) = transform_gyro(&r, gx as f64, gy as f64, gz as f64);
                filter.update_imu_with_frame(
                    row.timestamp_us,
                    ax_v as f32, ay_v as f32, az_v as f32,
                    gx_v as f32, gy_v as f32, gz_v as f32,
                    true,
                );
                if row.timestamp_us.saturating_sub(last_emit_ts) >= ahrs_period_us {
                    wtr.serialize(&filter.get_output_row(row.timestamp_us))?;
                    last_emit_ts = row.timestamp_us;
                }
            }
        }
    }

    wtr.flush()?;
    info!("Wrote synthesized to {:?}", synthesized_path);

    // Generate PNGs via track_plotter (sibling tool)
    let track_plotter_bin = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../track_plotter/target/debug/track_plotter");
    let plotter_cmd: &std::path::Path = if track_plotter_bin.exists() {
        track_plotter_bin.as_path()
    } else {
        Path::new("track_plotter")
    };

    if let Ok(status) = std::process::Command::new(plotter_cmd)
        .arg("--input")
        .arg(&synthesized_path)
        .arg("--output")
        .arg(parent.join(format!("{}_synthesized_track.png", stem)))
        .output()
    {
        if status.status.success() {
            info!("Generated synthesized_track.png");
        }
    }

    if gps_imu_only_path.exists() {
        let _ = std::process::Command::new(plotter_cmd)
        .arg("--input")
        .arg(&gps_imu_only_path)
        .arg("--output")
        .arg(parent.join(format!("{}_gps_imu_only_track.png", stem)))
        .output();
    }

    Ok(())
}
