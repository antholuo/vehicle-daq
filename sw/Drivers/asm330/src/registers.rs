#![allow(dead_code)]
use embedded_hal::spi::SpiBus;
use log::{debug, info, trace, warn};

use crate::{AccelFs, Odr, RawSensorData, SensorStatus};

const REG_CTRL1_XL: u8 = 0x10;
const CTRL1_XL_ODR_SAMT: u8 = 4;
const CTRL1_XL_FS_SAMT: u8 = 2;
const CTRL1_XL_LPF2_EN_MASK: u8 = 0b0000_0010; // NOT a shift amount

const REG_CTRL2_GY: u8 = 0x11;

const REG_STATUS_REG: u8 = 0x1E;

const REG_OUTX_L_G: u8 = 0x22;
const REG_OUTX_H_G: u8 = 0x23;
const REG_OUTY_L_G: u8 = 0x24;
const REG_OUTY_H_G: u8 = 0x25;
const REG_OUTZ_L_G: u8 = 0x26;
const REG_OUTZ_H_G: u8 = 0x27;

pub(super) const REG_OUTX_L_A: u8 = 0x28;
const REG_OUTX_H_A: u8 = 0x29;
const REG_OUTY_L_A: u8 = 0x2A;
const REG_OUTY_H_A: u8 = 0x2B;
const REG_OUTZ_L_A: u8 = 0x2C;
const REG_OUTZ_H_A: u8 = 0x2D;

pub fn read_register<S>(spi: &mut S, reg_addr: u8) -> Result<u8, S::Error>
where
    S: SpiBus<u8>,
    S::Error: core::fmt::Debug,
{
    let read_cmd = reg_addr | 0x80;
    let mut buf: [u8; 2] = [read_cmd, 0x00];

    spi.transfer_in_place(&mut buf)?;

    Ok(buf[1])
}

pub fn write_register<S>(spi: &mut S, reg_addr: u8, data: u8) -> Result<(), S::Error>
where
    S: SpiBus<u8>,
    S::Error: core::fmt::Debug,
{
    let write_cmd = reg_addr & 0x7f;
    let buf: [u8; 2] = [write_cmd, data];

    spi.write(&buf)?;

    Ok(())
}

pub fn get_status<S>(spi: &mut S) -> Result<SensorStatus, S::Error>
where
    S: SpiBus<u8>,
    S::Error: core::fmt::Debug,
{
    debug!("Trying to read status register!");
    let res = read_register(spi, REG_STATUS_REG)?;
    let stat = SensorStatus {
        tda: res & 0b0000_0100 != 0,
        gda: res & 0b0000_0010 != 0,
        xlda: res & 0b0000_0001 != 0,
    };
    info!(
        "Got raw status register as {:08b}, with sensor status: {:?}",
        res, stat
    );

    Ok(stat)
}

pub fn set_reg_ctrl1_xl<S>(
    spi: &mut S,
    odr: Odr,
    fsr: AccelFs,
    lpf2_en: bool,
) -> Result<(), S::Error>
where
    S: SpiBus<u8>,
    S::Error: core::fmt::Debug,
{
    debug!(
        "Setting reg ctrl1_xl with odr: {:?}, accel_fsr: {:?}, and lpf2_en: {}",
        odr, fsr, lpf2_en
    );
    let mut new_ctrl1_xl: u8 = (odr as u8) << CTRL1_XL_ODR_SAMT | (fsr as u8) << CTRL1_XL_FS_SAMT;
    if lpf2_en {
        new_ctrl1_xl |= CTRL1_XL_LPF2_EN_MASK;
    }

    info!("Constructed new_ctrl1_xl as: {:08b}", new_ctrl1_xl);

    write_register(spi, REG_CTRL1_XL, new_ctrl1_xl)
}
