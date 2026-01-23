/*
* Created date: 12/20/25
* File Description: common definitions used across files
*/

use thiserror_no_std::Error;
use modular_bitfield::prelude::*;

#[derive(Error, Debug)]
pub enum Asm330Error {
    #[error("Data not ready for read")]
    DataNotReadyError(),
    #[error("Spi error occured")]
    SpiError(),
}

#[derive(Clone, Copy, Specifier, Default)]
#[bits = 3]
pub enum FifoMode {
    #[default]
    Bypass = 0b000,
    Fifo = 0b001,
    ContToFifo = 0b011,
    BypassToCont = 0b100,
    Cont = 0b110,
    BypassToFifo = 0b111,
}

#[derive(Clone, Copy, Specifier, Default)]
#[bits = 2]
pub enum TempBatchOdr {
    #[default]
    NotBatched = 0b00,
    Batched1_6 = 0b01,
    Batched12_5 = 0b10,
    Batched52 = 0b11,
}

#[derive(Clone, Copy, Specifier, Default)]
#[bits = 2]
pub enum TimestampDecimation {
    #[default]
    NotBatched = 0b00,
    BDRFlat = 0b01,
    BDRBy8 = 0b10,
    BDRBy32 = 0b11,
}

#[derive(Clone, Copy, Specifier, Default, PartialOrd, PartialEq)]
#[bits = 4]
pub enum Odr {
    #[default]
    PowerDown = 0b0000,
    Hz12_5 = 0b0001,
    Hz26 = 0b0010,
    Hz52 = 0b0011,
    Hz104 = 0b0100,
    Hz208 = 0b0101,
    Hz417 = 0b0110,
    Hz833 = 0b0111,
    Hz1667 = 0b1000,
    Hz3333 = 0b1001,
    Hz6667 = 0b1010,
    Hz6_5 = 0b1011,
}

#[derive(Clone, Copy, Specifier, Default)]
#[bits = 2]
pub enum AccelScale {
    #[default]
    G2 = 0b00,
    G16 = 0b01,
    G4 = 0b10,
    G8 = 0b11,
}

#[derive(Clone, Copy, Specifier, Default)]
#[bits = 4]
pub enum GyroScale {
    #[default]
    Dps125 = 0b0010,
    Dps250 = 0b0000,
    Dps500 = 0b0100,
    Dps1000 = 0b1000,
    Dps2000 = 0b1100,
    Dps4000 = 0b0001,
}

#[derive(Clone, Copy, Specifier, Default)]
#[bits = 2]
pub enum Rounding {
    #[default]
    None = 0b00,
    AccelOnly = 0b01,
    GyroOnly = 0b10,
    Both = 0b11,
}

#[derive(Clone, Copy, Specifier, Default)]
#[bits = 2]
pub enum AccelST {
    #[default]
    Normal = 0b00,
    Positive = 0b01,
    Negative = 0b10,
}

#[derive(Clone, Copy, Specifier, Default)]
#[bits = 2]
pub enum GyroST {
    #[default]
    Normal = 0b00,
    Positive = 0b01,
    Negative = 0b11,
}

#[derive(Clone, Copy, Specifier, Default)]
#[bits = 3]
pub enum TriggerMode {
    #[default]
    DataEnableOff = 0b000,
    EdgeTrig = 0b100,
    LevelTrig = 0b010,
    LevelLatch = 0b011,
    LevelFifo = 0b110,
}

#[derive(Clone, Copy, Specifier, Default)]
#[bits = 2]
pub enum GyroHighPassMode {
    #[default]
    Mode16 = 0b00,
    Mode65 = 0b01,
    Mode280 = 0b10,
    Mode1_04 = 0b11,
}

#[derive(Clone, Copy, Specifier, Default)]
#[bits = 2]
pub enum InactMode {
    #[default]
    StationaryOrMotionInts = 0b00,
    Accel12_5GyroUnchanged = 0b01,
    Accel12_5GyroSleep = 0b10,
    Accel12_5GyroPowerDown = 0b11,
}

#[derive(Clone, Copy, Specifier, Default)]
#[bits = 2]
pub enum SixDThresh {
    #[default]
    Thresh80Deg = 0b00,
    Thresh70Deg = 0b01,
    Thresh60Deg = 0b10,
    Thresh50Deg = 0b11,
}

#[derive(Clone, Copy, Specifier, Default)]
#[bits = 3]
pub enum FreeFallThresh {
    #[default]
    Thresh156 = 0b000,
    Thresh219 = 0b001,
    Thresh250 = 0b010,
    Thresh312 = 0b011,
    Thresh344 = 0b100,
    Thresh406 = 0b101,
    Thresh469 = 0b110,
    Thresh500 = 0b111,
}
