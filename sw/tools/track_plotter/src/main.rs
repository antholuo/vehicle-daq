use clap::Parser;
use csv::Reader;
use image::{imageops, ImageBuffer, Rgba};
use reqwest::blocking::Client;
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(name = "track_plotter")]
#[command(about = "Generate a PNG track plot from AHRS postprocess CSV with the track overlaid on a satellite map")]
struct Args {
    /// Input AHRS postprocess CSV
    #[arg(short, long)]
    input: PathBuf,

    /// Output PNG path [default: <input_stem>_track.png]
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Output image width in pixels
    #[arg(long, default_value = "1024")]
    width: u32,

    /// Output image height in pixels
    #[arg(long, default_value = "1024")]
    height: u32,

    /// Margin (pixels) around track
    #[arg(long, default_value = "20")]
    margin: u32,

    /// Tile URL template for the satellite/base map (use {z}/{x}/{y} placeholders). Default is ArcGIS World Imagery (satellite).
    #[arg(long, default_value = "https://services.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/{z}/{y}/{x}")]
    tile_url_template: String,

    /// Zoom level override (0-19). If unset, auto-fit bounds.
    #[arg(long)]
    zoom: Option<u8>,

    /// Draw red lines indicating GPS heading at every 10th point (requires gps_heading_deg in input)
    #[arg(long)]
    plot_heading: bool,
}

#[derive(Debug, Deserialize)]
struct AhrsRow {
    timestamp_us: u64,
    lat_synth: f64,
    lon_synth: f64,
    v_north_mps: f64,
    v_east_mps: f64,
    #[serde(default)]
    gps_heading_deg: Option<u16>,
    /// IMU-derived vehicle heading for slide detection (yellow when plotting)
    #[serde(default)]
    imu_heading_deg: Option<f32>,
}

#[derive(Debug, Clone, Copy)]
struct Point {
    x: f64,
    y: f64,
}

const TILE_SIZE: f64 = 256.0;
const MAX_ZOOM: u8 = 19;

fn latlon_to_world_px(lat: f64, lon: f64, zoom: u8) -> (f64, f64) {
    let lat_rad = lat.to_radians();
    let n = 2.0_f64.powi(zoom as i32);
    let x = (lon + 180.0) / 360.0;
    let y = (1.0 - (lat_rad.tan() + 1.0 / lat_rad.cos()).ln() / std::f64::consts::PI) / 2.0;
    (x * TILE_SIZE * n, y * TILE_SIZE * n)
}

fn choose_zoom(bounds_min: Point, bounds_max: Point, width: u32, height: u32, margin: f64) -> u8 {
    let mut best = 0;
    for zoom in 0..=MAX_ZOOM {
        let (min_x, min_y) = latlon_to_world_px(bounds_min.y, bounds_min.x, zoom);
        let (max_x, max_y) = latlon_to_world_px(bounds_max.y, bounds_max.x, zoom);
        let span_x = (max_x - min_x).abs();
        let span_y = (max_y - min_y).abs();
        let max_w = (width as f64) - 2.0 * margin;
        let max_h = (height as f64) - 2.0 * margin;
        if span_x <= max_w && span_y <= max_h {
            best = zoom;
        } else {
            break;
        }
    }
    best
}

fn tile_url(template: &str, z: u8, x: i64, y: i64) -> String {
    template
        .replace("{z}", &z.to_string())
        .replace("{x}", &x.to_string())
        .replace("{y}", &y.to_string())
}

fn fetch_tile(client: &Client, url: &str) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    match client.get(url).send().and_then(|r| r.error_for_status()) {
        Ok(resp) => match resp.bytes() {
            Ok(bytes) => match image::load_from_memory(&bytes) {
                Ok(img) => img.to_rgba8(),
                Err(_) => ImageBuffer::from_pixel(256, 256, Rgba([230, 230, 230, 255])),
            },
            Err(_) => ImageBuffer::from_pixel(256, 256, Rgba([230, 230, 230, 255])),
        },
        Err(_) => ImageBuffer::from_pixel(256, 256, Rgba([230, 230, 230, 255])),
    }
}

fn draw_line(img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, a: (i32, i32), b: (i32, i32), color: Rgba<u8>) {
    let (mut x0, mut y0) = a;
    let (x1, y1) = b;
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        if x0 >= 0 && y0 >= 0 {
            let (xu, yu) = (x0 as u32, y0 as u32);
            if xu < img.width() && yu < img.height() {
                img.put_pixel(xu, yu, color);
            }
        }
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

fn output_path_for(input: &Path) -> PathBuf {
    let stem = input.file_stem().unwrap_or_default().to_string_lossy();
    let parent = input.parent().unwrap_or_else(|| Path::new("."));
    parent.join(format!("{}_track.png", stem))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let output_path = args.output.unwrap_or_else(|| output_path_for(&args.input));
    let mut rdr = Reader::from_path(&args.input)?;
    let mut rows: Vec<AhrsRow> = Vec::new();
    for result in rdr.deserialize() {
        let row: AhrsRow = result?;
        rows.push(row);
    }

    let width = args.width.max(1);
    let height = args.height.max(1);
    let margin = args.margin.min(width / 2).min(height / 2) as f64;

    if rows.is_empty() {
        let img: ImageBuffer<Rgba<u8>, Vec<u8>> =
            ImageBuffer::from_pixel(width, height, Rgba([255, 255, 255, 255]));
        img.save(&output_path)?;
        println!("Wrote {:?} (empty track)", output_path);
        return Ok(());
    }

    // Track points with optional GPS heading and IMU heading for each point.
    let latlon_track: Vec<(Point, Option<u16>, Option<f32>)> = rows
        .iter()
        .filter(|row| !(row.lat_synth == 0.0 && row.lon_synth == 0.0))
        .map(|row| {
            (
                Point {
                    x: row.lon_synth,
                    y: row.lat_synth,
                },
                row.gps_heading_deg,
                row.imu_heading_deg,
            )
        })
        .collect();

    // Use satellite (or other tile) map whenever we have any valid lat/lon; overlay track on it.
    let use_map = !latlon_track.is_empty();

    let track_points: Vec<(Point, Option<u16>, Option<f32>)> = if latlon_track.len() >= 2 {
        latlon_track
    } else {
        let mut pts: Vec<(Point, Option<u16>, Option<f32>)> = Vec::with_capacity(rows.len());
        let mut last_ts = rows[0].timestamp_us;
        let mut p_north = 0.0f64;
        let mut p_east = 0.0f64;
        pts.push((Point { x: p_east, y: p_north }, None, None));
        for row in rows.iter().skip(1) {
            let dt = (row.timestamp_us.saturating_sub(last_ts)) as f64 / 1_000_000.0;
            last_ts = row.timestamp_us;
            let dt = if dt <= 0.0 || dt > 10.0 { 0.0 } else { dt };
            p_north += row.v_north_mps * dt;
            p_east += row.v_east_mps * dt;
            pts.push((Point { x: p_east, y: p_north }, None, None));
        }
        pts
    };

    let points: Vec<Point> = track_points.iter().map(|(p, _, _)| *p).collect();

    let (mut min_x, mut max_x) = (points[0].x, points[0].x);
    let (mut min_y, mut max_y) = (points[0].y, points[0].y);
    for p in &points[1..] {
        if p.x < min_x {
            min_x = p.x;
        }
        if p.x > max_x {
            max_x = p.x;
        }
        if p.y < min_y {
            min_y = p.y;
        }
        if p.y > max_y {
            max_y = p.y;
        }
    }

    let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::from_pixel(width, height, Rgba([255, 255, 255, 255]));
    let mut to_pixel: Box<dyn Fn(Point) -> (i32, i32)> = Box::new(|p| {
        let x = p.x.round() as i32;
        let y = p.y.round() as i32;
        (x, y)
    });

    if use_map {
        let bounds_min = Point { x: min_x, y: min_y };
        let bounds_max = Point { x: max_x, y: max_y };
        let zoom = args
            .zoom
            .unwrap_or_else(|| choose_zoom(bounds_min, bounds_max, width, height, margin));
        let center = Point {
            x: (min_x + max_x) / 2.0,
            y: (min_y + max_y) / 2.0,
        };
        let (center_x, center_y) = latlon_to_world_px(center.y, center.x, zoom);
        let top_left_x = center_x - (width as f64) / 2.0;
        let top_left_y = center_y - (height as f64) / 2.0;
        let min_tile_x = (top_left_x / TILE_SIZE).floor() as i64;
        let min_tile_y = (top_left_y / TILE_SIZE).floor() as i64;
        let max_tile_x = ((top_left_x + (width as f64) - 1.0) / TILE_SIZE).floor() as i64;
        let max_tile_y = ((top_left_y + (height as f64) - 1.0) / TILE_SIZE).floor() as i64;

        let tiles_w = (max_tile_x - min_tile_x + 1) as u32;
        let tiles_h = (max_tile_y - min_tile_y + 1) as u32;
        let mut mosaic: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_pixel(
            (tiles_w as f64 * TILE_SIZE) as u32,
            (tiles_h as f64 * TILE_SIZE) as u32,
            Rgba([230, 230, 230, 255]),
        );

        let client = Client::builder()
            .user_agent("track_plotter/0.1")
            .build()?;
        for ty in min_tile_y..=max_tile_y {
            for tx in min_tile_x..=max_tile_x {
                let url = tile_url(&args.tile_url_template, zoom, tx, ty);
                let tile = fetch_tile(&client, &url);
                let px = ((tx - min_tile_x) as f64 * TILE_SIZE) as u32;
                let py = ((ty - min_tile_y) as f64 * TILE_SIZE) as u32;
                imageops::overlay(&mut mosaic, &tile, px.into(), py.into());
            }
        }

        let crop_x = (top_left_x - (min_tile_x as f64 * TILE_SIZE)).max(0.0);
        let crop_y = (top_left_y - (min_tile_y as f64 * TILE_SIZE)).max(0.0);
        let view = imageops::crop_imm(&mosaic, crop_x as u32, crop_y as u32, width, height);
        img = view.to_image();

        to_pixel = Box::new(move |p: Point| {
            let (wx, wy) = latlon_to_world_px(p.y, p.x, zoom);
            let x = (wx - top_left_x).round() as i32;
            let y = (wy - top_left_y).round() as i32;
            (x, y)
        });
    } else {
        let span_x = (max_x - min_x).max(1e-12);
        let span_y = (max_y - min_y).max(1e-12);
        let scale_x = ((width as f64) - 2.0 * margin) / span_x;
        let scale_y = ((height as f64) - 2.0 * margin) / span_y;
        let scale = scale_x.min(scale_y);
        let used_w = span_x * scale;
        let used_h = span_y * scale;
        let pad_x = ((width as f64) - used_w) / 2.0;
        let pad_y = ((height as f64) - used_h) / 2.0;
        to_pixel = Box::new(move |p: Point| {
            let px = pad_x + (p.x - min_x) * scale;
            let py = pad_y + (p.y - min_y) * scale;
            let x = px.round() as i32;
            let y = (height as f64 - py).round() as i32;
            (x, y)
        });
    }
    let track_color = Rgba([20, 92, 212, 255]);

    let mut last = to_pixel(points[0]);
    for p in points.iter().skip(1) {
        let current = to_pixel(*p);
        draw_line(&mut img, last, current, track_color);
        last = current;
    }
    // If only one point, draw at least a single pixel so the track is visible on the map
    if points.len() == 1 {
        let (x, y) = last;
        if x >= 0 && y >= 0 && (x as u32) < width && (y as u32) < height {
            img.put_pixel(x as u32, y as u32, track_color);
        }
    }

    // Draw GPS heading (red) every 10 points when --plot-heading
    if args.plot_heading {
        const HEADING_LINE_LEN: f64 = 25.0;
        let gps_heading_color = Rgba([220, 53, 69, 255]);   // red
        for (i, (point, gps_h, _imu_h)) in track_points.iter().enumerate() {
            if i % 10 == 0 {
                if let Some(h) = gps_h {
                    let rad = (*h as f64).to_radians();
                    let dx = rad.sin();
                    let dy = -rad.cos();
                    let start = to_pixel(*point);
                    let end = (
                        start.0 + (dx * HEADING_LINE_LEN).round() as i32,
                        start.1 + (dy * HEADING_LINE_LEN).round() as i32,
                    );
                    draw_line(&mut img, start, end, gps_heading_color);
                }
            }
        }
    }

    img.save(&output_path)?;
    println!("Wrote {:?}", output_path);
    Ok(())
}
