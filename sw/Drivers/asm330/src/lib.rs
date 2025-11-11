#![no_std]
use core::time::Duration;

use embedded_hal::{delay, spi::SpiBus};
use log::{debug, info, trace, warn};

mod reg_ctrl;
mod registers;
mod types;

pub use types::*;

use reg_ctrl::*;
use registers::*;

pub fn read_who_am_i<S>(spi: &mut S) -> Result<u8, S::Error>
where
    S: SpiBus<u8>,
    S::Error: core::fmt::Debug,
{
    debug!("Attempting to read WHOAMI");
    let result = read_register(spi, WHO_AM_I)?;
    info!("Read WHOAMI as {}, 0x{:X}, 0b{:b}", result, result, result);
    Ok(result)
}

// TODO: relocate this into a separate file
pub fn enable_xl_gy_outputs<S>(spi: &mut S, cfg: &OutputConfig) -> Result<(), S::Error>
where
    S: SpiBus<u8>,
    S::Error: core::fmt::Debug,
{
    write_register(
        spi,
        CTRL1_XL,
        make_ctrl1_xl_reg(cfg.xl_odr, cfg.xl_fsr, cfg.xl_lpf2_en),
    )?;

    write_register(spi, CTRL2_G, make_ctrl2_g_reg(cfg.gy_odr, cfg.gy_fsr))?;

    // let current_ctrl6_c = read_register(spi, CTRL6_C)?;
    // let new_ctrl6_c = (current_ctrl6_c & !CTRL6_C_GY_LPF1_MASK) | 0b010; // pick LPF1 setting
    // write_register(spi, CTRL6_C, new_ctrl6_c)?;

    Ok(())
}

pub fn poll_data<S>(spi: &mut S) -> Result<RawImuData, S::Error>
where
    S: SpiBus<u8>,
    S::Error: core::fmt::Debug,
{
    // Read status reg
    let status = read_register(spi, STATUS_REG)?;
    let xl_data_ready: bool = (status & 0b0000_0001) != 0; // XLDA
    let gy_data_ready: bool = (status & 0b0000_0010) != 0; // GDA
    let temp_data_ready: bool = (status & 0b0000_0100) != 0; // TDA
    debug!(
        "XLDA: {}, GDA: {}, TDA: {}",
        xl_data_ready, gy_data_ready, temp_data_ready
    );

    // Get timestamp
    const TIMESTAMP_READ_CMD: u8 = 0x80 | TIMESTAMP0;
    let mut ts_buf: [u8; 4] = [TIMESTAMP_READ_CMD, 0, 0, 0];
    spi.transfer_in_place(&mut ts_buf)?;
    let timestamp = u32::from_le_bytes([ts_buf[0], ts_buf[1], ts_buf[2], ts_buf[3]]);

    // Read TEMP/XL/GY in oneshot (all 14 regs)
    const SENS_READ_CMD: u8 = 0x80 | OUT_TEMP_L;
    let mut sens_buf: [u8; 14] = [0; 14];
    sens_buf[0] = SENS_READ_CMD;
    spi.transfer_in_place(&mut sens_buf)?;

    let temp_raw = u16::from_le_bytes([sens_buf[0], sens_buf[1]]);
    let gy_x = i16::from_le_bytes([sens_buf[2], sens_buf[3]]);
    let gy_y = i16::from_le_bytes([sens_buf[4], sens_buf[5]]);
    let gy_z = i16::from_le_bytes([sens_buf[6], sens_buf[7]]);
    let xl_x = i16::from_le_bytes([sens_buf[8], sens_buf[9]]);
    let xl_y = i16::from_le_bytes([sens_buf[10], sens_buf[11]]);
    let xl_z = i16::from_le_bytes([sens_buf[12], sens_buf[13]]);

    Ok(RawImuData {
        xl: RawSensorData {
            x: xl_x,
            y: xl_y,
            z: xl_z,
            valid: xl_data_ready,
        },
        gy: RawSensorData {
            x: gy_x,
            y: gy_y,
            z: gy_z,
            valid: gy_data_ready,
        },
        ts: Some(timestamp), // TODO: handle case where timestamping isn't enabled
        temp: if temp_data_ready {
            Some(temp_raw)
        } else {
            None
        },
    })
}

pub fn fs_a_to_g(raw: i16, fsr: &AccelFs) -> f32 {
    let sensitivity_mg = match fsr {
        AccelFs::G2 => 0.061,
        AccelFs::G4 => 0.122,
        AccelFs::G8 => 0.244,
        AccelFs::G16 => 0.488,
    };

    ((raw as f32) * sensitivity_mg) / 1000.0
}

pub fn reset_asm330<S>(spi: &mut S) -> Result<(), S::Error>
where
    S: SpiBus<u8>,
    S::Error: core::fmt::Debug,
{
    use log::{debug, info, warn};

    // Step 1: Issue software reset
    const SW_RESET_BIT: u8 = 0b0000_0001;
    write_register(spi, CTRL3_C, SW_RESET_BIT)?;
    info!("Issued software reset to ASM330...");

    // Step 2: Wait for reset to complete
    // The SW_RESET bit self-clears when done.
    let mut attempts = 0u8;
    loop {
        let reg = read_register(spi, CTRL3_C)?;
        if reg & SW_RESET_BIT == 0 {
            break;
        }
        attempts += 1;
        if attempts > 100 {
            warn!("ASM330 reset timed out!");
            break;
        }
    }

    info!("Reset complete after {} checks.", attempts);

    // Step 3 (optional): Reload calibration data
    const BOOT_BIT: u8 = 0b1000_0000;
    write_register(spi, CTRL3_C, BOOT_BIT)?;
    info!("Reloading trimming data from NVM...");

    // Step 4: Wait for boot to clear
    loop {
        let reg = read_register(spi, CTRL3_C)?;
        if reg & BOOT_BIT == 0 {
            break;
        }
    }

    info!("Boot sequence complete.");

    Ok(())
}
