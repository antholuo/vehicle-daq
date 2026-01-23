#![no_std]
/*
* Created date: 12/20/25
* File Description: asm330 device state
*/
use log::{trace};
use ux::*;

pub mod common;
mod reg;

pub use common::*;

use embedded_hal::spi::SpiBus;

pub struct Device<S>
where
    S: SpiBus<u8>,
{
    spibus: S,
}

#[derive(Default, Copy, Clone)]
pub enum UsrOffWeight {
    #[default]
    Small = 0,
    Large = 1,
}

#[derive(Default, Copy, Clone)]
pub enum BatchCounterSrc {
    #[default]
    Accel = 0,
    Gyro = 1,
}

#[derive(Default, Copy, Clone)]
pub enum InterruptPinMode {
    #[default]
    PushPull = 0,
    OpenDrain = 1,
}

#[derive(Default, Copy, Clone)]
pub enum SpiMode {
    #[default]
    FourWire = 0,
    ThreeWire = 1,
}

#[derive(Default, Copy, Clone)]
pub enum EventFilter {
    #[default]
    Slope = 0,
    HighPass = 1,
}

#[derive(Default, Copy, Clone)]
pub enum WakeThresholdWeight {
    #[default]
    Large = 0,
    Small = 1,
}

#[derive(Default, Copy, Clone)]
pub enum StampSensor {
    #[default]
    Gyro = 0,
    Accel = 1,
}

#[derive(Default, Copy, Clone)]
pub struct AccelConfig {
    pub odr: Odr,
    pub fs: AccelScale,
    pub lpf2_en: bool,
    pub high_pass_cutoff: u3,
    pub high_pass_ref_mode_en: bool,
    pub high_pass_fast_settle_en: bool,
    pub high_pass_slope_xl_en: bool,
    pub low_pass_on_6d_en: bool,
    pub usr_off_weight: UsrOffWeight,
    pub usr_off_on_out_en: bool,
    pub x_ofs_usr: u8,
    pub y_ofs_usr: u8,
    pub z_ofs_usr: u8,
}

#[derive(Default, Copy, Clone)]
pub struct GyroConfig {
    pub odr: Odr,
    pub fs: GyroScale,
    pub high_pass_en: bool,
    pub high_pass_mode: GyroHighPassMode,
    pub low_pass_filter_select: u3,
    pub low_pass_filter_en: bool,
}

#[derive(Default, Copy, Clone)]
pub struct FifoConfig {
    pub wtm: u9,
    pub odrchg_en: bool,
    pub stop_on_wtm: bool,
    pub bdr_gy: Odr,
    pub bdr_xl: Odr,
    pub timestamp_decimation: TimestampDecimation,
    pub temperature_batch_rate: TempBatchOdr,
    pub fifo_mode: FifoMode,
    pub data_ready_pulsed_en: bool,
    pub batch_counter_src: BatchCounterSrc,
    pub batch_event_threshold: u10,
}

#[derive(Default, Copy, Clone)]
pub struct SystemConfig {
    pub i2c_disable: bool,
    pub sdo_pullup_en: bool,
    pub block_data_update_en: bool,
    pub interrupt_pin_mode: InterruptPinMode,
    pub spi_mode_select: SpiMode,
    pub reg_addr_auto_increment_en: bool,
    pub rounding_mode: Rounding,
}

#[derive(Default, Copy, Clone)]
pub struct Interrupt1Mask {
    pub cnt_bdr: bool,
    pub fifo_full: bool,
    pub fifo_ovr: bool,
    pub fifo_th: bool,
    pub boot: bool,
    pub drdy_g: bool,
    pub drdy_xl: bool,
    pub sleep_change: bool,
    pub wu: bool,
    pub ff: bool,
    pub int_6d: bool,
    pub den_drdy: bool,
}

#[derive(Default, Copy, Clone)]
pub struct Interrupt2Mask {
    pub cnt_bdr: bool,
    pub fifo_full: bool,
    pub fifo_ovr: bool,
    pub fifo_th: bool,
    pub drdy_temp: bool,
    pub drdy_g: bool,
    pub drdy_xl: bool,
    pub sleep_change: bool,
    pub wu: bool,
    pub ff: bool,
    pub int_6d: bool,
    pub timestamp: bool,
}

#[derive(Default, Copy, Clone)]
pub struct InterruptConfig {
    pub int2_on_int1: bool,
    pub active_low: bool,
    pub drdy_mask_en: bool,
    pub clear_on_read_en: bool,
    pub sleep_status_en: bool,
    pub latched_en: bool,
    pub interrupts_en: bool,
    pub int1_mask: Interrupt1Mask,
    pub int2_mask: Interrupt2Mask,
}

#[derive(Default, Copy, Clone)]
pub struct EventConfig {
    pub event_filter_select: EventFilter,
    pub inact_en: InactMode,
    pub d4d_en: bool,
    pub sixd_threshold: SixDThresh,
    pub usr_off_on_wu: bool,
    pub wake_up_threshold: u6,
    pub wake_up_duration: u2,
    pub wake_up_threshold_weight: WakeThresholdWeight,
    pub sleep_duration: u4,
    pub ff_dur: u6,
    pub ff_ths: FreeFallThresh,
}

#[derive(Copy, Clone)]
pub struct DenConfig {
    pub on_x: bool,
    pub on_y: bool,
    pub on_z: bool,
    pub stamping_sensor: StampSensor,
    pub extend_to_accel: bool,
    pub active_high: bool,
    pub trigger_mode: TriggerMode,
}

#[derive(Default, Copy, Clone)]
pub struct AccelData {
    pub x: i16,
    pub y: i16,
    pub z: i16,
}

#[derive(Default, Copy, Clone)]
pub struct GyroData {
    pub x: i16,
    pub y: i16,
    pub z: i16,
}

impl Default for DenConfig {
    fn default() -> Self {
        Self {
            on_x: true,
            on_y: true,
            on_z: true,
            stamping_sensor: StampSensor::default(),
            extend_to_accel: false,
            active_high: false,
            trigger_mode: TriggerMode::default(),
        }
    }
}

impl<S> Device<S>
where
    S: SpiBus<u8>,
{
    pub fn new(spibus: S) -> Self {
        Self { spibus }
    }

    fn spi_read_reg(&mut self, reg_addr: u8) -> Result<u8, Asm330Error> {
        let read_cmd = reg_addr | 0x80;
        let mut buf: [u8; 2] = [read_cmd, 0x00];

        match self.spibus.transfer_in_place(&mut buf) {
            Ok(_) => {}
            Err(_) => {
                return Err(Asm330Error::SpiError());
            }
        }
        trace!(
            "spi_read_reg: read 0x{:02X} from address 0x{:02X}",
            buf[1],
            reg_addr
        );
        Ok(buf[1])
    }

    fn spi_write_reg(&mut self, reg_addr: u8, data: u8) -> Result<(), Asm330Error> {
        let write_cmd = reg_addr & !0x80;
        let buf: [u8; 2] = [write_cmd, data];

        match self.spibus.write(&buf) {
            Ok(_) => {}
            Err(_) => {
                return Err(Asm330Error::SpiError());
            }
        }
        trace!(
            "spi_write_reg: wrote 0x{:02X} to address 0x{:02X}",
            data,
            reg_addr
        );
        Ok(())
    }

    pub fn test_whoami(&mut self) -> Result<bool, Asm330Error> {
        let value: reg::WHO_AM_I = self.spi_read_reg(reg::WHO_AM_I::ADDR)?.into();
        if value.data() == 0x6b {
            return Ok(true);
        } else {
            return Ok(false);
        }
    }

    pub fn configure_accel(&mut self, cfg: AccelConfig) -> Result<(), Asm330Error> {
        assert!(cfg.odr < Odr::Hz6_5, "Invalid accel odr (see datasheet pg 49)!");
        let ctrl1_xl = reg::CTRL1_XL::from(self.spi_read_reg(reg::CTRL1_XL::ADDR)?)
            .with_lpf2_xl_en(cfg.lpf2_en.into())
            .with_fs_xl(cfg.fs)
            .with_odr_xl(cfg.odr);
        let ctrl8_xl = reg::CTRL8_XL::from(self.spi_read_reg(reg::CTRL8_XL::ADDR)?)
            .with_hpcf_xl(cfg.high_pass_cutoff.into())
            .with_hp_ref_mode_xl(cfg.high_pass_ref_mode_en.into())
            .with_fastsettle_mode_xl(cfg.high_pass_fast_settle_en.into())
            .with_hp_slope_xl_en(cfg.high_pass_slope_xl_en.into())
            .with_low_pass_on_6d(cfg.low_pass_on_6d_en.into());
        let ctrl6_c = reg::CTRL6_C::from(self.spi_read_reg(reg::CTRL6_C::ADDR)?)
            .with_usr_off_w(cfg.usr_off_weight as u8);
        let ctrl7_g = reg::CTRL7_G::from(self.spi_read_reg(reg::CTRL7_G::ADDR)?)
            .with_usr_off_on_out(cfg.usr_off_on_out_en.into());
        let x_ofs_usr = reg::X_OFS_USR::from(cfg.x_ofs_usr);
        let y_ofs_usr = reg::Y_OFS_USR::from(cfg.y_ofs_usr);
        let z_ofs_usr = reg::Z_OFS_USR::from(cfg.z_ofs_usr);
        self.spi_write_reg(reg::CTRL1_XL::ADDR, u8::from(ctrl1_xl))?;
        self.spi_write_reg(reg::CTRL8_XL::ADDR, u8::from(ctrl8_xl))?;
        self.spi_write_reg(reg::CTRL6_C::ADDR, u8::from(ctrl6_c))?;
        self.spi_write_reg(reg::CTRL7_G::ADDR, u8::from(ctrl7_g))?;
        self.spi_write_reg(reg::X_OFS_USR::ADDR, u8::from(x_ofs_usr))?;
        self.spi_write_reg(reg::Y_OFS_USR::ADDR, u8::from(y_ofs_usr))?;
        self.spi_write_reg(reg::Z_OFS_USR::ADDR, u8::from(z_ofs_usr))?;
        Ok(())
    }

    pub fn configure_gyro(&mut self, cfg: GyroConfig) -> Result<(), Asm330Error> {
        assert!(cfg.odr < Odr::Hz6_5, "Invalid Gyro odr (see datasheet pg 50)!");
        assert!(cfg.low_pass_filter_select < u3::new(0b100) || cfg.odr < Odr::Hz3333, "Invalid Gyro odr and low pass filter selection (see datasheet pg 54)!");
        let ctrl2_g = reg::CTRL2_G::from(self.spi_read_reg(reg::CTRL2_G::ADDR)?)
            .with_odr_g(cfg.odr.into())
            .with_fs_g(cfg.fs.into());
        let ctrl7_g = reg::CTRL7_G::from(self.spi_read_reg(reg::CTRL7_G::ADDR)?)
            .with_hp_en_g(cfg.high_pass_en.into())
            .with_hpm_g(cfg.high_pass_mode.into());
        let ctrl6_c = reg::CTRL6_C::from(self.spi_read_reg(reg::CTRL6_C::ADDR)?)
            .with_ftype(cfg.low_pass_filter_select.into());
        let ctrl4_c = reg::CTRL4_C::from(self.spi_read_reg(reg::CTRL4_C::ADDR)?)
            .with_lpf1_sel_g(cfg.low_pass_filter_en.into());
        self.spi_write_reg(reg::CTRL2_G::ADDR, u8::from(ctrl2_g))?;
        self.spi_write_reg(reg::CTRL7_G::ADDR, u8::from(ctrl7_g))?;
        self.spi_write_reg(reg::CTRL6_C::ADDR, u8::from(ctrl6_c))?;
        self.spi_write_reg(reg::CTRL4_C::ADDR, u8::from(ctrl4_c))?;
        Ok(())
    }

    pub fn configure_time(&mut self, enable: bool) -> Result<(), Asm330Error> {
        let ctrl10_c = reg::CTRL10_C::from(self.spi_read_reg(reg::CTRL10_C::ADDR)?).with_timestamp_en(enable.into());
        self.spi_write_reg(reg::CTRL10_C::ADDR, u8::from(ctrl10_c))?;
        Ok(())
    }

    pub fn configure_fifo(&mut self, cfg: FifoConfig) -> Result<(), Asm330Error> {
        assert!(cfg.bdr_xl < Odr::Hz6_5, "Invalid accel batched data rate (see datasheet pg 44)!");
        let fifo_ctrl1 = reg::FIFO_CTRL1::from(self.spi_read_reg(reg::FIFO_CTRL1::ADDR)?)
            .with_wtm_lower((u16::from(cfg.wtm) & 0xFF) as u8);
        let fifo_ctrl2 = reg::FIFO_CTRL2::from(self.spi_read_reg(reg::FIFO_CTRL2::ADDR)?)
            .with_wtm_upper((u16::from(cfg.wtm) >> 8) as u8)
            .with_odrchg_en(cfg.odrchg_en.into())
            .with_stop_on_wtm(cfg.stop_on_wtm.into());
        let fifo_ctrl3 = reg::FIFO_CTRL3::from(self.spi_read_reg(reg::FIFO_CTRL3::ADDR)?)
            .with_bdr_gy(cfg.bdr_gy)
            .with_bdr_xl(cfg.bdr_xl);
        let fifo_ctrl4 = reg::FIFO_CTRL4::from(self.spi_read_reg(reg::FIFO_CTRL4::ADDR)?)
            .with_fifo_mode(cfg.fifo_mode)
            .with_odr_t_batch(cfg.temperature_batch_rate)
            .with_dec_ts_batch(cfg.timestamp_decimation);
        let counter_bdr_reg1 = reg::COUNTER_BDR_REG1::from(self.spi_read_reg(reg::COUNTER_BDR_REG1::ADDR)?)
            .with_dataready_pulsed(cfg.data_ready_pulsed_en.into())
            .with_trig_counter_bdr(cfg.batch_counter_src as u8)
            .with_cnt_bdr_th_upper((u16::from(cfg.batch_event_threshold) >> 8) as u8);
        let counter_bdr_reg2 = reg::COUNTER_BDR_REG2::from(self.spi_read_reg(reg::COUNTER_BDR_REG2::ADDR)?)
            .with_cnt_bdr_th_lower((u16::from(cfg.batch_event_threshold) & 0xFF) as u8);
        self.spi_write_reg(reg::FIFO_CTRL1::ADDR, u8::from(fifo_ctrl1))?;
        self.spi_write_reg(reg::FIFO_CTRL2::ADDR, u8::from(fifo_ctrl2))?;
        self.spi_write_reg(reg::FIFO_CTRL3::ADDR, u8::from(fifo_ctrl3))?;
        self.spi_write_reg(reg::FIFO_CTRL4::ADDR, u8::from(fifo_ctrl4))?;
        self.spi_write_reg(reg::COUNTER_BDR_REG1::ADDR, u8::from(counter_bdr_reg1))?;
        self.spi_write_reg(reg::COUNTER_BDR_REG2::ADDR, u8::from(counter_bdr_reg2))?;
        Ok(())
    }

    pub fn configure_system(&mut self, cfg: SystemConfig) -> Result<(), Asm330Error> {
        let pin_ctrl = reg::PIN_CTRL::from(self.spi_read_reg(reg::PIN_CTRL::ADDR)?)
            .with_sdo_pu_en(cfg.sdo_pullup_en.into());
        let ctrl3_c = reg::CTRL3_C::from(self.spi_read_reg(reg::CTRL3_C::ADDR)?)
            .with_bdu(cfg.block_data_update_en.into())
            .with_pp_od(cfg.interrupt_pin_mode as u8)
            .with_sim(cfg.spi_mode_select as u8)
            .with_if_inc(cfg.reg_addr_auto_increment_en.into());
        let ctrl4_c = reg::CTRL4_C::from(self.spi_read_reg(reg::CTRL4_C::ADDR)?)
            .with_i2c_disable(cfg.i2c_disable.into());
        let ctrl5_c = reg::CTRL5_C::from(self.spi_read_reg(reg::CTRL5_C::ADDR)?)
            .with_rounding(cfg.rounding_mode);
        let ctrl9_xl = reg::CTRL9_XL::from(self.spi_read_reg(reg::CTRL9_XL::ADDR)?)
            .with_device_conf(1);
        self.spi_write_reg(reg::PIN_CTRL::ADDR, u8::from(pin_ctrl))?;
        self.spi_write_reg(reg::CTRL3_C::ADDR, u8::from(ctrl3_c))?;
        self.spi_write_reg(reg::CTRL4_C::ADDR, u8::from(ctrl4_c))?;
        self.spi_write_reg(reg::CTRL5_C::ADDR, u8::from(ctrl5_c))?;
        self.spi_write_reg(reg::CTRL9_XL::ADDR, u8::from(ctrl9_xl))?;
        Ok(())
    }

    pub fn configure_interrupt(&mut self, cfg: InterruptConfig) -> Result<(), Asm330Error> {
        assert!(cfg.clear_on_read_en && !cfg.latched_en, "Latched interrupts must be enabled to enable interrupt clear on read (see datasheet pg 65)!");
        let ctrl4_c = reg::CTRL4_C::from(self.spi_read_reg(reg::CTRL4_C::ADDR)?)
            .with_int2_on_int1(cfg.int2_on_int1.into())
            .with_drdy_mask(cfg.drdy_mask_en.into());
        let ctrl3_c = reg::CTRL3_C::from(self.spi_read_reg(reg::CTRL3_C::ADDR)?)
            .with_h_lactive(cfg.active_low.into());
        let int_cfg0 = reg::INT_CFG0::from(self.spi_read_reg(reg::INT_CFG0::ADDR)?)
            .with_int_clr_on_read(cfg.clear_on_read_en.into())
            .with_sleep_status_on_int(cfg.sleep_status_en.into())
            .with_lir(cfg.latched_en.into());
        let int_cfg1 = reg::INT_CFG1::from(self.spi_read_reg(reg::INT_CFG1::ADDR)?)
            .with_interrupts_enable(cfg.interrupts_en.into());
        let md1_cfg = reg::MD1_CFG::from(self.spi_read_reg(reg::MD1_CFG::ADDR)?)
            .with_int1_sleep_change(cfg.int1_mask.sleep_change.into())
            .with_int1_wu(cfg.int1_mask.wu.into())
            .with_int1_ff(cfg.int1_mask.ff.into())
            .with_int1_6d(cfg.int1_mask.int_6d.into());
        let md2_cfg = reg::MD2_CFG::from(self.spi_read_reg(reg::MD2_CFG::ADDR)?)
            .with_int2_sleep_change(cfg.int2_mask.sleep_change.into())
            .with_int2_wu(cfg.int2_mask.wu.into())
            .with_int2_ff(cfg.int2_mask.ff.into())
            .with_int2_6d(cfg.int2_mask.int_6d.into())
            .with_int2_timestamp(cfg.int2_mask.timestamp.into());
        let int1_ctrl = reg::INT1_CTRL::from(self.spi_read_reg(reg::INT1_CTRL::ADDR)?)
            .with_int1_drdy_xl(cfg.int1_mask.drdy_xl.into())
            .with_int1_drdy_g(cfg.int1_mask.drdy_g.into())
            .with_int1_boot(cfg.int1_mask.boot.into())
            .with_int1_fifo_th(cfg.int1_mask.fifo_th.into())
            .with_int1_fifo_ovr(cfg.int1_mask.fifo_ovr.into())
            .with_int1_fifo_full(cfg.int1_mask.fifo_full.into())
            .with_int1_cnt_bdr(cfg.int1_mask.cnt_bdr.into())
            .with_den_drdy_flag(cfg.int1_mask.den_drdy.into());
        let int2_ctrl = reg::INT2_CTRL::from(self.spi_read_reg(reg::INT2_CTRL::ADDR)?)
            .with_int2_drdy_xl(cfg.int2_mask.drdy_xl.into())
            .with_int2_drdy_g(cfg.int2_mask.drdy_g.into())
            .with_int2_drdy_temp(cfg.int2_mask.drdy_temp.into())
            .with_int2_fifo_th(cfg.int2_mask.fifo_th.into())
            .with_int2_fifo_ovr(cfg.int2_mask.fifo_ovr.into())
            .with_int2_fifo_full(cfg.int2_mask.fifo_full.into())
            .with_int2_cnt_bdr(cfg.int2_mask.cnt_bdr.into());
        self.spi_write_reg(reg::CTRL4_C::ADDR, u8::from(ctrl4_c))?;
        self.spi_write_reg(reg::CTRL3_C::ADDR, u8::from(ctrl3_c))?;
        self.spi_write_reg(reg::INT_CFG0::ADDR, u8::from(int_cfg0))?;
        self.spi_write_reg(reg::INT_CFG1::ADDR, u8::from(int_cfg1))?;
        self.spi_write_reg(reg::MD1_CFG::ADDR, u8::from(md1_cfg))?;
        self.spi_write_reg(reg::MD2_CFG::ADDR, u8::from(md2_cfg))?;
        self.spi_write_reg(reg::INT1_CTRL::ADDR, u8::from(int1_ctrl))?;
        self.spi_write_reg(reg::INT2_CTRL::ADDR, u8::from(int2_ctrl))?;
        Ok(())
    }

    pub fn configure_event(&mut self, cfg: EventConfig) -> Result<(), Asm330Error> {
        let int_cfg0 = reg::INT_CFG0::from(self.spi_read_reg(reg::INT_CFG0::ADDR)?)
            .with_slope_fds(cfg.event_filter_select as u8);
        let int_cfg1 = reg::INT_CFG1::from(self.spi_read_reg(reg::INT_CFG1::ADDR)?)
            .with_inact(cfg.inact_en);
        let ths_6d = reg::THS_6D::from(self.spi_read_reg(reg::THS_6D::ADDR)?)
            .with_d4d_en(cfg.d4d_en.into())
            .with_sixd_ths(cfg.sixd_threshold);
        let wake_up_ths = reg::WAKE_UP_THS::from(self.spi_read_reg(reg::WAKE_UP_THS::ADDR)?)
            .with_usr_off_on_wu(cfg.usr_off_on_wu.into())
            .with_wk_ths(cfg.wake_up_threshold.into());
        let wake_up_dur = reg::WAKE_UP_DUR::from(self.spi_read_reg(reg::WAKE_UP_DUR::ADDR)?)
            .with_wake_dur(cfg.wake_up_duration.into())
            .with_wake_ths_w(cfg.wake_up_threshold_weight as u8)
            .with_sleep_dur(cfg.sleep_duration.into())
            .with_ff_dur_upper((cfg.ff_dur >> 5).into());
        let free_fall = reg::FREE_FALL::from(self.spi_read_reg(reg::FREE_FALL::ADDR)?)
            .with_ff_dur_lower(u8::from(cfg.ff_dur) & 0b0001_1111)
            .with_ff_ths(cfg.ff_ths);
        self.spi_write_reg(reg::INT_CFG0::ADDR, u8::from(int_cfg0))?;
        self.spi_write_reg(reg::INT_CFG1::ADDR, u8::from(int_cfg1))?;
        self.spi_write_reg(reg::THS_6D::ADDR, u8::from(ths_6d))?;
        self.spi_write_reg(reg::WAKE_UP_THS::ADDR, u8::from(wake_up_ths))?;
        self.spi_write_reg(reg::WAKE_UP_DUR::ADDR, u8::from(wake_up_dur))?;
        self.spi_write_reg(reg::FREE_FALL::ADDR, u8::from(free_fall))?;
        Ok(())
    }

    pub fn configure_den(&mut self, cfg: DenConfig) -> Result<(), Asm330Error> {
        let ctrl9_xl = reg::CTRL9_XL::from(self.spi_read_reg(reg::CTRL9_XL::ADDR)?)
            .with_den_x(cfg.on_x.into())
            .with_den_y(cfg.on_y.into())
            .with_den_z(cfg.on_z.into())
            .with_den_xl_g(cfg.stamping_sensor as u8)
            .with_den_xl_en(cfg.extend_to_accel.into())
            .with_den_lh(cfg.active_high.into());
        let ctrl6_c = reg::CTRL6_C::from(self.spi_read_reg(reg::CTRL6_C::ADDR)?)
            .with_trig_mode(cfg.trigger_mode);
        self.spi_write_reg(reg::CTRL9_XL::ADDR, u8::from(ctrl9_xl))?;
        self.spi_write_reg(reg::CTRL6_C::ADDR, u8::from(ctrl6_c))?;
        Ok(())
    }

    pub fn read_gyro(&mut self) -> Result<GyroData, Asm330Error> {
        let outx_l_g = reg::OUTX_L_G::from(self.spi_read_reg(reg::OUTX_L_G::ADDR)?);
        let outx_h_g = reg::OUTX_H_G::from(self.spi_read_reg(reg::OUTX_H_G::ADDR)?);
        let outy_l_g = reg::OUTY_L_G::from(self.spi_read_reg(reg::OUTY_L_G::ADDR)?);
        let outy_h_g = reg::OUTY_H_G::from(self.spi_read_reg(reg::OUTY_H_G::ADDR)?);
        let outz_l_g = reg::OUTZ_L_G::from(self.spi_read_reg(reg::OUTZ_L_G::ADDR)?);
        let outz_h_g = reg::OUTZ_H_G::from(self.spi_read_reg(reg::OUTZ_H_G::ADDR)?);
        Ok(GyroData {
            x: (outx_l_g.data() as i16) | (outx_h_g.data() as i16) << 8,
            y: (outy_l_g.data() as i16) | (outy_h_g.data() as i16) << 8,
            z: (outz_l_g.data() as i16) | (outz_h_g.data() as i16) << 8,
        })
    }

    pub fn read_accel(&mut self) -> Result<AccelData, Asm330Error> {
        let outx_l_a = reg::OUTX_L_A::from(self.spi_read_reg(reg::OUTX_L_A::ADDR)?);
        let outx_h_a = reg::OUTX_H_A::from(self.spi_read_reg(reg::OUTX_H_A::ADDR)?);
        let outy_l_a = reg::OUTY_L_A::from(self.spi_read_reg(reg::OUTY_L_A::ADDR)?);
        let outy_h_a = reg::OUTY_H_A::from(self.spi_read_reg(reg::OUTY_H_A::ADDR)?);
        let outz_l_a = reg::OUTZ_L_A::from(self.spi_read_reg(reg::OUTZ_L_A::ADDR)?);
        let outz_h_a = reg::OUTZ_H_A::from(self.spi_read_reg(reg::OUTZ_H_A::ADDR)?);
        Ok(AccelData {
            x: (outx_l_a.data() as i16) | (outx_h_a.data() as i16) << 8,
            y: (outy_l_a.data() as i16) | (outy_h_a.data() as i16) << 8,
            z: (outz_l_a.data() as i16) | (outz_h_a.data() as i16) << 8,
        })
    }

    pub fn read_temp(&mut self) -> Result<i16, Asm330Error> {
        let out_temp_l = reg::OUT_TEMP_L::from(self.spi_read_reg(reg::OUT_TEMP_L::ADDR)?);
        let out_temp_h = reg::OUT_TEMP_H::from(self.spi_read_reg(reg::OUT_TEMP_H::ADDR)?);
        Ok((out_temp_l.data() as i16) | (out_temp_h.data() as i16) << 8)
    }

    pub fn read_internal_freq_fine(&mut self) -> Result<u8, Asm330Error> {
        let internal_freq_fine = reg::INTERNAL_FREQ_FINE::from(self.spi_read_reg(reg::INTERNAL_FREQ_FINE::ADDR)?);
        Ok(internal_freq_fine.data())
    }

    pub fn read_time(&mut self) -> Result<u32, Asm330Error> {
        let timestamp0 = reg::TIMESTAMP0::from(self.spi_read_reg(reg::TIMESTAMP0::ADDR)?);
        let timestamp1 = reg::TIMESTAMP1::from(self.spi_read_reg(reg::TIMESTAMP1::ADDR)?);
        let timestamp2 = reg::TIMESTAMP2::from(self.spi_read_reg(reg::TIMESTAMP2::ADDR)?);
        let timestamp3 = reg::TIMESTAMP3::from(self.spi_read_reg(reg::TIMESTAMP3::ADDR)?);
        Ok(
            (timestamp0.data() as u32)
            | (timestamp1.data() as u32) << 8
            | (timestamp2.data() as u32) << 16
            | (timestamp3.data() as u32) << 24
        )
    }

    pub fn read_fifo(&mut self) -> Result<(), Asm330Error> {
        unimplemented!()
    }

    pub fn reboot(&mut self) -> Result<(), Asm330Error> {
        let ctrl1_xl = reg::CTRL1_XL::from(self.spi_read_reg(reg::CTRL1_XL::ADDR)?);
        assert!(ctrl1_xl.odr_xl() > Odr::PowerDown, "Accelerometer must be on for reboot (see datasheet pg 51)!");
        let ctrl3_c = reg::CTRL3_C::from(self.spi_read_reg(reg::CTRL3_C::ADDR)?).with_boot(1);
        self.spi_write_reg(reg::CTRL3_C::ADDR, u8::from(ctrl3_c))?;
        Ok(())
    }

    pub fn sw_reset(&mut self) -> Result<(), Asm330Error> {
        let ctrl3_c = reg::CTRL3_C::from(self.spi_read_reg(reg::CTRL3_C::ADDR)?).with_sw_reset(1);
        self.spi_write_reg(reg::CTRL3_C::ADDR, u8::from(ctrl3_c))?;
        Ok(())
    }

    pub fn self_test_accel(&mut self) -> Result<(), Asm330Error> {
        unimplemented!()
        // pub st_xl: Setting<ADDR_CTRL5_C, 0, 2>,
    }

    pub fn self_test_gyro(&mut self) -> Result<(), Asm330Error> {
        unimplemented!()
        // pub st_g: Setting<ADDR_CTRL5_C, 2, 2>,
    }

    pub fn sleep_gyro(&mut self) -> Result<(), Asm330Error> {
        let ctrl4_c = reg::CTRL4_C::from(self.spi_read_reg(reg::CTRL4_C::ADDR)?)
            .with_sleep_g(1);
        self.spi_write_reg(reg::CTRL4_C::ADDR, u8::from(ctrl4_c))?;
        Ok(())
    }

    pub fn wake_gyro(&mut self) -> Result<(), Asm330Error> {
        let ctrl4_c = reg::CTRL4_C::from(self.spi_read_reg(reg::CTRL4_C::ADDR)?)
            .with_sleep_g(0);
        self.spi_write_reg(reg::CTRL4_C::ADDR, u8::from(ctrl4_c))?;
        Ok(())
    }

    pub fn reset_fifo_batch_counter(&mut self) -> Result<(), Asm330Error> {
        let counter_bdr_reg1 = reg::COUNTER_BDR_REG1::from(self.spi_read_reg(reg::COUNTER_BDR_REG1::ADDR)?)
            .with_rst_counter_bdr(1);
        self.spi_write_reg(reg::COUNTER_BDR_REG1::ADDR, u8::from(counter_bdr_reg1))?;
        Ok(())
    }

    // pub fn self_test(&mut self) -> Result<(), Asm330Error> {
    //     unimplemented!()
    // }
}

// pub fn test() {
//     let raw: u8 = 0b00111111;
//     let mut pin_ctrl = reg::PIN_CTRL::from_bytes([raw]);
//     pin_ctrl.set_sdo_pu_en(1);
//     debug!("before: {:08b}, after: {:08b}", raw, u8::from(pin_ctrl));
// }

// pub struct SystemConfig {
//     pub drdy_mask               : Field,
//     pub i2c_disable             : Field,
//     pub sdo_pu_en               : Field,
//     pub boot                    : Field,
//     pub bdu                     : Field,
//     pub pp_od                   : Field,
//     pub sim                     : Field,
//     pub if_inc                  : Field,
//     pub sw_reset                : Field,
//     pub rounding                : Field,
//     pub st_g                    : Field,
//     pub st_xl                   : Field,
//     pub device_conf             : Field,
// }

// impl SystemConfig {
//     pub fn new() -> Self {
//         Self {
//             drdy_mask               : Field::new_rw(ADDR_CTRL4_C             , 3, 1),
//             i2c_disable             : Field::new_rw(ADDR_CTRL4_C             , 2, 1),
//             sdo_pu_en               : Field::new_rw(ADDR_PIN_CTRL            , 6, 1),
//             boot                    : Field::new_rw(ADDR_CTRL3_C             , 7, 1),
//             bdu                     : Field::new_rw(ADDR_CTRL3_C             , 6, 1),
//             pp_od                   : Field::new_rw(ADDR_CTRL3_C             , 4, 1),
//             sim                     : Field::new_rw(ADDR_CTRL3_C             , 3, 1),
//             if_inc                  : Field::new(   ADDR_CTRL3_C             , 2, 1, false, 1, false, false),
//             sw_reset                : Field::new_rw(ADDR_CTRL3_C             , 0, 1),
//             rounding                : Field::new_rw(ADDR_CTRL5_C             , 5, 2),
//             st_g                    : Field::new_rw(ADDR_CTRL5_C             , 2, 2),
//             st_xl                   : Field::new_rw(ADDR_CTRL5_C             , 0, 2),
//             device_conf             : Field::new_rw(ADDR_CTRL9_XL            , 1, 1),
//         }
//     }
// }

// pub struct InterruptConfig {
//     pub int2_on_int1            : Field,
//     pub den_drdy_flag           : Field,
//     pub int1_cnt_bdr            : Field,
//     pub int1_fifo_full          : Field,
//     pub int1_fifo_ovr           : Field,
//     pub int1_fifo_th            : Field,
//     pub int1_boot               : Field,
//     pub int1_drdy_g             : Field,
//     pub int1_drdy_xl            : Field,
//     pub int2_cnt_bdr            : Field,
//     pub int2_fifo_full          : Field,
//     pub int2_fifo_ovr           : Field,
//     pub int2_fifo_th            : Field,
//     pub int2_drdy_temp          : Field,
//     pub int2_drdy_g             : Field,
//     pub int2_drdy_xl            : Field,
//     pub h_lactive               : Field,
//     pub int_clr_on_read         : Field,
//     pub sleep_status_on_int     : Field,
//     pub lir                     : Field,
//     pub interrupts_enable       : Field,
//     pub int1_sleep_change       : Field,
//     pub int1_wu                 : Field,
//     pub int1_ff                 : Field,
//     pub int1_6d                 : Field,
//     pub int2_sleep_change       : Field,
//     pub int2_wu                 : Field,
//     pub int2_ff                 : Field,
//     pub int2_6d                 : Field,
//     pub int2_timestamp          : Field,
// }

// impl InterruptConfig {
//     pub fn new() -> Self {
//         Self {
//             int2_on_int1            : Field::new_rw(ADDR_CTRL4_C             , 5, 1),
//             den_drdy_flag           : Field::new_rw(ADDR_INT1_CTRL           , 7, 1),
//             int1_cnt_bdr            : Field::new_rw(ADDR_INT1_CTRL           , 6, 1),
//             int1_fifo_full          : Field::new_rw(ADDR_INT1_CTRL           , 5, 1),
//             int1_fifo_ovr           : Field::new_rw(ADDR_INT1_CTRL           , 4, 1),
//             int1_fifo_th            : Field::new_rw(ADDR_INT1_CTRL           , 3, 1),
//             int1_boot               : Field::new_rw(ADDR_INT1_CTRL           , 2, 1),
//             int1_drdy_g             : Field::new_rw(ADDR_INT1_CTRL           , 1, 1),
//             int1_drdy_xl            : Field::new_rw(ADDR_INT1_CTRL           , 0, 1),
//             int2_cnt_bdr            : Field::new_rw(ADDR_INT1_CTRL           , 6, 1),
//             int2_fifo_full          : Field::new_rw(ADDR_INT1_CTRL           , 5, 1),
//             int2_fifo_ovr           : Field::new_rw(ADDR_INT1_CTRL           , 4, 1),
//             int2_fifo_th            : Field::new_rw(ADDR_INT1_CTRL           , 3, 1),
//             int2_drdy_temp          : Field::new_rw(ADDR_INT1_CTRL           , 2, 1),
//             int2_drdy_g             : Field::new_rw(ADDR_INT1_CTRL           , 1, 1),
//             int2_drdy_xl            : Field::new_rw(ADDR_INT1_CTRL           , 0, 1),
//             h_lactive               : Field::new_rw(ADDR_CTRL3_C             , 5, 1),
//             int_clr_on_read         : Field::new_rw(ADDR_INT_CFG0            , 6, 1),
//             sleep_status_on_int     : Field::new_rw(ADDR_INT_CFG0            , 5, 1),
//             lir                     : Field::new_rw(ADDR_INT_CFG0            , 0, 1),
//             interrupts_enable       : Field::new_rw(ADDR_INT_CFG1            , 7, 1),
//             int1_sleep_change       : Field::new_rw(ADDR_MD1_CFG             , 7, 1),
//             int1_wu                 : Field::new_rw(ADDR_MD1_CFG             , 5, 1),
//             int1_ff                 : Field::new_rw(ADDR_MD1_CFG             , 4, 1),
//             int1_6d                 : Field::new_rw(ADDR_MD1_CFG             , 2, 1),
//             int2_sleep_change       : Field::new_rw(ADDR_MD2_CFG             , 7, 1),
//             int2_wu                 : Field::new_rw(ADDR_MD2_CFG             , 5, 1),
//             int2_ff                 : Field::new_rw(ADDR_MD2_CFG             , 4, 1),
//             int2_6d                 : Field::new_rw(ADDR_MD2_CFG             , 2, 1),
//             int2_timestamp          : Field::new_rw(ADDR_MD2_CFG             , 0, 1),
//         }
//     }
// }

// pub struct EventConfig {
//     pub slope_fds               : Field,
//     pub inact_en                : Field,
//     pub d4d_en                  : Field,
//     pub sixd_ths                : Field,
//     pub usr_off_on_wu           : Field,
//     pub wk_ths                  : Field,
//     pub wake_dur                : Field,
//     pub wake_ths_w              : Field,
//     pub sleep_dur               : Field,
//     pub ff_dur                  : Field,
//     pub ff_ths                  : Field,
// }

// impl EventConfig {
//     pub fn new() -> Self {
//         Self {
//             slope_fds               : Field::new_rw(ADDR_INT_CFG0            , 4, 1),
//             inact_en                : Field::new_rw(ADDR_INT_CFG1            , 5, 2),
//             d4d_en                  : Field::new_rw(ADDR_THS_6D              , 7, 1),
//             sixd_ths                : Field::new_rw(ADDR_THS_6D              , 5, 2),
//             usr_off_on_wu           : Field::new_rw(ADDR_WAKE_UP_THS         , 6, 1),
//             wk_ths                  : Field::new_rw(ADDR_WAKE_UP_THS         , 0, 6),
//             wake_dur                : Field::new_rw(ADDR_WAKE_UP_DUR         , 5, 2),
//             wake_ths_w              : Field::new_rw(ADDR_WAKE_UP_DUR         , 4, 1),
//             sleep_dur               : Field::new_rw(ADDR_WAKE_UP_DUR         , 0, 4),
//             ff_dur                  : Field::new(   ADDR_FREE_FALL           , 3, 6, false, 0, true, false),
//             ff_ths                  : Field::new_rw(ADDR_FREE_FALL           , 0, 3),
//         }
//     }
// }

// pub struct DenConfig {
//     pub den_x                   : Field,
//     pub den_y                   : Field,
//     pub den_z                   : Field,
//     pub den_xl_g                : Field,
//     pub den_xl_en               : Field,
//     pub den_lh                  : Field,
//     pub trig_en                 : Field,
//     pub lvl1_en                 : Field,
//     pub lvl2_en                 : Field,
// }

// impl DenConfig {
//     pub fn new() -> Self {
//         Self {
//             den_x                   : Field::new(   ADDR_CTRL9_XL            , 7, 1, false, 1, false, false),
//             den_y                   : Field::new(   ADDR_CTRL9_XL            , 6, 1, false, 1, false, false),
//             den_z                   : Field::new(   ADDR_CTRL9_XL            , 5, 1, false, 1, false, false),
//             den_xl_g                : Field::new_rw(ADDR_CTRL9_XL            , 4, 1),
//             den_xl_en               : Field::new_rw(ADDR_CTRL9_XL            , 3, 1),
//             den_lh                  : Field::new_rw(ADDR_CTRL9_XL            , 2, 1),
//             trig_en                 : Field::new_rw(ADDR_CTRL6_C             , 7, 1),
//             lvl1_en                 : Field::new_rw(ADDR_CTRL6_C             , 6, 1),
//             lvl2_en                 : Field::new_rw(ADDR_CTRL6_C             , 5, 1),
//         }
//     }
// }

// pub struct XLConfig {
//     pub odr_xl                  : Field,
//     pub fs_xl                   : Field,
//     pub lpf2_xl_en              : Field,
//     pub hpcf_xl                 : Field,
//     pub hp_ref_mode_xl          : Field,
//     pub fast_settle_mode_xl     : Field,
//     pub hp_slope_xl_en          : Field,
//     pub low_pass_on_6d          : Field,
//     pub usr_off_w               : Field,
//     pub usr_off_on_out          : Field,
//     pub x_ofs_usr               : Field,
//     pub y_ofs_usr               : Field,
//     pub z_ofs_usr               : Field,
// }

// impl XLConfig {
//     pub fn new() -> Self {
//         Self {
//             odr_xl                  : Field::new_rw(ADDR_CTRL1_XL            , 4, 4),
//             fs_xl                   : Field::new_rw(ADDR_CTRL1_XL            , 2, 2),
//             lpf2_xl_en              : Field::new_rw(ADDR_CTRL1_XL            , 1, 1),
//             hpcf_xl                 : Field::new_rw(ADDR_CTRL8_XL            , 5, 3),
//             hp_ref_mode_xl          : Field::new_rw(ADDR_CTRL8_XL            , 4, 1),
//             fast_settle_mode_xl     : Field::new_rw(ADDR_CTRL8_XL            , 3, 1),
//             hp_slope_xl_en          : Field::new_rw(ADDR_CTRL8_XL            , 2, 1),
//             low_pass_on_6d          : Field::new_rw(ADDR_CTRL8_XL            , 0, 1),
//             usr_off_w               : Field::new_rw(ADDR_CTRL6_C             , 3, 1),
//             usr_off_on_out          : Field::new_rw(ADDR_CTRL6_C             , 1, 1),
//             x_ofs_usr               : Field::new_rw(ADDR_X_OFS_USR           , 0, 8),
//             y_ofs_usr               : Field::new_rw(ADDR_Y_OFS_USR           , 0, 8),
//             z_ofs_usr               : Field::new_rw(ADDR_Z_OFS_USR           , 0, 8),
//         }
//     }
// }

// pub struct GyroConfig {
//     pub odr_g                   : Field,
//     pub fs_g                    : Field,
//     pub fs_125                  : Field,
//     pub fs_4000                 : Field,
//     pub hp_en_g                 : Field,
//     pub hpm_g                   : Field,
//     pub ftype                   : Field,
//     pub lpf1_sel_g              : Field,
//     pub sleep_g                 : Field,
// }

// impl GyroConfig {
//     pub fn new() -> Self {
//         Self {
//             odr_g                   : Field::new_rw(ADDR_CTRL2_G             , 4, 4),
//             fs_g                    : Field::new_rw(ADDR_CTRL2_G             , 2, 2),
//             fs_125                  : Field::new_rw(ADDR_CTRL2_G             , 1, 1),
//             fs_4000                 : Field::new_rw(ADDR_CTRL2_G             , 0, 1),
//             hp_en_g                 : Field::new_rw(ADDR_CTRL7_G             , 6, 1),
//             hpm_g                   : Field::new_rw(ADDR_CTRL7_G             , 4, 2),
//             ftype                   : Field::new_rw(ADDR_CTRL6_C             , 0, 3),
//             lpf1_sel_g              : Field::new_rw(ADDR_CTRL4_C             , 1, 1),
//             sleep_g                 : Field::new_rw(ADDR_CTRL4_C             , 6, 1),
//         }
//     }
// }

// pub struct FifoConfig {
//     pub wtm                     : Field,
//     pub odrchg_en               : Field,
//     pub stop_on_wtm             : Field,
//     pub bdr_gy                  : Field,
//     pub bdr_xl                  : Field,
//     pub dec_ts_batch            : Field,
//     pub odr_t_batch             : Field,
//     pub fifo_mode               : Field,
//     pub dataready_pulsed        : Field,
//     pub rst_counter_bdr         : Field,
//     pub trig_counter_bdr        : Field,
//     pub cnt_bdr_th              : Field,
// }

// impl FifoConfig {
//     pub fn new() -> Self {
//         Self {
//             wtm                     : Field::new(   ADDR_FIFO_CTRL1          , 0, 9, false, 0, false, true),
//             odrchg_en               : Field::new_rw(ADDR_FIFO_CTRL2          , 4, 1),
//             stop_on_wtm             : Field::new_rw(ADDR_FIFO_CTRL2          , 7, 1),
//             bdr_gy                  : Field::new_rw(ADDR_FIFO_CTRL3          , 4, 4),
//             bdr_xl                  : Field::new_rw(ADDR_FIFO_CTRL3          , 0, 4),
//             dec_ts_batch            : Field::new_rw(ADDR_FIFO_CTRL4          , 6, 2),
//             odr_t_batch             : Field::new_rw(ADDR_FIFO_CTRL4          , 4, 2),
//             fifo_mode               : Field::new_rw(ADDR_FIFO_CTRL4          , 0, 3),
//             dataready_pulsed        : Field::new_rw(ADDR_COUNTER_BDR_REG1    , 7, 1),
//             rst_counter_bdr         : Field::new_rw(ADDR_COUNTER_BDR_REG1    , 6, 1),
//             trig_counter_bdr        : Field::new_rw(ADDR_COUNTER_BDR_REG1    , 5, 1),
//             cnt_bdr_th              : Field::new(   ADDR_COUNTER_BDR_REG2    , 0, 10, false, 0, true, true),
//         }
//     }
// }

// pub struct SensorReader {
//     pub outx_h_a                : Field,
//     pub outx_l_a                : Field,
//     pub outy_h_a                : Field,
//     pub outy_l_a                : Field,
//     pub outz_h_a                : Field,
//     pub outz_l_a                : Field,
//     pub outx_h_g                : Field,
//     pub outx_l_g                : Field,
//     pub outy_h_g                : Field,
//     pub outy_l_g                : Field,
//     pub outz_h_g                : Field,
//     pub outz_l_g                : Field,
//     pub timestamp0              : Field,
//     pub timestamp1              : Field,
//     pub timestamp2              : Field,
//     pub timestamp3              : Field,
//     pub out_temp_h              : Field,
//     pub out_temp_l              : Field,
//     pub tda                     : Field,
//     pub gda                     : Field,
//     pub xlda                    : Field,
//     pub freq_fine               : Field,
// }

// impl SensorReader {
//     pub fn new() -> Self {
//         Self {
//             outx_h_a                : Field::new_ro(ADDR_OUTX_H_A            , 0, 8),
//             outx_l_a                : Field::new_ro(ADDR_OUTX_L_A            , 0, 8),
//             outy_h_a                : Field::new_ro(ADDR_OUTY_H_A            , 0, 8),
//             outy_l_a                : Field::new_ro(ADDR_OUTY_L_A            , 0, 8),
//             outz_h_a                : Field::new_ro(ADDR_OUTZ_H_A            , 0, 8),
//             outz_l_a                : Field::new_ro(ADDR_OUTZ_L_A            , 0, 8),
//             outx_h_g                : Field::new_ro(ADDR_OUTX_H_G            , 0, 8),
//             outx_l_g                : Field::new_ro(ADDR_OUTX_L_G            , 0, 8),
//             outy_h_g                : Field::new_ro(ADDR_OUTY_H_G            , 0, 8),
//             outy_l_g                : Field::new_ro(ADDR_OUTY_L_G            , 0, 8),
//             outz_h_g                : Field::new_ro(ADDR_OUTZ_H_G            , 0, 8),
//             outz_l_g                : Field::new_ro(ADDR_OUTZ_L_G            , 0, 8),
//             timestamp0              : Field::new_ro(ADDR_TIMESTAMP0_REG      , 0, 8),
//             timestamp1              : Field::new_ro(ADDR_TIMESTAMP1_REG      , 0, 8),
//             timestamp2              : Field::new_ro(ADDR_TIMESTAMP2_REG      , 0, 8),
//             timestamp3              : Field::new_ro(ADDR_TIMESTAMP3_REG      , 0, 8),
//             out_temp_h              : Field::new_ro(ADDR_OUT_TEMP_H          , 0, 8),
//             out_temp_l              : Field::new_ro(ADDR_OUT_TEMP_L          , 0, 8),
//             tda                     : Field::new_ro(ADDR_STATUS_REG          , 2, 1),
//             gda                     : Field::new_ro(ADDR_STATUS_REG          , 1, 1),
//             xlda                    : Field::new_ro(ADDR_STATUS_REG          , 0, 1),
//             freq_fine               : Field::new_ro(ADDR_INTERNAL_FREQ_FINE  , 0, 8),
//         }
//     }
// }

// pub struct FifoReader {
//     pub diff_fifo                   : Field,
//     pub fifo_wtm_ia                 : Field,
//     pub fifo_ovr_ia                 : Field,
//     pub fifo_full_ia                : Field,
//     pub counter_bdr_ia              : Field,
//     pub fifo_ovr_latched            : Field,
//     pub tag_sensor                  : Field,
//     pub tag_cnt                     : Field,
//     pub tag_parity                  : Field,
//     pub fifo_data_out_x_h           : Field,
//     pub fifo_data_out_x_l           : Field,
//     pub fifo_data_out_y_h           : Field,
//     pub fifo_data_out_y_l           : Field,
//     pub fifo_data_out_z_h           : Field,
//     pub fifo_data_out_z_l           : Field,
// }

// impl FifoReader {
//     pub fn new() -> Self {
//         Self {
//             diff_fifo                   : Field::new(   ADDR_FIFO_STATUS1        , 0, 10, true, 0, false, true),
//             fifo_wtm_ia                 : Field::new_ro(ADDR_FIFO_STATUS2        , 7, 1),
//             fifo_ovr_ia                 : Field::new_ro(ADDR_FIFO_STATUS2        , 6, 1),
//             fifo_full_ia                : Field::new_ro(ADDR_FIFO_STATUS2        , 5, 1),
//             counter_bdr_ia              : Field::new_ro(ADDR_FIFO_STATUS2        , 4, 1),
//             fifo_ovr_latched            : Field::new_ro(ADDR_FIFO_STATUS2        , 3, 1),
//             tag_sensor                  : Field::new_ro(ADDR_FIFO_DATA_OUT_TAG   , 3, 5),
//             tag_cnt                     : Field::new_ro(ADDR_FIFO_DATA_OUT_TAG   , 1, 2),
//             tag_parity                  : Field::new_ro(ADDR_FIFO_DATA_OUT_TAG   , 0, 1),
//             fifo_data_out_x_h           : Field::new_ro(ADDR_FIFO_DATA_OUT_X_H   , 0, 8),
//             fifo_data_out_x_l           : Field::new_ro(ADDR_FIFO_DATA_OUT_X_L   , 0, 8),
//             fifo_data_out_y_h           : Field::new_ro(ADDR_FIFO_DATA_OUT_Y_H   , 0, 8),
//             fifo_data_out_y_l           : Field::new_ro(ADDR_FIFO_DATA_OUT_Y_L   , 0, 8),
//             fifo_data_out_z_h           : Field::new_ro(ADDR_FIFO_DATA_OUT_Z_H   , 0, 8),
//             fifo_data_out_z_l           : Field::new_ro(ADDR_FIFO_DATA_OUT_Z_L   , 0, 8),
//         }
//     }
// }

// pub struct InterruptReader {
//     pub all_timestamp_end_count : Field,
//     pub all_sleep_change_ia     : Field,
//     pub all_d6d_ia              : Field,
//     pub all_wu_ia               : Field,
//     pub all_ff_ia               : Field,
//     pub wake_sleep_change_ia    : Field,
//     pub wake_ff_ia              : Field,
//     pub wake_sleep_state        : Field,
//     pub wake_wu_ia              : Field,
//     pub wake_x_wu               : Field,
//     pub wake_y_wu               : Field,
//     pub wake_z_wu               : Field,
//     pub d6d_den_drdy            : Field,
//     pub d6d_d6d_ia              : Field,
//     pub d6d_zh                  : Field,
//     pub d6d_zl                  : Field,
//     pub d6d_yh                  : Field,
//     pub d6d_yl                  : Field,
//     pub d6d_xh                  : Field,
//     pub d6d_xl                  : Field,
// }

// impl InterruptReader {
//     pub fn new() -> Self {
//         Self {
//             all_timestamp_end_count : Field::new_ro(ADDR_ALL_INT_SRC         , 7, 1),
//             all_sleep_change_ia     : Field::new_ro(ADDR_ALL_INT_SRC         , 5, 1),
//             all_d6d_ia              : Field::new_ro(ADDR_ALL_INT_SRC         , 4, 1),
//             all_wu_ia               : Field::new_ro(ADDR_ALL_INT_SRC         , 1, 1),
//             all_ff_ia               : Field::new_ro(ADDR_ALL_INT_SRC         , 0, 1),
//             wake_sleep_change_ia    : Field::new_ro(ADDR_WAKE_UP_SRC         , 6, 1),
//             wake_ff_ia              : Field::new_ro(ADDR_WAKE_UP_SRC         , 5, 1),
//             wake_sleep_state        : Field::new_ro(ADDR_WAKE_UP_SRC         , 4, 1),
//             wake_wu_ia              : Field::new_ro(ADDR_WAKE_UP_SRC         , 3, 1),
//             wake_x_wu               : Field::new_ro(ADDR_WAKE_UP_SRC         , 2, 1),
//             wake_y_wu               : Field::new_ro(ADDR_WAKE_UP_SRC         , 1, 1),
//             wake_z_wu               : Field::new_ro(ADDR_WAKE_UP_SRC         , 0, 1),
//             d6d_den_drdy            : Field::new_ro(ADDR_D6D_SRC             , 7, 1),
//             d6d_d6d_ia              : Field::new_ro(ADDR_D6D_SRC             , 6, 1),
//             d6d_zh                  : Field::new_ro(ADDR_D6D_SRC             , 5, 1),
//             d6d_zl                  : Field::new_ro(ADDR_D6D_SRC             , 4, 1),
//             d6d_yh                  : Field::new_ro(ADDR_D6D_SRC             , 3, 1),
//             d6d_yl                  : Field::new_ro(ADDR_D6D_SRC             , 2, 1),
//             d6d_xh                  : Field::new_ro(ADDR_D6D_SRC             , 1, 1),
//             d6d_xl                  : Field::new_ro(ADDR_D6D_SRC             , 0, 1),
//         }
//     }
// }

// pub struct WhoAmIReader {
//     pub who_am_i                : Field,
// }

// impl WhoAmIReader {
//     pub fn new() -> Self {
//         Self {
//             who_am_i                : Field::new_ro(ADDR_WHO_AM_I            , 0, 8),
//         }
//     }

//     pub fn read(&mut self, spibus: &mut impl SpiBus<u8>) -> Result<u8, Asm330Error> {
//         Ok(self.who_am_i.read(&mut self.spibus)? as u8)
//     }
// }

// pub struct Device<S: SpiBus<u8>> {
//     spibus: S,
//     whoamireader: WhoAmIReader,
// }

// impl<S: SpiBus<u8>> Device<S> {
//     pub fn new(spibus: impl SpiBus<u8>) -> Device<S> {
//         Self {
//             spibus,
//             whoamireader
//         }
//     }
//     pub fn read_whoami(&mut self) -> Result<u8, Asm330Error> {
//         self.whoamireader.read(self.spibus)
//     }
// }

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
