#![allow(dead_code)]
use embedded_hal::spi::SpiBus;

#[derive(Debug, Clone, Copy)]
pub struct RawSensorData {
    pub x: i16,
    pub y: i16,
    pub z: i16,
    pub valid: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct RawImuData {
    pub xl: RawSensorData,
    pub gy: RawSensorData,
    pub ts: Option<u32>,
    pub temp: Option<u16>,
}

#[rustfmt::skip]
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum GyroFs {
    DPS250  = 0b00,
    DPS500  = 0b01,
    DPS1000 = 0b10,
    DPS2000 = 0b11,
}

#[rustfmt::skip]
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum AccelFs {
    G2  = 0b00, 
    G4  = 0b10, 
    G8  = 0b11, 
    G16 = 0b01, 
}

#[rustfmt::skip]
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Odr {
    Hz12_5 = 0b0001,
    Hz26   = 0b0010,
    Hz52   = 0b0011,
    Hz104  = 0b0100,
    Hz208  = 0b0101,
    Hz417  = 0b0110,
    Hz833  = 0b0111,
    Hz1667 = 0b1000,
    Hz3333 = 0b1001,
    Hz6667 = 0b1010,
}

#[derive(Debug, Clone, Copy)]
pub struct OutputConfig {
    pub xl_odr: Odr,
    pub xl_fsr: AccelFs,
    pub xl_lpf2_en: bool,
    pub gy_odr: Odr,
    pub gy_fsr: GyroFs,
    pub gy_lpf1_en: bool,
    pub block_data_en: bool,
    pub timestamp_en: bool,
}
