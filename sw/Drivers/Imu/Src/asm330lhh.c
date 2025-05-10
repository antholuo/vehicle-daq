/**
 * asm330lhh.c
 *
 * Created on: 2025-MAY-10
 *     Author: Anni
 *   Modified: 2025-MAY-10
 *
 * Functions for using ASM330LHH IMU
 */

#include "imu.h"
#include "asm330lhh.h"
#include "asm330lhh_reg.h"

#include "gpio.h"
#include "spi.h"

#include <stdint.h>
#include <stdbool.h>

////////
// Private prototypes
int32_t platform_write(void *handle, uint8_t reg, const uint8_t *bufp,
                              uint16_t len);
int32_t platform_read(void *handle, uint8_t reg, uint8_t *bufp,
                             uint16_t len);
//static void tx_com( uint8_t *tx_buffer, uint16_t len );
void platform_delay(uint32_t ms);

//////// // Eventually we need these to be configurable...
extern SPI_HandleTypeDef hspi1;     // ASM330LHH SPI

////////
// Private variables
static stmdev_ctx_t asm330lhh_dev_ctx;
static asm330lhh_reg_t asm330lhh_reg;
static uint_fast32_t timestamp;
static uint8_t asm330lhh_whoami, rst;

ImuStatus_E setup_imu_asm330lhh() {
    ImuStatus_E retVal = IMU_STATUS_OK;
    asm330lhh_dev_ctx.write_reg = platform_write;
    asm330lhh_dev_ctx.read_reg = platform_read;
    asm330lhh_dev_ctx.mdelay = platform_delay;
    asm330lhh_dev_ctx.handle = &hspi1; // TODO: eventualy derive this from an "IMU CONFIGURATION"

    // Wait IMU boot time and check ID
    HAL_Delay(100);
    asm330lhh_device_id_get(&asm330lhh_dev_ctx, &asm330lhh_whoami);
    if (asm330lhh_whoami != ASM330LHH_ID) {
        retVal = IMU_STATUS_ERR;
    }

    // Restore Default Configurations
    asm330lhh_reset_set(&asm330lhh_dev_ctx, PROPERTY_ENABLE);
    do {
        asm330lhh_reset_get(&asm330lhh_dev_ctx, &rst);
    } while (rst);

    // TODO: verify these values are good / usable
    // Configure IMU
    asm330lhh_device_conf_set(&asm330lhh_dev_ctx, PROPERTY_ENABLE);           // Set device configuration (no clue what this does)
    asm330lhh_block_data_update_set(&asm330lhh_dev_ctx, PROPERTY_ENABLE);     // Enable block data update
    asm330lhh_xl_data_rate_set(&asm330lhh_dev_ctx, ASM330LHH_XL_ODR_417Hz);   // TODO: determine the correct output rates
    asm330lhh_gy_data_rate_set(&asm330lhh_dev_ctx, ASM330LHH_GY_ODR_417Hz);
    asm330lhh_xl_full_scale_set(&asm330lhh_dev_ctx, ASM330LHH_2g);            // TODO: determine the correct scaling
    asm330lhh_gy_full_scale_set(&asm330lhh_dev_ctx, ASM330LHH_2000dps);
    asm330lhh_timestamp_set(&asm330lhh_dev_ctx, PROPERTY_ENABLE);             // Enable timestamping

    // Configure filtering chain(No aux interface)
    // Accelerometer - LPF1 + LPF2 path
    asm330lhh_xl_hp_path_on_out_set(&asm330lhh_dev_ctx, ASM330LHH_LP_ODR_DIV_100);

    return retVal;
}

////////
// platform functions for IMU setup
int32_t platform_write(void *handle, uint8_t reg, const uint8_t *bufp, uint16_t len) {
    HAL_GPIO_WritePin(SPI_CS_GPIO_Port, SPI_CS_Pin, GPIO_PIN_RESET);
    HAL_SPI_Transmit(handle, &reg, 1, 1000);
    HAL_SPI_Transmit(handle, (uint8_t*) bufp, len, 1000);
    HAL_GPIO_WritePin(SPI_CS_GPIO_Port, SPI_CS_Pin, GPIO_PIN_SET);
    return 0;
}

int32_t platform_read(void *handle, uint8_t reg, uint8_t *bufp, uint16_t len) {
    reg |= 0x80;

    // This SPI pin is currently hardcoded, eventually we want to add this to some sort of configuration file
    HAL_GPIO_WritePin(SPI_CS_GPIO_Port, SPI_CS_Pin, GPIO_PIN_RESET);
    HAL_SPI_Transmit(handle, &reg, 1, 1000);
    HAL_SPI_Receive(handle, bufp, len, 1000);
    HAL_GPIO_WritePin(SPI_CS_GPIO_Port, SPI_CS_Pin, GPIO_PIN_SET);
    return 0;
}

void platform_delay(uint32_t ms) {
    HAL_Delay(ms);
}
