use core::fmt::Write;
use esp_hal::Async;
use esp_hal::uart::Uart;
use heapless::{String, Vec};
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use crate::types::{GpsData, GpsTime};

const NMEA_0183_MAX_LENGTH: usize = 83;
const MAX_SENTENCE_LENGTH: usize = NMEA_0183_MAX_LENGTH + 2; // give ourselves buffer for newlines
const NUM_NMEA_SENTENCES: usize = 3; // RMC, VTG, GGA
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

async fn send_ubx_packet(
    uart: &mut Uart<'static, Async>,
    class: u8,
    id: u8,
    payload: &[u8],
    name: &str,
) {
    let len = payload.len() as u16;
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

    // TODO: Configure GPS for 10hz updates & 115200 baud
    // ONLY DO THIS WHEN THE NEW GPS COMES IN, since we will have to set baud rate at configuration
    // time, meaning we must set the baud for all the gps's, then change the code, then use the GPS

    // This sets baud to 460800
    // default for M8 is 9600, default for F10 is 38400. Reset to default, set high baud, then set
    // high baud. Theoretically we can reconfigure the uart on the fly by dropping and re-creating
    // but that seems really difficult and I don't want to do that
    // send_nmea_command(&mut gps2_uart, "PUBX,41,1,3,3,460800,0", "SET BAUD 460800").await;

    // This *should* set 10hz updates but I need to implement send_ubx_packet
    const UBX_CFG_VALSET_10HZ: [u8; 17] = [
        0xB5, 0x62, // Sync
        0x06, 0x8A, // Class: CFG, ID: VALSET
        0x09, 0x00, // Length: 9 bytes
        0x00, // Version 0
        0x07, // Layers (1 = RAM only, use 0x07 for RAM+Flash+BBR)
        0x00, 0x00, // Reserved
        0x01, 0x00, 0x21, 0x30, // Key ID: CFG-RATE-MEAS (0x30210001) - Little Endian
        0x64, // 0x64 == 100 ms
        0x60, 0x34, // Checksum A/B
    ];
    send_ubx_packet(
        &mut gps2_uart,
        0x06, // class: CFG
        0x8A, // id: valset
        &UBX_CFG_VALSET_10HZ,
        "CFG-RATE-MEAS 10Hz", // update this if 25
    )
    .await;

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
                            info!(
                                "GPGGA FIX: Lat={}, Lon={}, HDOP={}, SATS={}",
                                gga.latitude.unwrap_or(0.0),
                                gga.longitude.unwrap_or(0.0),
                                gga.hdop.unwrap_or(0.0),
                                gga.satellite_count.unwrap_or(0),
                            );

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
                        }
                        nmea_parser::ParsedMessage::Rmc(rmc) => {
                            if let Some(time) = rmc.timestamp {
                                info!("GPS rmc time is: {}", time);

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
                            info!(
                                "GNVTG VELOCITY: Speed={:.2} knots ({:.2} km/h); Heading: {}",
                                speed_knots, speed_kph, course
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
