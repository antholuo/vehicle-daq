#![allow(dead_code)]

use embedded_hal_async::spi::SpiDevice;
use log::{debug, info};

// ----- Register Addresses -----
// --- FIFO ---
const REG_FIFO_CTRL1: u8 = 0x07;

const REG_FIFO_CTRL2: u8 = 0x08;
const WTM8_MASK: u8 = 0b0000_00001;

const REG_FIFO_CTRL3: u8 = 0x09;
const BDR_GY_MASK: u8 = 0b1111_0000;
const BDR_XL_MASK: u8 = 0b0000_1111;

const REG_FIFO_CTRL4: u8 = 0x0A;
const FIFO_MODE_MASK: u8 = 0b0000_0111; // Bits [2:0] in FIFO_CTRL4
const ODR_T_BATCH_MASK: u8 = 0b0011_0000; // Bits [5:4] in FIFO_CTRL4. Temp data batch rate
const DEC_TS_MASK: u8 = 0b1100_0000; // Bits [7:6] in FIFO_CTRL4. TS decimation

const REG_COUNTER_BDR1: u8 = 0x0B;
const REG_COUNTER_BDR2: u8 = 0x0C;
// --- ---

const REG_INT1_CTRL: u8 = 0x0D;
const REG_INT2_CTRL: u8 = 0x0C;

const REG_WHOAMI: u8 = 0x0F;

const REG_CTRL1_XL: u8 = 0x10;
const REG_CTRL2_G: u8 = 0x11;
const REG_CTRL3_C: u8 = 0x12;
const REG_CTRL4_C: u8 = 0x13;
const REG_CTRL5_C: u8 = 0x14;
const REG_CTRL6_C: u8 = 0x15;
const REG_CTRL7_G: u8 = 0x16;
const REG_CTRL8_XL: u8 = 0x17;
const REG_CTRL9_XL: u8 = 0x18;
const REG_CTRL10_C: u8 = 0x19;

const REG_STATUS_REG: u8 = 0x1E;

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

const REG_TIMESTAMP0_REG: u8 = 0x40;
const REG_TIMESTAMP1_REG: u8 = 0x41;
const REG_TIMESTAMP2_REG: u8 = 0x42;
const REG_TIMESTAMP3_REG: u8 = 0x43;

// const REG_FIFO_DATA_OUT_X_L = 0x79;
// const REG_FIFO_DATA_OUT_X_H = 0x7A;
// const REG_FIFO_DATA_OUT_Y_L = 0x7B;
// const REG_FIFO_DATA_OUT_Y_H = 0x7C;
// const REG_FIFO_DATA_OUT_Z_L = 0x7D;
// const REG_FIFO_DATA_OUT_Z_H = 0x7E

// ----- Bitmasks -----
const GYRO_FSR_MASK: u8 = 0b0000_1110; // bits [3:1] in CTRL2_G
const GYRO_ODR_MASK: u8 = 0b1111_0000; // bits [7:4] in CTRL2_G
const ACCEL_FSR_MASK: u8 = 0b0000_1100; // bits [3:2] in CTRL1_XL
const ACCEL_ODR_MASK: u8 = 0b1111_0000; // bits [7:4] in CTRL1_XL

const TIMER_EN_MASK: u8 = 0b0010_0000; // Bit 5 in CTRL10_C
// TODO: Fix the way that our program communicates with the IMU lmao

const BDU_MASK: u8 = 0b0100_0000; // Bit 6 in CTRL3_C (Block Data Update)

const H_LACTIVE_MASK: u8 = 0b0010_0000; // Polarity: 0 = active high, 1 = active low
const INT1_FIFO_TH_MASK: u8 = 0b0000_1000; // FIFO Watermark interrupt

#[derive(Debug, Clone, Copy)]
pub struct GyRawData {
    pub x: i16,
    pub y: i16,
    pub z: i16,
}

#[derive(Debug, Clone, Copy)]
pub struct XlRawData {
    pub x: i16,
    pub y: i16,
    pub z: i16,
}

#[derive(Debug, Clone, Copy)]
pub struct RawImuData {
    pub xl: XlRawData,
    pub gy: GyRawData,
    pub ts: u32,
}

#[derive(Debug, Clone, Copy)]
pub enum GyroFs {
    DPS250,
    DPS500,
    DPS1000,
    DPS2000,
}

#[derive(Debug, Clone, Copy)]
pub enum AccelFs {
    G2,  // +/- 2G
    G4,  // +/- 4G
    G8,  // +/- 8G
    G16, // +/- 16G
}

#[rustfmt::skip]
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Odr {
    Hz12_5 = 0b0001_0000,
    Hz26   = 0b0010_0000,
    Hz52   = 0b0011_0000,
    Hz104  = 0b0100_0000,
    Hz208  = 0b0101_0000,
    Hz417  = 0b0110_0000,
    Hz833  = 0b0111_0000,
    Hz1667 = 0b1000_0000,
    Hz3333 = 0b1001_0000,
    Hz6667 = 0b1010_0000,
}

pub async fn check_who_am_i<S>(spi: &mut S) -> Result<u8, S::Error>
where
    S: SpiDevice<u8>,
    S::Error: core::fmt::Debug,
{
    info!("Attempting to read WHOAMI at reg: 0x{:02X}", REG_WHOAMI);
    let mut buf: [u8; 2] = [0x8F, 0x00];
    spi.transfer_in_place(&mut buf).await?;
    Ok(buf[1])
}

////////////////////////
/// HELPER FUNCTIONS
////////////////////////
async fn read_register<S>(spi: &mut S, reg_addr: u8) -> Result<u8, S::Error>
where
    S: SpiDevice<u8>,
    S::Error: core::fmt::Debug,
{
    let read_cmd = reg_addr | 0x80;
    let mut buf: [u8; 2] = [read_cmd, 0x00];

    spi.transfer_in_place(&mut buf).await?;

    Ok(buf[1])
}

async fn write_register<S>(spi: &mut S, reg_addr: u8, data: u8) -> Result<(), S::Error>
where
    S: SpiDevice<u8>,
    S::Error: core::fmt::Debug,
{
    let write_cmd = reg_addr & 0x7F;
    let buf: [u8; 2] = [write_cmd, data];

    spi.write(&buf).await?;

    Ok(())
}

////////////////////////
/// Configuration
////////////////////////
pub async fn enable_timestamp<S>(spi: &mut S) -> Result<(), S::Error>
where
    S: SpiDevice<u8>,
    S::Error: core::fmt::Debug,
{
    let current_ctrl10_c = read_register(spi, REG_CTRL10_C).await?;
    let new_ctrl10_c = current_ctrl10_c | TIMER_EN_MASK;
    write_register(spi, REG_CTRL10_C, new_ctrl10_c).await
}

////////////////////////
/// Gyro
////////////////////////
pub async fn set_gyro_config<S>(spi: &mut S, odr: Odr, fsr: GyroFs) -> Result<(), S::Error>
where
    S: SpiDevice<u8>,
    S::Error: core::fmt::Debug,
{
    let odr_bits = odr as u8;
    let fsr_bits = match fsr {
        GyroFs::DPS250 => 0b0000_0000,
        GyroFs::DPS500 => 0b0000_0100,
        GyroFs::DPS1000 => 0b0000_1000,
        GyroFs::DPS2000 => 0b0000_1100,
    };

    let current_ctrl2_g = read_register(spi, REG_CTRL2_G).await?;
    let new_val = (current_ctrl2_g & !(GYRO_ODR_MASK | GYRO_FSR_MASK)) | odr_bits | fsr_bits;

    write_register(spi, REG_CTRL2_G, new_val).await?;

    Ok(())
}

pub async fn read_gy_z<S>(spi: &mut S) -> Result<i16, S::Error>
where
    S: SpiDevice<u8>,
    S::Error: core::fmt::Debug,
{
    const READ_CMD: u8 = 0x80 | REG_OUTZ_L_G;
    let mut buf: [u8; 3] = [READ_CMD, 0x00, 0x00];

    spi.transfer_in_place(&mut buf).await?;

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

pub fn fs_g_to_dps(raw: i16, fsr: &GyroFs) -> f32 {
    // Sensitivities in mdps/LSB (millidegrees per second per LSB)
    let sensitivity_mdps = match fsr {
        GyroFs::DPS250 => 8.75,
        GyroFs::DPS500 => 17.50,
        GyroFs::DPS1000 => 35.0,
        GyroFs::DPS2000 => 70.0,
    };

    // Convert millidegrees/sec to degrees/sec
    ((raw as f32) * sensitivity_mdps) / 1000.0
}

pub async fn set_xl_fsr<S>(spi: &mut S, fsr: &AccelFs) -> Result<(), S::Error>
where
    S: SpiDevice<u8>,
    S::Error: core::fmt::Debug,
{
    let fsr_bits = match fsr {
        AccelFs::G2 => 0b0000_0000,
        AccelFs::G4 => 0b0000_1000,
        AccelFs::G8 => 0b0000_1100,
        AccelFs::G16 => 0b0000_0100,
    };
    let current_ctrl1_xl = read_register(spi, REG_CTRL1_XL).await?;
    let new_val = (current_ctrl1_xl & !ACCEL_FSR_MASK) | fsr_bits;
    write_register(spi, REG_CTRL1_XL, new_val).await?;

    Ok(())
}

pub async fn set_xl_odr<S>(spi: &mut S, odr: Odr) -> Result<(), S::Error>
where
    S: SpiDevice<u8>,
    S::Error: core::fmt::Debug,
{
    let odr_bits = odr as u8;

    let current_ctrl1_xl = read_register(spi, REG_CTRL1_XL).await?;
    let new_val = (current_ctrl1_xl & !ACCEL_ODR_MASK) | odr_bits;

    write_register(spi, REG_CTRL1_XL, new_val).await?;

    Ok(())
}

pub async fn read_xl_xyz<S>(spi: &mut S) -> Result<XlRawData, S::Error>
where
    S: SpiDevice<u8>,
    S::Error: core::fmt::Debug,
{
    debug!("Attempting to read raw XL XYZ registers");
    const READ_CMD: u8 = 0x80 | REG_OUTX_L_A;
    // Need 7 bytes: 1 command + 6 data bytes (X_L, X_H, Y_L, Y_H, Z_L, Z_H)
    let mut buf: [u8; 7] = [READ_CMD, 0, 0, 0, 0, 0, 0];
    spi.transfer_in_place(&mut buf).await?;
    // After transfer: buf[0] = garbage, buf[1..7] = data
    let x = i16::from_le_bytes([buf[1], buf[2]]);
    let y = i16::from_le_bytes([buf[3], buf[4]]);
    let z = i16::from_le_bytes([buf[5], buf[6]]);

    debug!("Got X, Y, Z raw as {} {} {}", x, y, z);

    Ok(XlRawData { x, y, z })
}

pub async fn read_xl_x<S>(spi: &mut S) -> Result<i16, S::Error>
where
    S: SpiDevice<u8>,
    S::Error: core::fmt::Debug,
{
    const READ_CMD: u8 = 0x80 | REG_OUTX_L_A;
    let mut buf: [u8; 3] = [READ_CMD, 0x00, 0x00];

    spi.transfer_in_place(&mut buf).await?;

    let x_lo = buf[1];
    let x_hi = buf[2];

    let x_raw = i16::from_le_bytes([x_lo, x_hi]);

    Ok(x_raw)
}

pub async fn read_xl_y<S>(spi: &mut S) -> Result<i16, S::Error>
where
    S: SpiDevice<u8>,
    S::Error: core::fmt::Debug,
{
    const READ_CMD: u8 = 0x80 | REG_OUTY_L_A;
    let mut buf: [u8; 3] = [READ_CMD, 0x00, 0x00];

    spi.transfer_in_place(&mut buf).await?;

    let y_lo = buf[1];
    let y_hi = buf[2];

    let y_raw = i16::from_le_bytes([y_lo, y_hi]);

    Ok(y_raw)
}

pub async fn read_xl_z<S>(spi: &mut S) -> Result<i16, S::Error>
where
    S: SpiDevice<u8>,
    S::Error: core::fmt::Debug,
{
    const READ_CMD: u8 = 0x80 | REG_OUTZ_L_A;
    let mut buf: [u8; 3] = [READ_CMD, 0x00, 0x00];

    spi.transfer_in_place(&mut buf).await?;

    let z_lo = buf[1];
    let z_hi = buf[2];

    let z_raw = i16::from_le_bytes([z_lo, z_hi]);

    Ok(z_raw)
}

pub async fn read_gy_xyz<S>(spi: &mut S) -> Result<GyRawData, S::Error>
where
    S: SpiDevice<u8>,
    S::Error: core::fmt::Debug,
{
    debug!("Attempting to read raw GY XYZ registers");
    const READ_CMD: u8 = 0x80 | REG_OUTX_L_G;
    // Need 7 bytes: 1 command + 6 data bytes (X_L, X_H, Y_L, Y_H, Z_L, Z_H)
    let mut buf: [u8; 7] = [READ_CMD, 0, 0, 0, 0, 0, 0];
    spi.transfer_in_place(&mut buf).await?;

    let x = i16::from_le_bytes([buf[1], buf[2]]);
    let y = i16::from_le_bytes([buf[3], buf[4]]);
    let z = i16::from_le_bytes([buf[5], buf[6]]);

    Ok(GyRawData { x, y, z })
}

/// Configure both accelerometer (XL) and gyroscope (GY) ODR and FS ranges.
/// Also enables the sensor timestamp counter.
pub async fn configure_imu<S>(
    spi: &mut S,
    odr: Odr,
    accel_fs: AccelFs,
    gyro_fs: GyroFs,
    enable_bdu: bool,
) -> Result<(), S::Error>
where
    S: SpiDevice<u8>,
    S::Error: core::fmt::Debug,
{
    // Set accelerometer ODR and full-scale range
    set_xl_odr(spi, odr).await?;
    set_xl_fsr(spi, &accel_fs).await?;

    // Set gyroscope ODR and full-scale range
    set_gyro_config(spi, odr, gyro_fs).await?;

    // Enable sensor timestamp so we can read timestamps when polling
    enable_timestamp(spi).await?;

    // Configure Block Data Update if requested
    let current_ctrl3 = read_register(spi, REG_CTRL3_C).await?;
    let new_ctrl3 = if enable_bdu {
        current_ctrl3 | BDU_MASK
    } else {
        current_ctrl3 & !BDU_MASK
    };
    write_register(spi, REG_CTRL3_C, new_ctrl3).await?;

    Ok(())
}

/// Poll both accelerometer and gyroscope and return combined raw data with a timestamp.
pub async fn poll_xl_gy_combined<S>(spi: &mut S) -> Result<RawImuData, S::Error>
where
    S: SpiDevice<u8>,
    S::Error: core::fmt::Debug,
{
    debug!("Attempting combined SPI read for GY then XL registers");
    const READ_CMD: u8 = 0x80 | REG_OUTX_L_G;
    // 1 command byte + 12 data bytes (GY X/Y/Z, XL X/Y/Z)
    let mut buf: [u8; 13] = [READ_CMD, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    spi.transfer_in_place(&mut buf).await?;

    let gx = i16::from_le_bytes([buf[1], buf[2]]);
    let gyy = i16::from_le_bytes([buf[3], buf[4]]);
    let gz = i16::from_le_bytes([buf[5], buf[6]]);
    let ax = i16::from_le_bytes([buf[7], buf[8]]);
    let ay = i16::from_le_bytes([buf[9], buf[10]]);
    let az = i16::from_le_bytes([buf[11], buf[12]]);

    let gyro = GyRawData {
        x: gx,
        y: gyy,
        z: gz,
    };
    let xl = XlRawData {
        x: ax,
        y: ay,
        z: az,
    };

    let ts = get_timestamp(spi).await?;

    Ok(RawImuData { xl, gy: gyro, ts })
}

pub async fn get_timestamp<S>(spi: &mut S) -> Result<u32, S::Error>
where
    S: SpiDevice<u8>,
    S::Error: core::fmt::Debug,
{
    const READ_CMD: u8 = 0x80 | REG_TIMESTAMP0_REG;
    let mut buf: [u8; 5] = [READ_CMD, 0, 0, 0, 0];
    spi.transfer_in_place(&mut buf).await?;
    let ts = u32::from_le_bytes([buf[1], buf[2], buf[3], buf[4]]);
    Ok(ts)
}

#[derive(Debug, Clone, Copy)]
pub struct StatusDataAvailable {
    pub xlda: bool,
    pub gda: bool,
    pub tda: bool,
}

pub async fn read_status_data<S>(spi: &mut S) -> Result<StatusDataAvailable, S::Error>
where
    S: SpiDevice<u8>,
    S::Error: core::fmt::Debug,
{
    let status = read_register(spi, REG_STATUS_REG).await?;
    let xlda = (status & 0b0000_0001) != 0;
    let gda = (status & 0b0000_0010) != 0;
    let tda = (status & 0b0000_0100) != 0;
    Ok(StatusDataAvailable { xlda, gda, tda })
}
