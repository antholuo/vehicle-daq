use embedded_hal::spi::SpiBus;
/// xl_ctrl.rs
///
/// Exclusively accelerometer control (since this is what we care about the most)
/// will potentially be re-implemented later in sens_ctrl.rs or something, but whatever.

#[allow(unused_imports)]
use log::{debug, info, trace};

use crate::{AccelFs, Odr, RawSensorData, registers::*};

pub fn set_bdu<S>(spi: &mut S, bdu_en: bool) -> Result<(), S::Error>
where
    S: SpiBus<u8>,
    S::Error: core::fmt::Debug,
{
    Ok(())
}

pub fn enable_xl<S>(spi: &mut S, odr: Odr, fsr: AccelFs, lpf2_en: bool) -> Result<(), S::Error>
where
    S: SpiBus<u8>,
    S::Error: core::fmt::Debug,
{
    set_reg_ctrl1_xl(spi, odr, fsr, lpf2_en)
}

pub fn read_raw_xl_xyz<S>(spi: &mut S) -> Result<RawSensorData, S::Error>
where
    S: SpiBus<u8>,
    S::Error: core::fmt::Debug,
{
    debug!("Attempting to read raw XL XYZ registers");
    const READ_CMD: u8 = 0x80 | REG_OUTX_L_A;
    let mut buf: [u8; 7] = [READ_CMD, 0, 0, 0, 0, 0, 0];
    spi.transfer_in_place(&mut buf)?;

    Ok(RawSensorData {
        x: i16::from_le_bytes([buf[1], buf[2]]),
        y: i16::from_le_bytes([buf[3], buf[4]]),
        z: i16::from_le_bytes([buf[5], buf[6]]),
        valid: false,
        ts: 0,
    })
}
