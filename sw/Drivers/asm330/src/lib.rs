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

pub struct Setting<
    const ADDR: u8,
    const OFFSET: u8,
    const WIDTH: u8,
    const DEFAULT: u16 = 0x0000,
    const LEAK_REG: bool = LEAK_REG_DOWN,
    const LEAK_PORTION: bool = LEAK_PORTION_LOWER,
> {
    /// A representation of a read/write field within the asm330 address space
    /// that changes the behavior of an asm330.
    value: u16,
}

impl<
        const ADDR: u8,
        const OFFSET: u8,
        const WIDTH: u8,
        const DEFAULT: u16,
        const LEAK_REG: bool,
        const LEAK_PORTION: bool,
    > Default for Setting<ADDR, OFFSET, WIDTH, DEFAULT, LEAK_REG, LEAK_PORTION>
{
    fn default() -> Self {
        const {
            assert!(OFFSET <= 8, "Setting offset must be less than 8");
            assert!(WIDTH > 0, "Setting width must be greater than 0");
            assert!(
                DEFAULT < (0x0001 << WIDTH),
                "Setting default exceeds max width"
            );
            Setting { value: DEFAULT }
        }
    }
}

impl<
        const ADDR: u8,
        const OFFSET: u8,
        const WIDTH: u8,
        const DEFAULT: u16,
        const LEAK_REG: bool,
        const LEAK_PORTION: bool,
    > Setting<ADDR, OFFSET, WIDTH, DEFAULT, LEAK_REG, LEAK_PORTION>
{
    pub const fn new<const VALUE: u16>() -> Self {
        const {
            assert!(OFFSET <= 8, "Setting offset must be less than 8");
            assert!(WIDTH > 0, "Setting width must be greater than 0");
            assert!(VALUE < (1 << WIDTH), "New setting value exceeds max width");
        }
        Self { value: VALUE }
    }

    pub const fn set<const VALUE: u16>(&mut self) {
        const {
            assert!(OFFSET <= 8, "Setting offset must be less than 8");
            assert!(WIDTH > 0, "Setting width must be greater than 0");
            assert!(VALUE < (1 << WIDTH), "New setting value exceeds max width");
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
            if LEAK_PORTION == LEAK_PORTION_UPPER {
                upper <<= 16 - WIDTH;
            }
            if LEAK_REG == LEAK_REG_DOWN {
                spi_write_reg(spi, ADDR - 1, upper);
            } else {
                spi_write_reg(spi, ADDR + 1, upper)?;
            }
            return Ok(());
        } else {
            spi_write_reg(spi, ADDR, (self.value & 0x00ff) as u8)?;
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
}

trait Config {
    /// A collection of settings that modify the behavior of a unit within
    /// an asm330.
    fn write_config<S: SpiBus<u8>>(&mut self, spi: &mut S) -> Result<(), Asm330Error>;
}

#[derive(Default)]
pub struct SystemConfig {
    pub drdy_mask: Setting<ADDR_CTRL4_C, 3, 1>,
    pub i2c_disable: Setting<ADDR_CTRL4_C, 2, 1>,
    pub sdo_pu_en: Setting<ADDR_PIN_CTRL, 6, 1>,
    pub boot: Setting<ADDR_CTRL3_C, 7, 1>,
    pub bdu: Setting<ADDR_CTRL3_C, 6, 1>,
    pub pp_od: Setting<ADDR_CTRL3_C, 4, 1>,
    pub sim: Setting<ADDR_CTRL3_C, 3, 1>,
    pub if_inc: Setting<ADDR_CTRL3_C, 2, 1, 0x0001>,
    pub sw_reset: Setting<ADDR_CTRL3_C, 0, 1>,
    pub rounding: Setting<ADDR_CTRL5_C, 5, 2>,
    pub st_g: Setting<ADDR_CTRL5_C, 2, 2>,
    pub st_xl: Setting<ADDR_CTRL5_C, 0, 2>,
    pub device_conf: Setting<ADDR_CTRL9_XL, 1, 1>,
}

impl Config for SystemConfig {
    fn write_config<S: SpiBus<u8>>(&mut self, spi: &mut S) -> Result<(), Asm330Error> {
        panic!("not implemented")
    }
}

#[derive(Default)]
pub struct InterruptConfig {
    pub int2_on_int1: Setting<ADDR_CTRL4_C, 5, 1>,
    pub den_drdy_flag: Setting<ADDR_INT1_CTRL, 7, 1>,
    pub int1_cnt_bdr: Setting<ADDR_INT1_CTRL, 6, 1>,
    pub int1_fifo_full: Setting<ADDR_INT1_CTRL, 5, 1>,
    pub int1_fifo_ovr: Setting<ADDR_INT1_CTRL, 4, 1>,
    pub int1_fifo_th: Setting<ADDR_INT1_CTRL, 3, 1>,
    pub int1_boot: Setting<ADDR_INT1_CTRL, 2, 1>,
    pub int1_drdy_g: Setting<ADDR_INT1_CTRL, 1, 1>,
    pub int1_drdy_xl: Setting<ADDR_INT1_CTRL, 0, 1>,
    pub int2_cnt_bdr: Setting<ADDR_INT1_CTRL, 6, 1>,
    pub int2_fifo_full: Setting<ADDR_INT1_CTRL, 5, 1>,
    pub int2_fifo_ovr: Setting<ADDR_INT1_CTRL, 4, 1>,
    pub int2_fifo_th: Setting<ADDR_INT1_CTRL, 3, 1>,
    pub int2_drdy_temp: Setting<ADDR_INT1_CTRL, 2, 1>,
    pub int2_drdy_g: Setting<ADDR_INT1_CTRL, 1, 1>,
    pub int2_drdy_xl: Setting<ADDR_INT1_CTRL, 0, 1>,
    pub h_lactive: Setting<ADDR_CTRL3_C, 5, 1>,
    pub int_clr_on_read: Setting<ADDR_INT_CFG0, 6, 1>,
    pub sleep_status_on_int: Setting<ADDR_INT_CFG0, 5, 1>,
    pub lir: Setting<ADDR_INT_CFG0, 0, 1>,
    pub interrupts_enable: Setting<ADDR_INT_CFG1, 7, 1>,
    pub int1_sleep_change: Setting<ADDR_MD1_CFG, 7, 1>,
    pub int1_wu: Setting<ADDR_MD1_CFG, 5, 1>,
    pub int1_ff: Setting<ADDR_MD1_CFG, 4, 1>,
    pub int1_6d: Setting<ADDR_MD1_CFG, 2, 1>,
    pub int2_sleep_change: Setting<ADDR_MD2_CFG, 7, 1>,
    pub int2_wu: Setting<ADDR_MD2_CFG, 5, 1>,
    pub int2_ff: Setting<ADDR_MD2_CFG, 4, 1>,
    pub int2_6d: Setting<ADDR_MD2_CFG, 2, 1>,
    pub int2_timestamp: Setting<ADDR_MD2_CFG, 0, 1>,
}

impl Config for InterruptConfig {
    fn write_config<S: SpiBus<u8>>(&mut self, spi: &mut S) -> Result<(), Asm330Error> {
        panic!("not implemented")
    }
}

#[derive(Default)]
pub struct EventConfig {
    pub slope_fds: Setting<ADDR_INT_CFG0, 4, 1>,
    pub inact_en: Setting<ADDR_INT_CFG1, 5, 2>,
    pub d4d_en: Setting<ADDR_THS_6D, 7, 1>,
    pub sixd_ths: Setting<ADDR_THS_6D, 5, 2>,
    pub usr_off_on_wu: Setting<ADDR_WAKE_UP_THS, 6, 1>,
    pub wk_ths: Setting<ADDR_WAKE_UP_THS, 0, 6>,
    pub wake_dur: Setting<ADDR_WAKE_UP_DUR, 5, 2>,
    pub wake_ths_w: Setting<ADDR_WAKE_UP_DUR, 4, 1>,
    pub sleep_dur: Setting<ADDR_WAKE_UP_DUR, 0, 4>,
    pub ff_dur: Setting<ADDR_FREE_FALL, 3, 6, 0x0000, LEAK_REG_DOWN, LEAK_PORTION_UPPER>,
    pub ff_ths: Setting<ADDR_FREE_FALL, 0, 3>,
}

impl Config for EventConfig {
    fn write_config<S: SpiBus<u8>>(&mut self, spi: &mut S) -> Result<(), Asm330Error> {
        panic!("not implemented")
    }
}

#[derive(Default)]
pub struct DenConfig {
    pub den_x: Setting<ADDR_CTRL9_XL, 7, 1, 0x1>,
    pub den_y: Setting<ADDR_CTRL9_XL, 6, 1, 0x1>,
    pub den_z: Setting<ADDR_CTRL9_XL, 5, 1, 0x1>,
    pub den_xl_g: Setting<ADDR_CTRL9_XL, 4, 1>,
    pub den_xl_en: Setting<ADDR_CTRL9_XL, 3, 1>,
    pub den_lh: Setting<ADDR_CTRL9_XL, 2, 1>,
    pub trig_en: Setting<ADDR_CTRL6_C, 7, 1>,
    pub lvl1_en: Setting<ADDR_CTRL6_C, 6, 1>,
    pub lvl2_en: Setting<ADDR_CTRL6_C, 5, 1>,
}

impl Config for DenConfig {
    fn write_config<S: SpiBus<u8>>(&mut self, spi: &mut S) -> Result<(), Asm330Error> {
        panic!("not implemented")
    }
}

#[derive(Default)]
pub struct XLConfig {
    pub odr_xl: Setting<ADDR_CTRL1_XL, 4, 4>,
    pub fs_xl: Setting<ADDR_CTRL1_XL, 2, 2>,
    pub lpf2_xl_en: Setting<ADDR_CTRL1_XL, 1, 1>,
    pub hpcf_xl: Setting<ADDR_CTRL8_XL, 5, 3>,
    pub hp_ref_mode_xl: Setting<ADDR_CTRL8_XL, 4, 1>,
    pub fast_settle_mode_xl: Setting<ADDR_CTRL8_XL, 3, 1>,
    pub hp_slope_xl_en: Setting<ADDR_CTRL8_XL, 2, 1>,
    pub low_pass_on_6d: Setting<ADDR_CTRL8_XL, 0, 1>,
    pub usr_off_w: Setting<ADDR_CTRL6_C, 3, 1>,
    pub usr_off_on_out: Setting<ADDR_CTRL6_C, 1, 1>,
    pub x_ofs_usr: Setting<ADDR_X_OFS_USR, 0, 8>,
    pub y_ofs_usr: Setting<ADDR_Y_OFS_USR, 0, 8>,
    pub z_ofs_usr: Setting<ADDR_Z_OFS_USR, 0, 8>,
}

impl Config for XLConfig {
    fn write_config<S: SpiBus<u8>>(&mut self, spi: &mut S) -> Result<(), Asm330Error> {
        panic!("not implemented")
    }
}

#[derive(Default)]
pub struct GyroConfig {
    pub odr_g: Setting<ADDR_CTRL2_G, 4, 4>,
    pub fs_g: Setting<ADDR_CTRL2_G, 2, 2>,
    pub fs_125: Setting<ADDR_CTRL2_G, 1, 1>,
    pub fs_4000: Setting<ADDR_CTRL2_G, 0, 1>,
    pub hp_en_g: Setting<ADDR_CTRL7_G, 6, 1>,
    pub hpm_g: Setting<ADDR_CTRL7_G, 4, 2>,
    pub ftype: Setting<ADDR_CTRL6_C, 0, 3>,
    pub lpf1_sel_g: Setting<ADDR_CTRL4_C, 1, 1>,
    pub sleep_g: Setting<ADDR_CTRL4_C, 6, 1>,
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
    pub timestamp_en: Setting<ADDR_CTRL10_C, 5, 1>,
}

impl Config for TimeConfig {
    fn write_config<S: SpiBus<u8>>(&mut self, spi: &mut S) -> Result<(), Asm330Error> {
        panic!("not implemented")
    }
}

#[derive(Default)]
pub struct FifoConfig {
    pub wtm: Setting<ADDR_FIFO_CTRL1, 0, 9, 0x0000, LEAK_REG_UP, LEAK_PORTION_LOWER>,
    pub odrchg_en: Setting<ADDR_FIFO_CTRL2, 4, 1>,
    pub stop_on_wtm: Setting<ADDR_FIFO_CTRL2, 7, 1>,
    pub bdr_gy: Setting<ADDR_FIFO_CTRL3, 4, 4>,
    pub bdr_xl: Setting<ADDR_FIFO_CTRL3, 0, 4>,
    pub dec_ts_batch: Setting<ADDR_FIFO_CTRL4, 6, 2>,
    pub odr_t_batch: Setting<ADDR_FIFO_CTRL4, 4, 2>,
    pub fifo_mode: Setting<ADDR_FIFO_CTRL4, 0, 3>,
    pub dataready_pulsed: Setting<ADDR_COUNTER_BDR_REG1, 7, 1>,
    pub rst_counter_bdr: Setting<ADDR_COUNTER_BDR_REG1, 6, 1>,
    pub trig_counter_bdr: Setting<ADDR_COUNTER_BDR_REG1, 5, 1>,
    pub cnt_bdr_th:
        Setting<ADDR_COUNTER_BDR_REG2, 0, 10, 0x0000, LEAK_REG_DOWN, LEAK_PORTION_LOWER>,
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

struct Device {
    _running: bool,
    _config_written: bool,
    pub device_conf: DeviceConfig,
}

impl Device {
    fn power_off() -> Result<(), Asm330Error> {
        Ok(())
    }
    fn read_accel() -> Result<(i16, i16, i16), Asm330Error> {
        Ok((0, 0, 0))
    }
    fn read_gyro() -> Result<(i16, i16, i16), Asm330Error> {
        Ok((0, 0, 0))
    }
    fn read_temp() -> Result<i16, Asm330Error> {
        Ok(0)
    }
    fn read_time() -> Result<u64, Asm330Error> {
        Ok(0)
    }
    // fn empty_fifo() -> Result<Vec<;
}

/* setting
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
