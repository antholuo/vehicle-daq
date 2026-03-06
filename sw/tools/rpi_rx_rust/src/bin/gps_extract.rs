//! Extract GPS-only rows from a raw log CSV, with optional IMU-derived heading.
//! Output: timestamp_us, lat_synth, lon_synth, v_north_mps, v_east_mps, gps_heading_deg, imu_heading_deg.
//! IMU heading comes from Madgwick on the GPS node's IMU (orientation only, for slide detection).
//! Output: <input_stem>_gps_only.csv

use clap::Parser;
use csv::{Reader, Writer};
use nalgebra::{Rotation3, Vector3};
use rpi_rx_rust::ahrs::AhrsFilter;
use rpi_rx_rust::board_calib::{board_to_vehicle_rotation, transform_accel, transform_gyro};
use serde::Serialize;
use std::path::Path;

const REST_PERIOD_US: u64 = 2 * 1_000_000; // 2 s steady state for board calibration
const AHRS_SAMPLE_PERIOD_S: f64 = 1.0 / 400.0;
const AHRS_BETA: f64 = 0.1;
/// Re-align IMU heading to GPS every N seconds; between alignments IMU captures rapid changes
const HEADING_ALIGN_INTERVAL_US: u64 = 5 * 1_000_000; // 5 s

#[derive(Parser, Debug)]
#[command(name = "gps_extract")]
#[command(about = "Extract GPS rows with GPS and IMU heading for track_plotter")]
struct Args {
    /// Input raw log CSV (from rpi_rx_rust)
    #[arg(short, long)]
    input: std::path::PathBuf,

    /// Output CSV path [default: <input_stem>_gps_only.csv]
    #[arg(short, long)]
    output: Option<std::path::PathBuf>,

    /// Generate *_gps_only_heading.png via track_plotter (blue track, red GPS heading, yellow IMU heading)
    #[arg(long)]
    generate_plot: bool,
}

#[derive(Debug, serde::Deserialize)]
struct RawCsvRow {
    src_mac: String,
    #[serde(rename = "timestamp_us")]
    timestamp_us: u64,
    #[serde(rename = "message_type")]
    message_type: String,
    #[serde(default)]
    accel_x: Option<f32>,
    #[serde(default)]
    accel_y: Option<f32>,
    #[serde(default)]
    accel_z: Option<f32>,
    #[serde(default)]
    gyro_x: Option<f32>,
    #[serde(default)]
    gyro_y: Option<f32>,
    #[serde(default)]
    gyro_z: Option<f32>,
    #[serde(default)]
    lat: Option<f64>,
    #[serde(default)]
    lon: Option<f64>,
    #[serde(default)]
    heading: Option<u16>,
}

/// One row of GPS-only output (track_plotter-compatible).
#[derive(Debug, Serialize)]
struct GpsOnlyRow {
    timestamp_us: u64,
    lat_synth: f64,
    lon_synth: f64,
    v_north_mps: f64,
    v_east_mps: f64,
    /// GPS heading in degrees [0, 360) from raw NMEA/GPS data
    gps_heading_deg: Option<u16>,
    /// IMU-derived heading (vehicle yaw) in degrees [0, 360); for slide detection vs GPS heading
    imu_heading_deg: Option<f32>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let stem = args.input.file_stem().unwrap_or_default().to_string_lossy();
    let parent = args.input.parent().unwrap_or(Path::new("."));
    let output_path = args.output.unwrap_or_else(|| {
        parent.join(format!("{}_gps_only.csv", stem))
    });

    // Load and sort rows
    let mut rdr = Reader::from_path(&args.input)?;
    let mut rows: Vec<RawCsvRow> = Vec::new();
    for result in rdr.deserialize() {
        let row: RawCsvRow = result?;
        if row.message_type == "Imu" || row.message_type == "Gps" {
            rows.push(row);
        }
    }
    rows.sort_by_key(|r| r.timestamp_us);

    // Find GPS node (first node that has Gps with lat/lon)
    let gps_node = rows
        .iter()
        .find(|r| {
            r.message_type == "Gps" && r.lat.is_some() && r.lon.is_some()
        })
        .map(|r| r.src_mac.clone());

    let mut filter: Option<AhrsFilter> = None;
    let mut r_b2v: Option<Rotation3<f64>> = None;

    if let Some(ref gps_mac) = gps_node {
        // Board calibration from first 2s of GPS node's IMU
        let first_ts = rows
            .iter()
            .filter(|r| r.message_type == "Imu" && r.src_mac == *gps_mac)
            .map(|r| r.timestamp_us)
            .min()
            .unwrap_or(0);
        let rest_end_ts = first_ts + REST_PERIOD_US;

        let mut accel_sum = (0.0_f64, 0.0_f64, 0.0_f64);
        let mut accel_n = 0u32;

        for row in &rows {
            if row.message_type == "Imu"
                && row.src_mac == *gps_mac
                && row.timestamp_us < rest_end_ts
            {
                if let (Some(ax), Some(ay), Some(az)) =
                    (row.accel_x, row.accel_y, row.accel_z)
                {
                    accel_sum.0 += ax as f64;
                    accel_sum.1 += ay as f64;
                    accel_sum.2 += az as f64;
                    accel_n += 1;
                }
            }
        }

        if accel_n > 0 {
            let mean = Vector3::new(
                accel_sum.0 / accel_n as f64,
                accel_sum.1 / accel_n as f64,
                accel_sum.2 / accel_n as f64,
            );
            r_b2v = Some(board_to_vehicle_rotation(mean));
        } else {
            r_b2v = Some(Rotation3::identity());
        }

        filter = Some(AhrsFilter::new(AHRS_SAMPLE_PERIOD_S, AHRS_BETA));
    }

    let mut wtr = Writer::from_path(&output_path)?;
    let mut count = 0u64;
    let mut prev_imu_ts: u64 = 0;
    // Re-align IMU to GPS periodically; between alignments IMU captures rapid changes (e.g. slide)
    let mut heading_offset: Option<f32> = None;
    let mut last_align_ts_us: u64 = 0;

    for row in &rows {
        if row.message_type == "Gps" {
            if let (Some(lat), Some(lon)) = (row.lat, row.lon) {
                let imu_heading = filter.as_ref().map(|f| f.get_vehicle_heading_deg());

                // Re-compute offset when first fix or when align interval elapsed
                let imu_aligned = if let Some(imu_h) = imu_heading {
                    if let Some(gps_h) = row.heading {
                        let should_align = heading_offset.is_none()
                            || row.timestamp_us.saturating_sub(last_align_ts_us)
                                >= HEADING_ALIGN_INTERVAL_US;
                        if should_align {
                            let diff = (gps_h as f32) - imu_h;
                            heading_offset = Some(if diff > 180.0 {
                                diff - 360.0
                            } else if diff < -180.0 {
                                diff + 360.0
                            } else {
                                diff
                            });
                            last_align_ts_us = row.timestamp_us;
                        }
                        let off = heading_offset.unwrap();
                        let aligned = imu_h + off;
                        Some(if aligned >= 360.0 {
                            aligned - 360.0
                        } else if aligned < 0.0 {
                            aligned + 360.0
                        } else {
                            aligned
                        })
                    } else if let Some(off) = heading_offset {
                        let aligned = imu_h + off;
                        Some(if aligned >= 360.0 {
                            aligned - 360.0
                        } else if aligned < 0.0 {
                            aligned + 360.0
                        } else {
                            aligned
                        })
                    } else {
                        Some(imu_h)
                    }
                } else {
                    None
                };

                wtr.serialize(GpsOnlyRow {
                    timestamp_us: row.timestamp_us,
                    lat_synth: lat,
                    lon_synth: lon,
                    v_north_mps: 0.0,
                    v_east_mps: 0.0,
                    gps_heading_deg: row.heading,
                    imu_heading_deg: imu_aligned,
                })?;
                count += 1;
            }
        } else if row.message_type == "Imu" {
            if let (Some(ref mut f), Some(ref r)) = (filter.as_mut(), r_b2v.as_ref()) {
                if row.src_mac == *gps_node.as_ref().unwrap() {
                    if let (Some(ax), Some(ay), Some(az), Some(gx), Some(gy), Some(gz)) = (
                        row.accel_x, row.accel_y, row.accel_z,
                        row.gyro_x, row.gyro_y, row.gyro_z,
                    ) {
                        if row.timestamp_us > prev_imu_ts {
                            prev_imu_ts = row.timestamp_us;
                            let (ax_v, ay_v, az_v) =
                                transform_accel(r, ax as f64, ay as f64, az as f64);
                            let (gx_v, gy_v, gz_v) =
                                transform_gyro(r, gx as f64, gy as f64, gz as f64);
                            f.update_imu_with_frame(
                                row.timestamp_us,
                                ax_v as f32, ay_v as f32, az_v as f32,
                                gx_v as f32, gy_v as f32, gz_v as f32,
                                true,
                            );
                        }
                    }
                }
            }
        }
    }

    wtr.flush()?;
    eprintln!("Wrote {} GPS rows to {:?}", count, output_path);

    if args.generate_plot && count > 0 {
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
        let plotter_debug = manifest.join("../track_plotter/target/debug/track_plotter");
        let plotter_release = manifest.join("../track_plotter/target/release/track_plotter");
        let plotter: &std::path::Path = if plotter_release.exists() {
            plotter_release.as_path()
        } else if plotter_debug.exists() {
            plotter_debug.as_path()
        } else {
            Path::new("track_plotter")
        };
        let heading_png = parent.join(format!("{}_gps_only_heading.png", stem));
        if let Ok(status) = std::process::Command::new(plotter)
            .arg("--input")
            .arg(&output_path)
            .arg("--output")
            .arg(&heading_png)
            .arg("--plot-heading")
            .output()
        {
            if status.status.success() {
                eprintln!("Generated {:?}", heading_png);
            }
        }
    }

    Ok(())
}
