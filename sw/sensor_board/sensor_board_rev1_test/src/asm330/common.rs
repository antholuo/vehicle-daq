#![allow(dead_code)]

// ----- Register Addresses -----
const REG_FIFO_CTRL1: u8 = 0x07;
const REG_FIFO_CTRL2: u8 = 0x08;
const REG_FIFO_CTRL3: u8 = 0x09;
const REG_FIFO_CTRL4: u8 = 0x0A;
const REG_COUNTER_BDR1: u8 = 0x0B;
const REG_COUNTER_BDR2: u8 = 0x0C;

const REG_INT1_CTRL: u8 = 0x0D;
const REG_INT2_CTRL: u8 = 0x0C;

const REG_WHOAMI: u8 = 0x0F;

const REG_CTRL1_XL: u8 = 0x10;
const REG_CTRL2_G: u8 = 0x11;
const REG_CTRL3_C: u8 = 0x12;
const REG_CTRL4_C: u8 = 0x13;
const REG_CTRL5_C: u8 = 0x14;
const REG_CTRL6_C: u8 = 0x15;
const REG_CTRL7_G: u8 = 0x16;
const REG_CTRL8_XL: u8 = 0x17;
const REG_CTRL9_XL: u8 = 0x18;
const REG_CTRL10_C: u8 = 0x19;

const REG_OUTX_L_G: u8 = 0x22;
const REG_OUTX_H_G: u8 = 0x23;
const REG_OUTY_L_G: u8 = 0x24;
const REG_OUTY_H_G: u8 = 0x25;
const REG_OUTZ_L_G: u8 = 0x26;
const REG_OUTZ_H_G: u8 = 0x27;

const REG_OUTX_L_A: u8 = 0x28;
const REG_OUTX_H_A: u8 = 0x29;
const REG_OUTY_L_A: u8 = 0x2A;
const REG_OUTY_H_A: u8 = 0x2B;
const REG_OUTZ_L_A: u8 = 0x2C;
const REG_OUTZ_H_A: u8 = 0x2D;

const REG_FIFO_DATA_OUT_X_L: u8 = 0x79;
const REG_FIFO_DATA_OUT_X_H: u8 = 0x7A;
const REG_FIFO_DATA_OUT_Y_L: u8 = 0x7B;
const REG_FIFO_DATA_OUT_Y_H: u8 = 0x7C;
const REG_FIFO_DATA_OUT_Z_L: u8 = 0x7D;
const REG_FIFO_DATA_OUT_Z_H: u8 = 0x7E;
