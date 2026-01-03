#![no_std]
/*
* Created date: 12/20/25
* File Description: asm330 device state
*/

use embedded_hal::spi::SpiBus;

mod common;
mod reg;

use common::*;
use reg::*;

// using const vars instead of enum/struct because of const generic support
const LEAK_REG_UP: bool = true;
const LEAK_REG_DOWN: bool = false;
const LEAK_PORTION_UPPER: bool = true;
const LEAK_PORTION_LOWER: bool = false;

pub struct Field<
    const ADDR: u8,
    const OFFSET: u8,
    const WIDTH: u8,
    const DEFAULT: u16 = 0x0000,
    const LEAK_REG: bool = LEAK_REG_DOWN,
    const LEAK_PORTION: bool = LEAK_PORTION_LOWER,
> {
    /// A representation of a read/write field within the asm330 address space
    /// that changes the behavior of an asm330.
    addr: u8,
    value: u16,
}

impl<
        const ADDR: u8,
        const OFFSET: u8,
        const WIDTH: u8,
        const DEFAULT: u16,
        const LEAK_REG: bool,
        const LEAK_PORTION: bool,
    > Default for Field<ADDR, OFFSET, WIDTH, DEFAULT, LEAK_REG, LEAK_PORTION>
{
    fn default() -> Self {
        const {
            assert!(OFFSET <= 8, "Field offset must be less than 8");
            assert!(WIDTH > 0, "Field width must be greater than 0");
            assert!(
                DEFAULT < (0x0001 << WIDTH),
                "Field default exceeds max width"
            );
            Field { addr: ADDR, value: DEFAULT }
        }
    }
}

fn safe_write<S: SpiBus<u8>>(spi: &mut S, addr: u8, value: u8, mask: u8) -> Result<(), Asm330Error> {
    let mut reg_value = spi_read_reg(spi, addr)?;
    reg_value &= mask;
    reg_value |= value;
    spi_write_reg(spi, addr, reg_value)?;
    Ok(())
}

impl<
        const ADDR: u8,
        const OFFSET: u8,
        const WIDTH: u8,
        const DEFAULT: u16,
        const LEAK_REG: bool,
        const LEAK_PORTION: bool,
    > Field<ADDR, OFFSET, WIDTH, DEFAULT, LEAK_REG, LEAK_PORTION>
{
    pub const fn new<const VALUE: u16>() -> Self {
        const {
            assert!(OFFSET <= 8, "Field offset must be less than 8");
            assert!(WIDTH > 0, "Field width must be greater than 0");
            assert!(VALUE < (1 << WIDTH), "New Field value exceeds max width");
        }
        Self { addr: ADDR, value: VALUE }
    }

    pub const fn set<const VALUE: u16>(&mut self) {
        const {
            assert!(OFFSET <= 8, "Field offset must be less than 8");
            assert!(WIDTH > 0, "Field width must be greater than 0");
            assert!(VALUE < (1 << WIDTH), "New Field value exceeds max width");
        }
        self.value = VALUE;
    }

    pub fn write<S: SpiBus<u8>>(&self, spi: &mut S) -> Result<(), Asm330Error> {
        if WIDTH > 16 {
            panic!("not implemented");
        } else if WIDTH > 8 {
            let mut upper: u8 = (self.value >> 8) as u8;
            let lower: u8 = (self.value & 0x00ff) as u8;
            spi_write_reg(spi, ADDR, lower)?;
            let mut mask = 0xff >> (16-WIDTH);
            if LEAK_PORTION == LEAK_PORTION_UPPER {
                upper <<= 16 - WIDTH;
                mask <<= 16 - WIDTH;
            }
            if LEAK_REG == LEAK_REG_DOWN {
                safe_write(spi, ADDR - 1, upper, mask)?;
            } else {
                safe_write(spi, ADDR + 1, upper, mask)?;
            }
            return Ok(());
        } else {
            let mask = (0xff >> (8-WIDTH)) << OFFSET;
            safe_write(spi, ADDR, (self.value as u8) << OFFSET, mask)?;
            return Ok(());
        }
    }

    pub fn read<S: SpiBus<u8>>(&self, spi: &mut S) -> Result<u16, Asm330Error> {
        if WIDTH > 16 {
            panic!("not implemented");
        } else if WIDTH > 8 {
            let lower = spi_read_reg(spi, ADDR)?;
            let mut upper = 0;
            if LEAK_REG == LEAK_REG_DOWN {
                upper = spi_read_reg(spi, ADDR - 1)?;
            } else {
                upper = spi_read_reg(spi, ADDR + 1)?;
            }
            if LEAK_PORTION == LEAK_PORTION_UPPER {
                upper >>= 16 - WIDTH;
            }
            let big_upper = (upper as u16) << 8;
            let big_lower = lower as u16;
            let bitmask = 0xffff >> 16 - WIDTH;
            return Ok((big_upper | big_lower) & bitmask);
        } else {
            let value = spi_read_reg(spi, ADDR)?;
            return Ok(value as u16);
        }
    }

    pub fn read_from_unshifted(value: u8) -> u8 {
        if WIDTH > 16 {
            panic!("not implemented");
        } else if WIDTH > 8 {
            panic!("not implemented");
        } else {
            let mask = (0xff >> (8-WIDTH)) << OFFSET;
            let masked = value & mask;
            return masked;
        }
    }

    pub fn read_from(value: u8) -> u8 {
        Self::read_from_unshifted(value) >> OFFSET
    }
}

trait Config {
    /// A collection of Fields that modify the behavior of a unit within
    /// an asm330.
    fn write_config<S: SpiBus<u8>>(&mut self, spi: &mut S) -> Result<(), Asm330Error>;
}

#[derive(Default)]
pub struct SystemConfig {
    pub drdy_mask: Field<ADDR_CTRL4_C, 3, 1>,
    pub i2c_disable: Field<ADDR_CTRL4_C, 2, 1>,
    pub sdo_pu_en: Field<ADDR_PIN_CTRL, 6, 1>,
    pub boot: Field<ADDR_CTRL3_C, 7, 1>,
    pub bdu: Field<ADDR_CTRL3_C, 6, 1>,
    pub pp_od: Field<ADDR_CTRL3_C, 4, 1>,
    pub sim: Field<ADDR_CTRL3_C, 3, 1>,
    pub if_inc: Field<ADDR_CTRL3_C, 2, 1, 0x0001>,
    pub sw_reset: Field<ADDR_CTRL3_C, 0, 1>,
    pub rounding: Field<ADDR_CTRL5_C, 5, 2>,
    pub st_g: Field<ADDR_CTRL5_C, 2, 2>,
    pub st_xl: Field<ADDR_CTRL5_C, 0, 2>,
    pub device_conf: Field<ADDR_CTRL9_XL, 1, 1>,
}

impl Config for SystemConfig {
    fn write_config<S: SpiBus<u8>>(&mut self, spi: &mut S) -> Result<(), Asm330Error> {
        panic!("not implemented")
    }
}

#[derive(Default)]
pub struct InterruptConfig {
    pub int2_on_int1: Field<ADDR_CTRL4_C, 5, 1>,
    pub den_drdy_flag: Field<ADDR_INT1_CTRL, 7, 1>,
    pub int1_cnt_bdr: Field<ADDR_INT1_CTRL, 6, 1>,
    pub int1_fifo_full: Field<ADDR_INT1_CTRL, 5, 1>,
    pub int1_fifo_ovr: Field<ADDR_INT1_CTRL, 4, 1>,
    pub int1_fifo_th: Field<ADDR_INT1_CTRL, 3, 1>,
    pub int1_boot: Field<ADDR_INT1_CTRL, 2, 1>,
    pub int1_drdy_g: Field<ADDR_INT1_CTRL, 1, 1>,
    pub int1_drdy_xl: Field<ADDR_INT1_CTRL, 0, 1>,
    pub int2_cnt_bdr: Field<ADDR_INT1_CTRL, 6, 1>,
    pub int2_fifo_full: Field<ADDR_INT1_CTRL, 5, 1>,
    pub int2_fifo_ovr: Field<ADDR_INT1_CTRL, 4, 1>,
    pub int2_fifo_th: Field<ADDR_INT1_CTRL, 3, 1>,
    pub int2_drdy_temp: Field<ADDR_INT1_CTRL, 2, 1>,
    pub int2_drdy_g: Field<ADDR_INT1_CTRL, 1, 1>,
    pub int2_drdy_xl: Field<ADDR_INT1_CTRL, 0, 1>,
    pub h_lactive: Field<ADDR_CTRL3_C, 5, 1>,
    pub int_clr_on_read: Field<ADDR_INT_CFG0, 6, 1>,
    pub sleep_status_on_int: Field<ADDR_INT_CFG0, 5, 1>,
    pub lir: Field<ADDR_INT_CFG0, 0, 1>,
    pub interrupts_enable: Field<ADDR_INT_CFG1, 7, 1>,
    pub int1_sleep_change: Field<ADDR_MD1_CFG, 7, 1>,
    pub int1_wu: Field<ADDR_MD1_CFG, 5, 1>,
    pub int1_ff: Field<ADDR_MD1_CFG, 4, 1>,
    pub int1_6d: Field<ADDR_MD1_CFG, 2, 1>,
    pub int2_sleep_change: Field<ADDR_MD2_CFG, 7, 1>,
    pub int2_wu: Field<ADDR_MD2_CFG, 5, 1>,
    pub int2_ff: Field<ADDR_MD2_CFG, 4, 1>,
    pub int2_6d: Field<ADDR_MD2_CFG, 2, 1>,
    pub int2_timestamp: Field<ADDR_MD2_CFG, 0, 1>,
}

impl Config for InterruptConfig {
    fn write_config<S: SpiBus<u8>>(&mut self, spi: &mut S) -> Result<(), Asm330Error> {
        panic!("not implemented")
    }
}

#[derive(Default)]
pub struct EventConfig {
    pub slope_fds: Field<ADDR_INT_CFG0, 4, 1>,
    pub inact_en: Field<ADDR_INT_CFG1, 5, 2>,
    pub d4d_en: Field<ADDR_THS_6D, 7, 1>,
    pub sixd_ths: Field<ADDR_THS_6D, 5, 2>,
    pub usr_off_on_wu: Field<ADDR_WAKE_UP_THS, 6, 1>,
    pub wk_ths: Field<ADDR_WAKE_UP_THS, 0, 6>,
    pub wake_dur: Field<ADDR_WAKE_UP_DUR, 5, 2>,
    pub wake_ths_w: Field<ADDR_WAKE_UP_DUR, 4, 1>,
    pub sleep_dur: Field<ADDR_WAKE_UP_DUR, 0, 4>,
    pub ff_dur: Field<ADDR_FREE_FALL, 3, 6, 0x0000, LEAK_REG_DOWN, LEAK_PORTION_UPPER>,
    pub ff_ths: Field<ADDR_FREE_FALL, 0, 3>,
}

impl Config for EventConfig {
    fn write_config<S: SpiBus<u8>>(&mut self, spi: &mut S) -> Result<(), Asm330Error> {
        panic!("not implemented")
    }
}

#[derive(Default)]
pub struct DenConfig {
    pub den_x: Field<ADDR_CTRL9_XL, 7, 1, 0x1>,
    pub den_y: Field<ADDR_CTRL9_XL, 6, 1, 0x1>,
    pub den_z: Field<ADDR_CTRL9_XL, 5, 1, 0x1>,
    pub den_xl_g: Field<ADDR_CTRL9_XL, 4, 1>,
    pub den_xl_en: Field<ADDR_CTRL9_XL, 3, 1>,
    pub den_lh: Field<ADDR_CTRL9_XL, 2, 1>,
    pub trig_en: Field<ADDR_CTRL6_C, 7, 1>,
    pub lvl1_en: Field<ADDR_CTRL6_C, 6, 1>,
    pub lvl2_en: Field<ADDR_CTRL6_C, 5, 1>,
}

impl Config for DenConfig {
    fn write_config<S: SpiBus<u8>>(&mut self, spi: &mut S) -> Result<(), Asm330Error> {
        panic!("not implemented")
    }
}

#[derive(Default)]
pub struct XLConfig {
    pub odr_xl: Field<ADDR_CTRL1_XL, 4, 4>,
    pub fs_xl: Field<ADDR_CTRL1_XL, 2, 2>,
    pub lpf2_xl_en: Field<ADDR_CTRL1_XL, 1, 1>,
    pub hpcf_xl: Field<ADDR_CTRL8_XL, 5, 3>,
    pub hp_ref_mode_xl: Field<ADDR_CTRL8_XL, 4, 1>,
    pub fast_settle_mode_xl: Field<ADDR_CTRL8_XL, 3, 1>,
    pub hp_slope_xl_en: Field<ADDR_CTRL8_XL, 2, 1>,
    pub low_pass_on_6d: Field<ADDR_CTRL8_XL, 0, 1>,
    pub usr_off_w: Field<ADDR_CTRL6_C, 3, 1>,
    pub usr_off_on_out: Field<ADDR_CTRL6_C, 1, 1>,
    pub x_ofs_usr: Field<ADDR_X_OFS_USR, 0, 8>,
    pub y_ofs_usr: Field<ADDR_Y_OFS_USR, 0, 8>,
    pub z_ofs_usr: Field<ADDR_Z_OFS_USR, 0, 8>,
}

impl Config for XLConfig {
    fn write_config<S: SpiBus<u8>>(&mut self, spi: &mut S) -> Result<(), Asm330Error> {
        panic!("not implemented")
    }
}

#[derive(Default)]
pub struct GyroConfig {
    pub odr_g: Field<ADDR_CTRL2_G, 4, 4>,
    pub fs_g: Field<ADDR_CTRL2_G, 2, 2>,
    pub fs_125: Field<ADDR_CTRL2_G, 1, 1>,
    pub fs_4000: Field<ADDR_CTRL2_G, 0, 1>,
    pub hp_en_g: Field<ADDR_CTRL7_G, 6, 1>,
    pub hpm_g: Field<ADDR_CTRL7_G, 4, 2>,
    pub ftype: Field<ADDR_CTRL6_C, 0, 3>,
    pub lpf1_sel_g: Field<ADDR_CTRL4_C, 1, 1>,
    pub sleep_g: Field<ADDR_CTRL4_C, 6, 1>,
}

impl Config for GyroConfig {
    fn write_config<S: SpiBus<u8>>(&mut self, spi: &mut S) -> Result<(), Asm330Error> {
        panic!("not implemented")
    }
}

#[derive(Default)]
pub struct ThermoConfig {}

impl Config for ThermoConfig {
    fn write_config<S: SpiBus<u8>>(&mut self, spi: &mut S) -> Result<(), Asm330Error> {
        panic!("not implemented")
    }
}

#[derive(Default)]
pub struct TimeConfig {
    pub timestamp_en: Field<ADDR_CTRL10_C, 5, 1>,
}

impl Config for TimeConfig {
    fn write_config<S: SpiBus<u8>>(&mut self, spi: &mut S) -> Result<(), Asm330Error> {
        panic!("not implemented")
    }
}

#[derive(Default)]
pub struct FifoConfig {
    pub wtm: Field<ADDR_FIFO_CTRL1, 0, 9, 0x0000, LEAK_REG_UP, LEAK_PORTION_LOWER>,
    pub odrchg_en: Field<ADDR_FIFO_CTRL2, 4, 1>,
    pub stop_on_wtm: Field<ADDR_FIFO_CTRL2, 7, 1>,
    pub bdr_gy: Field<ADDR_FIFO_CTRL3, 4, 4>,
    pub bdr_xl: Field<ADDR_FIFO_CTRL3, 0, 4>,
    pub dec_ts_batch: Field<ADDR_FIFO_CTRL4, 6, 2>,
    pub odr_t_batch: Field<ADDR_FIFO_CTRL4, 4, 2>,
    pub fifo_mode: Field<ADDR_FIFO_CTRL4, 0, 3>,
    pub dataready_pulsed: Field<ADDR_COUNTER_BDR_REG1, 7, 1>,
    pub rst_counter_bdr: Field<ADDR_COUNTER_BDR_REG1, 6, 1>,
    pub trig_counter_bdr: Field<ADDR_COUNTER_BDR_REG1, 5, 1>,
    pub cnt_bdr_th:
        Field<ADDR_COUNTER_BDR_REG2, 0, 10, 0x0000, LEAK_REG_DOWN, LEAK_PORTION_LOWER>,
}

impl Config for FifoConfig {
    fn write_config<S: SpiBus<u8>>(&mut self, spi: &mut S) -> Result<(), Asm330Error> {
        panic!("not implemented")
    }
}

#[derive(Default)]
pub struct DeviceConfig {
    sys_conf: SystemConfig,
    xl_conf: XLConfig,
    gryo_conf: GyroConfig,
    thermo_conf: ThermoConfig,
    time_conf: TimeConfig,
    fifo_conf: FifoConfig,
}

impl Config for DeviceConfig {
    fn write_config<S: SpiBus<u8>>(&mut self, spi: &mut S) -> Result<(), Asm330Error> {
        self.sys_conf.write_config(spi)?;
        self.xl_conf.write_config(spi)?;
        self.gryo_conf.write_config(spi)?;
        self.thermo_conf.write_config(spi)?;
        self.time_conf.write_config(spi)?;
        self.fifo_conf.write_config(spi)?;
        Ok(())
    }
}

fn read_u8<S: SpiBus<u8>, const A: u8, const O: u8, const W: u8>(spi: &mut S, field: &Field<A, O, W>) -> Result<u8, Asm330Error> {
    spi_read_reg(spi, A)
}

fn read_i8<S: SpiBus<u8>, const A: u8, const O: u8, const W: u8>(spi: &mut S, field: &Field<A, O, W>) -> Result<i8, Asm330Error> {
    Ok(spi_read_reg(spi, A)? as i8)
}

fn read_i16<S: SpiBus<u8>, const A0: u8, const O0: u8, const W0: u8, const A1: u8, const O1: u8, const W1: u8>(spi: &mut S, lower_field: &Field<A0, O0, W0>, upper_field: &Field<A1, O1, W1>) -> Result<i16, Asm330Error> {
    let l = spi_read_reg(spi, A0)? as i16;
    let u = spi_read_reg(spi, A1)? as i16;
    Ok((u << 8) | l)
}

fn read_u32<S: SpiBus<u8>, const A0: u8, const O0: u8, const W0: u8, const A1: u8, const O1: u8, const W1: u8, const A2: u8, const O2: u8, const W2: u8, const A3: u8, const O3: u8, const W3: u8>(spi: &mut S, field0: &Field<A0, O0, W0>, field1: &Field<A1, O1, W1>, field2: &Field<A2, O2, W2>, field3: &Field<A3, O3, W3>) -> Result<u32, Asm330Error> {
    let v0 = spi_read_reg(spi, A0)? as u32;
    let v1 = spi_read_reg(spi, A1)? as u32;
    let v2 = spi_read_reg(spi, A2)? as u32;
    let v3 = spi_read_reg(spi, A3)? as u32;
    Ok(v0 | (v1 << 8) | (v2 << 16) | (v3 << 24))
}

pub struct SensorReader {
    outx_h_a: Field<ADDR_OUTX_H_A, 0, 8>,
    outx_l_a: Field<ADDR_OUTX_L_A, 0, 8>,
    outy_h_a: Field<ADDR_OUTY_H_A, 0, 8>,
    outy_l_a: Field<ADDR_OUTY_L_A, 0, 8>,
    outz_h_a: Field<ADDR_OUTZ_H_A, 0, 8>,
    outz_l_a: Field<ADDR_OUTZ_L_A, 0, 8>,
    outx_h_g: Field<ADDR_OUTX_H_G, 0, 8>,
    outx_l_g: Field<ADDR_OUTX_L_G, 0, 8>,
    outy_h_g: Field<ADDR_OUTY_H_G, 0, 8>,
    outy_l_g: Field<ADDR_OUTY_L_G, 0, 8>,
    outz_h_g: Field<ADDR_OUTZ_H_G, 0, 8>,
    outz_l_g: Field<ADDR_OUTZ_L_G, 0, 8>,
    timestamp0: Field<ADDR_TIMESTAMP0_REG, 0, 8>,
    timestamp1: Field<ADDR_TIMESTAMP1_REG, 0, 8>,
    timestamp2: Field<ADDR_TIMESTAMP2_REG, 0, 8>,
    timestamp3: Field<ADDR_TIMESTAMP3_REG, 0, 8>,
    out_temp_h: Field<ADDR_OUT_TEMP_H, 0, 8>,
    out_temp_l: Field<ADDR_OUT_TEMP_L, 0, 8>,
    pub tda: Field<ADDR_STATUS_REG, 2, 1>,
    pub gda: Field<ADDR_STATUS_REG, 1, 1>,
    pub xlda: Field<ADDR_STATUS_REG, 0, 1>,
    pub freq_fine: Field<ADDR_INTERNAL_FREQ_FINE, 0, 8>,
}

impl SensorReader {
    pub fn read_xl<S: SpiBus<u8>>(&self, spi: &mut S) -> Result<(i16, i16, i16), Asm330Error> {
        Ok((
            read_i16(spi, &self.outx_l_a, &self.outx_h_a)?,
            read_i16(spi, &self.outy_l_a, &self.outy_h_a)?,
            read_i16(spi, &self.outz_l_a, &self.outz_h_a)?,
        ))
    }

    pub fn read_g<S: SpiBus<u8>>(&self, spi: &mut S) -> Result<(i16, i16, i16), Asm330Error> {
        Ok((
            read_i16(spi, &self.outx_l_g, &self.outx_h_g)?,
            read_i16(spi, &self.outy_l_g, &self.outy_h_g)?,
            read_i16(spi, &self.outz_l_g, &self.outz_h_g)?,
        ))
    }

    pub fn read_timestamp<S: SpiBus<u8>>(&self, spi: &mut S) -> Result<u32, Asm330Error> {
        Ok(read_u32(spi, &self.timestamp0, &self.timestamp1, &self.timestamp2, &self.timestamp3)?)
    }

    pub fn read_temp<S: SpiBus<u8>>(&self, spi: &mut S) -> Result<i16, Asm330Error> {
        Ok(read_i16(spi, &self.out_temp_l, &self.out_temp_h)?)
    }

    pub fn read_freq_fine<S: SpiBus<u8>>(&self, spi: &mut S) -> Result<i8, Asm330Error> {
        Ok(read_i8(spi, &self.freq_fine)?)
    }
}

pub struct FifoReader {
    diff_fifo: Field<ADDR_FIFO_STATUS1, 0, 10, 0x0, LEAK_REG_UP, LEAK_PORTION_LOWER>,
    fifo_wtm_ia: Field<ADDR_FIFO_STATUS2, 7, 1>,
    fifo_ovr_ia: Field<ADDR_FIFO_STATUS2, 6, 1>,
    fifo_full_ia: Field<ADDR_FIFO_STATUS2, 5, 1>,
    counter_bdr_ia: Field<ADDR_FIFO_STATUS2, 4, 1>,
    fifo_ovr_latched: Field<ADDR_FIFO_STATUS2, 3, 1>,
    tag_sensor: Field<ADDR_FIFO_DATA_OUT_TAG, 3, 5>,
    tag_cnt: Field<ADDR_FIFO_DATA_OUT_TAG, 1, 2>,
    tag_parity: Field<ADDR_FIFO_DATA_OUT_TAG, 0, 1>,
    fifo_data_out_x_h: Field<ADDR_FIFO_DATA_OUT_X_H, 0, 8>,
    fifo_data_out_x_l: Field<ADDR_FIFO_DATA_OUT_X_L, 0, 8>,
    fifo_data_out_y_h: Field<ADDR_FIFO_DATA_OUT_Y_H, 0, 8>,
    fifo_data_out_y_l: Field<ADDR_FIFO_DATA_OUT_Y_L, 0, 8>,
    fifo_data_out_z_h: Field<ADDR_FIFO_DATA_OUT_Z_H, 0, 8>,
    fifo_data_out_z_l: Field<ADDR_FIFO_DATA_OUT_Z_L, 0, 8>,
}

pub enum FifoData {
    Accel(i16, i16, i16),
    Gyro(i16, i16, i16),
    Temp(i16),
    Time(u32),
    CfgChange(u64),

}

impl FifoReader {
    pub fn read_single<S: SpiBus<u8>>(&self, spi: &mut S) -> Result<FifoData, Asm330Error> {
        panic!("not implemented")
    }

    // pub fn read_n<S: SpiBus<u8>>(&self, spi: &mut S, n: u16) -> Result<
}

pub struct InterruptReader {
    // including prefix because ALL_INT_SRC, WAKE_UP_SRC, and D6D_SRC registers
    // have conflicts in field names
    pub all_timestamp_end_count: Field<ADDR_ALL_INT_SRC, 7, 1>,
    pub all_sleep_change_ia: Field<ADDR_ALL_INT_SRC, 5, 1>,
    pub all_d6d_ia: Field<ADDR_ALL_INT_SRC, 4, 1>,
    pub all_wu_ia: Field<ADDR_ALL_INT_SRC, 1, 1>,
    pub all_ff_ia: Field<ADDR_ALL_INT_SRC, 0, 1>,
    pub wake_sleep_change_ia: Field<ADDR_WAKE_UP_SRC, 6, 1>,
    pub wake_ff_ia: Field<ADDR_WAKE_UP_SRC, 5, 1>,
    pub wake_sleep_state: Field<ADDR_WAKE_UP_SRC, 4, 1>,
    pub wake_wu_ia: Field<ADDR_WAKE_UP_SRC, 3, 1>,
    pub wake_x_wu: Field<ADDR_WAKE_UP_SRC, 2, 1>,
    pub wake_y_wu: Field<ADDR_WAKE_UP_SRC, 1, 1>,
    pub wake_z_wu: Field<ADDR_WAKE_UP_SRC, 0, 1>,
    pub d6d_den_drdy: Field<ADDR_D6D_SRC, 7, 1>,
    pub d6d_d6d_ia: Field<ADDR_D6D_SRC, 6, 1>,
    pub d6d_zh: Field<ADDR_D6D_SRC, 5, 1>,
    pub d6d_zl: Field<ADDR_D6D_SRC, 4, 1>,
    pub d6d_yh: Field<ADDR_D6D_SRC, 3, 1>,
    pub d6d_yl: Field<ADDR_D6D_SRC, 2, 1>,
    pub d6d_xh: Field<ADDR_D6D_SRC, 1, 1>,
    pub d6d_xl: Field<ADDR_D6D_SRC, 0, 1>,
}

impl InterruptReader {
    pub fn read_all_int_src<S: SpiBus<u8>>(&self, spi: &mut S) -> Result<u8, Asm330Error> {
        read_u8(spi, &self.all_timestamp_end_count)
    }

    pub fn read_wake_up_src<S: SpiBus<u8>>(&self, spi: &mut S) -> Result<u8, Asm330Error> {
        read_u8(spi, &self.wake_sleep_change_ia)
    }

    pub fn read_d6d_src<S: SpiBus<u8>>(&self, spi: &mut S) -> Result<u8, Asm330Error> {
        read_u8(spi, &self.d6d_den_drdy)
    }
}

pub struct DataReader {
    pub sensor: SensorReader,
    pub fifo: FifoReader,
    pub interrupt: InterruptReader,
}

pub struct Device<S: SpiBus<u8>> {
    pub device_conf: DeviceConfig,
    pub data_reader: DataReader,
    pub spi: S,
}

impl<S: SpiBus<u8>> Device<S> {
    // fn power_off() -> Result<(), Asm330Error> {
    //     Ok(())
    // }
    
    // fn read_single_accel(&mut self) -> Result<(i16, i16, i16), Asm330Error> {
    //     let status_val = spi_read_reg(spi, ADDR_STATUS_REG).unwrap();
    //     if status_val & BITMASK_STATUS_REG_XLDA != BITMASK_STATUS_REG_XLDA {
    //         return Err(Asm330Error::DataNotReadyError());
    //     }
    //     let accel_x_h = spi_read_reg(spi, ADDR_OUTX_H_A).unwrap() as i16;
    //     let accel_x_l = spi_read_reg(spi, ADDR_OUTX_L_A).unwrap() as i16;
    //     let accel_y_h = spi_read_reg(spi, ADDR_OUTY_H_A).unwrap() as i16;
    //     let accel_y_l = spi_read_reg(spi, ADDR_OUTY_L_A).unwrap() as i16;
    //     let accel_z_h = spi_read_reg(spi, ADDR_OUTZ_H_A).unwrap() as i16;
    //     let accel_z_l = spi_read_reg(spi, ADDR_OUTZ_L_A).unwrap() as i16;
    //     Ok((accel_x_h << 8 | accel_x_l, accel_y_h << 8 | accel_y_l, accel_z_h << 8 | accel_z_l))
    // }
    
    // fn read_single_gyro() -> Result<(i16, i16, i16), Asm330Error> {
    //     Ok((0, 0, 0))
    // }
    
    // fn read_single_temp() -> Result<i16, Asm330Error> {
    //     Ok(0)
    // }
    
    // fn read_single_time(&self) -> Result<u32, Asm330Error> {
    //     let ts0 = spi_read_reg(&self.spi, ADDR_TIMESTAMP0_REG).unwrap();
    //     let ts1 = spi_read_reg(&self.spi, ADDR_TIMESTAMP1_REG).unwrap();
    //     let ts2 = spi_read_reg(&self.spi, ADDR_TIMESTAMP2_REG).unwrap();
    //     let ts3 = spi_read_reg(&self.spi, ADDR_TIMESTAMP3_REG).unwrap();
    //     let ts = (ts0 as u32) | (ts1 as u32) << 8 | (ts2 as u32) << 16 | (ts3 as u32) << 24;
    //     Ok(ts)
    // }

    // fn empty_fifo() -> Result<Vec<;
}

/* Field
 * variables: address, offset, width, value
 * functions: set, get
*/

/* config
 * variables: each config?
 * functions: write_config
*/

/* Device
 * variables: running
 * functions:
 * write_config
 * stop
 * read_gyro
 * read_accel
 * read_temp
 * read_time
*/
