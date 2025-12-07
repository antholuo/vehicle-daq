use core::fmt::Write;

use esp_hal::Async;
use esp_hal::uart::Uart;
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

extern crate alloc;
use alloc::string::String; // Need this import if using heap/alloc crate

const NMEA_0183_MAX_LENGTH: usize = 83;
const MAX_SENTENCE_LENGTH: usize = NMEA_0183_MAX_LENGTH + 2; // give ourselves buffer for newlines

// TODO: create initialization function which disables GSV/GSA/GLL messages
// Then: make buffer sizing constant since we only receive RMC/VTG/GGA
// --- NMEA Checksum function (from above) ---
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
    let mut incomplete_line_buffer = String::new(); // Requires the 'alloc' crate to be used

    loop {
        let mut buf = [0u8; 500];
        let n = match gps2_uart.read_async(&mut buf[..]).await {
            Ok(bytes) => bytes,
            Err(e) => {
                error!("UART Read Error: {:?}", e);
                continue;
            }
        };
        let chunk = core::str::from_utf8(&buf[..n]).unwrap_or("");

        incomplete_line_buffer.push_str(chunk);
        let lines: alloc::vec::Vec<&str> = incomplete_line_buffer.split('\n').collect();

        let (sentences_to_parse, next_buffer) = if incomplete_line_buffer.ends_with('\n') {
            // If the buffer ended with a newline, all lines are complete
            (lines, "")
        } else {
            // Otherwise, the last item in the vector is the incomplete line
            let last_line = lines[lines.len() - 1];
            let complete_lines = &lines[..lines.len() - 1];
            (complete_lines.to_vec(), last_line)
        };

        for line in sentences_to_parse {
            let sentence = line.trim(); // Clean up whitespace/newline residue
            //
            if sentence.len() < 5 || sentence.len() > NMEA_0183_MAX_LENGTH {
                warn!(
                    "Skipping non-compliant NMEA sentence (len={}): '{}'",
                    sentence.len(),
                    sentence
                );
            }

            if !sentence.is_empty() {
                match parser.parse_sentence(sentence) {
                    Ok(parsed_data) => {
                        match parsed_data {
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
                                // Velocity over ground in knots (N) and kilometers per hour (K)
                                let speed_knots = vtg.sog_knots.unwrap_or(0.0);
                                let speed_kph = vtg.sog_kph.unwrap_or(0.0);
                                let course = vtg.cog_true.unwrap_or(0.0);

                                info!(
                                    "GNVTG VELOCITY: Speed={:.2} knots ({:.2} km/h); Heading: {}",
                                    speed_knots, speed_kph, course
                                );
                            }
                            other_message => {
                                info!("parsed message of type: {:?}", other_message);
                            } // Ignore other sentence types for now
                        }
                    }
                    Err(e) => {
                        warn!("NMEA Parse Error: {} for sentence: '{}'", e, sentence);
                    }
                }
            }
        }

        incomplete_line_buffer = String::from(next_buffer);
        if incomplete_line_buffer.len() > 1024 {
            warn!("GPS Buffer overflow, clearing");
            incomplete_line_buffer.clear();
        }
    }
}
