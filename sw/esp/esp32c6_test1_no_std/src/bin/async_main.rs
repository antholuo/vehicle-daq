#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use log::info;

use embassy_sync::{blocking_mutex::raw::NoopRawMutex, signal::Signal};
use esp_hal::{
    gpio::lp_io::{LowPowerInput, LowPowerOutput},
    load_lp_code,
    lp_core::{LpCore, LpCoreWakeupSource},
    main,
    rmt::Rmt,
    time::Rate,
    uart::{lp_uart::LpUart, AtCmdConfig, Config, RxConfig, Uart, UartRx, UartTx},
    Async,
};
use esp_hal_smartled::{buffer_size_async, SmartLedsAdapterAsync};
use smart_leds::{
    brightness, gamma,
    hsv::{hsv2rgb, Hsv},
    SmartLedsWriteAsync, RGB8,
};

extern crate alloc;

const READ_BUF_SIZE: usize = 64;
const AT_CMD: u8 = 0x04;

#[embassy_executor::task]
async fn writer(mut tx: UartTx<'static, Async>, signal: &'static Signal<NoopRawMutex, usize>) {
    use core::fmt::Write;
    embedded_io_async::Write::write(
        &mut tx,
        b"Hello async serial. Enter something ended with EOT (CTRL-D).\r\n",
    )
    .await
    .unwrap();
    embedded_io_async::Write::flush(&mut tx).await.unwrap();
    loop {
        let bytes_read = signal.wait().await;
        signal.reset();
        write!(&mut tx, "\r\n-- received {} bytes --\r\n", bytes_read).unwrap();
        embedded_io_async::Write::flush(&mut tx).await.unwrap();
    }
}

#[embassy_executor::task]
async fn reader(mut rx: UartRx<'static, Async>, signal: &'static Signal<NoopRawMutex, usize>) {
    const MAX_BUFFER_SIZE: usize = 10 * READ_BUF_SIZE + 16;

    let mut rbuf: [u8; MAX_BUFFER_SIZE] = [0u8; MAX_BUFFER_SIZE];
    let mut offset = 0;
    loop {
        let r = embedded_io_async::Read::read(&mut rx, &mut rbuf[offset..]).await;
        match r {
            Ok(len) => {
                offset += len;
                esp_println::println!("Read: {len}, data: {:?}", &rbuf[..offset]);
                offset = 0;
                signal.signal(len);
            }
            Err(e) => esp_println::println!("RX Error: {:?}", e),
        }
    }
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // generator version: 0.2.2

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(size: 72 * 1024);

    esp_println::logger::init_logger_from_env();

    let timer0 = esp_hal::timer::systimer::SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(timer0.alarm0);

    info!("Embassy initialized!");

    let timg_0 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0);
    let timg_1 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG1);

    // wifi init
    // let _init = esp_wifi::init(
    //     timg_0.timer0,
    //     esp_hal::rng::Rng::new(peripherals.RNG),
    //     peripherals.RADIO_CLK,
    // )
    // .unwrap();

    // Turn on that stupid RGB LED
    // esp_hal_embassy::init(timg_1.timer0);

    // Configure RMT (Remote Control Transceiver) peripheral globally
    // <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/peripherals/rmt.html>
    let rmt: Rmt<'_, esp_hal::Async> = {
        let frequency: Rate = { Rate::from_mhz(80) };
        Rmt::new(peripherals.RMT, frequency)
    }
    .expect("Failed to initialize RMT")
    .into_async();

    // We use one of the RMT channels to instantiate a `SmartLedsAdapterAsync` which can
    // be used directly with all `smart_led` implementations
    let rmt_channel = rmt.channel0;
    let rmt_buffer = [0_u32; buffer_size_async(1)];

    let mut led: SmartLedsAdapterAsync<_, 25> =
        { SmartLedsAdapterAsync::new(rmt_channel, peripherals.GPIO8, rmt_buffer) };
    let mut color = Hsv {
        hue: 0,
        sat: 255,
        val: 255,
    };

    let mut hue_val: u8 = 0;
    let mut sat_val: u8 = 128;
    let mut data = RGB8 {
        r: 0,
        g: 128,
        b: 255,
    };
    let level = 10;

    // UART test setup
    //
    let lp_pin = LowPowerOutput::new(peripherals.GPIO1);

    // let (lp_uart_rx_pin, lp_uart_tx_pin) = (peripherals.GPIO4, peripherals.GPIO5);
    let lp_uart_config = Config::default()
        .with_rx(RxConfig::default().with_fifo_full_threshold(READ_BUF_SIZE as u16));

    let mut lp_uart = LpUart::new(
        peripherals.LP_UART,
        lp_uart_config,
        LowPowerOutput::new(peripherals.GPIO5),
        LowPowerInput::new(peripherals.GPIO4),
    );

    let mut lp_core = LpCore::new(peripherals.LP_CORE);
    lp_core.stop();
    info!("lp_core stopped");

    let lp_core_code = load_lp_code!(
        "../esp-hal/esp-lp-hal/target/riscv32imac-unknown-none-elf/debug/examples/blinky"
    );

    lp_core_code.run(&mut lp_core, LpCoreWakeupSource::HpCpu, lp_pin, lp_uart);
    info!("lp_core run");

    // let mut lp_uart = Uart::new(peripherals.LP_UART, lp_uart_config)
    //     .unwrap()
    //     .with_tx(lp_uart_tx_pin)
    //     .with_rx(lp_uart_rx_pin)
    // .into_async();
    // TODO: Spawn some tasks
    let _ = spawner;

    loop {
        // info!("Hello world!");
        color.hue = hue_val;
        // color.sat = sat_val;

        data = hsv2rgb(color);

        // data.r = data.r.wrapping_add(1);
        // data.b = data.b.wrapping_add(1);
        // data.g = data.g.wrapping_add(1);

        led.write(brightness(gamma([data].into_iter()), level))
            .await
            .unwrap();

        hue_val = hue_val.wrapping_add(1);

        if hue_val >= 255 {
            sat_val = sat_val.wrapping_add(3);
        } else if hue_val <= 0 {
        }
        let data = (0x5000_2000) as *mut u32;
        info!("Current {:x}           \u{000d}", unsafe {
            data.read_volatile()
        });

        Timer::after(Duration::from_secs(1)).await;
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/v0.23.1/examples/src/bin
}
