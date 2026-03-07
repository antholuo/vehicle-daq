# rpi_rx_rust

Raspberry Pi receiver for the sensor_board COBS-over-serial stream. Decodes IMU, GPS, and heartbeat messages, writes CSV logs, and optionally runs real-time AHRS (attitude/heading reference system) with synthesized GPS and velocity integration. Can also request floating GPS for track capture (cone/waypoint logging).

## Features

- **COBS decoding**: Consumes COBS-encoded frames from serial (or stdin), decodes header (MAC, node ID, timestamp, message type) and payloads for IMU, GPS, and heartbeat.
- **CSV logging**: Writes one row per decoded message to a raw CSV (MAC, node position/instance, timestamp, type, and type-specific fields).
- **Real-time AHRS** (optional): Madgwick orientation filter plus velocity/position integration in NED. Uses the first IMU node as primary; first 10 seconds are treated as “at rest” for gyro bias and velocity zeroing. Outputs synthesized lat/lon/alt, vehicle heading, ground track, roll/pitch/yaw, and NED velocity at a fixed rate (default 10 Hz).
- **Post-process AHRS**: The `ahrs_postprocess` binary runs AHRS on an existing raw log CSV (e.g. for re-runs with different parameters or when real-time AHRS was not used).
- **Unified log layout**: Logs under `~/daq/logs/<date>/` with `<time>_raw.csv` and `<time>_postprocess.csv` (postprocess = AHRS output). Track capture CSVs under `~/daq/tracks/<date>/`.
- **GPIO-controlled service**: The `rpi_rx_rust_gpio` binary (Linux, `--features gpio`) uses GPIOs to control logging, video recording, and track capture with no CLI, suitable for systemd.
- **Track capture**: Request 5-sample averaged GPS from a floating sensor node; log cone/waypoint positions to CSV (GPIO or interactive `rpi_rx_rust_track_test`). When armed, the Pi relies on a **track-capture heartbeat** (message type 0xFE) from the bridge every 500 ms; if no heartbeat for 3 s the Pi auto-ends track capture. If the Pi stops sending (e.g. crash), the bridge auto-ends after 2 s and other nodes resume.

## Building all binaries

From this directory (`sw/tools/rpi_rx_rust/`):

| Binary | Build command |
|--------|----------------|
| `rpi_rx_rust` | `cargo build --release` |
| `ahrs_postprocess` | `cargo build --release --bin ahrs_postprocess` |
| `gps_extract` | `cargo build --release --bin gps_extract` |
| `rpi_rx_rust_gpio` | `cargo build --release --features gpio` (Linux, libgpiod) |
| `rpi_rx_rust_track_test` | `cargo build --release --bin rpi_rx_rust_track_test` |

One-liner to build everything that doesn’t require optional features:

```bash
cargo build --release --bin rpi_rx_rust --bin ahrs_postprocess --bin gps_extract --bin rpi_rx_rust_track_test
cargo build --release --features gpio  # separately, for GPIO binary (Linux)
```

## Binaries

| Binary | Purpose |
|--------|--------|
| `rpi_rx_rust` | Interactive/CLI: read from serial (or stdin), log raw CSV and optionally real-time AHRS. |
| `ahrs_postprocess` | Offline: run AHRS on a raw log CSV; output `<stem>_ahrs_postprocess.csv`. |
| `gps_extract` | Extract or filter GPS data from logs (see binary help). |
| `rpi_rx_rust_gpio` | Service: GPIO-controlled logging, video, and track capture; no CLI. Requires `--features gpio` (Linux). |
| `rpi_rx_rust_track_test` | Interactive track capture: type “arm track capture”, “take location”, “end track capture” to test without GPIO. |

## Usage

### Summary: how you interact with each binary

- **rpi_rx_rust**: CLI flags + serial (or stdin). No GPIO.
- **rpi_rx_rust_gpio**: GPIO only (11, 19, 26, 10, 9). No CLI; for systemd.
- **rpi_rx_rust_track_test**: Serial + typed commands (“arm track capture”, “take location”, “end track capture”, “quit”). For testing track capture without GPIO.
- **ahrs_postprocess** / **gps_extract**: Offline; input/output files and CLI args.

### rpi_rx_rust (CLI)

```bash
# Serial only, raw CSV (default: ~/daq/logs/<date>/<time>.csv or <time>_raw.csv with --with-ahrs)
cargo run

# Serial + real-time AHRS (raw + ahrs CSVs; default 10 Hz)
cargo run -- --with-ahrs

# Custom AHRS rate (e.g. 50 Hz)
cargo run -- --with-ahrs --ahrs-hz 50
```

Serial defaults: `/dev/ttyACM0`, 115200 baud. If the port cannot be opened, input falls back to stdin. Set `RPI_RX_SERIAL_PORT` if the bridge is on another port.

### rpi_rx_rust_track_test

```bash
cargo run --bin rpi_rx_rust_track_test
# or: ./target/release/rpi_rx_rust_track_test
```

Then type: `arm track capture`, then `take location` for each cone/waypoint, then `end track capture` when done. Optional: `RPI_RX_SERIAL_PORT=/dev/ttyACM0` (default).

### ahrs_postprocess

```bash
cargo run --bin ahrs_postprocess -- --input path/to/raw.csv [--output path/to/out.csv] [--ahrs-hz 10]
```

Assumes the first 10 seconds of the log are at rest. Uses first GPS fix as origin. Output defaults to `<input_stem>_ahrs_postprocess.csv`.

### rpi_rx_rust_gpio (service binary) — GPIO “HMI”

All interaction is via GPIO; no CLI.

| GPIO (BCM) | Function |
|------------|----------|
| **11** | Init — start raw logging; AHRS init (vehicle assumed level and stationary). |
| **19** | Postprocessed — when high, also log AHRS at 10 Hz. |
| **26** | Video — high = start recording, low = stop (camera server on `localhost:8888`). |
| **10** | Track arm — **rising edge** opens a new track file under `~/daq/tracks/<date>/<time>.csv` and resets cone #. |
| **9** | Track capture — **rising edge** (with track armed) requests floating GPS, averages 5 samples, appends one row (Cone #, Lat, Lon, Alt) and increments cone #. |

- Data logging runs while **either** GPIO #11 or #19 is high. Logs **stop only when both are low**.
- Video is independent of logging (GPIO #26).
- Track capture only runs when a session is active (GPIO 11 or 19 high). Arm with GPIO 10 (high), then trigger each cone/waypoint with a rising edge on GPIO 9.

### rpi_rx_rust_track_test — interactive “HMI”

No GPIO; all interaction is by typing commands (bridge must be on serial, default `/dev/ttyACM0`):

| Command | Action |
|---------|--------|
| `arm track capture` | Create a new track file and arm; bridge sends track-capture heartbeats (0xFE) every 500 ms so the Pi knows it’s alive. |
| `take location` | Send RequestFloatingGps to bridge; receive 5 GPS samples, average, append one row (Cone #, Lat, Lon, Alt). |
| `end track capture` | Close the current track file and tell the bridge to end. |
| `quit` | Exit. |

- **Heartbeat**: Only the bridge’s **track-capture heartbeat** (type 0xFE) updates the “last activity” time. If no heartbeat for **3 s** (e.g. bridge unplugged), the Pi auto-ends track capture and closes the file.
- **Logs**: Only messages from node `"Floating"` are printed to the console; other nodes are still received and forwarded but not logged here.
- Track files: `~/daq/tracks/<date>/<time>.csv` (or `RPI_RX_OUTPUT_DIR`/tracks). Override serial port with `RPI_RX_SERIAL_PORT`.

### rpi_rx_rust (CLI) — no physical HMI

Interaction is via command-line flags and serial/stdin; see “Usage” below.

- **Build** (on Linux, with libgpiod):  
  `cargo build --release --features gpio`
- **Install**: Copy the binary to e.g. `/opt/rpi_rx_rust/bin/rpi_rx_rust_gpio`.
- **Log paths**: `~/daq/logs/<date>/<time>_raw.csv` and `<time>_postprocess.csv` (when GPIO19 is high). Track files: `~/daq/tracks/<date>/<time>.csv`. Base path override: `RPI_RX_OUTPUT_DIR` (logs under `<base>/logs/<date>/`, tracks under `<base>/tracks/<date>/`).
- **Serial port**: Default `/dev/ttyACM0` (bridge). Which device is ACM0 vs ACM1 depends on USB enumeration order (e.g. bridge may be `/dev/ttyACM1` and sensor board `/dev/ttyACM0`). Set `RPI_RX_SERIAL_PORT` to the **bridge** port so the RPi talks to the bridge (e.g. `RPI_RX_SERIAL_PORT=/dev/ttyACM1`).
- **Environment** (optional):
  - `RPI_RX_OUTPUT_DIR`: base directory (default `$HOME`; then `daq/logs/<date>/` and `daq/tracks/<date>/`).
  - `RPI_RX_SERIAL_PORT`: serial device for bridge (default: `/dev/ttyACM0`).

## Systemd service

A unit file `rpi-rx-rust.service` is provided so the GPIO-controlled binary can run as a service that starts on boot.

**Install the binary for systemd (one command):**

From the project root, run:

```bash
# Create install dir and make it writable by your user (once)
sudo mkdir -p /opt/rpi_rx_rust && sudo chown $USER: /opt/rpi_rx_rust

# Build and install the GPIO binary to /opt/rpi_rx_rust/bin/ (where the service expects it)
cargo install-service
```

The `.cargo/config.toml` alias `install-service` runs `cargo install --path . --root /opt/rpi_rx_rust --bin rpi_rx_rust_gpio --features gpio`, so the release binary is placed in `/opt/rpi_rx_rust/bin/rpi_rx_rust_gpio`.

**Then enable the service** (adjust `User`/`Group` in the unit if needed):
   ```bash
   sudo cp rpi-rx-rust.service /etc/systemd/system/
   sudo systemctl daemon-reload
   sudo systemctl enable rpi-rx-rust
   sudo systemctl start rpi-rx-rust
   ```
Ensure the service user is in the `gpio` group so libgpiod can access the GPIOs.

Start/stop of logging is then fully controlled by the two GPIOs; the process stays running and only starts or stops sessions when GPIOs change.

## Dependencies

- **Rust** (edition 2021).
- **Linux** (for `rpi_rx_rust_gpio`): system libgpiod (e.g. `libgpiod2` or `libgpiod-dev`) and the `gpiod` crate (included via `--features gpio`).

## License

Same as the parent vehicle-daq project.
