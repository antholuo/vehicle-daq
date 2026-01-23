#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use modular_bitfield::prelude::*;

use crate::common::*;

/// Defines an 8-bit register with bitfields and an associated address
/// Note: first field is least significant (right most in this case)
macro_rules! define_register {
    ($name:ident, $addr:expr, { $($field:tt)* }) => {
        #[bitfield(bits = 8)]
        #[repr(u8)]
        #[derive(Clone, Copy)]
        pub struct $name {
            $($field)*
        }

        impl $name {
            pub const ADDR: u8 = $addr;
        }
    };
}

define_register!(PIN_CTRL, 0x02, {
    #[skip] __: B6,
    pub sdo_pu_en: B1,
    #[skip] __: B1,
});

define_register!(FIFO_CTRL1, 0x07, {
    pub wtm_lower: B8,
});

define_register!(FIFO_CTRL2, 0x08, {
    pub wtm_upper: B1,
    #[skip] __: B3,
    pub odrchg_en: B1,
    #[skip] __: B2,
    pub stop_on_wtm: B1
});

define_register!(FIFO_CTRL3, 0x09, {
    pub bdr_xl: Odr,
    pub bdr_gy: Odr,
});

define_register!(FIFO_CTRL4, 0x0A, {
    pub fifo_mode: FifoMode,
    #[skip] __: B1,
    pub odr_t_batch: TempBatchOdr,
    pub dec_ts_batch: TimestampDecimation,
});

define_register!(COUNTER_BDR_REG1, 0x0B, {
    pub cnt_bdr_th_upper: B2,
    #[skip] __: B3,
    pub trig_counter_bdr: B1,
    pub rst_counter_bdr: B1,
    pub dataready_pulsed: B1,
});

define_register!(COUNTER_BDR_REG2, 0x0C, { pub cnt_bdr_th_lower: B8 });

define_register!(INT1_CTRL, 0x0D, {
    pub int1_drdy_xl: B1,
    pub int1_drdy_g: B1,
    pub int1_boot: B1,
    pub int1_fifo_th: B1,
    pub int1_fifo_ovr: B1,
    pub int1_fifo_full: B1,
    pub int1_cnt_bdr: B1,
    pub den_drdy_flag: B1
});

define_register!(INT2_CTRL, 0x0E, {
    pub int2_drdy_xl: B1,
    pub int2_drdy_g: B1,
    pub int2_drdy_temp: B1,
    pub int2_fifo_th: B1,
    pub int2_fifo_ovr: B1,
    pub int2_fifo_full: B1,
    pub int2_cnt_bdr: B1,
    #[skip] __: B1,
});

define_register!(WHO_AM_I, 0x0F, { pub data: B8 });

define_register!(CTRL1_XL, 0x10, {
    #[skip] __: B1,
    pub lpf2_xl_en: B1,
    pub fs_xl: AccelScale,
    pub odr_xl: Odr,
});

define_register!(CTRL2_G, 0x11, {
    pub odr_g: Odr,
    pub fs_g: GyroScale,
});

define_register!(CTRL3_C, 0x12, {
    pub sw_reset: B1,
    #[skip] __: B1,
    pub if_inc: B1,
    pub sim: B1,
    pub pp_od: B1,
    pub h_lactive: B1,
    pub bdu: B1,
    pub boot: B1,
});

define_register!(CTRL4_C, 0x13, {
    #[skip] __: B1,
    pub lpf1_sel_g: B1,
    pub i2c_disable: B1,
    pub drdy_mask: B1,
    #[skip] __: B1,
    pub int2_on_int1: B1,
    pub sleep_g: B1,
    #[skip] __: B1,
});

define_register!(CTRL5_C, 0x14, {
    pub st_xl: AccelST,
    pub st_g: GyroST,
    #[skip] __: B1,
    pub rounding: Rounding,
    #[skip] __: B1,
});

define_register!(CTRL6_C, 0x15, {
    pub ftype: B3,
    pub usr_off_w: B1,
    #[skip] __: B1,
    pub trig_mode: TriggerMode
});

define_register!(CTRL7_G, 0x16, {
    #[skip] __: B1,
    pub usr_off_on_out: B1,
    #[skip] __: B2,
    pub hpm_g: GyroHighPassMode,
    pub hp_en_g: B1,
    #[skip] __: B1,
});

define_register!(CTRL8_XL, 0x17, {
    pub low_pass_on_6d: B1,
    #[skip] __: B1,
    pub hp_slope_xl_en: B1,
    pub fastsettle_mode_xl: B1,
    pub hp_ref_mode_xl: B1,
    pub hpcf_xl: B3,
});

define_register!(CTRL9_XL, 0x18, {
    #[skip] __: B1,
    pub device_conf: B1,
    pub den_lh: B1,
    pub den_xl_en: B1,
    pub den_xl_g: B1,
    pub den_z: B1,
    pub den_y: B1,
    pub den_x: B1,
});

define_register!(CTRL10_C, 0x19, {
    #[skip] __: B5,
    pub timestamp_en: B1,
    #[skip] __: B2,
});

define_register!(ALL_INT_SRC, 0x1A, {
    pub ff_ia: B1,
    pub wu_ia: B1,
    #[skip] __: B2,
    pub d6d_ia: B1,
    pub sleep_change_ia: B1,
    #[skip] __: B1,
    pub timestamp_endcount: B1,
});

define_register!(WAKE_UP_SRC, 0x1B, {
    pub z_wu: B1,
    pub y_wu: B1,
    pub x_wu: B1,
    pub wu_ia: B1,
    pub sleep_state: B1,
    pub ff_ia: B1,
    pub sleep_change_ia: B1,
    #[skip] __: B1
});

define_register!(D6D_SRC, 0x1D, {
    pub xl: B1,
    pub xh: B1,
    pub yl: B1,
    pub yh: B1,
    pub zl: B1,
    pub zh: B1,
    pub d6d_ia: B1,
    pub den_drdy: B1,
});

define_register!(STATUS_REG, 0x1E, {
    pub xlda: B1,
    pub gda: B1,
    pub tda: B1,
    #[skip] __: B5,
});

define_register!(OUT_TEMP_L, 0x20, {pub data: B8 });
define_register!(OUT_TEMP_H, 0x21, {pub data: B8 });

define_register!(OUTX_L_G, 0x22, { pub data: B8 });
define_register!(OUTX_H_G, 0x23, { pub data: B8 });
define_register!(OUTY_L_G, 0x24, { pub data: B8 });
define_register!(OUTY_H_G, 0x25, { pub data: B8 });
define_register!(OUTZ_L_G, 0x26, { pub data: B8 });
define_register!(OUTZ_H_G, 0x27, { pub data: B8 });

define_register!(OUTX_L_A, 0x28, { pub data: B8 });
define_register!(OUTX_H_A, 0x29, { pub data: B8 });
define_register!(OUTY_L_A, 0x2A, { pub data: B8 });
define_register!(OUTY_H_A, 0x2B, { pub data: B8 });
define_register!(OUTZ_L_A, 0x2C, { pub data: B8 });
define_register!(OUTZ_H_A, 0x2D, { pub data: B8 });

define_register!(FIFO_STATUS1, 0x3A, {pub diff_fifo_lower: B8});

define_register!(FIFO_STATUS2, 0x3B, {
    pub diff_fifo_upper: B2,
    #[skip] __: B1,
    pub fifo_ovr_latched: B1,
    pub counter_bdr_ia: B1,
    pub fifo_full_ia: B1,
    pub fifo_ovr_ia: B1,
    pub fifo_wtm_ia: B1,
});

define_register!(TIMESTAMP0, 0x40, { pub data: B8 });
define_register!(TIMESTAMP1, 0x41, { pub data: B8 });
define_register!(TIMESTAMP2, 0x42, { pub data: B8 });
define_register!(TIMESTAMP3, 0x43, { pub data: B8 });

define_register!(INT_CFG0, 0x56, {
    pub lir: B1,
    #[skip] __: B3,
    pub slope_fds: B1,
    pub sleep_status_on_int: B1,
    pub int_clr_on_read: B1,
    #[skip] __: B1,
});

define_register!(INT_CFG1, 0x58, {
    #[skip] __: B5,
    pub inact: InactMode,
    pub interrupts_enable: B1,
});

define_register!(THS_6D, 0x59, {
    #[skip] __: B5,
    pub sixd_ths: SixDThresh,
    pub d4d_en: B1,
});

define_register!(WAKE_UP_THS, 0x5B, {
    pub wk_ths: B6,
    pub usr_off_on_wu: B1,
    #[skip] __: B1,
});

define_register!(WAKE_UP_DUR, 0x5C, {
    pub sleep_dur: B4,
    pub wake_ths_w: B1,
    pub wake_dur: B2,
    pub ff_dur_upper: B1,
});

define_register!(FREE_FALL, 0x5D, {
    pub ff_ths: FreeFallThresh,
    pub ff_dur_lower: B5
});

define_register!(MD1_CFG, 0x5E, {
    #[skip] __: B2,
    pub int1_6d: B1,
    #[skip] __: B1,
    pub int1_ff: B1,
    pub int1_wu: B1,
    #[skip] __: B1,
    pub int1_sleep_change: B1,
});

define_register!(MD2_CFG, 0x5F, {
    pub int2_timestamp: B1,
    #[skip] __: B1,
    pub int2_6d: B1,
    #[skip] __: B1,
    pub int2_ff: B1,
    pub int2_wu: B1,
    #[skip] __: B1,
    pub int2_sleep_change: B1,
});

define_register!(INTERNAL_FREQ_FINE, 0x63, { pub data: B8 });

define_register!(X_OFS_USR, 0x73, { pub data: B8 });
define_register!(Y_OFS_USR, 0x74, { pub data: B8 });
define_register!(Z_OFS_USR, 0x75, { pub data: B8 });

define_register!(FIFO_DATA_OUT_TAG, 0x78, { pub data: B8 });

define_register!(FIFO_DATA_OUT_X_L, 0x79, { pub data: B8 });
define_register!(FIFO_DATA_OUT_X_H, 0x7A, { pub data: B8 });

define_register!(FIFO_DATA_OUT_Y_L, 0x7B, { pub data: B8 });
define_register!(FIFO_DATA_OUT_Y_H, 0x7C, { pub data: B8 });

define_register!(FIFO_DATA_OUT_Z_L, 0x7D, { pub data: B8 });
define_register!(FIFO_DATA_OUT_Z_H, 0x7E, { pub data: B8 });
