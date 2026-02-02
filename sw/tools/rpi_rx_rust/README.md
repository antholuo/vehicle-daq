# rpi_rx_rust

Raspberry Pi receiver for the sensor_board COBS-over-serial stream. Decodes IMU, GPS, and heartbeat messages, writes CSV logs, and optionally runs real-time AHRS (attitude/heading reference system) with synthesized GPS and velocity integration.

## Features

- **COBS decoding**: Consumes COBS-encoded frames from serial (or stdin), decodes header (MAC, node ID, timestamp, message type) and payloads for IMU, GPS, and heartbeat.
- **CSV logging**: Writes one row per decoded message to a raw CSV (MAC, node position/instance, timestamp, type, and type-specific fields).
- **Real-time AHRS** (optional): Madgwick orientation filter plus velocity/position integration in NED. Uses the first IMU node as primary; first 10 seconds are treated as “at rest” for gyro bias and velocity zeroing. Outputs synthesized lat/lon/alt, vehicle heading, ground track, roll/pitch/yaw, and NED velocity at a fixed rate (default 10 Hz).
- **Post-process AHRS**: The `ahrs_postprocess` binary runs AHRS on an existing raw log CSV (e.g. for re-runs with different parameters or when real-time AHRS was not used).
- **Unified log layout**: All binaries write under `~/daq/logs/<date>/` with `<time>_raw.csv` and `<time>_postprocess.csv` (postprocess = AHRS output).
- **GPIO-controlled service**: The `rpi_rx_rust_gpio` binary (Linux, `--features gpio`) uses two GPIOs to control logging with no CLI flags, suitable for a systemd service that starts on boot.

## Binaries

| Binary              | Purpose |
|---------------------|--------|
| `rpi_rx_rust`      | Interactive/CLI: read from serial (or stdin), log raw CSV and optionally real-time AHRS. |
| `ahrs_postprocess`  | Offline: run AHRS on a raw log CSV; output `<stem>_ahrs_postprocess.csv`. |
| `rpi_rx_rust_gpio` | Service: GPIO-controlled start/stop; no CLI. Build with `cargo build --release --features gpio` (Linux only). |

## Usage

### rpi_rx_rust (CLI)

```bash
# Serial only, raw CSV (default: ~/daq/logs/<date>/<time>.csv or <time>_raw.csv with --with-ahrs)
cargo run

# Serial + real-time AHRS (raw + ahrs CSVs; default 10 Hz)
cargo run -- --with-ahrs

# Custom AHRS rate (e.g. 50 Hz)
cargo run -- --with-ahrs --ahrs-hz 50
```

Serial defaults: `/dev/ttyACM0`, 115200 baud. If the port cannot be opened, input falls back to stdin.

### ahrs_postprocess

```bash
cargo run --bin ahrs_postprocess -- --input path/to/raw.csv [--output path/to/out.csv] [--ahrs-hz 10]
```

Assumes the first 10 seconds of the log are at rest. Uses first GPS fix as origin. Output defaults to `<input_stem>_ahrs_postprocess.csv`.

### rpi_rx_rust_gpio (service binary)

- **GPIO #11 (BCM 11)**: Init — start raw logging and allow AHRS to initialize (vehicle assumed level and stationary).
- **GPIO #5 (BCM 5)**: Postprocessed — when high, also log AHRS output at the default rate (10 Hz).

Logging runs while **either** GPIO is high. Logs **stop only when both GPIOs are low**. Temporary loss of serial data does not stop the session; only both GPIOs going low does.

- **Build** (on Linux, with libgpiod):  
  `cargo build --release --features gpio`
- **Install**: Copy the binary to e.g. `/opt/rpi_rx_rust/bin/rpi_rx_rust_gpio`.
- **Log paths**: Same as CLI — `~/daq/logs/<date>/<time>_raw.csv` and `<time>_postprocess.csv` (when GPIO5 is high). Default is `$HOME/daq/logs/<date>/`; override with `RPI_RX_OUTPUT_DIR` (base path; logs go under `<RPI_RX_OUTPUT_DIR>/logs/<date>/`).
- **Environment** (optional):
  - `RPI_RX_OUTPUT_DIR`: base directory for logs (default: `$HOME`; then logs under `daq/logs/<date>/`).
  - `RPI_RX_SERIAL_PORT`: serial device (default: `/dev/ttyACM0`).

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
