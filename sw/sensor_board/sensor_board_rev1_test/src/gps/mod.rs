#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use esp_hal::Async;
use esp_hal::uart::Uart;

extern crate alloc;

use alloc::string::String; // Need this import if using heap/alloc crate

pub async fn start_gps(mut gps2_uart: Uart<'static, Async>) {
    let mut parser = nmea_parser::NmeaParser::new();

    // Use a String to buffer incomplete lines across loop iterations
    let mut incomplete_line_buffer = String::new(); // Requires the 'alloc' crate to be used

    loop {
        let mut buf = [0u8; 500];
        let n = gps2_uart.read_async(&mut buf[..]).await.unwrap();
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

            if !sentence.is_empty() {
                match parser.parse_sentence(sentence) {
                    Ok(parsed_data) => {
                        match parsed_data {
                            nmea_parser::ParsedMessage::Gga(gga) => {
                                if let Some(time) = gga.timestamp {
                                    info!("GPS time is: {}", time);
                                }
                                info!(
                                    "GPGGA FIX: Lat={}, Lon={}, HDOP={}, SATS={}",
                                    gga.latitude.unwrap_or(0.0),
                                    gga.longitude.unwrap_or(0.0),
                                    gga.hdop.unwrap_or(0.0),
                                    gga.satellite_count.unwrap_or(0),
                                );
                            }
                            nmea_parser::ParsedMessage::Rmc(rmc) => {
                                debug!("GPRMC Status: {:?}", rmc);
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
                            _ => {} // Ignore other sentence types for now
                        }
                    }
                    Err(e) => {
                        warn!("NMEA Parse Error: {} for sentence: '{}'", e, sentence);
                    }
                }
            }
        }

        incomplete_line_buffer = String::from(next_buffer);
    }
}
