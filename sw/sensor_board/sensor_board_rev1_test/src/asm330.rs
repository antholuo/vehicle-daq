#![allow(dead_code)]

use embedded_hal::spi::SpiBus;
use log::{debug, info, trace};

const REG_WHOAMI: u8 = 0x0F;

const REG_CTRL1_XL: u8 = 0x10;

const REG_OUTX_L_G: u8 = 0x22;
const REG_OUTX_H_G: u8 = 0x23;
const REG_OUTY_L_G: u8 = 0x24;
const REG_OUTY_H_G: u8 = 0x25;
const REG_OUTZ_L_G: u8 = 0x26;
const REG_OUTZ_H_G: u8 = 0x27;

const REG_OUTX_L_A: u8 = 0x28;
const REG_OUTX_H_A: u8 = 0x29;
const REG_OUTY_L_A: u8 = 0x2A;
const REG_OUTY_H_A: u8 = 0x2B;
const REG_OUTZ_L_A: u8 = 0x2C;
const REG_OUTZ_H_A: u8 = 0x2D;

const ACCEL_FSR_MASK: u8 = 0b0000_1100; // bits [3:2] in CTRL1_XL
const ACCEL_ODR_MASK: u8 = 0b1111_0000; // bits [7:4] in CTRL1_XL

#[derive(Debug, Clone, Copy)]
pub struct XlRawData {
    pub x: i16,
    pub y: i16,
    pub z: i16,
}

#[derive(Debug, Clone, Copy)]
pub struct GyRawData {
    pub x: i16,
    pub y: i16,
    pub z: i16,
}

#[derive(Debug, Clone, Copy)]
pub struct ImuRawData {
    pub xl: XlRawData,
    pub gy: GyRawData,
    pub ts: u64,
}

#[derive(Debug)]
pub enum AccelFs {
    G2,  // +/- 2G
    G4,  // +/- 4G
    G8,  // +/- 8G
    G16, // +/- 16G
}

#[derive(Debug)]
pub enum AccelOdr {
    Hz12_5,
    Hz26,
    Hz52,
    Hz104,
    Hz208,
}

pub fn check_who_am_i<S>(spi: &mut S) -> Result<u8, S::Error>
where
    S: SpiBus<u8>,
    S::Error: core::fmt::Debug,
{
    info!("Attempting to read WHOAMI at reg: 0x{:02X}", REG_WHOAMI);
    let mut buf: [u8; 2] = [0x8F, 0x00];
    spi.transfer_in_place(&mut buf)?;
    Ok(buf[1])
}

////////////////////////
/// HELPER FUNCTIONS
////////////////////////
fn read_register<S>(spi: &mut S, reg_addr: u8) -> Result<u8, S::Error>
where
    S: SpiBus<u8>,
    S::Error: core::fmt::Debug,
{
    let read_cmd = reg_addr | 0x80;
    let mut buf: [u8; 2] = [read_cmd, 0x00];

    spi.transfer_in_place(&mut buf)?;

    Ok(buf[1])
}

fn write_register<S>(spi: &mut S, reg_addr: u8, data: u8) -> Result<(), S::Error>
where
    S: SpiBus<u8>,
    S::Error: core::fmt::Debug,
{
    let write_cmd = reg_addr & 0x7F;
    let buf: [u8; 2] = [write_cmd, data];

    spi.write(&buf)?;

    Ok(())
}

pub fn read_gy_z<S>(spi: &mut S) -> Result<i16, S::Error>
where
    S: SpiBus<u8>,
    S::Error: core::fmt::Debug,
{
    const READ_CMD: u8 = 0x80 | REG_OUTZ_L_G;
    let mut buf: [u8; 3] = [READ_CMD, 0x00, 0x00];

    spi.transfer_in_place(&mut buf)?;

    let z_low = buf[1];
    let z_high = buf[2];

    let z_raw = i16::from_le_bytes([z_low, z_high]);

    Ok(z_raw)
}

////////////////////////
/// Accelerometer
////////////////////////

pub fn fs_a_to_g(raw: i16, fsr: &AccelFs) -> f32 {
    let sensitivity_mg = match fsr {
        AccelFs::G2 => 0.061,
        AccelFs::G4 => 0.122,
        AccelFs::G8 => 0.244,
        AccelFs::G16 => 0.488,
    };

    ((raw as f32) * sensitivity_mg) / 1000.0
}

pub fn set_xl_fsr<S>(spi: &mut S, fsr: &AccelFs) -> Result<(), S::Error>
where
    S: SpiBus<u8>,
    S::Error: core::fmt::Debug,
{
    let fsr_bits = match fsr {
        AccelFs::G2 => 0b0000_0000,
        AccelFs::G4 => 0b0000_1000,
        AccelFs::G8 => 0b0000_1100,
        AccelFs::G16 => 0b0000_0100,
    };
    let current_ctrl1_xl = read_register(spi, REG_CTRL1_XL)?;
    let new_val = (current_ctrl1_xl & !ACCEL_FSR_MASK) | fsr_bits;
    write_register(spi, REG_CTRL1_XL, new_val)?;

    Ok(())
}

pub fn set_xl_odr<S>(spi: &mut S, odr: &AccelOdr) -> Result<(), S::Error>
where
    S: SpiBus<u8>,
    S::Error: core::fmt::Debug,
{
    let odr_bits = match odr {
        AccelOdr::Hz12_5 => 0b0001_0000, // 0001 << 4
        AccelOdr::Hz26 => 0b0010_0000,   // 0010 << 4
        AccelOdr::Hz52 => 0b0011_0000,   // 0010 << 4
        AccelOdr::Hz104 => 0b0100_0000,  // 0100 << 4
        AccelOdr::Hz208 => 0b0101_0000,  // 0101 << 4
    };

    let current_ctrl1_xl = read_register(spi, REG_CTRL1_XL)?;
    let new_val = (current_ctrl1_xl & !ACCEL_ODR_MASK) | odr_bits;

    write_register(spi, REG_CTRL1_XL, new_val)?;

    Ok(())
}

pub fn read_xl_xyz<S>(spi: &mut S) -> Result<XlRawData, S::Error>
where
    S: SpiBus<u8>,
    S::Error: core::fmt::Debug,
{
    debug!("Attempting to read raw XL XYZ registers");
    const READ_CMD: u8 = 0x80 | REG_OUTX_L_A; // in place read all 6 bytes
    let mut buf: [u8; 6] = [READ_CMD, 0, 0, 0, 0, 0];
    spi.transfer_in_place(&mut buf)?;
    let x = i16::from_le_bytes([buf[0], buf[1]]);
    let y = i16::from_le_bytes([buf[2], buf[3]]);
    let z = i16::from_le_bytes([buf[4], buf[5]]);

    debug!("Got X, Y, Z raw as {} {} {}", x, y, z);

    Ok(XlRawData { x: x, y: y, z: z })
}

pub fn read_xl_x<S>(spi: &mut S) -> Result<i16, S::Error>
where
    S: SpiBus<u8>,
    S::Error: core::fmt::Debug,
{
    const READ_CMD: u8 = 0x80 | REG_OUTX_L_A;
    let mut buf: [u8; 3] = [READ_CMD, 0x00, 0x00];

    spi.transfer_in_place(&mut buf)?;

    let x_lo = buf[1];
    let x_hi = buf[2];

    let x_raw = i16::from_le_bytes([x_lo, x_hi]);

    Ok(x_raw)
}

pub fn read_xl_y<S>(spi: &mut S) -> Result<i16, S::Error>
where
    S: SpiBus<u8>,
    S::Error: core::fmt::Debug,
{
    const READ_CMD: u8 = 0x80 | REG_OUTY_L_A;
    let mut buf: [u8; 3] = [READ_CMD, 0x00, 0x00];

    spi.transfer_in_place(&mut buf)?;

    let y_lo = buf[1];
    let y_hi = buf[2];

    let y_raw = i16::from_le_bytes([y_lo, y_hi]);

    Ok(y_raw)
}

pub fn read_xl_z<S>(spi: &mut S) -> Result<i16, S::Error>
where
    S: SpiBus<u8>,
    S::Error: core::fmt::Debug,
{
    const READ_CMD: u8 = 0x80 | REG_OUTZ_L_A;
    let mut buf: [u8; 3] = [READ_CMD, 0x00, 0x00];

    spi.transfer_in_place(&mut buf)?;

    let z_lo = buf[1];
    let z_hi = buf[2];

    let z_raw = i16::from_le_bytes([z_lo, z_hi]);

    Ok(z_raw)
}
