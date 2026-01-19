#![no_std]
/*
* Created date: 12/20/25
* File Description: asm330 device state
*/

mod field;
mod common;
mod registers;

use common::*;
use field::*;
use registers::*;

use embedded_hal::spi::SpiBus;

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
