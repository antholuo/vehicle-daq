/*
 * imu.c
 *
 *  Created on: Feb 22, 2025
 *      Author: Anthony Luo
 */

#include "imu.h"
#include "gpio.h"
#include "spi.h"
#include "tim.h"

#include "asm330lhh_reg.h"

extern SPI_HandleTypeDef hspi1;		// ASM330 SPI
extern TIM_HandleTypeDef htim3;		// GPIO LED (1 & 2)
extern UART_HandleTypeDef huart1;	// UART1 (No cts/rts) for sensor data output stream.

void run_imu_basic() {
	TIM3->CCR1 = TIM3->ARR / 2;
	HAL_TIM_PWM_Start(&htim3, TIM_CHANNEL_1);

	// Protobuf setup
	uint8_t workspace[300];
	uint8_t encoded[300];
	struct raw_imu_data_t *raw_imu_pb_p;
	raw_imu_pb_p = raw_imu_data_new(&workspace[0], sizeof(workspace));


// IMU Setup
	stmdev_ctx_t dev_ctx;
	asm330lhh_reg_t reg;
	uint32_t timestamp;

	dev_ctx.write_reg = platform_write;
	dev_ctx.read_reg = platform_read;
	dev_ctx.mdelay = platform_delay;
	dev_ctx.handle = &hspi1;

	// Wait IMU boot time.
	HAL_Delay(100);

	// Check device ID
	asm330lhh_device_id_get(&dev_ctx, &asm330_wai);
	if (asm330_wai != ASM330LHH_ID) {
		TIM3->ARR = TIM3->ARR / 10;
		TIM3->CCR1 = TIM3->ARR / 2;
//		TIM3->CCR2 = 0;
		while (1)
			;
	}

	// Restore Default Configuration
	asm330lhh_reset_set(&dev_ctx, PROPERTY_ENABLE);
	do {
		asm330lhh_reset_get(&dev_ctx, &rst);
	} while (rst);

	// Configure IMU
	asm330lhh_device_conf_set(&dev_ctx, PROPERTY_ENABLE);// Set device configuration (no clue what this does)
	asm330lhh_block_data_update_set(&dev_ctx, PROPERTY_ENABLE);	// Enable block data update
	asm330lhh_xl_data_rate_set(&dev_ctx, ASM330LHH_XL_ODR_417Hz);// TODO: determine the correct output rates
	asm330lhh_gy_data_rate_set(&dev_ctx, ASM330LHH_GY_ODR_417Hz);
	asm330lhh_xl_full_scale_set(&dev_ctx, ASM330LHH_2g);// TODO: determine the correct scaling
	asm330lhh_gy_full_scale_set(&dev_ctx, ASM330LHH_2000dps);
	asm330lhh_timestamp_set(&dev_ctx, PROPERTY_ENABLE);	// Enable timestamping

	/* Configure filtering chain(No aux interface)
	 * Accelerometer - LPF1 + LPF2 path
	 */
	asm330lhh_xl_hp_path_on_out_set(&dev_ctx, ASM330LHH_LP_ODR_DIV_100);
	asm330lhh_xl_filter_lp2_set(&dev_ctx, PROPERTY_ENABLE);

	imuRawData_S raw_data;
	while (1) {
		HAL_GPIO_WritePin(GPIO_LED2_GPIO_Port, GPIO_LED2_Pin, GPIO_PIN_RESET);	// Turn LED2 on (indicating that we are calculating something)
		asm330lhh_status_reg_get(&dev_ctx, &raw_data.status_reg);

		if (raw_data.status_reg.xlda || raw_data.status_reg.gda) {
			asm330lhh_timestamp_raw_get(&dev_ctx, &timestamp);
			raw_data.timestamp = timestamp;
		}

		if (raw_data.status_reg.xlda) {
			asm330lhh_acceleration_raw_get(&dev_ctx,
					data_raw_acceleration);
#if DO_FP
			acceleration_mg[0] = asm330lhh_from_fs2g_to_mg(
					raw_data.acceleration.i16bit[0]);
			acceleration_mg[1] = asm330lhh_from_fs2g_to_mg(
					raw_data.acceleration.i16bit[1]);
			acceleration_mg[2] = asm330lhh_from_fs2g_to_mg(
					raw_data.acceleration.i16bit[2]);

			acceleration_g[0] = acceleration_mg[0] / 1000;
			acceleration_g[1] = acceleration_mg[1] / 1000;
			acceleration_g[2] = acceleration_mg[2] / 1000;
#endif
		}

		if (raw_data.status_reg.gda) {
			asm330lhh_angular_rate_raw_get(&dev_ctx,
					data_raw_angular_rate);
#if DO_FP
			angular_rate_mdps[0] = asm330lhh_from_fs2000dps_to_mdps(
					raw_data.angular_rate.i16bit[0]);
			angular_rate_mdps[1] = asm330lhh_from_fs2000dps_to_mdps(
					raw_data.angular_rate.i16bit[1]);
			angular_rate_mdps[2] = asm330lhh_from_fs2000dps_to_mdps(
					raw_data.angular_rate.i16bit[2]);

			angular_rate_dps[0] = angular_rate_mdps[0] / 1000;
			angular_rate_dps[1] = angular_rate_mdps[1] / 1000;
			angular_rate_dps[2] = angular_rate_mdps[2] / 1000;
#endif
		}

		// Protobuf encode data
		map_data_to_pb(raw_imu_pb_p, data_raw_acceleration, data_raw_angular_rate, &raw_data.status_reg, timestamp);
		size_t pb_size = raw_imu_data_encode(raw_imu_pb_p, &encoded[0], sizeof(encoded));

		// Transmit encoded data over uart
		HAL_UART_Transmit(&huart1, &encoded, pb_size, 100);

		HAL_GPIO_WritePin(GPIO_LED2_GPIO_Port, GPIO_LED2_Pin, GPIO_PIN_SET);	// Turn LED2 off (we are done!)

		HAL_Delay(1000);
	}
}

void map_data_to_pb(struct raw_imu_data_t *raw_pb, int16_t *data_raw_acceleration, int16_t *data_raw_angular_rate, asm330lhh_status_reg_t *sr, uint32_t ts) {
	raw_pb->timestamp = ts;

	raw_pb->accel_x = data_raw_acceleration[0];
	raw_pb->accel_y = data_raw_acceleration[1];
	raw_pb->accel_z = data_raw_acceleration[2];

	raw_pb->gyro_x = data_raw_angular_rate[0];
	raw_pb->gyro_y = data_raw_angular_rate[1];
	raw_pb->gyro_z = data_raw_angular_rate[2];

	raw_pb->gda = sr->gda;
	raw_pb->xlda = sr->xlda;
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

static void platform_delay(uint32_t ms) {
	HAL_Delay(ms);
}
