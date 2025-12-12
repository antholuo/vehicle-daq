#![allow(dead_code)]
/*
 * Created date: 12/6/25
 * File Description: Register addresses, values, offsets, and manipulation for asm330 driver
*/

use embedded_hal::spi::SpiBus;
use log::trace;
use core::result::Result;
use core::result::Result::Ok;

// ----- Register Addresses -----
pub const ADDR_PIN_CTRL                  : u8 = 0x02;
pub const ADDR_RESERVED__0               : u8 = 0x04;
pub const ADDR_RESERVED__1               : u8 = 0x05;
pub const ADDR_RESERVED__2               : u8 = 0x06;
pub const ADDR_FIFO_CTRL1                : u8 = 0x07;
pub const ADDR_FIFO_CTRL2                : u8 = 0x08;
pub const ADDR_FIFO_CTRL3                : u8 = 0x09;
pub const ADDR_FIFO_CTRL4                : u8 = 0x0A;
pub const ADDR_COUNTER_BDR_REG1          : u8 = 0x0B;
pub const ADDR_COUNTER_BDR_REG2          : u8 = 0x0C;
pub const ADDR_INT1_CTRL                 : u8 = 0x0D;
pub const ADDR_INT2_CTRL                 : u8 = 0x0E;
pub const ADDR_WHO_AM_I                  : u8 = 0x0F;
pub const ADDR_CTRL1_XL                  : u8 = 0x10;
pub const ADDR_CTRL2_G                   : u8 = 0x11;
pub const ADDR_CTRL3_C                   : u8 = 0x12;
pub const ADDR_CTRL4_C                   : u8 = 0x13;
pub const ADDR_CTRL5_C                   : u8 = 0x14;
pub const ADDR_CTRL6_C                   : u8 = 0x15;
pub const ADDR_CTRL7_G                   : u8 = 0x16;
pub const ADDR_CTRL8_XL                  : u8 = 0x17;
pub const ADDR_CTRL9_XL                  : u8 = 0x18;
pub const ADDR_CTRL10_C                  : u8 = 0x19;
pub const ADDR_ALL_INT_SRC               : u8 = 0x1A;
pub const ADDR_WAKE_UP_SRC               : u8 = 0x1B;
pub const ADDR_RESERVED__3               : u8 = 0x1C;
pub const ADDR_D6D_SRC                   : u8 = 0x1D;
pub const ADDR_STATUS_REG                : u8 = 0x1E;
pub const ADDR_RESERVED__4               : u8 = 0x1F;
pub const ADDR_OUT_TEMP_L                : u8 = 0x20;
pub const ADDR_OUT_TEMP_H                : u8 = 0x21;
pub const ADDR_OUTX_L_G                  : u8 = 0x22;
pub const ADDR_OUTX_H_G                  : u8 = 0x23;
pub const ADDR_OUTY_L_G                  : u8 = 0x24;
pub const ADDR_OUTY_H_G                  : u8 = 0x25;
pub const ADDR_OUTZ_L_G                  : u8 = 0x26;
pub const ADDR_OUTZ_H_G                  : u8 = 0x27;
pub const ADDR_OUTX_L_A                  : u8 = 0x28;
pub const ADDR_OUTX_H_A                  : u8 = 0x29;
pub const ADDR_OUTY_L_A                  : u8 = 0x2A;
pub const ADDR_OUTY_H_A                  : u8 = 0x2B;
pub const ADDR_OUTZ_L_A                  : u8 = 0x2C;
pub const ADDR_OUTZ_H_A                  : u8 = 0x2D;
pub const ADDR_RESERVED__5               : u8 = 0x2E;
pub const ADDR_RESERVED__6               : u8 = 0x2F;
pub const ADDR_RESERVED__7               : u8 = 0x30;
pub const ADDR_RESERVED__8               : u8 = 0x31;
pub const ADDR_RESERVED__9               : u8 = 0x32;
pub const ADDR_RESERVED__10              : u8 = 0x33;
pub const ADDR_RESERVED__11              : u8 = 0x34;
pub const ADDR_RESERVED__12              : u8 = 0x35;
pub const ADDR_RESERVED__13              : u8 = 0x36;
pub const ADDR_RESERVED__14              : u8 = 0x37;
pub const ADDR_RESERVED__15              : u8 = 0x38;
pub const ADDR_RESERVED__16              : u8 = 0x39;
pub const ADDR_FIFO_STATUS1              : u8 = 0x3A;
pub const ADDR_FIFO_STATUS2              : u8 = 0x3B;
pub const ADDR_RESERVED__17              : u8 = 0x3C;
pub const ADDR_RESERVED__18              : u8 = 0x3E;
pub const ADDR_RESERVED__19              : u8 = 0x3F;
pub const ADDR_TIMESTAMP0_REG            : u8 = 0x40;
pub const ADDR_TIMESTAMP1_REG            : u8 = 0x41;
pub const ADDR_TIMESTAMP2_REG            : u8 = 0x42;
pub const ADDR_TIMESTAMP3_REG            : u8 = 0x43;
pub const ADDR_RESERVED__20              : u8 = 0x44;
pub const ADDR_RESERVED__21              : u8 = 0x45;
pub const ADDR_RESERVED__22              : u8 = 0x46;
pub const ADDR_RESERVED__23              : u8 = 0x47;
pub const ADDR_RESERVED__24              : u8 = 0x48;
pub const ADDR_RESERVED__25              : u8 = 0x49;
pub const ADDR_RESERVED__26              : u8 = 0x4A;
pub const ADDR_RESERVED__27              : u8 = 0x4B;
pub const ADDR_RESERVED__28              : u8 = 0x4C;
pub const ADDR_RESERVED__29              : u8 = 0x4D;
pub const ADDR_RESERVED__30              : u8 = 0x4E;
pub const ADDR_RESERVED__31              : u8 = 0x4F;
pub const ADDR_RESERVED__32              : u8 = 0x50;
pub const ADDR_RESERVED__33              : u8 = 0x51;
pub const ADDR_RESERVED__34              : u8 = 0x52;
pub const ADDR_RESERVED__35              : u8 = 0x53;
pub const ADDR_RESERVED__36              : u8 = 0x54;
pub const ADDR_RESERVED__37              : u8 = 0x55;
pub const ADDR_INT_CFG0                  : u8 = 0x56;
pub const ADDR_RESERVED__38              : u8 = 0x57;
pub const ADDR_INT_CFG1                  : u8 = 0x58;
pub const ADDR_THS_6D                    : u8 = 0x59;
pub const ADDR_RESERVED__39              : u8 = 0x5A;
pub const ADDR_WAKE_UP_THS               : u8 = 0x5B;
pub const ADDR_WAKE_UP_DUR               : u8 = 0x5C;
pub const ADDR_FREE_FALL                 : u8 = 0x5D;
pub const ADDR_MD1_CFG                   : u8 = 0x5E;
pub const ADDR_MD2_CFG                   : u8 = 0x5F;
pub const ADDR_RESERVED__40              : u8 = 0x60;
pub const ADDR_RESERVED__41              : u8 = 0x61;
pub const ADDR_RESERVED__42              : u8 = 0x62;
pub const ADDR_INTERNAL_FREQ_FINE        : u8 = 0x63;
pub const ADDR_RESERVED__43              : u8 = 0x64;
pub const ADDR_RESERVED__44              : u8 = 0x65;
pub const ADDR_RESERVED__45              : u8 = 0x66;
pub const ADDR_RESERVED__46              : u8 = 0x67;
pub const ADDR_RESERVED__47              : u8 = 0x68;
pub const ADDR_RESERVED__48              : u8 = 0x69;
pub const ADDR_RESERVED__49              : u8 = 0x6A;
pub const ADDR_RESERVED__50              : u8 = 0x6B;
pub const ADDR_RESERVED__51              : u8 = 0x6C;
pub const ADDR_RESERVED__52              : u8 = 0x6D;
pub const ADDR_RESERVED__53              : u8 = 0x6E;
pub const ADDR_RESERVED__54              : u8 = 0x6F;
pub const ADDR_RESERVED__55              : u8 = 0x70;
pub const ADDR_RESERVED__56              : u8 = 0x71;
pub const ADDR_RESERVED__57              : u8 = 0x72;
pub const ADDR_X_OFS_USR                 : u8 = 0x73;
pub const ADDR_Y_OFS_USR                 : u8 = 0x74;
pub const ADDR_Z_OFS_USR                 : u8 = 0x75;
pub const ADDR_RESERVED__58              : u8 = 0x76;
pub const ADDR_RESERVED__59              : u8 = 0x77;
pub const ADDR_FIFO_DATA_OUT_TAG         : u8 = 0x78;
pub const ADDR_FIFO_DATA_OUT_X_L         : u8 = 0x79;
pub const ADDR_FIFO_DATA_OUT_X_H         : u8 = 0x7A;
pub const ADDR_FIFO_DATA_OUT_Y_L         : u8 = 0x7B;
pub const ADDR_FIFO_DATA_OUT_Y_H         : u8 = 0x7C;
pub const ADDR_FIFO_DATA_OUT_Z_L         : u8 = 0x7D;
pub const ADDR_FIFO_DATA_OUT_Z_H         : u8 = 0x7E;
pub const ADDR_RESERVED__60              : u8 = 0x7F;

// ----- Register Values -----
pub const VALUE_WHO_AM_I                 : u8 = 0x6B;

pub const VALUE_ODR_00000                : u8 = 0x00;
pub const VALUE_ODR_00125                : u8 = 0x10;
pub const VALUE_ODR_00260                : u8 = 0x20;
pub const VALUE_ODR_00520                : u8 = 0x30;
pub const VALUE_ODR_01040                : u8 = 0x40;
pub const VALUE_ODR_02080                : u8 = 0x50;
pub const VALUE_ODR_04170                : u8 = 0x60;
pub const VALUE_ODR_08330                : u8 = 0x70;
pub const VALUE_ODR_16670                : u8 = 0x80;
pub const VALUE_ODR_33330                : u8 = 0x90;
pub const VALUE_ODR_66670                : u8 = 0xA0;

pub const VALUE_FS_00                    : u8 = 0x00;
pub const VALUE_FS_01                    : u8 = 0x04;
pub const VALUE_FS_10                    : u8 = 0x08;
pub const VALUE_FS_11                    : u8 = 0x0C;

pub const VALUE_HPCF_XL_0                : u8 = 0x00;
pub const VALUE_HPCF_XL_1                : u8 = 0x20;
pub const VALUE_HPCF_XL_2                : u8 = 0x40;
pub const VALUE_HPCF_XL_3                : u8 = 0x60;
pub const VALUE_HPCF_XL_4                : u8 = 0x80;
pub const VALUE_HPCF_XL_5                : u8 = 0xA0;
pub const VALUE_HPCF_XL_6                : u8 = 0xC0;
pub const VALUE_HPCF_XL_7                : u8 = 0xE0;

pub const VALUE_FTYPE_000                : u8 = 0x00;
pub const VALUE_FTYPE_001                : u8 = 0x01;
pub const VALUE_FTYPE_010                : u8 = 0x02;
pub const VALUE_FTYPE_011                : u8 = 0x03;
pub const VALUE_FTYPE_100                : u8 = 0x04;
pub const VALUE_FTYPE_101                : u8 = 0x05;
pub const VALUE_FTYPE_110                : u8 = 0x06;
pub const VALUE_FTYPE_111                : u8 = 0x07;

//----- Bit Masks -----
pub const BITMASK_SPI_RW                 : u8 = 0x80;

pub const BITMASK_LPF2_XL_EN             : u8 = 0x02;

pub const BITMASK_FS_125                 : u8 = 0x02;
pub const BITMASK_FS_4000                : u8 = 0x01;

pub const BITMASK_DEVICE_CONF            : u8 = 0x02;
pub const BITMASK_I2C_DISABLE            : u8 = 0x04;
pub const BITMASK_SW_RESET               : u8 = 0x01;

pub const BITMASK_LOW_PASS_ON_6D         : u8 = 0x01;
pub const BITMASK_HP_SLOPE_XL_EN         : u8 = 0x04;
pub const BITMASK_FASTSETTL_MODE_XL      : u8 = 0x08;
pub const BITMASK_HP_REF_MODE_XL         : u8 = 0x10;

pub const BITMASK_CTRL6_C_DISABLE_DEVICE : u8 = 0x10;
pub const BITMASK_STATUS_REG_XLDA        : u8 = 0x01;

pub const BITMASK_CTRL3_C_BOOT           : u8 = 0x80;


pub const BITMASK_CTRL2_FS_125           : u8 = 0x02;
pub const BITMASK_CTRL2_FS_4000          : u8 = 0x01;
pub const BITMASK_CTRL7_HP_EN_G          : u8 = 0x40;

pub const BITMASK_CTRL7_HPM_00           : u8 = 0x00;
pub const BITMASK_CTRL7_HPM_01           : u8 = 0x10;
pub const BITMASK_CTRL7_HPM_10           : u8 = 0x20;
pub const BITMASK_CTRL7_HPM_11           : u8 = 0x30;

pub const BITMASK_CTRL4_LPF1_SEL_G       : u8 = 0x02;
pub const BITMASK_CTRL4_SLEEP_G          : u8 = 0x40;

pub const BITMASK_STATUS_REG_GDA         : u8 = 0x02;

// ----- SPI Interface -----

pub fn spi_read_reg<S>(spi: &mut S, reg_addr: u8) -> Result<u8, S::Error>
where
    S: SpiBus<u8>,
    S::Error: core::fmt::Debug,
{
    let read_cmd = reg_addr | BITMASK_SPI_RW;
    let mut buf: [u8; 2] = [read_cmd, 0x00];

    spi.transfer_in_place(&mut buf)?;
    trace!("spi_read_reg: read 0x{:02X} from address 0x{:02X}", buf[1], reg_addr);
    Ok(buf[1])
}

pub fn spi_write_reg<S>(spi: &mut S, reg_addr: u8, data: u8) -> Result<(), S::Error>
where
    S: SpiBus<u8>,
    S::Error: core::fmt::Debug,
{
    let write_cmd = reg_addr & !BITMASK_SPI_RW;
    let buf: [u8; 2] = [write_cmd, data];

    spi.write(&buf)?;
    trace!("spi_write_reg: wrote 0x{:02X} to address 0x{:02X}", data, reg_addr);
    Ok(())
}
