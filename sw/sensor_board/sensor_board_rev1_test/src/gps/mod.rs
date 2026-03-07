use core::fmt::Write;
use esp_hal::Async;
#[cfg(feature = "floating")]
use esp_hal::uart::Config;
use esp_hal::uart::Uart;
use heapless::String;
#[cfg(feature = "set_gps_10hz")]
use heapless::Vec;
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use crate::types::{GpsData, GpsTime};
use embassy_time::Instant;

const NMEA_0183_MAX_LENGTH: usize = 83;
const LAST_RAW_GGA_CAP: usize = 84;

/// Last successfully parsed GGA sentence (raw NMEA), for diagnostic logging when capture gets 0 samples.
/// Stored as bytes to avoid Rust 2024 static_mut_refs; copied out byte-wise in with_last_raw_gga.
static mut LAST_RAW_GGA_BUF: [u8; LAST_RAW_GGA_CAP] = [0; LAST_RAW_GGA_CAP];
static mut LAST_RAW_GGA_LEN: u8 = 0;

/// Store the raw GGA sentence (call when a GGA is parsed). Truncates to fit.
pub fn set_last_raw_gga(sentence: &str) {
    let end = sentence
        .char_indices()
        .nth(NMEA_0183_MAX_LENGTH)
        .map(|(i, _)| i)
        .unwrap_or(sentence.len());
    let trunc = &sentence[..end.min(sentence.len())];
    let bytes = trunc.as_bytes();
    let len = bytes.len().min(LAST_RAW_GGA_CAP) as u8;
    critical_section::with(|_| unsafe {
        for (i, &b) in bytes.iter().take(LAST_RAW_GGA_CAP).enumerate() {
            LAST_RAW_GGA_BUF[i] = b;
        }
        LAST_RAW_GGA_LEN = len;
    });
}

/// Run a closure with the last raw GGA sentence (if any), for logging.
pub fn with_last_raw_gga<F, R>(f: F) -> R
where
    F: FnOnce(Option<&str>) -> R,
{
    let mut local_buf = [0u8; LAST_RAW_GGA_CAP];
    let len = critical_section::with(|_| unsafe {
        let n = LAST_RAW_GGA_LEN as usize;
        for i in 0..n {
            local_buf[i] = LAST_RAW_GGA_BUF[i];
        }
        n
    });
    let opt_str = if len > 0 {
        core::str::from_utf8(&local_buf[..len]).ok()
    } else {
        None
    };
    f(opt_str)
}
const MAX_SENTENCE_LENGTH: usize = NMEA_0183_MAX_LENGTH + 2; // give ourselves buffer for newlines
const NUM_NMEA_SENTENCES: usize = 5; // RMC, VTG, GGA
const CHUNK_BUFF_SIZE: usize = MAX_SENTENCE_LENGTH * (NUM_NMEA_SENTENCES + 1); // buffer
const UNPROCESSED_BUFF_SIZE: usize = CHUNK_BUFF_SIZE * 2; // hold 2 chunks

fn calculate_nmea_checksum(data: &[u8]) -> u8 {
    let mut checksum: u8 = 0;
    for byte in data {
        checksum ^= *byte;
    }
    checksum
}
// ------------------------------------------

async fn send_nmea_command(uart: &mut Uart<'static, Async>, payload: &str, name: &str) {
    let checksum = calculate_nmea_checksum(payload.as_bytes());

    let mut full_command: heapless::String<128> = heapless::String::new();

    let result = write!(&mut full_command, "${}*{:02X}\r\n", payload, checksum);

    if result.is_err() {
        error!("Heapless string creation failed during PUBX message creation");
        return;
    }

    let command_bytes = full_command.as_bytes();

    match uart.write_async(command_bytes).await {
        Ok(_) => info!(
            "GPS Config: Sent command to disable {} ({})",
            name,
            full_command.trim()
        ),
        Err(e) => error!("GPS Config: Failed to send command for {}: {:?}", name, e),
    }
}

#[cfg(feature = "set_gps_10hz")]
async fn send_ubx_packet(
    uart: &mut Uart<'static, Async>,
    class: u8,
    id: u8,
    payload: &[u8],
    name: &str,
) {
    let len = payload.len() as u16;
    info!("payload len for msg UBX rate resolved to {}", len);
    let len_bytes = len.to_le_bytes(); // UBX is Little Endian

    // 1. Build the part of the packet used for checksum calculation
    // Checksum is calculated over: Class, ID, Length, and Payload
    let mut checksum_data = Vec::<u8, 128>::new(); // Use a fixed-capacity stack vector
    checksum_data.push(class).ok();
    checksum_data.push(id).ok();
    checksum_data.extend_from_slice(&len_bytes).ok();
    checksum_data.extend_from_slice(payload).ok();

    // 2. Calculate Fletcher-8 Checksum
    let mut ck_a: u8 = 0;
    let mut ck_b: u8 = 0;
    for &byte in checksum_data.iter() {
        ck_a = ck_a.wrapping_add(byte);
        ck_b = ck_b.wrapping_add(ck_a);
    }

    // 3. Assemble the full frame
    // [Sync1, Sync2, ChecksumData..., CK_A, CK_B]
    let mut full_packet = Vec::<u8, 134>::new();
    full_packet.push(0xB5).ok(); // Sync 1
    full_packet.push(0x62).ok(); // Sync 2
    full_packet.extend_from_slice(&checksum_data).ok();
    full_packet.push(ck_a).ok();
    full_packet.push(ck_b).ok();

    info!("Sending UBX {} command...", name);

    // 4. Send over UART
    match uart.write_async(&full_packet).await {
        Ok(_) => info!("UBX {} sent.", name),
        Err(e) => error!("Failed to send UBX {}: {:?}", name, e),
    }
}

/// Baud rates to try when auto-detecting GPS (before sending any configuration).
#[cfg(feature = "floating")]
const DETECT_BAUD_RATES: &[u32] = &[9600, 38400, 115200, 460800];
/// Time to listen at each baud rate for a GGA sentence (GPS may be slow after power-up).
#[cfg(feature = "floating")]
const DETECT_READ_MS: u64 = 1200;
#[cfg(feature = "floating")]
const DETECT_CHUNK_MS: u64 = 50;
/// Delay before first baud try so the GPS module has time to power up and start sending NMEA.
#[cfg(feature = "floating")]
const DETECT_STARTUP_DELAY_MS: u64 = 2500;

/// Try common baud rates, read NMEA without sending config, and return (uart at detected baud, detected baud).
/// Logs "Trying baud X...", "Could not parse..." or "Parsed GG* ... NUM SATS: N".
#[cfg(feature = "floating")]
pub async fn detect_gps_baud_and_init(
    mut uart: Uart<'static, esp_hal::Blocking>,
) -> Uart<'static, Async> {
    use embassy_time::Timer;

    info!(
        "GPS baud detection: waiting {} ms for module to start sending...",
        DETECT_STARTUP_DELAY_MS
    );
    Timer::after_millis(DETECT_STARTUP_DELAY_MS).await;

    let mut parser = nmea_parser::NmeaParser::new();
    let mut line_buf: heapless::String<{ NMEA_0183_MAX_LENGTH + 2 }> = heapless::String::new();
    let mut read_buf = [0u8; 128];
    let mut detected_baud: Option<u32> = None;

    for &baud in DETECT_BAUD_RATES {
        info!("Trying baud {}...", baud);
        let config = Config::default().with_baudrate(baud);
        if uart.apply_config(&config).is_err() {
            warn!("Could not set baud {} on UART", baud);
            continue;
        }
        // Drain any stale bytes from previous baud
        loop {
            match uart.read_buffered(&mut read_buf) {
                Ok(0) => break,
                Ok(_) => {}
                Err(_) => break,
            }
        }
        Timer::after_millis(DETECT_CHUNK_MS).await;

        let deadline = embassy_time::Instant::now() + embassy_time::Duration::from_millis(DETECT_READ_MS);
        let mut got_gga = false;
        let mut num_sats: Option<u8> = None;

        while embassy_time::Instant::now() < deadline {
            match uart.read_buffered(&mut read_buf) {
                Ok(n) if n > 0 => {
                    for &b in &read_buf[..n] {
                        if b == b'\n' || b == b'\r' {
                            if !line_buf.is_empty() {
                                let sentence = line_buf.as_str().trim();
                                if sentence.len() >= 7 && (sentence.starts_with("$GPGGA") || sentence.starts_with("$GNGGA")) {
                                    match parser.parse_sentence(sentence) {
                                        Ok(nmea_parser::ParsedMessage::Gga(gga)) => {
                                            let sats = gga.satellite_count.unwrap_or(0);
                                            num_sats = Some(sats);
                                            got_gga = true;
                                            info!(
                                                "Parsed {} ... NUM SATS: {}",
                                                &sentence[..sentence.len().min(20)],
                                                sats
                                            );
                                            break;
                                        }
                                        _ => {}
                                    }
                                }
                                line_buf.clear();
                            }
                        } else if line_buf.push(b as char).is_err() {
                            line_buf.clear();
                        }
                    }
                    if got_gga {
                        break;
                    }
                }
                _ => {}
            }
            Timer::after_millis(10).await;
        }

        if got_gga {
            detected_baud = Some(baud);
            info!("GPS baud detected: {} (NUM SATS: {:?})", baud, num_sats);
            break;
        }
        info!("Could not parse GGA at baud {} (no valid $GP* / $GN* GGA in {} ms)", baud, DETECT_READ_MS);
    }

    let baud = detected_baud.unwrap_or(9600);
    if detected_baud.is_none() {
        warn!("Using default baud {} (no GGA seen at any tried rate)", baud);
    }

    let uart_async = uart.into_async();
    init_gps(uart_async).await
}

pub async fn init_gps(mut gps2_uart: Uart<'static, Async>) -> esp_hal::uart::Uart<'static, Async> {
    info!("Starting GPS configuration using $PUBX,40 commands...");

    // The payloads (excluding the leading '$' and trailing '*cs')
    const GSA_PAYLOAD: &str = "PUBX,40,GSA,0,0,0,0,0,0";
    const GSV_PAYLOAD: &str = "PUBX,40,GSV,0,0,0,0,0,0";
    const GLL_PAYLOAD: &str = "PUBX,40,GLL,0,0,0,0,0,0";

    // Send commands to disable the unwanted messages (GSA, GSV, GLL)
    send_nmea_command(&mut gps2_uart, GSA_PAYLOAD, "GSA").await;
    send_nmea_command(&mut gps2_uart, GSV_PAYLOAD, "GSV").await;
    send_nmea_command(&mut gps2_uart, GLL_PAYLOAD, "GLL").await;

    #[cfg(feature = "set_gps_high_baud")]
    {
        info!("Setting GPS Baud to 460800 (Persistent)...");

        // USE THIS FOR A NEO F10
        // Key ID for CFG-UART1-BAUDRATE: 0x40520001
        // Value for 460800: 0x00070800 (Little Endian: 0x00, 0x08, 0x07, 0x00)
        const UBX_CFG_VALSET_BAUD_460800: [u8; 12] = [
            0x00, // Version 0
            0x07, // Layer: 7 = RAM + Flash + BBR (Persistent)
            0x00, 0x00, // Reserved
            0x01, 0x00, 0x52, 0x40, // Key ID: CFG-UART1-BAUDRATE
            0x00, 0x08, 0x07, 0x00, // Value: 460800
        ];
        send_ubx_packet(
            &mut gps2_uart,
            0x06, // class: CFG
            0x8A, // id: VALSET
            &UBX_CFG_VALSET_BAUD_460800,
            "CFG-UART1-BAUDRATE",
        )
        .await;

        // USE THIS FOR A NEO M8
        // UBX-CFG-PRT for NEO-M8 to set UART1 = 460800 baud
        // const UBX_CFG_PRT_UART1_460800: [u8; 20] = [
        //     0x01, 0x00, // portID=1 (UART1), reserved
        //     0x00, 0x00, // txReady
        //     0xD0, 0x08, 0x00, 0x00, // mode = 0x08D0 (8N1, no parity)
        //     0x00, 0x08, 0x07, 0x00, // baudrate = 460800 (LE)
        //     0x03, 0x00, // inProtoMask = UBX + NMEA
        //     0x03, 0x00, // outProtoMask = UBX + NMEA
        //     0x00, 0x00, // flags
        //     0x00, 0x00, // reserved
        // ];
        // send_ubx_packet(
        //     &mut gps2_uart,
        //     0x06, // class: CFG
        //     0x00, // id: none
        //     &UBX_CFG_PRT_UART1_460800,
        //     "CFG-UART1-BAUDRATE",
        // )
        // .await;
        // // UBX-CFG-CFG: Save current configuration
        // const UBX_CFG_SAVE: [u8; 12] = [
        //     0x00, 0x00, 0x00, 0x00, // clearMask (do nothing)
        //     0xFF, 0xFF, 0x00, 0x00, // saveMask (save all sections)
        //     0x00, 0x00, 0x00, 0x00, // loadMask (do nothing)
        // ];
        // send_ubx_packet(
        //     &mut gps2_uart,
        //     0x06, // class: CFG
        //     0x09, // id: CFG
        //     &UBX_CFG_SAVE,
        //     "CFG-SAVE",
        // )
        // .await;

        info!("ESP32 UART switched to 460800 baud.");
    }

    #[cfg(feature = "set_gps_10hz")]
    {
        // This *should* set 10hz updates but I need to implement send_ubx_packet
        const UBX_CFG_VALSET_10HZ_PAYLOAD: [u8; 10] = [
            0x00, // Version 0
            0x07, // Layer: 7 = RAM + Flash + BBR (Persistent)
            0x00, 0x00, // Reserved
            0x01, 0x00, 0x21, 0x30, // Key ID: CFG-RATE-MEAS
            0x64, 0x00, // Value: 100ms (Little Endian U2)
        ];
        // UNCOMMENT BELOW IF CONFIGURING A NEW GPS (saved to ROM)
        send_ubx_packet(
            &mut gps2_uart,
            0x06, // class: CFG
            0x8A, // id: valset
            &UBX_CFG_VALSET_10HZ_PAYLOAD,
            "CFG-RATE-MEAS 10Hz",
        )
        .await;
    }

    gps2_uart
}

/// Start GPS task with a callback for each data sample
///
/// The callback `on_data` is invoked whenever a valid GPS fix is available.
/// This allows the caller to decide what to do with the data (log, send, etc.)
/// without the driver needing to know about channels or networking.
///
/// # Arguments
/// * `gps2_uart` - The UART peripheral for GPS communication
/// * `on_data` - Callback invoked with each GPS fix
pub async fn start_gps<F>(mut gps2_uart: Uart<'static, Async>, on_data: F)
where
    F: Fn(GpsData),
{
    use nmea_parser::chrono::{Datelike, Timelike};

    let mut parser = nmea_parser::NmeaParser::new();

    // Use a String to buffer incomplete lines across loop iterations
    let mut incomplete_line_buffer: heapless::String<UNPROCESSED_BUFF_SIZE> =
        heapless::String::new();

    // Track latest values from different NMEA sentences
    let mut last_lat: Option<f64> = None;
    let mut last_lon: Option<f64> = None;
    let mut last_alt: Option<f32> = None;
    let mut last_speed_kts: Option<f32> = None;
    let mut last_heading: Option<u16> = None;
    let mut last_time: Option<GpsTime> = None;

    loop {
        let mut buf = [0u8; CHUNK_BUFF_SIZE];
        let n = match gps2_uart.read_async(&mut buf[..]).await {
            Ok(bytes) => bytes,
            Err(e) => {
                error!("UART Read Error: {:?}", e);
                continue;
            }
        };

        if n > 0 {
            let mut hmi = crate::hmi::state::HMI_STATE.0.lock().await;
            hmi.last_gps_rx_timestamp = Some(Instant::now());
        }

        let chunk = core::str::from_utf8(&buf[..n]).unwrap_or("");

        if incomplete_line_buffer.push_str(chunk).is_err() {
            warn!("Did not succesfully push chunk to incomplete_line_buffer..continuing anyways");
        }

        let mut start_idx = 0;
        let mut _sentences_processed = 0;

        while let Some(end_idx) = incomplete_line_buffer.as_str()[start_idx..].find('\n') {
            let full_end_idx = start_idx + end_idx;
            let line_slice = &incomplete_line_buffer.as_str()[start_idx..full_end_idx];

            let sentence = line_slice.trim();
            if sentence.len() < 5 || sentence.len() > NMEA_0183_MAX_LENGTH {
                warn! {
                    "skipping non-compliant NMEA sentence (len={}): '{}'",
                    sentence.len(),
                    sentence
                }
            } else if !sentence.is_empty() {
                match parser.parse_sentence(sentence) {
                    Ok(parsed_data) => match parsed_data {
                        nmea_parser::ParsedMessage::Gga(gga) => {
                            set_last_raw_gga(sentence);
                            let elapsed_s = crate::timebase::elapsed_seconds();
                            let has_fix = gga.latitude.is_some() && gga.longitude.is_some();
                            let sat_count = gga.satellite_count.unwrap_or(0);

                            if has_fix {
                                info!(
                                    "t={:.3}s GPGGA ✓ VALID FIX: Lat={}, Lon={}, HDOP={}, SATS={}",
                                    elapsed_s,
                                    gga.latitude.unwrap_or(0.0),
                                    gga.longitude.unwrap_or(0.0),
                                    gga.hdop.unwrap_or(0.0),
                                    sat_count,
                                );
                            } else {
                                warn!(
                                    "t={:.3}s GPGGA ✗ NO FIX: SATS={}, HDOP={}",
                                    elapsed_s,
                                    sat_count,
                                    gga.hdop.unwrap_or(99.99),
                                );
                            }

                            // Update cached values from GGA
                            if let Some(lat) = gga.latitude {
                                last_lat = Some(lat);
                            }
                            if let Some(lon) = gga.longitude {
                                last_lon = Some(lon);
                            }
                            if let Some(alt) = gga.altitude {
                                last_alt = Some(alt as f32);
                            }

                            // Try to construct and send GPS data if we have position
                            try_send_gps_data(
                                &on_data,
                                last_lat,
                                last_lon,
                                last_alt,
                                last_speed_kts,
                                last_heading,
                                last_time,
                            );

                            // Update HMI state: GPS timestamp and fix flag
                            let mut hmi = crate::hmi::state::HMI_STATE.0.lock().await;
                            hmi.last_gps_timestamp = Some(Instant::now());
                            // Only set fix=true if we have valid position data
                            hmi.gps_fix = gga.latitude.is_some() && gga.longitude.is_some();
                        }
                        nmea_parser::ParsedMessage::Rmc(rmc) => {
                            if let Some(time) = rmc.timestamp {
                                let elapsed_s = crate::timebase::elapsed_seconds();
                                info!("t={:.3}s GPS rmc time is: {}", elapsed_s, time);

                                // Update cached time from RMC
                                last_time = Some(GpsTime {
                                    year: time.year() as u16,
                                    month: time.month() as u8,
                                    day: time.day() as u8,
                                    hours: time.hour() as u8,
                                    minutes: time.minute() as u8,
                                    seconds: time.second() as u8,
                                    millis: (time.nanosecond() / 1_000_000) as u16,
                                });
                            }

                            // RMC also contains position data
                            if let Some(lat) = rmc.latitude {
                                last_lat = Some(lat);
                            }
                            if let Some(lon) = rmc.longitude {
                                last_lon = Some(lon);
                            }
                            if let Some(sog) = rmc.sog_knots {
                                last_speed_kts = Some(sog as f32);
                            }
                            if let Some(bearing) = rmc.bearing {
                                last_heading = Some(bearing as u16);
                            }
                        }
                        nmea_parser::ParsedMessage::Vtg(vtg) => {
                            let speed_knots = vtg.sog_knots.unwrap_or(0.0);
                            let speed_kph = vtg.sog_kph.unwrap_or(0.0);
                            let course = vtg.cog_true.unwrap_or(0.0);
                            let elapsed_s = crate::timebase::elapsed_seconds();
                            info!(
                                "t={:.3}s GNVTG VELOCITY: Speed={:.2} knots ({:.2} km/h); Heading: {}",
                                elapsed_s, speed_knots, speed_kph, course
                            );

                            // Update cached values from VTG
                            if vtg.sog_knots.is_some() {
                                last_speed_kts = Some(speed_knots as f32);
                            }
                            if vtg.cog_true.is_some() {
                                last_heading = Some(course as u16);
                            }

                            // Try to construct and send GPS data after VTG
                            try_send_gps_data(
                                &on_data,
                                last_lat,
                                last_lon,
                                last_alt,
                                last_speed_kts,
                                last_heading,
                                last_time,
                            );
                        }
                        other_message => {
                            debug!("Ignoring other NMEA message type: {:?}", other_message);
                        }
                    },
                    Err(e) => {
                        warn!("NMEA Parse Error: {} for sentence: '{}'", e, sentence);
                    }
                }
            }

            start_idx = full_end_idx + 1;
            _sentences_processed += 1;
        }

        if start_idx < incomplete_line_buffer.len() {
            let remainder = &incomplete_line_buffer.as_str()[start_idx..];

            let mut next_buff: String<UNPROCESSED_BUFF_SIZE> = String::new();
            if next_buff.push_str(remainder).is_err() {
                error!("Heapless next buff push failed during remainder assignment");
            }

            incomplete_line_buffer.clear();
            incomplete_line_buffer = next_buff;
        } else {
            incomplete_line_buffer.clear(); // everything processed
        }
    }
}

/// Helper to construct and send GPS data if we have at least position
fn try_send_gps_data<F>(
    on_data: &F,
    lat: Option<f64>,
    lon: Option<f64>,
    alt: Option<f32>,
    speed_kts: Option<f32>,
    heading: Option<u16>,
    time: Option<GpsTime>,
) where
    F: Fn(GpsData),
{
    // Only send if we have at least lat/lon
    if let (Some(lat), Some(lon)) = (lat, lon) {
        let gps_data = GpsData {
            lat,
            lon,
            alt: alt.unwrap_or(0.0),
            speed_kts: speed_kts.unwrap_or(0.0),
            heading: heading.unwrap_or(0),
            utc_time: time.unwrap_or(GpsTime {
                year: 0,
                month: 0,
                day: 0,
                hours: 0,
                minutes: 0,
                seconds: 0,
                millis: 0,
            }),
        };
        on_data(gps_data);
    }
}
