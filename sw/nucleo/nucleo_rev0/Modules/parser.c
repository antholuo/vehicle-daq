/*
 * parser.c
 *
 *  Created on: Feb 22, 2025
 *      Author: antho
 */

#include "imu.h"
#include "parser.h"
#include "imu_pb.h"
#include "pbtools.h"

#include "usart.h"
#include <stdbool.h>

extern UART_HandleTypeDef huart1; // where we receive all the raw data
extern UART_HandleTypeDef huart2; // where we send the pretty data (our computer)

static float_t acceleration_mg[3];
static float_t acceleration_g[3];
static float_t angular_rate_mdps[3];
static float_t angular_rate_dps[3];

uint8_t console_buf[100];

uint8_t encoded[100];	// Encoded data
uint8_t workspace[300]; // Workspace for PB tools;
bool new_data_available;
uint8_t indx = 0;

static void tx_com(uint8_t *tx_buffer, uint16_t len);

void start_parsing_data() {
	new_data_available = false;

	struct raw_imu_data_t *raw_data_p;

	raw_data_p = raw_imu_data_new(&workspace[0], sizeof(workspace));

	HAL_UARTEx_ReceiveToIdle_IT(&huart1, encoded, sizeof(encoded));

	while (1) {
		if (new_data_available) {
			size_t data_size = raw_imu_data_decode(raw_data_p, encoded, indx);

			snprintf((char*) console_buf, sizeof(console_buf),
					"Received new data with size: %d and count %d\r\n",
					data_size, raw_data_p->timestamp);

			tx_com(console_buf, strlen((char const *) console_buf));
			new_data_available = false;
		}
		HAL_Delay(10);
	}
}

void HAL_UARTEx_RxEventCallback(UART_HandleTypeDef *huart, uint16_t Size) {
	indx = Size;
	new_data_available = true;
	HAL_GPIO_TogglePin(GPIO_LED2_GPIO_Port, GPIO_LED2_Pin);
	HAL_UARTEx_ReceiveToIdle_IT(&huart1, encoded, sizeof(encoded));
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
	new_data_available = true;

	HAL_UART_Receive_IT(&huart1, encoded, sizeof(encoded));
}


static void tx_com(uint8_t *tx_buffer, uint16_t len)
{
  HAL_UART_Transmit(&huart2, tx_buffer, len, 1000);
}
