use embedded_hal::spi::SpiBus;
use log::{debug, info, trace, warn};

use crate::registers::*;
use crate::types::*;

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

pub fn write_register<s>(spi: &mut s, reg_addr: u8, data: u8) -> Result<(), s::Error>
where
    s: SpiBus<u8>,
    s::Error: core::fmt::Debug,
{
    let write_cmd = reg_addr & 0x7f;
    let buf: [u8; 2] = [write_cmd, data];

    spi.write(&buf)?;

    Ok(())
}

pub fn make_ctrl1_xl_reg(odr: Odr, fs: AccelFs, lpf2_en: bool) -> u8 {
    ((odr as u8) << CTRL1_XL_ODR_SAMT)
        | ((fs as u8) << CTRL1_XL_FS_SAMT)
        | ((lpf2_en as u8) << CTRL1_XL_LPF2_EN_SAMT)
}

pub fn make_ctrl2_g_reg(odr: Odr, fs: GyroFs) -> u8 {
    ((odr as u8) << CTRL2_G_ODR_SAMT) | ((fs as u8) << CTRL2_G_FS_SAMT)
}

pub fn configure_ctrl3_c_reg<S>(
    // NOTE: bunch of these are unused right now
    spi: &mut S,
    _sw_reset: Option<bool>,
    _if_inc: Option<bool>,
    _sim: Option<bool>,
    _pp_od: Option<bool>,
    _h_lactive: Option<bool>,
    bdu: Option<bool>,
    _boot: Option<bool>,
) -> Result<(), S::Error>
where
    S: SpiBus<u8>,
    S::Error: core::fmt::Debug,
{
    let mut reg_val = read_register(spi, CTRL3_C)?;

    // Update bits only if corresponding Option is Some(...)
    if let Some(enable) = bdu {
        if enable {
            reg_val |= (enable as u8) << CTRL3_C_BDU_SAMT;
        } else {
            reg_val &= !((enable as u8) << CTRL3_C_BDU_SAMT);
        }
    }

    write_register(spi, CTRL3_C, reg_val)?;

    Ok(())
}

// TODO: enable timestamp with a `configure_ctrl10_c_reg function`
pub fn enable_timestamp<S>(spi: &mut S) -> Result<(), S::Error>
where
    S: SpiBus<u8>,
    S::Error: core::fmt::Debug,
{
    let current_ctrl10_c = read_register(spi, CTRL10_C)?;
    let new_ctrl10_c = current_ctrl10_c | (CTRL10_C_TIMER_EN_MASK);
    write_register(spi, CTRL10_C, new_ctrl10_c)
}
