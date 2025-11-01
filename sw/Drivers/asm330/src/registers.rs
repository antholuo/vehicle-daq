#![allow(dead_code)] // allow dead code
#![cfg_attr(rustfmt, rustfmt_skip)] // skip formatting (alignment)

pub(super) const PIN_CTRL: u8 = 0x02;

pub(super) const FIFO_CTRL1: u8 = 0x07;
pub(super) const FIFO_CTRL2: u8 = 0x08;
pub(super) const FIFO_CTRL3: u8 = 0x09;
pub(super) const FIFO_CTRL4: u8 = 0x0A;

pub(super) const COUNTER_BDR_REG1: u8 = 0x0B;
pub(super) const COUNTER_BDR_REG2: u8 = 0x0C;

pub(super) const INT1_CTRL: u8 = 0x0D;
pub(super) const INT2_CTRL: u8 = 0x0E;

pub(super) const WHO_AM_I: u8 = 0x0F;

pub(super) const CTRL1_XL: u8 = 0x10;
pub(super) const CTRL2_G:  u8 = 0x11;
pub(super) const CTRL3_C:  u8 = 0x12;
pub(super) const CTRL4_C:  u8 = 0x13;
pub(super) const CTRL5_C:  u8 = 0x14;
pub(super) const CTRL6_C:  u8 = 0x15;
pub(super) const CTRL7_G:  u8 = 0x16;
pub(super) const CTRL8_XL: u8 = 0x17;
pub(super) const CTRL9_XL: u8 = 0x18;
pub(super) const CTRL10_C: u8 = 0x19;

pub(super) const ALL_INT_SRC: u8 = 0x1A;
pub(super) const WAKE_UP_SRC: u8 = 0x1B;

pub(super) const D6D_SRC: u8 = 0x1D;

pub(super) const STATUS_REG: u8 = 0x1E;

pub(super) const OUT_TEMP_L: u8 = 0x20;
pub(super) const OUT_TEMP_H: u8 = 0x21;

pub(super) const OUTX_L_G: u8 = 0x22;
pub(super) const OUTX_H_G: u8 = 0x23;
pub(super) const OUTY_L_G: u8 = 0x24;
pub(super) const OUTY_H_G: u8 = 0x25;
pub(super) const OUTZ_L_G: u8 = 0x26;
pub(super) const OUTZ_H_G: u8 = 0x27;
pub(super) const OUTX_L_A: u8 = 0x28;
pub(super) const OUTX_H_A: u8 = 0x29;
pub(super) const OUTY_L_A: u8 = 0x2A;
pub(super) const OUTY_H_A: u8 = 0x2B;
pub(super) const OUTZ_L_A: u8 = 0x2C;
pub(super) const OUTZ_H_A: u8 = 0x2D;

pub(super) const FIFO_STATUS1: u8 = 0x3A;
pub(super) const FIFO_STATUS2: u8 = 0x3B;

pub(super) const TIMESTAMP0: u8 = 0x40;
pub(super) const TIMESTAMP1: u8 = 0x41;
pub(super) const TIMESTAMP2: u8 = 0x42;
pub(super) const TIMESTAMP3: u8 = 0x43;

pub(super) const INT_CFG0: u8 = 0x56;
pub(super) const INT_CFG1: u8 = 0x58;

pub(super) const THS_6D: u8 = 0x59;
pub(super) const WAKE_UP_THS: u8 = 0x5B;
pub(super) const WAKE_UP_DUR: u8 = 0x5C;

pub(super) const FREE_FALL: u8 = 0x5D;

pub(super) const MD1_CFG: u8 = 0x5E;
pub(super) const MD2_CFG: u8 = 0x5F;

pub(super) const INTERNAL_FREQ_FINE: u8 = 0x63;

pub(super) const X_OFS_USR: u8 = 0x73;
pub(super) const Y_OFS_USR: u8 = 0x74;
pub(super) const Z_OFS_USR: u8 = 0x75;

pub(super) const FIFO_DATA_OUT_TAG: u8 = 0x78;
pub(super) const FIFO_DATA_OUT_X_L: u8 = 0x79;
pub(super) const FIFO_DATA_OUT_X_H: u8 = 0x7A;
pub(super) const FIFO_DATA_OUT_Y_L: u8 = 0x7B;
pub(super) const FIFO_DATA_OUT_Y_H: u8 = 0x7C;
pub(super) const FIFO_DATA_OUT_Z_L: u8 = 0x7D;
pub(super) const FIFO_DATA_OUT_Z_H: u8 = 0x7E;
