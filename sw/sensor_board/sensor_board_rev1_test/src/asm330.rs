use embedded_hal::spi::SpiBus;
use log::info;

pub fn check_who_am_i<S>(spi: &mut S) -> Result<u8, S::Error>
where
    S: SpiBus<u8>,
    S::Error: core::fmt::Debug,
{
    let mut buf: [u8; 2] = [0x8F, 0x00];
    spi.transfer_in_place(&mut buf)?;
    Ok(buf[1])
}
