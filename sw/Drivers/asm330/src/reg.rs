/*
* Created date: 12/6/25
* File Description: Register addresses, values, offsets, and manipulation for asm330 driver
*/

use embedded_hal::spi::SpiBus;

#![allow(dead_code)]
// ----- Register Addresses -----
const ADDR_PIN_CTRL            : u8 = 0x02;
const ADDR_RESERVED__0         : u8 = 0x04;
const ADDR_RESERVED__1         : u8 = 0x05;
const ADDR_RESERVED__2         : u8 = 0x06;
const ADDR_FIFO_CTRL1          : u8 = 0x07;
const ADDR_FIFO_CTRL2          : u8 = 0x08;
const ADDR_FIFO_CTRL3          : u8 = 0x09;
const ADDR_FIFO_CTRL4          : u8 = 0x0A;
const ADDR_COUNTER_BDR_REG1    : u8 = 0x0B;
const ADDR_COUNTER_BDR_REG2    : u8 = 0x0C;
const ADDR_INT1_CTRL           : u8 = 0x0D;
const ADDR_INT2_CTRL           : u8 = 0x0E;
const ADDR_WHO_AM_I            : u8 = 0x0F;
const VALUE_WHO_AM_I           : u8 = 0x6A;
const ADDR_CTRL1_XL            : u8 = 0x10;
const ADDR_CTRL2_G             : u8 = 0x11;
const ADDR_CTRL3_C             : u8 = 0x12;
const ADDR_CTRL4_C             : u8 = 0x13;
const ADDR_CTRL5_C             : u8 = 0x14;
const ADDR_CTRL6_C             : u8 = 0x15;
const ADDR_CTRL7_G             : u8 = 0x16;
const ADDR_CTRL8_XL            : u8 = 0x17;
const ADDR_CTRL9_XL            : u8 = 0x18;
const ADDR_CTRL10_C            : u8 = 0x19;
const ADDR_ALL_INT_SRC         : u8 = 0x1A;
const ADDR_WAKE_UP_SRC         : u8 = 0x1B;
const ADDR_RESERVED__3         : u8 = 0x1C;
const ADDR_D6D_SRC             : u8 = 0x1D;
const ADDR_STATUS_REG          : u8 = 0x1E;
const ADDR_RESERVED__4         : u8 = 0x1F;
const ADDR_OUT_TEMP_L          : u8 = 0x20;
const ADDR_OUT_TEMP_H          : u8 = 0x21;
const ADDR_OUTX_L_G            : u8 = 0x22;
const ADDR_OUTX_H_G            : u8 = 0x23;
const ADDR_OUTY_L_G            : u8 = 0x24;
const ADDR_OUTY_H_G            : u8 = 0x25;
const ADDR_OUTZ_L_G            : u8 = 0x26;
const ADDR_OUTZ_H_G            : u8 = 0x27;
const ADDR_OUTX_L_A            : u8 = 0x28;
const ADDR_OUTX_H_A            : u8 = 0x29;
const ADDR_OUTY_L_A            : u8 = 0x2A;
const ADDR_OUTY_H_A            : u8 = 0x2B;
const ADDR_OUTZ_L_A            : u8 = 0x2C;
const ADDR_OUTZ_H_A            : u8 = 0x2D;
const ADDR_RESERVED__5         : u8 = 0x2E;
const ADDR_RESERVED__6         : u8 = 0x2F;
const ADDR_RESERVED__7         : u8 = 0x30;
const ADDR_RESERVED__8         : u8 = 0x31;
const ADDR_RESERVED__9         : u8 = 0x32;
const ADDR_RESERVED__10        : u8 = 0x33;
const ADDR_RESERVED__11        : u8 = 0x34;
const ADDR_RESERVED__12        : u8 = 0x35;
const ADDR_RESERVED__13        : u8 = 0x36;
const ADDR_RESERVED__14        : u8 = 0x37;
const ADDR_RESERVED__15        : u8 = 0x38;
const ADDR_RESERVED__16        : u8 = 0x39;
const ADDR_FIFO_STATUS1        : u8 = 0x3A;
const ADDR_FIFO_STATUS2        : u8 = 0x3B;
const ADDR_RESERVED__17        : u8 = 0x3C;
const ADDR_RESERVED__18        : u8 = 0x3E;
const ADDR_RESERVED__19        : u8 = 0x3F;
const ADDR_TIMESTAMP0_REG      : u8 = 0x40;
const ADDR_TIMESTAMP1_REG      : u8 = 0x41;
const ADDR_TIMESTAMP2_REG      : u8 = 0x42;
const ADDR_TIMESTAMP3_REG      : u8 = 0x43;
const ADDR_RESERVED__20        : u8 = 0x44;
const ADDR_RESERVED__21        : u8 = 0x45;
const ADDR_RESERVED__22        : u8 = 0x46;
const ADDR_RESERVED__23        : u8 = 0x47;
const ADDR_RESERVED__24        : u8 = 0x48;
const ADDR_RESERVED__25        : u8 = 0x49;
const ADDR_RESERVED__26        : u8 = 0x4A;
const ADDR_RESERVED__27        : u8 = 0x4B;
const ADDR_RESERVED__28        : u8 = 0x4C;
const ADDR_RESERVED__29        : u8 = 0x4D;
const ADDR_RESERVED__30        : u8 = 0x4E;
const ADDR_RESERVED__31        : u8 = 0x4F;
const ADDR_RESERVED__32        : u8 = 0x50;
const ADDR_RESERVED__33        : u8 = 0x51;
const ADDR_RESERVED__34        : u8 = 0x52;
const ADDR_RESERVED__35        : u8 = 0x53;
const ADDR_RESERVED__36        : u8 = 0x54;
const ADDR_RESERVED__37        : u8 = 0x55;
const ADDR_INT_CFG0            : u8 = 0x56;
const ADDR_RESERVED__38        : u8 = 0x57;
const ADDR_INT_CFG1            : u8 = 0x58;
const ADDR_THS_6D              : u8 = 0x59;
const ADDR_RESERVED__39        : u8 = 0x5A;
const ADDR_WAKE_UP_THS         : u8 = 0x5B;
const ADDR_WAKE_UP_DUR         : u8 = 0x5C;
const ADDR_FREE_FALL           : u8 = 0x5D;
const ADDR_MD1_CFG             : u8 = 0x5E;
const ADDR_MD2_CFG             : u8 = 0x5F;
const ADDR_RESERVED__40        : u8 = 0x60;
const ADDR_RESERVED__41        : u8 = 0x61;
const ADDR_RESERVED__42        : u8 = 0x62;
const ADDR_INTERNAL_FREQ_FINE  : u8 = 0x63;
const ADDR_RESERVED__43        : u8 = 0x64;
const ADDR_RESERVED__44        : u8 = 0x65;
const ADDR_RESERVED__45        : u8 = 0x66;
const ADDR_RESERVED__46        : u8 = 0x67;
const ADDR_RESERVED__47        : u8 = 0x68;
const ADDR_RESERVED__48        : u8 = 0x69;
const ADDR_RESERVED__49        : u8 = 0x6A;
const ADDR_RESERVED__50        : u8 = 0x6B;
const ADDR_RESERVED__51        : u8 = 0x6C;
const ADDR_RESERVED__52        : u8 = 0x6D;
const ADDR_RESERVED__53        : u8 = 0x6E;
const ADDR_RESERVED__54        : u8 = 0x6F;
const ADDR_RESERVED__55        : u8 = 0x70;
const ADDR_RESERVED__56        : u8 = 0x71;
const ADDR_RESERVED__57        : u8 = 0x72;
const ADDR_X_OFS_USR           : u8 = 0x73;
const ADDR_Y_OFS_USR           : u8 = 0x74;
const ADDR_Z_OFS_USR           : u8 = 0x75;
const ADDR_RESERVED__58        : u8 = 0x76;
const ADDR_RESERVED__59        : u8 = 0x77;
const ADDR_FIFO_DATA_OUT_TAG   : u8 = 0x78;
const ADDR_FIFO_DATA_OUT_X_L   : u8 = 0x79;
const ADDR_FIFO_DATA_OUT_X_H   : u8 = 0x7A;
const ADDR_FIFO_DATA_OUT_Y_L   : u8 = 0x7B;
const ADDR_FIFO_DATA_OUT_Y_H   : u8 = 0x7C;
const ADDR_FIFO_DATA_OUT_Z_L   : u8 = 0x7D;
const ADDR_FIFO_DATA_OUT_Z_H   : u8 = 0x7E;
const ADDR_RESERVED__60        : u8 = 0x7F;

fn read_register<S>(spi: &mut S, reg_addr: u8) -> Result<u8, S::Error>
where
    S: SpiBus<u8>,
    S::Error: core::fmt::Debug,
{
    let read_cmd = reg_addr | 0x80;
    let mut buf: [u8; 2] = [read_cmd, 0x00];

    spi.transfer_in_place(&mut buf)?;

    Ok(buf[1])
}

fn write_register<S>(spi: &mut S, reg_addr: u8, data: u8) -> Result<(), S::Error>
where
    S: SpiBus<u8>,
    S::Error: core::fmt::Debug,
{
    let write_cmd = reg_addr & 0x7F;
    let buf: [u8; 2] = [write_cmd, data];

    spi.write(&buf)?;

    Ok(())
}

// const ADDR_FIFO_CTRL1: u8 = 0x07;
// const ADDR_FIFO_CTRL2: u8 = 0x08;
// const ADDR_FIFO_CTRL3: u8 = 0x09;
// const ADDR_FIFO_CTRL4: u8 = 0x0A;
// const ADDR_COUNTER_BDR1: u8 = 0x0B;
// const ADDR_COUNTER_BDR2: u8 = 0x0C;

// const ADDR_INT1_CTRL: u8 = 0x0D;
// const ADDR_INT2_CTRL: u8 = 0x0C;

// const ADDR_WHOAMI: u8 = 0x0F;

// const ADDR_CTRL1_XL: u8 = 0x10;
// const ADDR_CTRL2_G: u8 = 0x11;
// const ADDR_CTRL3_C: u8 = 0x12;
// const ADDR_CTRL4_C: u8 = 0x13;
// const ADDR_CTRL5_C: u8 = 0x14;
// const ADDR_CTRL6_C: u8 = 0x15;
// const ADDR_CTRL7_G: u8 = 0x16;
// const ADDR_CTRL8_XL: u8 = 0x17;
// const ADDR_CTRL9_XL: u8 = 0x18;
// const ADDR_CTRL10_C: u8 = 0x19;

// const ADDR_OUTX_L_G: u8 = 0x22;
// const ADDR_OUTX_H_G: u8 = 0x23;
// const ADDR_OUTY_L_G: u8 = 0x24;
// const ADDR_OUTY_H_G: u8 = 0x25;
// const ADDR_OUTZ_L_G: u8 = 0x26;
// const ADDR_OUTZ_H_G: u8 = 0x27;

// const ADDR_OUTX_L_A: u8 = 0x28;
// const ADDR_OUTX_H_A: u8 = 0x29;
// const ADDR_OUTY_L_A: u8 = 0x2A;
// const ADDR_OUTY_H_A: u8 = 0x2B;
// const ADDR_OUTZ_L_A: u8 = 0x2C;
// const ADDR_OUTZ_H_A: u8 = 0x2D;

// const ADDR_FIFO_DATA_OUT_X_L: u8 = 0x79;
// const ADDR_FIFO_DATA_OUT_X_H: u8 = 0x7A;
// const ADDR_FIFO_DATA_OUT_Y_L: u8 = 0x7B;
// const ADDR_FIFO_DATA_OUT_Y_H: u8 = 0x7C;
// const ADDR_FIFO_DATA_OUT_Z_L: u8 = 0x7D;
// const ADDR_FIFO_DATA_OUT_Z_H: u8 = 0x7E;
