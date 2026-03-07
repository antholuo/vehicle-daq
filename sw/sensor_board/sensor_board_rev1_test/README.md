# sensor_board_rev1_test

This project was originally supposed to just be a test project for the ESP-HAL:1.0.0 release, but we continued to use it since it just worked and it kinda snowballed. You will find firmware for our Rev1 DAQ Nodes (ESP32-C6 based) operating both as sensor acquisition nodes and floating gps locator nodes, as well as firmware for ESP32-C6 devkits acting as our aircomm bridge board. In theory, a Rev1 DAQ node could act as a bridge node, except we messed up our board design so the second UART doesn't work :(.

## Binaries

`rev1_board` | **Sensor node**: streams IMU + GPS + heartbeat over ESP-NOW. Pauses transmissions when track capture is armed (to avoid dropping floating-node packets). Optional position feature (e.g. `pos_center`) for node ID.
`sensorboard_floating` | **Floating (GPS-only) Sensor node**: waits for RequestGpsCapture over ESP-NOW, replies with the next 5 GPS samples. Used with RPi track capture.
`devkit_c` | **Bridge node**: receives ESP-NOW from nodes, forwards to host over USB (COBS). Accepts TimeSync, Arm/End track, RequestFloatingGps, CaptureSuccess/CaptureTimeout. When track capture is armed, sends a **track-capture heartbeat** (0xFE) to the Pi every 500 ms; if no USB from Pi for 2 s, auto-ends and broadcasts TrackCaptureEnded.

## Build commands

All builds from this directory (`sw/sensor_board/sensor_board_rev1_test/`).

### Sensor node (default: HMI + IMU + GPS + WiFi)

```bash
# Default (includes hmi, imu, gps, wifi)
cargo run # You do not need to include anything, sensor node will run by default
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

If the devkit has a separate serial port (UART), you can send log output there and keep USB for RPi/bridge traffic (flashing still over USB):

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

### NeoPixel

**Bridge (`devkit_c`)**: No IMU/GPS; LED reflects bridge state:
  - **Armed** (track capture): **4×** blink rate (red).
  - **Sending request**: **Orange**.
  - **Success** (Pi got 5 GPS): **Solid green** for 1 s.
  - **Timeout** (Pi did not get 5 GPS): **Rainbow** for 1 s.

**Sensor node (`rev1_board`)**:
  - **Color** by IMU data age: **Green** &lt; 20 ms, **Blue** &lt; 10 s, **Red** otherwise.
  - **Pattern** by GPS: **Solid** = fix; **Double pulse** = connected no fix; **Slow blink** = no GPS.
  - When **GPS fix** and idle: **Green breathing** (25% → 100% brightness, 2 s cycle).

**Sensorode (`floating`)**:
- **Idle** (no capture in progress), **green only** (no IMU):
  - **GPS fix**: **Green breathing** (25% → 100% brightness, 2 s cycle).
  - **Double-pulse green**: GPS connected, no fix.
  - **Slow blink green**: No GPS data.
- **Acquiring** (after RequestGpsCapture):
  - **Solid orange** for **500 ms** (operator notice).
  - Then **rapid orange blink** for up to ~2 s while collecting 5 GPS samples.
  - Then back to idle (green) behaviour.

## Task management

Firmware uses the **Embassy** async executor. Each binary spawns a fixed set of tasks; the main coroutine then sleeps in a loop so the process does not exit.

### Sensor node (`rev1_board`)

| Task | Role |
|------|------|
| **IMU** | Reads IMU over SPI at fixed rate; pushes `ImuData` into `SENSOR_CHANNEL`. |
| **GPS** | After `init_gps`, reads NMEA from UART; pushes `GpsData` into `SENSOR_CHANNEL`. |
| **ESP-NOW sender** | Pulls from `SENSOR_CHANNEL` (capacity 8), sends IMU/GPS over ESP-NOW. Also receives TimeSync and TrackCaptureArmed/Ended; when armed, pauses sending so the floating node’s packets are not dropped. Sends periodic heartbeats. |
| **HMI** | Reads `HMI_STATE` (mutex), drives user LED (1 Hz) and NeoPixel (color/pattern by IMU age and GPS). |

Shared state: `SENSOR_CHANNEL` (Channel), `HMI_STATE` (mutex), `TRACK_CAPTURE_ARMED` (atomic), timebase (program start + sync offset).

### Bridge (`devkit_c`)

| Task | Role |
|------|------|
| **ESP-NOW bridge** | Polls USB RX for commands (TimeSync, Arm/End track, RequestFloatingGps, CaptureSuccess/Timeout); when armed, sends track-capture heartbeat (0xFE) to the Pi every 500 ms; receives ESP-NOW, assigns NodeIds by MAC discovery, forwards messages to USB (COBS). Auto-ends track capture if no USB from Pi for 2 s. |
| **HMI** | Drives LED/NeoPixel from `HMI_STATE` (armed blink, orange/green/rainbow feedback). |

Before the bridge task is spawned, the app waits up to 3 s for USB host presence (SOF interrupt), with a rainbow NeoPixel animation.

### Floating node (`sensorboard_floating`)

| Task | Role |
|------|------|
| **ESP-NOW** | Listens for RequestGpsCapture; sets `CAPTURE_MODE`, collects 5 GPS from `CAPTURE_CHANNEL`, replies with 5 GpsData messages. |
| **GPS** | Optional baud detection (floating feature) then NMEA parsing; pushes GGA-derived `GpsData` to `CAPTURE_CHANNEL` when in capture mode, and updates `HMI_STATE` (fix, last timestamp). |
| **HMI floating** | User LED 1 Hz; NeoPixel: green breathing when idle with fix, orange when acquiring. |
| **GPS stale watchdog** | Every 1 s, warns if no GGA in 1 s, errors if none in 5 s. |

Shared state: `CAPTURE_CHANNEL` (capacity 5), `CAPTURE_MODE` (atomic), `HMI_STATE`.

---

## Board hardware initialization

Initialization is driven by the **`BoardPeripherals`** trait: each board (Rev1, DevkitC, Floating) implements `take_*` methods that hand out peripherals once. The app calls them in a fixed order so that tasks receive owned resources.

### Order in `app_run` (sensor node / bridge)

1. **Timebase** — `set_program_start()` (used for elapsed time and sync).
2. **HMI** — `take_user_led()`, `take_neopixel()`; 3 s rainbow on NeoPixel (all binaries).
3. **Display SPI** — `take_disp_spi_device()` (held but not used by current tasks).
4. **IMU** — `take_imu_spi_device()` (sensor node only).
5. **GPS** — `take_gps2_uart()`; then `init_gps()` (configures NMEA/UBX, 10 Hz, etc.).
6. **WiFi** — `take_wifi()`; create `AirCommTransceiver`, then spawn ESP-NOW task(s).
7. **USB** — (Bridge only) `take_usb_serial_tx()`, `take_usb_serial_rx()`; wait for host, then spawn bridge task.
8. **HMI task** — spawned last with user_led and neopixel.

### Order in `app_run_floating`

1. Timebase, user_led, neopixel, 3 s rainbow.
2. `take_disp_spi_device()`, then GPS: if `take_gps2_uart_blocking()` is `Some`, run **baud detection** (9600, 38400, 115200, 460800) and then switch to async UART; else `init_gps(take_gps2_uart())`.
3. WiFi, ESP-NOW task, GPS task, HMI floating task, GPS stale watchdog.

### Rev1 hardware mapping

| Peripheral | Pin / config |
|------------|----------------|
| User LED | GPIO19 |
| NeoPixel (WS2812) | GPIO18, RMT, 80 MHz |
| GPS UART | UART1, default 460800, RX=GPIO23, TX=GPIO22 |
| SPI (IMU + display) | SPI2, 100 kHz, Mode 0; SCK=6, MOSI=7, MISO=0; IMU CS=GPIO1, display CS=GPIO10 |

### DevkitC (bridge)

Same idea; NeoPixel often on GPIO8. USB Serial/JTAG for host; no IMU/GPS in normal bridge use. UART1 at 9600 if present.

---

## Multi-node synchronization

### TimeSync (RPi session time)

The Raspberry Pi sends **TimeSync** over USB (session time in microseconds). The bridge receives it and **broadcasts** the same TimeSync on ESP-NOW. Data nodes (sender or transceiver) receive it and call `timebase::apply_sync(session_time_us)`. All nodes then use `timebase::synced_timestamp_us()` for message timestamps, so logs and forwarded data are aligned to the Pi’s session clock. If no TimeSync has been applied yet, `synced_timestamp_us()` falls back to local monotonic time.

### Node identity on the bridge

The bridge does **not** configure node IDs; each node sends a **Heartbeat** with its configured position (e.g. FrontLeft, Center, Floating). On first Heartbeat from a given MAC, the bridge assigns an **instance** number (0, 1, …) per position and builds a `mac_to_node_id` map. All messages forwarded to the Pi carry this (position, instance) so the host can distinguish multiple nodes of the same type.

### Track capture coordination

When the Pi arms track capture, it sends **ArmTrack**; the bridge sets `track_armed`, broadcasts **TrackCaptureArmed** on ESP-NOW, and starts sending **track-capture heartbeats** (0xFE) to the Pi every 500 ms. Non-floating nodes receive TrackCaptureArmed and set `TRACK_CAPTURE_ARMED`; they then **pause** IMU/GPS/heartbeat transmission until **TrackCaptureEnded**. That keeps the air clear for the floating node’s 5 GPS replies. When the Pi ends capture (or the bridge times out after 2 s without USB), the bridge broadcasts TrackCaptureEnded and nodes resume.
