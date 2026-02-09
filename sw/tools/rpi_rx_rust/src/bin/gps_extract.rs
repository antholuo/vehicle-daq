//! Extract GPS-only rows from a raw log CSV (no AHRS).
//! Output format matches track_plotter input: timestamp_us, lat_synth, lon_synth, v_north_mps, v_east_mps.
//! Output: <input_stem>_gps_only.csv

use clap::Parser;
use csv::{Reader, Writer};
use serde::Serialize;
use std::path::Path;

#[derive(Parser, Debug)]
#[command(name = "gps_extract")]
#[command(about = "Extract GPS-only rows from raw log CSV for track_plotter (no AHRS)")]
struct Args {
    /// Input raw log CSV (from rpi_rx_rust)
    #[arg(short, long)]
    input: std::path::PathBuf,

    /// Output CSV path [default: <input_stem>_gps_only.csv]
    #[arg(short, long)]
    output: Option<std::path::PathBuf>,
}

#[derive(Debug, serde::Deserialize)]
struct RawCsvRow {
    #[serde(rename = "timestamp_us")]
    timestamp_us: u64,
    #[serde(rename = "message_type")]
    message_type: String,
    lat: Option<f64>,
    lon: Option<f64>,
}

/// One row of GPS-only output (track_plotter-compatible).
#[derive(Debug, Serialize)]
struct GpsOnlyRow {
    timestamp_us: u64,
    lat_synth: f64,
    lon_synth: f64,
    v_north_mps: f64,
    v_east_mps: f64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let output_path = args.output.unwrap_or_else(|| {
        let stem = args.input.file_stem().unwrap_or_default();
        let parent = args.input.parent().unwrap_or(Path::new("."));
        parent.join(format!("{}_gps_only.csv", stem.to_string_lossy()))
    });

    let mut rdr = Reader::from_path(&args.input)?;
    let mut wtr = Writer::from_path(&output_path)?;

    let mut count = 0u64;
    for result in rdr.deserialize() {
        let row: RawCsvRow = result?;
        if row.message_type == "Gps" {
            if let (Some(lat), Some(lon)) = (row.lat, row.lon) {
                // Skip invalid (0,0) if you want; track_plotter filters these too
                wtr.serialize(GpsOnlyRow {
                    timestamp_us: row.timestamp_us,
                    lat_synth: lat,
                    lon_synth: lon,
                    v_north_mps: 0.0,
                    v_east_mps: 0.0,
                })?;
                count += 1;
            }
        }
    }

    wtr.flush()?;
    eprintln!("Wrote {} GPS rows to {:?}", count, output_path);
    Ok(())
}
