#![no_std]
/*
* Created date: 12/6/25
* File Description: Public top interface level of asm330 driver
*/

#[allow(unused_imports)]
use embedded_hal::spi::SpiBus;
use embedded_hal::spi::ErrorType;
use core::result::Result::Ok;
use log::{debug};
use thiserror_no_std::Error;
use core::fmt;
use core::prelude::v1::Err;
use esp_hal::time::{Duration, Instant, Rate};
mod reg;
mod device;

#[derive(Error, Debug)]
pub enum Asm330Error {
    #[error("Data not ready for read")]
    DataNotReadyError(),
}

#[derive(Copy, Clone)]
pub enum OdrFreq {
    Odr00125,
    Odr00260,
    Odr00520,
    Odr01040,
    Odr02080,
    Odr04170,
    Odr08330,
    Odr16670,
    Odr33330,
    Odr66670,
}

#[derive(Copy, Clone)]
pub enum AccelScale {
    Accel2G,
    Accel4G,
    Accel8G,
    Accel16G,
}

#[derive(Copy, Clone)]
pub enum AccelBandwidth {
    LowOdrBy2 = 0,
    LowOdrBy4 = 1,
    LowOdrBy10 = 2,
    LowOdrBy20 = 3,
    LowOdrBy45 = 4,
    LowOdrBy100 = 5,
    LowOdrBy200 = 6,
    LowOdrBy400 = 7,
    LowOdrBy800 = 8,
    HighOdrBy4 = 9,
    HighOdrBy10 = 10,
    HighOdrBy20 = 11,
    HighOdrBy45 = 12,
    HighOdrBy100 = 13,
    HighOdrBy200 = 14,
    HighOdrBy400 = 15,
    HighOdrBy800 = 16,
}

#[derive(Copy, Clone)]
pub enum GyroScale {
    Gyro125Dps,
    Gyro250Dps,
    Gyro500Dps,
    Gyro1000Dps,
    Gyro2000Dps,
    Gyro4000Dps,
}

#[derive(Copy, Clone)]
pub enum GyroFType {
    GyroFT000,
    GyroFT001,
    GyroFT010,
    GyroFT011,
    GyroFT100,
    GyroFT101,
    GyroFT110,
    GyroFT111,
}

#[derive(Copy, Clone)]
pub enum GyroHighPassCutoff {
    Gyro0016Hz,
    Gyro0065Hz,
    Gyro0260Hz,
    Gyro1040Hz,
}

pub fn is_who_am_i_good<S>(spi: &mut S) -> Result<bool, S::Error>
where
    S: SpiBus<u8>,
    S::Error: core::fmt::Debug,
{
    let val = reg::spi_read_reg(spi, reg::ADDR_WHO_AM_I)?;
    debug!("WHOAMI register at 0x{:02X} reads as: 0x{:02X}", reg::ADDR_WHO_AM_I, val);
    Ok(val == reg::VALUE_WHO_AM_I)
}

pub fn configure_spi<S>(spi: &mut S) -> Result<(), S::Error>
where
    S: SpiBus<u8>,
    S::Error: core::fmt::Debug,
{
    let _ = reg::spi_write_reg(spi, reg::ADDR_CTRL9_XL, reg::BITMASK_DEVICE_CONF)?;
    let _ = reg::spi_write_reg(spi, reg::ADDR_CTRL4_C, reg::BITMASK_I2C_DISABLE)?;
    let _ = reg::spi_write_reg(spi, reg::ADDR_CTRL9_XL, 0x00)?;
    Ok(())
}

pub fn sw_reset<S>(spi: &mut S) -> Result<(), S::Error>
where
    S: SpiBus<u8>,
    S::Error: core::fmt::Debug,
{
    let _ = reg::spi_write_reg(spi, reg::ADDR_CTRL3_C, reg::BITMASK_SW_RESET);
    Ok(())
}

pub fn enable_accel<S>(
    spi: &mut S,
    odr: OdrFreq,
    fs: AccelScale,
    bw: AccelBandwidth,
    low_pass_on_6d: bool,
    fast_settle_mode: bool,
    high_pass_ref_mode: bool,
    ) -> Result<(), S::Error>
where
    S: SpiBus<u8>,
    S::Error: core::fmt::Debug,
{
    let mut ctrl1_val = 0x00;
    let mut ctrl8_val: u8 = 0x00;
    let ctrl2_val = reg::spi_read_reg(spi, reg::ADDR_CTRL2_G)?;
    let mut ctrl6_val = reg::spi_read_reg(spi, reg::ADDR_CTRL6_C)?;
    let bw_dec = bw as u8;
    if bw_dec >= 9 {
        ctrl8_val |= reg::BITMASK_HP_SLOPE_XL_EN;
    }
    else {
        if bw_dec != 0 {
            ctrl1_val |= reg::BITMASK_LPF2_XL_EN;
        }
        else {
            ctrl1_val &= !reg::BITMASK_LPF2_XL_EN;
        }
    }
    match odr {
    OdrFreq::Odr00125 => {
        ctrl1_val |= reg::VALUE_ODR_00125;
    }
    OdrFreq::Odr00260 => {
        ctrl1_val |= reg::VALUE_ODR_00260;
    }
    OdrFreq::Odr00520 => {
        ctrl1_val |= reg::VALUE_ODR_00520;
    }
    OdrFreq::Odr01040 => {
        ctrl1_val |= reg::VALUE_ODR_01040;
    }
    OdrFreq::Odr02080 => {
        ctrl1_val |= reg::VALUE_ODR_02080;
    }
    OdrFreq::Odr04170 => {
        ctrl1_val |= reg::VALUE_ODR_04170;
    }
    OdrFreq::Odr08330 => {
        ctrl1_val |= reg::VALUE_ODR_08330;
    }
    OdrFreq::Odr16670 => {
        ctrl1_val |= reg::VALUE_ODR_16670;
    }
    OdrFreq::Odr33330 => {
        ctrl1_val |= reg::VALUE_ODR_33330;
    }
    OdrFreq::Odr66670 => {
        ctrl1_val |= reg::VALUE_ODR_66670;
    }
    }
    match fs {
    AccelScale::Accel2G => {
        ctrl1_val |= reg::VALUE_FS_00;
    }
    AccelScale::Accel4G => {
        ctrl1_val |= reg::VALUE_FS_10;
    }
    AccelScale::Accel8G => {
        ctrl1_val |= reg::VALUE_FS_11;
    }
    AccelScale::Accel16G => {
        ctrl1_val |= reg::VALUE_FS_01;
    }
    }
    match bw {
        AccelBandwidth::LowOdrBy2 => {}
        AccelBandwidth::LowOdrBy4 => {
            ctrl8_val |= reg::VALUE_HPCF_XL_0;
        }
        AccelBandwidth::LowOdrBy10 => {
            ctrl8_val |= reg::VALUE_HPCF_XL_1;
        }
        AccelBandwidth::LowOdrBy20 => {
            ctrl8_val |= reg::VALUE_HPCF_XL_2;
        }
        AccelBandwidth::LowOdrBy45 => {
            ctrl8_val |= reg::VALUE_HPCF_XL_3;
        }
        AccelBandwidth::LowOdrBy100 => {
            ctrl8_val |= reg::VALUE_HPCF_XL_4;
        }
        AccelBandwidth::LowOdrBy200 => {
            ctrl8_val |= reg::VALUE_HPCF_XL_5;
        }
        AccelBandwidth::LowOdrBy400 => {
            ctrl8_val |= reg::VALUE_HPCF_XL_6;
        }
        AccelBandwidth::LowOdrBy800 => {
            ctrl8_val |= reg::VALUE_HPCF_XL_7;
        }
        AccelBandwidth::HighOdrBy4 => {
            ctrl8_val |= reg::VALUE_HPCF_XL_0; 
        }
        AccelBandwidth::HighOdrBy10 => {
            ctrl8_val |= reg::VALUE_HPCF_XL_1; 
        }
        AccelBandwidth::HighOdrBy20 => {
            ctrl8_val |= reg::VALUE_HPCF_XL_2; 
        }
        AccelBandwidth::HighOdrBy45 => {
            ctrl8_val |= reg::VALUE_HPCF_XL_3; 
        }
        AccelBandwidth::HighOdrBy100 => {
            ctrl8_val |= reg::VALUE_HPCF_XL_4; 
        }
        AccelBandwidth::HighOdrBy200 => {
            ctrl8_val |= reg::VALUE_HPCF_XL_5; 
        }
        AccelBandwidth::HighOdrBy400 => {
            ctrl8_val |= reg::VALUE_HPCF_XL_6; 
        }
        AccelBandwidth::HighOdrBy800 => {
            ctrl8_val |= reg::VALUE_HPCF_XL_7; 
        }
    }
    if low_pass_on_6d {
        ctrl8_val |= reg::BITMASK_LOW_PASS_ON_6D;
    }
    if fast_settle_mode {
        ctrl8_val |= reg::BITMASK_FASTSETTL_MODE_XL;
    }
    if high_pass_ref_mode {
        ctrl8_val |= reg::BITMASK_HP_REF_MODE_XL;
    }
    reg::spi_write_reg(spi, reg::ADDR_CTRL8_XL, ctrl8_val)?;
    // see note in section 3 intro of application notes
    // if gyro is not in power-down mode
    if ctrl2_val >> 0x04 > 0x00 {
        ctrl6_val |= reg::BITMASK_CTRL6_C_DISABLE_DEVICE;
        reg::spi_write_reg(spi, reg::ADDR_CTRL6_C, ctrl6_val)?;
        reg::spi_write_reg(spi, reg::ADDR_CTRL1_XL, reg::VALUE_ODR_02080)?;
        let _ = reg::spi_read_reg(spi, reg::ADDR_OUTZ_H_A)?;
        loop {
            let status_reg_val = reg::spi_read_reg(spi, reg::ADDR_STATUS_REG)?;
            if status_reg_val & reg::BITMASK_STATUS_REG_XLDA == 0x01 {
                break;
            }
        }
        ctrl6_val &= !reg::BITMASK_CTRL6_C_DISABLE_DEVICE;
        reg::spi_write_reg(spi, reg::ADDR_CTRL6_C, ctrl6_val)?;
    }
    reg::spi_write_reg(spi, reg::ADDR_CTRL1_XL, ctrl1_val)?;
    Ok(())
}

pub fn disable_accel<S>(spi: &mut S) -> Result<(), Asm330Error>
where
    S: SpiBus<u8>,
{
    reg::spi_write_reg(spi, reg::ADDR_CTRL8_XL, 0x00).unwrap();
    reg::spi_write_reg(spi, reg::ADDR_CTRL1_XL, 0x00).unwrap();
    Ok(())
}

pub fn read_single_accel<S>(spi: &mut S) -> Result<(i16, i16, i16), Asm330Error>
where 
    S: SpiBus<u8>,
{
    let status_val = reg::spi_read_reg(spi, reg::ADDR_STATUS_REG).unwrap();
    if status_val & reg::BITMASK_STATUS_REG_XLDA != reg::BITMASK_STATUS_REG_XLDA {
        return Err(Asm330Error::DataNotReadyError());
    }
    let accel_x_h = reg::spi_read_reg(spi, reg::ADDR_OUTX_H_A).unwrap() as i16;
    let accel_x_l = reg::spi_read_reg(spi, reg::ADDR_OUTX_L_A).unwrap() as i16;
    let accel_y_h = reg::spi_read_reg(spi, reg::ADDR_OUTY_H_A).unwrap() as i16;
    let accel_y_l = reg::spi_read_reg(spi, reg::ADDR_OUTY_L_A).unwrap() as i16;
    let accel_z_h = reg::spi_read_reg(spi, reg::ADDR_OUTZ_H_A).unwrap() as i16;
    let accel_z_l = reg::spi_read_reg(spi, reg::ADDR_OUTZ_L_A).unwrap() as i16;
    Ok((accel_x_h << 8 | accel_x_l, accel_y_h << 8 | accel_y_l, accel_z_h << 8 | accel_z_l))
}

pub fn enable_gyro<S>(
    spi: &mut S,
    odr: OdrFreq,
    fs: GyroScale,
    ftype: GyroFType,
    high_pass_cutoff: GyroHighPassCutoff,
    hp_en: bool,
    lpf1_sel: bool,
    sleep: bool
) -> Result<(), Asm330Error>
where
    S: SpiBus<u8>
{
    let mut ctrl2_val = 0x00;
    let mut ctrl7_val = reg::spi_read_reg(spi, reg::ADDR_CTRL7_G).unwrap();
    let mut ctrl6_val = reg::spi_read_reg(spi, reg::ADDR_CTRL6_C).unwrap();
    let mut ctrl4_val = reg::spi_read_reg(spi, reg::ADDR_CTRL4_C).unwrap();

    match odr {
        OdrFreq::Odr00125 => {
            ctrl2_val |= reg::VALUE_ODR_00125;
        }
        OdrFreq::Odr00260 => {
            ctrl2_val |= reg::VALUE_ODR_00260;
        }
        OdrFreq::Odr00520 => {
            ctrl2_val |= reg::VALUE_ODR_00520;
        }
        OdrFreq::Odr01040 => {
            ctrl2_val |= reg::VALUE_ODR_01040;
        }
        OdrFreq::Odr02080 => {
            ctrl2_val |= reg::VALUE_ODR_02080;
        }
        OdrFreq::Odr04170 => {
            ctrl2_val |= reg::VALUE_ODR_04170;
        }
        OdrFreq::Odr08330 => {
            ctrl2_val |= reg::VALUE_ODR_08330;
        }
        OdrFreq::Odr16670 => {
            ctrl2_val |= reg::VALUE_ODR_16670;
        }
        OdrFreq::Odr33330 => {
            ctrl2_val |= reg::VALUE_ODR_33330;
        }
        OdrFreq::Odr66670 => {
            ctrl2_val |= reg::VALUE_ODR_66670;
        }
    }
    match fs {
        GyroScale::Gyro125Dps => {
            ctrl2_val |= reg::BITMASK_CTRL2_FS_125;
        }
        GyroScale::Gyro250Dps => {
            ctrl2_val |= reg::VALUE_FS_00;
        }
        GyroScale::Gyro500Dps => {
            ctrl2_val |= reg::VALUE_FS_01;
        }
        GyroScale::Gyro1000Dps => {
            ctrl2_val |= reg::VALUE_FS_10;
        }
        GyroScale::Gyro2000Dps => {
            ctrl2_val |= reg::VALUE_FS_11;
        }
        GyroScale::Gyro4000Dps => {
            ctrl2_val |= reg::BITMASK_CTRL2_FS_4000;
        }
    }
    if hp_en {
        ctrl7_val |= reg::BITMASK_CTRL7_HP_EN_G;
        ctrl7_val &= !reg::BITMASK_CTRL7_HPM_11;
        match high_pass_cutoff {
            GyroHighPassCutoff::Gyro0016Hz => {
                ctrl7_val |= reg::BITMASK_CTRL7_HPM_00;
            }
            GyroHighPassCutoff::Gyro0065Hz => {
                ctrl7_val |= reg::BITMASK_CTRL7_HPM_01;
            }
            GyroHighPassCutoff::Gyro0260Hz => {
                ctrl7_val |= reg::BITMASK_CTRL7_HPM_10;
            }
            GyroHighPassCutoff::Gyro1040Hz => {
                ctrl7_val |= reg::BITMASK_CTRL7_HPM_11;
            }
        }
    }
    else {
        ctrl7_val &= !reg::BITMASK_CTRL7_HP_EN_G;
    }
    ctrl6_val &= !reg::VALUE_FTYPE_111;
    match ftype {
        GyroFType::GyroFT000 => {
            ctrl6_val |= reg::VALUE_FTYPE_000;
        }
        GyroFType::GyroFT001 => {
            ctrl6_val |= reg::VALUE_FTYPE_001;
        }
        GyroFType::GyroFT010 => {
            ctrl6_val |= reg::VALUE_FTYPE_010;
        }
        GyroFType::GyroFT011 => {
            ctrl6_val |= reg::VALUE_FTYPE_011;
        }
        GyroFType::GyroFT100 => {
            ctrl6_val |= reg::VALUE_FTYPE_100;
        }
        GyroFType::GyroFT101 => {
            ctrl6_val |= reg::VALUE_FTYPE_101;
        }
        GyroFType::GyroFT110 => {
            ctrl6_val |= reg::VALUE_FTYPE_110;
        }
        GyroFType::GyroFT111 => {
            ctrl6_val |= reg::VALUE_FTYPE_111;
        }
    }
    if sleep {
        ctrl4_val |= reg::BITMASK_CTRL4_SLEEP_G;
    }
    else {
        if lpf1_sel {
            ctrl4_val |= reg::BITMASK_CTRL4_LPF1_SEL_G;
        }
        else {
            ctrl4_val &= !reg::BITMASK_CTRL4_LPF1_SEL_G;
        }
    }
    reg::spi_write_reg(spi, reg::ADDR_CTRL2_G, ctrl2_val).unwrap();
    reg::spi_write_reg(spi, reg::ADDR_CTRL7_G, ctrl7_val).unwrap();
    reg::spi_write_reg(spi, reg::ADDR_CTRL6_C, ctrl6_val).unwrap();
    reg::spi_write_reg(spi, reg::ADDR_CTRL4_C, ctrl4_val).unwrap();
    Ok(())
}

pub fn disable_gyro<S>(spi: &mut S) -> Result<(), Asm330Error>
where
    S: SpiBus<u8>,
{
    reg::spi_write_reg(spi, reg::ADDR_CTRL2_G, 0x00).unwrap();
    Ok(())
}

pub fn read_single_gyro<S>(spi: &mut S) -> Result<(i16, i16, i16), Asm330Error>
where 
    S: SpiBus<u8>,
{
    let status_val = reg::spi_read_reg(spi, reg::ADDR_STATUS_REG).unwrap();
    if status_val & reg::BITMASK_STATUS_REG_GDA != reg::BITMASK_STATUS_REG_GDA {
        return Err(Asm330Error::DataNotReadyError());
    }
    let gyro_x_h = reg::spi_read_reg(spi, reg::ADDR_OUTX_H_G).unwrap() as i16;
    let gyro_x_l = reg::spi_read_reg(spi, reg::ADDR_OUTX_L_G).unwrap() as i16;
    let gyro_y_h = reg::spi_read_reg(spi, reg::ADDR_OUTY_H_G).unwrap() as i16;
    let gyro_y_l = reg::spi_read_reg(spi, reg::ADDR_OUTY_L_G).unwrap() as i16;
    let gyro_z_h = reg::spi_read_reg(spi, reg::ADDR_OUTZ_H_G).unwrap() as i16;
    let gyro_z_l = reg::spi_read_reg(spi, reg::ADDR_OUTZ_L_G).unwrap() as i16;
    Ok((gyro_x_h << 8 | gyro_x_l, gyro_y_h << 8 | gyro_y_l, gyro_z_h << 8 | gyro_z_l))
}

pub fn enable_timestamp<S>(spi: &mut S) -> Result<(), Asm330Error>
where
    S: SpiBus<u8>,
{
    reg::spi_write_reg(spi, reg::ADDR_CTRL10_C, reg::BITMASK_CTRL10_TIMESTAMP_EN).unwrap();
    Ok(())
}

pub fn disable_timestamp<S>(spi: &mut S) -> Result<(), Asm330Error>
where
    S: SpiBus<u8>,
{
    reg::spi_write_reg(spi, reg::ADDR_CTRL10_C, 0x00).unwrap();
    Ok(())
}

pub fn read_single_timestamp<S>(spi: &mut S) -> Result<u32, Asm330Error>
where
    S: SpiBus<u8>,
{
    let ts0 = reg::spi_read_reg(spi, reg::ADDR_TIMESTAMP0_REG).unwrap();
    let ts1 = reg::spi_read_reg(spi, reg::ADDR_TIMESTAMP1_REG).unwrap();
    let ts2 = reg::spi_read_reg(spi, reg::ADDR_TIMESTAMP2_REG).unwrap();
    let ts3 = reg::spi_read_reg(spi, reg::ADDR_TIMESTAMP3_REG).unwrap();
    let ts = (ts0 as u32) | (ts1 as u32) << 8 | (ts2 as u32) << 16 | (ts3 as u32) << 24;
    Ok(ts)
}

// pub fn configure_den<S>(
//     spi: &mut S,
//     enabled: bool,
//     stamp_on_accel: bool,
//     active_low: bool
//     ) -> Result<bool, S::Error>
// where
//     S: SpiBus<u8>,
//     S::Error: core::fmt::Debug,
// {
    
// }

#[cfg(debug_assertions)]
pub fn test_who_am_i<S>(spi: &mut S)
where
    S: SpiBus<u8>
{
    match is_who_am_i_good(spi) {
        Ok(pass) => {
            assert!(pass, "Who am I is bad");
        }
        Err(error) => {
            panic!("Read who am I failed: {:?}", error);
        }
    }
}

#[cfg(debug_assertions)]
pub fn test_configure_spi<S>(spi: &mut S)
where
    S: SpiBus<u8>
{
    configure_spi(spi).expect("configure_spi failed");
    match reg::spi_read_reg(spi, reg::ADDR_CTRL4_C) {
        Ok(val) => {
            assert_eq!(val, 0x04);
        }
        Err(error) => {
            panic!("Read ctrl4 register failed: {:?}", error);
        }
    }
    match reg::spi_read_reg(spi, reg::ADDR_CTRL9_XL) {
        Ok(val) => {
            assert_eq!(val, 0x00);
        }
        Err(err) => {
            panic!("Read ctrl9 register failed: {:?}", err);
        }
    }
}

#[cfg(debug_assertions)]
pub fn test_sw_reset<S>(spi: &mut S)
where
    S: SpiBus<u8>
{
    configure_spi(spi).expect("configure_spi failed");
    sw_reset(spi).expect("sw_reset failed");
    match reg::spi_read_reg(spi, reg::ADDR_CTRL4_C) {
        Ok(val) => {
            assert_eq!(val, 0x00);
        }
        Err(err) => {
            panic!("Read ctrl4 register failed: {:?}", err);
        }
    }
}

#[cfg(debug_assertions)]
pub fn test_enable_accel<S>(spi: &mut S)
where
    S: SpiBus<u8>
{
    let mut val = 0x00;
    enable_accel(
        spi,
        OdrFreq::Odr01040,
        AccelScale::Accel4G,
        AccelBandwidth::HighOdrBy10,
        false,
        true,
        false).expect("enable_accel failed");
    val = reg::spi_read_reg(spi, reg::ADDR_CTRL1_XL).expect("failed to read ctrl1 register");
    assert_eq!(val, 0x48);
    val = reg::spi_read_reg(spi, reg::ADDR_CTRL8_XL).expect("failed to read ctrl8 register");
    assert_eq!(val, 0b0010_1100);

    enable_accel(
        spi,
        OdrFreq::Odr16670,
        AccelScale::Accel8G,
        AccelBandwidth::LowOdrBy45,
        true,
        false,
        true).expect("enable_accel failed");
    val = reg::spi_read_reg(spi, reg::ADDR_CTRL1_XL).expect("failed to read ctrl1 register");
    assert_eq!(val, 0x8E);
    val = reg::spi_read_reg(spi, reg::ADDR_CTRL8_XL).expect("failed to read ctrl8 register");
    assert_eq!(val, 0b0111_0001);

    enable_accel(
        spi,
        OdrFreq::Odr00125,
        AccelScale::Accel2G,
        AccelBandwidth::LowOdrBy2,
        true,
        true,
        true).expect("enable_accel failed");
    val = reg::spi_read_reg(spi, reg::ADDR_CTRL1_XL).expect("failed to read ctrl1 register");
    assert_eq!(val, 0x10);
    val = reg::spi_read_reg(spi, reg::ADDR_CTRL8_XL).expect("failed to read ctrl8 register");
    assert_eq!(val, 0b0001_1001);
}

#[cfg(debug_assertions)]
pub fn test_disable_accel<S>(spi: &mut S)
where
    S: SpiBus<u8>
{
    enable_accel(
        spi,
        OdrFreq::Odr00125,
        AccelScale::Accel2G,
        AccelBandwidth::LowOdrBy2,
        true,
        true,
        true).expect("enable_accel failed");
    disable_accel(spi).expect("disable_accel failed");
    let ctrl1_val = reg::spi_read_reg(spi, reg::ADDR_CTRL1_XL).expect("failed to read ctrl1 register");
    assert_eq!(ctrl1_val, 0x00);
    let ctrl8_val = reg::spi_read_reg(spi, reg::ADDR_CTRL8_XL).expect("failed to read ctrl8 register");
    assert_eq!(ctrl8_val, 0x00);
}

#[cfg(debug_assertions)]
pub fn test_read_single_accel<S>(spi: &mut S)
where
    S: SpiBus<u8>
{
    sw_reset(spi).expect("sw_reset failed");
    read_single_accel(spi).expect("read_single_accel failed");
    match read_single_accel(spi) {
        Ok(_) => {
            panic!("read_single_accel saw data even though accel is off");
        }
        Err(err) => {}
    }
    enable_accel(
        spi,
        OdrFreq::Odr00125,
        AccelScale::Accel2G,
        AccelBandwidth::LowOdrBy2,
        false,
        false,
        false).expect("enable_accel failed");
    // discard first 2-3 samples according to application notes sec 3.5
    let mut rd_delay_start = Instant::now();
    while rd_delay_start.elapsed() < Duration::from_millis(100) {}
    let mut val = read_single_accel(spi).expect("read_single_accel_failed");
    rd_delay_start = Instant::now();
    while rd_delay_start.elapsed() < Duration::from_millis(100) {}
    val = read_single_accel(spi).expect("read_single_accel_failed");
    rd_delay_start = Instant::now();
    while rd_delay_start.elapsed() < Duration::from_millis(100) {}
    val = read_single_accel(spi).expect("read_single_accel_failed");
    // rd_delay_start = Instant::now();
    // loop {
    //     while rd_delay_start.elapsed() < Duration::from_millis(80) {}
    //     val = read_single_accel(spi).expect("read_single_accel_failed");
    //     // debug!("{:?}: (0x{:04X}, 0x{:04X}, 0x{:04X})", val, val.0, val.1, val.2);
    //     debug!("converted: ({:.2}mg, {:.2}mg, {:.2}mg)", val.0 as f64 * 0.0610, val.1 as f64 * 0.0610, val.2 as f64 * 0.0610);
    //     rd_delay_start = Instant::now();
    // }
}

#[cfg(debug_assertions)]
pub fn test_enable_gyro<S>(spi: &mut S)
where
    S: SpiBus<u8>
{
    let mut val = 0x00;
    enable_gyro(
        spi,
        OdrFreq::Odr00125,
        GyroScale::Gyro500Dps,
        GyroFType::GyroFT100,
        GyroHighPassCutoff::Gyro0016Hz,
        false,
        true,
        false).expect("enable_gyro failed");
    val = reg::spi_read_reg(spi, reg::ADDR_CTRL2_G).expect("failed to read ctrl2 register");
    assert_eq!(val, 0x14);
    val = reg::spi_read_reg(spi, reg::ADDR_CTRL7_G).expect("failed to read ctrl7 register");
    assert_eq!(val, 0x00);
    val = reg::spi_read_reg(spi, reg::ADDR_CTRL6_C).expect("failed to read ctrl6 register");
    assert_eq!(val, 0x04);
    val = reg::spi_read_reg(spi, reg::ADDR_CTRL4_C).expect("failed to read ctrl4 register");
    assert_eq!(val, 0x02);

    enable_gyro(
        spi,
        OdrFreq::Odr16670,
        GyroScale::Gyro125Dps,
        GyroFType::GyroFT000,
        GyroHighPassCutoff::Gyro0260Hz,
        true,
        false,
        false).expect("enable_gyro failed");
    val = reg::spi_read_reg(spi, reg::ADDR_CTRL2_G).expect("failed to read ctrl2 register");
    assert_eq!(val, 0x82);
    val = reg::spi_read_reg(spi, reg::ADDR_CTRL7_G).expect("failed to read ctrl7 register");
    assert_eq!(val, 0x60);
    val = reg::spi_read_reg(spi, reg::ADDR_CTRL6_C).expect("failed to read ctrl6 register");
    assert_eq!(val, 0x00);
    val = reg::spi_read_reg(spi, reg::ADDR_CTRL4_C).expect("failed to read ctrl4 register");
    assert_eq!(val, 0x00);

    enable_gyro(
        spi,
        OdrFreq::Odr02080,
        GyroScale::Gyro4000Dps,
        GyroFType::GyroFT010,
        GyroHighPassCutoff::Gyro1040Hz,
        true,
        true,
        false).expect("enable_gyro failed");
    val = reg::spi_read_reg(spi, reg::ADDR_CTRL2_G).expect("failed to read ctrl2 register");
    assert_eq!(val, 0x51);
    val = reg::spi_read_reg(spi, reg::ADDR_CTRL7_G).expect("failed to read ctrl7 register");
    assert_eq!(val, 0x70);
    val = reg::spi_read_reg(spi, reg::ADDR_CTRL6_C).expect("failed to read ctrl6 register");
    assert_eq!(val, 0x02);
    val = reg::spi_read_reg(spi, reg::ADDR_CTRL4_C).expect("failed to read ctrl4 register");
    assert_eq!(val, 0x02);

    enable_gyro(
        spi,
        OdrFreq::Odr02080,
        GyroScale::Gyro4000Dps,
        GyroFType::GyroFT010,
        GyroHighPassCutoff::Gyro1040Hz,
        false,
        false,
        true).expect("enable_gyro failed");
    val = reg::spi_read_reg(spi, reg::ADDR_CTRL4_C).expect("failed to read ctrl4 register");
    assert_eq!(val, 0x42);
}

#[cfg(debug_assertions)]
pub fn test_disable_gyro<S>(spi: &mut S)
where
    S: SpiBus<u8>
{
    disable_gyro(spi);
    let val = reg::spi_read_reg(spi, reg::ADDR_CTRL2_G).unwrap();
    assert_eq!(val, 0x00);
}

#[cfg(debug_assertions)]
pub fn test_read_single_gyro<S>(spi: &mut S)
where
    S: SpiBus<u8>
{
    sw_reset(spi).expect("sw_reset failed");
    read_single_gyro(spi).expect("read_single_gyro failed");
    match read_single_gyro(spi) {
        Ok(_) => {
            panic!("read_single_gyro saw data even though gyro is off");
        }
        Err(err) => {}
    }
    enable_gyro(
        spi,
        OdrFreq::Odr00125,
        GyroScale::Gyro250Dps,
        GyroFType::GyroFT010,
        GyroHighPassCutoff::Gyro1040Hz,
        true,
        true,
        false).expect("enable_gyro failed");
    // discard first 2-6 samples according to application notes sec 3.7
    let mut rd_delay_start = Instant::now();
    while rd_delay_start.elapsed() < Duration::from_millis(100) {}
    let mut val = read_single_gyro(spi).expect("read_single_gyro_failed");
    rd_delay_start = Instant::now();
    while rd_delay_start.elapsed() < Duration::from_millis(100) {}
    val = read_single_gyro(spi).expect("read_single_gyro_failed");
    rd_delay_start = Instant::now();
    while rd_delay_start.elapsed() < Duration::from_millis(100) {}
    val = read_single_gyro(spi).expect("read_single_gyro_failed");
    rd_delay_start = Instant::now();
    // loop {
    //     while rd_delay_start.elapsed() < Duration::from_millis(100) {}
    //     val = read_single_gyro(spi).expect("read_single_gyro_failed");
    //     debug!("converted: ({:.2}dps, {:.2}dps, {:.2}dps)", val.0 as f64 * 0.00875, val.1 as f64 * 0.00875, val.2 as f64 * 0.00875);
    //     rd_delay_start = Instant::now();
    // }
}

#[cfg(debug_assertions)]
pub fn test_single_timestamp<S>(spi: &mut S)
where
    S: SpiBus<u8>
{
    enable_timestamp(spi).expect("enable_timestamp failed");
    let mut val0 = read_single_timestamp(spi).expect("read_single_timestamp failed");
    let mut val1 = read_single_timestamp(spi).expect("read_single_timestamp failed");
    assert_ne!(val0, val1);
    disable_timestamp(spi).expect("disable_timestamp failed");
    val0 = read_single_timestamp(spi).expect("read_single_timestamp failed");
    val1 = read_single_timestamp(spi).expect("read_single_timestamp failed");
    assert_eq!(val0, val1);
    enable_timestamp(spi).expect("enable_timestamp failed");
    let mut rd_delay_start = Instant::now();
    loop {
        while rd_delay_start.elapsed() < Duration::from_millis(100) {}
        val0 = read_single_timestamp(spi).expect("read_single_timestamp failed");
        debug!("timestamp: {:.2}", (val0 as f32) * 0.000025);
        rd_delay_start = Instant::now();
    }
}

// #[cfg(debug_assertions)]
// pub fn test_enable_gyro_then_enable_accel<S>(spi: &mut S)
// where
//     S: SpiBus<u8>
// {
// }

#[cfg(debug_assertions)]
pub fn test_asm330<S>(spi: &mut S)
where
    S: SpiBus<u8>
{
    test_who_am_i(spi);
    test_configure_spi(spi);
    test_sw_reset(spi);
    test_enable_accel(spi);
    test_disable_accel(spi);
    test_read_single_accel(spi);
    test_enable_gyro(spi);
    test_disable_gyro(spi);
    test_read_single_gyro(spi);
    test_single_timestamp(spi);
    let mut imu_read_1hz = Instant::now();
    loop {
        if imu_read_1hz.elapsed() > Duration::from_secs(1) {
            imu_read_1hz = Instant::now();
        }
        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_millis(100) {}
    }
}

/*
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
*/

