/*
 * parser.c
 *
 *  Created on: Feb 22, 2025
 *      Author: antho
 */

#include "imu.h"
#include "parser.h"
#include "usart.h"

extern UART_HandleTypeDef huart1; // where we receive all the raw data
extern UART_HandleTypeDef huart2; // where we send the pretty data

static float_t acceleration_mg[3];
static float_t acceleration_g[3];
static float_t angular_rate_mdps[3];
static float_t angular_rate_dps[3];

imuRawData_S raw_data;

void start_parsing_data() {

	HAL_UART_Receive_IT(&huart1, &raw_data, sizeof(raw_data));
	while (1) {
//		HAL_UART_Receive(&huart1, &raw_data, sizeof(raw_data), 100);
//
//		acceleration_mg[0] = asm330lhh_from_fs2g_to_mg(
//				raw_data.acceleration.i16bit[0]);
//		acceleration_mg[1] = asm330lhh_from_fs2g_to_mg(
//				raw_data.acceleration.i16bit[1]);
//		acceleration_mg[2] = asm330lhh_from_fs2g_to_mg(
//				raw_data.acceleration.i16bit[2]);
//
//		acceleration_g[0] = acceleration_mg[0] / 1000;
//		acceleration_g[1] = acceleration_mg[1] / 1000;
//		acceleration_g[2] = acceleration_mg[2] / 1000;
//		angular_rate_mdps[0] = asm330lhh_from_fs2000dps_to_mdps(
//				raw_data.angular_rate.i16bit[0]);
//		angular_rate_mdps[1] = asm330lhh_from_fs2000dps_to_mdps(
//				raw_data.angular_rate.i16bit[1]);
//		angular_rate_mdps[2] = asm330lhh_from_fs2000dps_to_mdps(
//				raw_data.angular_rate.i16bit[2]);
//
//		angular_rate_dps[0] = angular_rate_mdps[0] / 1000;
//		angular_rate_dps[1] = angular_rate_mdps[1] / 1000;
//		angular_rate_dps[2] = angular_rate_mdps[2] / 1000;

//		HAL_GPIO_TogglePin(GPIO_LED2_GPIO_Port, GPIO_LED2_Pin);
		HAL_Delay(10);
	}
}

void HAL_UART_RxCpltCallback(UART_HandleTypeDef *huart)
{
//	acceleration_mg[0] = asm330lhh_from_fs2g_to_mg(
//			raw_data.acceleration.i16bit[0]);
//	acceleration_mg[1] = asm330lhh_from_fs2g_to_mg(
//			raw_data.acceleration.i16bit[1]);
//	acceleration_mg[2] = asm330lhh_from_fs2g_to_mg(
//			raw_data.acceleration.i16bit[2]);
//
//	acceleration_g[0] = acceleration_mg[0] / 1000;
//	acceleration_g[1] = acceleration_mg[1] / 1000;
//	acceleration_g[2] = acceleration_mg[2] / 1000;
//	angular_rate_mdps[0] = asm330lhh_from_fs2000dps_to_mdps(
//			raw_data.angular_rate.i16bit[0]);
//	angular_rate_mdps[1] = asm330lhh_from_fs2000dps_to_mdps(
//			raw_data.angular_rate.i16bit[1]);
//	angular_rate_mdps[2] = asm330lhh_from_fs2000dps_to_mdps(
//			raw_data.angular_rate.i16bit[2]);
//
//	angular_rate_dps[0] = angular_rate_mdps[0] / 1000;
//	angular_rate_dps[1] = angular_rate_mdps[1] / 1000;
//	angular_rate_dps[2] = angular_rate_mdps[2] / 1000;

	HAL_GPIO_TogglePin(GPIO_LED2_GPIO_Port, GPIO_LED2_Pin);

	HAL_UART_Receive_IT(&huart1, &raw_data, sizeof(raw_data));
}


