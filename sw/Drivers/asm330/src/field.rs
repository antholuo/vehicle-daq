// use embedded_hal::spi::SpiBus;

// use crate::common::*;
// use crate::reg::*;

// pub struct Field {
//     address: u8,
//     offset: u8,
//     width: u8,
//     readonly: bool,
//     default: u16,
//     leak_below: bool,
//     leak_to_lower: bool,
//     value: u16,
// }

// fn safe_write<S: SpiBus<u8>>(spi: &mut S, addr: u8, value: u8, mask: u8) -> Result<(), Asm330Error> {
//     let mut reg_value = spi_read_reg(spi, addr)?;
//     reg_value &= mask;
//     reg_value |= value;
//     spi_write_reg(spi, addr, reg_value)?;
//     Ok(())
// }

// impl Field {
//     pub fn new(address: u8, offset: u8, width: u8, readonly: bool, default: u16, leak_below: bool, leak_to_lower: bool) -> Self {
//         Self {
//             address,
//             offset,
//             width,
//             readonly,
//             default,
//             leak_below,
//             leak_to_lower,
//             value: default,
//         }
//     }

//     pub fn new_rw(address: u8, offset: u8, width: u8) -> Self {
//         Self::new(
//             address,
//             offset,
//             width,
//             false,
//             0,
//             false,
//             false,
//         )
//     }

//     pub fn new_ro(address: u8, offset: u8, width: u8) -> Self {
//         Self::new(
//             address,
//             offset,
//             width,
//             true,
//             0,
//             false,
//             false,
//         )
//     }

//     pub fn write<S: SpiBus<u8>>(&self, spi: &mut S) -> Result<(), Asm330Error> {
//         if self.width > 16 {
//             panic!("not implemented");
//         } else if self.width > 8 {
//             let mut upper: u8 = (self.value >> 8) as u8;
//             let lower: u8 = (self.value & 0x00ff) as u8;
//             spi_write_reg(spi, self.address, lower)?;
//             let mut mask = 0xff >> (16-self.width);
//             if !self.leak_to_lower {
//                 upper <<= 16 - self.width;
//                 mask <<= 16 - self.width;
//             }
//             if self.leak_below {
//                 safe_write(spi, self.address - 1, upper, mask)?;
//             } else {
//                 safe_write(spi, self.address + 1, upper, mask)?;
//             }
//             return Ok(());
//         } else {
//             let mask = (0xff >> (8-self.width)) << self.offset;
//             safe_write(spi, self.address, (self.value as u8) << self.offset, mask)?;
//             return Ok(());
//         }
//     }

//     pub fn read<S: SpiBus<u8>>(&self, spi: &mut S) -> Result<u16, Asm330Error> {
//         if self.width > 16 {
//             panic!("not implemented");
//         } else if self.width > 8 {
//             let lower = spi_read_reg(spi, self.address)?;
//             let mut upper = 0;
//             if self.leak_below {
//                 upper = spi_read_reg(spi, self.address - 1)?;
//             } else {
//                 upper = spi_read_reg(spi, self.address + 1)?;
//             }
//             if !self.leak_to_lower {
//                 upper >>= 16 - self.width;
//             }
//             let big_upper = (upper as u16) << 8;
//             let big_lower = lower as u16;
//             let bitmask = 0xffff >> 16 - self.width;
//             return Ok((big_upper | big_lower) & bitmask);
//         } else {
//             let value = spi_read_reg(spi, self.address)?;
//             return Ok(value as u16);
//         }
//     }

//     pub fn read_from_unshifted(&self, value: u8) -> u8 {
//         if self.width > 8 {
//             panic!("not implemented");
//         } else {
//             let mask = (0xff >> (8-self.width)) << self.offset;
//             let masked = value & mask;
//             return masked;
//         }
//     }

//     pub fn read_from(&self, value: u8) -> u8 {
//         self.read_from_unshifted(value) >> self.offset
//     }
// }