#![allow(non_snake_case)]

use embedded_interfaces::codegen::interface_objects;

//
// Common enums used by bitfields
//

#[derive(Clone, Copy)]
pub enum Odr {
    PowerDown = 0b0000,
    Hz12_5    = 0b0001,
    Hz26      = 0b0010,
    Hz52      = 0b0011,
    Hz104     = 0b0100,
    Hz208     = 0b0101,
    Hz416     = 0b0110,
    Hz833     = 0b0111,
    Hz1660    = 0b1000,
    Hz3330    = 0b1001,
    Hz6660    = 0b1010,
}

#[derive(Clone, Copy)]
pub enum AccelScale {
    G2  = 0b00,
    G4  = 0b01,
    G8  = 0b10,
    G16 = 0b11,
}

#[derive(Clone, Copy)]
pub enum GyroScale {
    Dps125  = 0b0010,
    Dps250  = 0b0000,
    Dps500  = 0b0100,
    Dps1000 = 0b1000,
    Dps2000 = 0b1100,
    Dps4000 = 0b0001, // per datasheet CTRL2_G encoding
}

type UnsupportedI2CCodec = embedded_interfaces::registers::i2c::codecs::unsupported_codec::UnsupportedCodec<()>;
use embedded_interfaces::registers::spi::codecs::standard_codec::StandardCodec;
type Asm330SpiCodec = StandardCodec<1, 6, 0, 7, true, 0>;

interface_objects! {
    register_defaults {
        codec_error = (),
        i2c_codec = UnsupportedI2CCodec,
        spi_codec = Asm330SpiCodec,
    }

    register CTRL1_XL(addr = 0x10, mode = rw, size = 1) {
        bw0_xl: bool,

        lpf2_xl_en: bool,

        fs_xl: u8{2},

        odr_xl: u8{4},
    }

}
