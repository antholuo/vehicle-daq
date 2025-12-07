use core::fmt::Write;
use esp_hal::Async;
use esp_hal::uart::Uart;
use heapless::String;
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

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

    gps2_uart
}

pub async fn start_gps(mut gps2_uart: Uart<'static, Async>) {
    let mut parser = nmea_parser::NmeaParser::new();

    // Use a String to buffer incomplete lines across loop iterations
    let mut incomplete_line_buffer: heapless::String<UNPROCESSED_BUFF_SIZE> =
        heapless::String::new();

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

        incomplete_line_buffer.push_str(chunk);

        let mut start_idx = 0;
        let mut sentences_processed = 0;

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
                        }
                        nmea_parser::ParsedMessage::Rmc(rmc) => {
                            if let Some(time) = rmc.timestamp {
                                info!("GPS rmc time is: {}", time);
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
            sentences_processed += 1;
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
