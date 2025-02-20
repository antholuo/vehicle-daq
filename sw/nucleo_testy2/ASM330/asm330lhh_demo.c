/*
 * asm330lhh_demo.c
 *
 *  Created on: Feb 12, 2025
 *      Author: antho
 */
#include <string.h>
#include <stdint.h>
#include <stdio.h>

#include "asm330lhh_demo.h"
#include "asm330lhh_reg.h"

#include "gpio.h"
#include "usart.h"
#include "spi.h"

extern SPI_HandleTypeDef hspi1;
extern UART_HandleTypeDef huart2;

static int16_t data_raw_acceleration[3];
static int16_t data_raw_angular_rate[3];
static int16_t data_raw_temperature;
static uint8_t tx_buffer[200];
static float_t temperature_degC;
static uint8_t whoamI, rst;

static int32_t platform_write(void *handle, uint8_t reg, const uint8_t *bufp,
		uint16_t len);
static int32_t platform_read(void *handle, uint8_t reg, uint8_t *bufp,
		uint16_t len);
static void tx_com(uint8_t *tx_buffer, uint16_t len);
static void platform_delay(uint32_t ms);

void start_demo() {

	// Initialize the IMU driver interface
	stmdev_ctx_t dev_ctx;
	dev_ctx.write_reg = platform_write;
	dev_ctx.read_reg = platform_read;
	dev_ctx.mdelay = platform_delay;
	dev_ctx.handle = &hspi1;

	/* Restore default configuration. */
	asm330lhh_reset_set(&dev_ctx, PROPERTY_ENABLE);
	asm330lhh_fifo_ctrl3_t fifo_ctrl3_c;

	/* Check device ID */
	do {
		asm330lhh_device_id_get(&dev_ctx, &whoamI);
		snprintf((char*) tx_buffer, sizeof(tx_buffer),
				"whoami is %d\r\n", whoamI);
		tx_com(tx_buffer, strlen((char const*) tx_buffer));

		HAL_Delay(10);

		uint32_t ret = asm330lhh_read_reg(&dev_ctx, ASM330LHH_FIFO_CTRL3,
					&fifo_ctrl3_c, 1);

		HAL_Delay(10);

		asm330lhh_xl_data_rate_set(&dev_ctx, ASM330LHH_XL_ODR_12Hz5);
		HAL_Delay(100);
	} while (whoamI != ASM330LHH_ID);


	do {
		asm330lhh_reset_get(&dev_ctx, &rst);
	} while (rst);

	/* Start device configuration. */
	asm330lhh_device_conf_set(&dev_ctx, PROPERTY_ENABLE);
	/* Enable Block Data Update. */
	asm330lhh_block_data_update_set(&dev_ctx, PROPERTY_ENABLE);
	/* Set Output Data Rate. */
	asm330lhh_xl_data_rate_set(&dev_ctx, ASM330LHH_XL_ODR_12Hz5);
	asm330lhh_gy_data_rate_set(&dev_ctx, ASM330LHH_GY_ODR_12Hz5);
	/* Set full scale. */
	asm330lhh_xl_full_scale_set(&dev_ctx, ASM330LHH_2g);
	asm330lhh_gy_full_scale_set(&dev_ctx, ASM330LHH_2000dps);
	/* Enable timestamp. */
	asm330lhh_timestamp_set(&dev_ctx, PROPERTY_ENABLE);
	/* Configure filtering chain(No aux interface)
	 * Accelerometer - LPF1 + LPF2 path
	 */
	asm330lhh_xl_hp_path_on_out_set(&dev_ctx, ASM330LHH_LP_ODR_DIV_100);
	asm330lhh_xl_filter_lp2_set(&dev_ctx, PROPERTY_ENABLE);

	/////////////////
	// Testing bits
	/////////////////
	// Read FIFO control registers
	uint32_t ret = asm330lhh_read_reg(&dev_ctx, ASM330LHH_FIFO_CTRL3,
			&fifo_ctrl3_c, 1);

	// Read CTRL6 (DEN)
	asm330lhh_ctrl6_c_t ctrl6_c;
	ret = asm330lhh_read_reg(&dev_ctx, ASM330LHH_CTRL6_C, &ctrl6_c, 1);

	asm330lhh_int1_ctrl_t int1_ctrl_c;
	ret = asm330lhh_read_reg(&dev_ctx, ASM330LHH_INT1_CTRL, &int1_ctrl_c, 1);

	// Initialize gyroscope to 416Hz (High Performance mode) by writing CTRL2_G = 60h
	// 0x60 == 0110 0000
	// ODR 0110 (417Hz)
	// FS 00 (250dps)
//	asm330lhh_gy_data_rate_set(&dev_ctx, ASM330LHH_GY_ODR_417Hz);
//	asm330lhh_gy_full_scale_set(&dev_ctx, ASM330LHH_250dps);

	// Enable block data update
	asm330lhh_block_data_update_set(&dev_ctx, PROPERTY_ENABLE);

	while (1) {
		HAL_GPIO_TogglePin(GPIO_LED_GPIO_Port, GPIO_LED_Pin);

		asm330lhh_reg_t reg;
		uint32_t timestamp;
		/* Read output only if new value is available. */
		asm330lhh_status_reg_get(&dev_ctx, &reg.status_reg);

		if (reg.status_reg.xlda || reg.status_reg.gda || reg.status_reg.tda) {
			asm330lhh_timestamp_raw_get(&dev_ctx, &timestamp);
		}

		if (reg.status_reg.xlda) {
			/* Read acceleration field data */
			memset(data_raw_acceleration, 0x00, 3 * sizeof(int16_t));
			asm330lhh_acceleration_raw_get(&dev_ctx, data_raw_acceleration);
			snprintf((char*) tx_buffer, sizeof(tx_buffer),
					"Acceleration [mg]:%d\t%d\t%d %lu\r\n",
					data_raw_acceleration[0], data_raw_acceleration[1],
					data_raw_acceleration[2], timestamp);
			tx_com(tx_buffer, strlen((char const*) tx_buffer));
		}

		if (reg.status_reg.gda) {
			/* Read angular rate field data */
			memset(data_raw_angular_rate, 0x00, 3 * sizeof(int16_t));
			asm330lhh_angular_rate_raw_get(&dev_ctx, data_raw_angular_rate);
			snprintf((char*) tx_buffer, sizeof(tx_buffer),
					"Angular rate [mdps]:%d\t%d\t%d %lu\r\n",
					data_raw_angular_rate[0], data_raw_angular_rate[1],
					data_raw_angular_rate[2], timestamp);
			tx_com(tx_buffer, strlen((char const*) tx_buffer));
		}

//		if (reg.status_reg.tda) {
//			/* Read temperature data */
//			memset(&data_raw_temperature, 0x00, sizeof(int16_t));
//			asm330lhh_temperature_raw_get(&dev_ctx, &data_raw_temperature);
//			temperature_degC = asm330lhh_from_lsb_to_celsius(
//					data_raw_temperature);
//			snprintf((char*) tx_buffer, sizeof(tx_buffer),
//					"Temperature [degC]:%d %lu\r\n", temperature_degC,
//					timestamp);
//			tx_com(tx_buffer, strlen((char const*) tx_buffer));
//		}

//		transmit_hello_world();
		HAL_Delay(10);
	}
}

static int32_t platform_write(void *handle, uint8_t reg, const uint8_t *bufp,
		uint16_t len) {
	HAL_GPIO_WritePin(SPI_CS_GPIO_Port, SPI_CS_Pin, GPIO_PIN_RESET);
	HAL_SPI_Transmit(handle, &reg, 1, 1000);
	HAL_SPI_Transmit(handle, (uint8_t*) bufp, len, 1000);
	HAL_GPIO_WritePin(SPI_CS_GPIO_Port, SPI_CS_Pin, GPIO_PIN_SET);
	return 0;
}

static int32_t platform_read(void *handle, uint8_t reg, uint8_t *bufp,
		uint16_t len) {
	reg |= 0x80;
	HAL_GPIO_WritePin(SPI_CS_GPIO_Port, SPI_CS_Pin, GPIO_PIN_RESET);
	HAL_SPI_Transmit(handle, &reg, 1, 1000);
	HAL_SPI_Receive(handle, bufp, len, 1000);
	HAL_GPIO_WritePin(SPI_CS_GPIO_Port, SPI_CS_Pin, GPIO_PIN_SET);
	return 0;
}

static void tx_com(uint8_t *tx_buffer, uint16_t len) {
	HAL_UART_Transmit(&huart2, tx_buffer, len, 1000);
}

static void platform_delay(uint32_t ms) {
	HAL_Delay(ms);
}

void transmit_hello_world() {
	uint8_t data[] = "Hello World\n";
	HAL_UART_Transmit(&huart2, data, 12, 10);
}
