/*
* Created date: 12/7/25
* File Description: asm330 device state
*/

#![no_std]

pub struct Device {
    spi_full_duplex: bool,

}

pub init_device() -> Device{
    Device {
        
    }
}

pub set_spi_half_duplex(dev: &mut Device) {

}