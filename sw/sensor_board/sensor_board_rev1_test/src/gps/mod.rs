#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use embassy_time::{Duration, Timer};
use esp_hal::Async;
use esp_hal::uart::Uart;

pub async fn start_gps(mut gps2_uart: Uart<'static, Async>) {
    // let mut parser = nmea_parser::NmeaParser::new();

    loop {
        let mut buf = [0u8; 800];
        let n = gps2_uart.read_async(&mut buf[..]).await.unwrap();
        let line = core::str::from_utf8(&buf[..n]).unwrap_or("");

        // if let Ok(msg) = parser.parse(line) {
        //     if let nmea_parser::ParsedMessage::Rmc(rmc) = msg {
        //         info!("Received RMC message: {}", msg);
        //     }
        // }
        info!("GPS2 received {} bytes of information: \n{:?}\n", n, line);
    }
}
