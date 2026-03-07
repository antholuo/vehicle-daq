# sensor_board_rev1_test

ESP32-C6 sensor board firmware: data-acquisition nodes, USB bridge, and GPS-only “floating” node for track capture. Uses ESP-NOW for wireless data and COBS over USB to the Raspberry Pi.

## Binaries

| Binary | Purpose |
|--------|--------|
| `rev1_board` | **Sensor node**: streams IMU + GPS + heartbeat over ESP-NOW. Optional position feature (e.g. `pos_center`) for node ID. |
| `devkit_c` | **Bridge**: receives ESP-NOW from nodes, forwards to host over USB serial (COBS). Accepts TimeSync and RequestFloatingGps from host. |
| `sensorboard_floating` | **Floating (GPS-only) node**: waits for RequestGpsCapture over ESP-NOW, replies with the next 5 GPS samples. Used with RPi track capture. |

## Build commands

All builds from this directory (`sw/sensor_board/sensor_board_rev1_test/`).

### Sensor node (default: HMI + IMU + GPS + WiFi)

```bash
# Default (includes hmi, imu, gps, wifi)
cargo build --bin rev1_board

# With car position (for node ID)
cargo build --bin rev1_board --features pos_center
# Other positions: pos_front_left, pos_front_center, pos_front_right, pos_left, pos_right,
#                  pos_rear_left, pos_rear_center, pos_rear_right, pos_roof
```

### Bridge (USB forwarder for RPi)

By default, both **flashing/debug log** and **RPi communication** use the same USB (USB-JTAG):

```bash
cargo build --bin devkit_c --no-default-features --features bridge
```

If the devkit has a **separate serial port** (UART), you can send log output there and keep USB **only** for RPi/bridge traffic (flashing still over USB):

```bash
cargo build --bin devkit_c --no-default-features --features bridge_uart_log
```

- **`bridge`**: Log and RPi COBS both on USB. Use for a single USB connection.
- **`bridge_uart_log`**: Log on UART (ROM default, typically UART0); RPi on USB. Connect a serial adapter to the UART port for `info!`/`warn!` etc., and plug USB to the RPi for clean COBS.

**Viewing UART debug on Linux** (when using `bridge_uart_log`): the ROM UART console is usually **115200 8N1**. Find the serial device (e.g. `/dev/ttyUSB0` for an FTDI adapter, or the second port if the board exposes two):

```bash
# List serial ports (plug in the UART adapter, then run)
ls /dev/ttyUSB* /dev/ttyACM* 2>/dev/null

# Connect with picocom (Ctrl+A Ctrl+X to exit)
picocom -b 115200 /dev/ttyUSB0

# Or with screen
screen /dev/ttyUSB0 115200
# Detach: Ctrl+A then K, then Y

# Or with minicom
minicom -D /dev/ttyUSB0 -b 115200
```

Use the device that corresponds to the **UART/serial** connector (not the USB-JTAG port used for RPi or flashing).

### Floating node (GPS-only, track capture)

```bash
cargo build --bin sensorboard_floating --no-default-features --features floating
```

Build all three binaries in one go:

```bash
cargo build --bin rev1_board
cargo build --bin devkit_c --no-default-features --features bridge
cargo build --bin sensorboard_floating --no-default-features --features floating
```

### Optional features (sensor / floating)

- `set_gps_high_baud`: configure GPS UART to 460800 (e.g. NEO-F10).
- `set_gps_10hz`: set GPS to 10 Hz (floating feature already enables this).

Example with high baud and 10 Hz on the default sensor node:

```bash
cargo build --bin rev1_board --features set_gps_high_baud,set_gps_10hz
```

## Features reference

| Feature | Description |
|---------|-------------|
| `hmi` | User LED + NeoPixel (WS2812) + display SPI. |
| `imu` | IMU over SPI (e.g. ASM330). |
| `gps` | GPS over UART (NMEA). |
| `wifi` | WiFi + ESP-NOW. |
| `usb` | USB Serial/JTAG for bridge → host. |
| `bridge` | Shorthand: `hmi`, `usb`, `wifi`, `jtag-log` (devkit_c; log + RPi on USB). |
| `bridge_uart_log` | Like bridge but log on UART, RPi on USB only (for devkit with separate serial port). |
| `floating` | Shorthand: `hmi`, `gps`, `wifi`, `set_gps_10hz` (for sensorboard_floating). |
| `pos_*` | Car position for node ID (pick one). |

## HMI (user LED + NeoPixel)

Hardware: **User LED** (GPIO19) and **NeoPixel** (GPIO18, WS2812).

### User LED

- Toggles at 1 Hz as a **heartbeat** on all binaries that enable HMI.

### NeoPixel (sensor node `rev1_board` and bridge `devkit_c`)

- **Color** reflects **IMU data age** (sensor node only; bridge runs HMI but has no IMU, so typically red/off):
  - **Green**: IMU data &lt; 20 ms old.
  - **Blue**: IMU data &lt; 10 s old.
  - **Red**: No IMU or &gt; 10 s old.
- **Pattern** reflects **GPS**:
  - **Solid**: GPS fix (position valid).
  - **Double pulse** (on 0.1 s, off 0.1 s, on 0.1 s, off 0.7 s): GPS connected, no fix.
  - **Slow blink**: No GPS data (or no fix).

### NeoPixel (floating node `sensorboard_floating`)

- **Idle** (no capture in progress): same as above but **green only** (no IMU):
  - **Solid green**: GPS fix.
  - **Double-pulse green**: GPS connected, no fix.
  - **Slow blink green**: No GPS data.
- **Acquiring** (after RequestGpsCapture):
  - **Solid orange** for 50 ms.
  - Then **rapid orange blink** (~50 Hz) for up to ~600 ms while collecting 5 GPS samples.
  - Then back to idle (green) behaviour.

## Target

ESP32-C6 (e.g. ESP32-C6-WROOM, DevKit-C). Flashing is typically done with `espflash` or the ESP-IDF toolchain; see the main vehicle-daq docs for flash layout and commands.
