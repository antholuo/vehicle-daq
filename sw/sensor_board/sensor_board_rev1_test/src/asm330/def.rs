/*
* Created date: 12/6/25
* File Description: Common definitions used throughout IMU driver
*/

#![allow(dead_code)]

// ----- Register Addresses -----
const REG_PIN_CTRL            : u8 = 0x00;
const REG_FIFO_CTRL1          : u8 = 0x00;
const REG_FIFO_CTRL2          : u8 = 0x00;
const REG_FIFO_CTRL3          : u8 = 0x00;
const REG_FIFO_CTRL4          : u8 = 0x00;
const REG_COUNTER_BDR_REG1    : u8 = 0x00;
const REG_COUNTER_BDR_REG2:   : u8 = 0x00;
const REG_INT1_CTRL           : u8 = 0x00;
const REG_INT2_CTRL           : u8 = 0x00;
const REG_WHO_AM_I            : u8 = 0x00;
const REG_CTRL1_XL            : u8 = 0x00;
const REG_CTRL2_G             : u8 = 0x00;
const REG_CTRL3_C             : u8 = 0x00;
const REG_CTRL4_C             : u8 = 0x00;
const REG_CTRL5_C             : u8 = 0x00;
const REG_CTRL6_C             : u8 = 0x00;
const REG_CTRL7_G             : u8 = 0x00;
const REG_CTRL8_XL            : u8 = 0x00;
const REG_CTRL9_XL            : u8 = 0x00;
const REG_CTRL10_C            : u8 = 0x00;
const REG_ALL_INT_SRC         : u8 = 0x00;
const REG_WAKE_UP_SRC         : u8 = 0x00;
const REG_D6D_SRC             : u8 = 0x00;
const REG_STATUS_REG          : u8 = 0x00;
const REG_OUT_TEMP_L          : u8 = 0x00;
const REG_OUT_TEMP_H          : u8 = 0x00;
const REG_OUTX_L_G            : u8 = 0x00;
const REG_OUTX_H_G            : u8 = 0x00;
const REG_OUTY_L_G            : u8 = 0x00;
const REG_OUTY_H_G            : u8 = 0x00;
const REG_OUTZ_L_G            : u8 = 0x00;
const REG_OUTZ_H_G            : u8 = 0x00;
const REG_OUTX_L_A            : u8 = 0x00;
const REG_OUTX_H_A            : u8 = 0x00;
const REG_OUTY_L_A            : u8 = 0x00;
const REG_OUTY_H_A            : u8 = 0x00;
const REG_OUTZ_L_A            : u8 = 0x00;
const REG_OUTZ_H_A            : u8 = 0x00;
const REG_FIFO_STATUS1        : u8 = 0x00;
const REG_FIFO_STATUS2        : u8 = 0x00;
const REG_TIMESTAMP0_REG      : u8 = 0x00;
const REG_TIMESTAMP1_REG      : u8 = 0x00;
const REG_TIMESTAMP2_REG      : u8 = 0x00;
const REG_TIMESTAMP3_REG      : u8 = 0x00;
const REG_INT_CFG0            : u8 = 0x00;
const REG_INT_CFG1            : u8 = 0x00;
const REG_THS_6D              : u8 = 0x00;
const REG_WAKE_UP_THS         : u8 = 0x00;
const REG_WAKE_UP_DUR         : u8 = 0x00;
const REG_FREE_FALL           : u8 = 0x00;
const REG_MD1_CFG             : u8 = 0x00;
const REG_MD2_CFG             : u8 = 0x00;
const REG_INTERNAL_FREQ_FINE  : u8 = 0x00;
const REG_X_OFS_USR           : u8 = 0x00;
const REG_Y_OFS_USR           : u8 = 0x00;
const REG_Z_OFS_USR           : u8 = 0x00;
const REG_FIFO_DATA_OUT_TAG   : u8 = 0x00;
const REG_FIFO_DATA_OUT_X_L   : u8 = 0x00;
const REG_FIFO_DATA_OUT_X_H   : u8 = 0x00;
const REG_FIFO_DATA_OUT_Y_L   : u8 = 0x00;
const REG_FIFO_DATA_OUT_Y_H   : u8 = 0x00;
const REG_FIFO_DATA_OUT_Z_L   : u8 = 0x00;
const REG_FIFO_DATA_OUT_Z_H   : u8 = 0x00;

// const REG_FIFO_CTRL1: u8 = 0x07;
// const REG_FIFO_CTRL2: u8 = 0x08;
// const REG_FIFO_CTRL3: u8 = 0x09;
// const REG_FIFO_CTRL4: u8 = 0x0A;
// const REG_COUNTER_BDR1: u8 = 0x0B;
// const REG_COUNTER_BDR2: u8 = 0x0C;

// const REG_INT1_CTRL: u8 = 0x0D;
// const REG_INT2_CTRL: u8 = 0x0C;

// const REG_WHOAMI: u8 = 0x0F;

// const REG_CTRL1_XL: u8 = 0x10;
// const REG_CTRL2_G: u8 = 0x11;
// const REG_CTRL3_C: u8 = 0x12;
// const REG_CTRL4_C: u8 = 0x13;
// const REG_CTRL5_C: u8 = 0x14;
// const REG_CTRL6_C: u8 = 0x15;
// const REG_CTRL7_G: u8 = 0x16;
// const REG_CTRL8_XL: u8 = 0x17;
// const REG_CTRL9_XL: u8 = 0x18;
// const REG_CTRL10_C: u8 = 0x19;

// const REG_OUTX_L_G: u8 = 0x22;
// const REG_OUTX_H_G: u8 = 0x23;
// const REG_OUTY_L_G: u8 = 0x24;
// const REG_OUTY_H_G: u8 = 0x25;
// const REG_OUTZ_L_G: u8 = 0x26;
// const REG_OUTZ_H_G: u8 = 0x27;

// const REG_OUTX_L_A: u8 = 0x28;
// const REG_OUTX_H_A: u8 = 0x29;
// const REG_OUTY_L_A: u8 = 0x2A;
// const REG_OUTY_H_A: u8 = 0x2B;
// const REG_OUTZ_L_A: u8 = 0x2C;
// const REG_OUTZ_H_A: u8 = 0x2D;

// const REG_FIFO_DATA_OUT_X_L: u8 = 0x79;
// const REG_FIFO_DATA_OUT_X_H: u8 = 0x7A;
// const REG_FIFO_DATA_OUT_Y_L: u8 = 0x7B;
// const REG_FIFO_DATA_OUT_Y_H: u8 = 0x7C;
// const REG_FIFO_DATA_OUT_Z_L: u8 = 0x7D;
// const REG_FIFO_DATA_OUT_Z_H: u8 = 0x7E;
