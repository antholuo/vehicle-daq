/*
* Created date: 12/6/25
* File Description: Public top interface level of asm330 driver
*/

#![no_std]
mod reg;
mod device;

pub struct Driver<SpiBus> {
    dev: device::Device,
    spi: SpiBus
}

pub fn init_asm330<SpiBus>(spi: &mut SpiBus) -> Driver<SpiBus>{
    Driver {
        
    }
}

pub fn is_who_am_i_good<S>(spi: &mut S) -> Result<u8, S::Error>
where
    S: SpiBus<u8>,
    S::Error: core::fmt::Debug,
{
    info!("Attempting to read WHOAMI at reg: 0x{:02X}", REG_WHOAMI);
    let mut buf: [u8; 2] = [reg::ADDR_WHO_AM_I, 0x00];
    spi.transfer_in_place(&mut buf)?;
    Ok(buf[1] == reg::VALUE_WHO_AM_I);
}
