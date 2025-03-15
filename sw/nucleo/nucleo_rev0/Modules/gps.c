/*
 * gps.c
 *
 *  Created on: Mar 13, 2025
 *      Author: antho
 */


/*
 * gnss_demo.c
 *
 *  Created on: Mar 1, 2025
 *      Author: antho
 */
#include <nmea.h>
#include <stdio.h>
#include <string.h>
#include <stdbool.h>

#include "usart.h"

#include "nmea.h"
#include "gps.h"

extern UART_HandleTypeDef huartt1; // Where we are receiving the raw GNSS data
extern UART_HandleTypeDef huart2; // where we send the pretty data (our computer)

extern RmcData_T data_RMC;

uint8_t console_buf2[100];
uint8_t nmea_raw[2000];	// Encoded NMEA data. Usually around ~500 bytes long.
uint16_t nmea_idx = 0;
int8_t new_data_ready = -5;

static void tx_com(uint8_t *tx_buffer, uint16_t len);

void run_gnss_demo() {

	HAL_UARTEx_ReceiveToIdle_DMA(&huart1, nmea_raw, sizeof(nmea_raw));

	while (1) {
		if (new_data_ready >= 1) {
			// Simply dump out the data for now....
			snprintf((char*) console_buf2, sizeof(console_buf2),
					"Received new GPS data, size is: %d\r\n", nmea_idx);
			tx_com(console_buf2, strlen((char const*) console_buf2));
			tx_com(nmea_raw, nmea_idx);

			// this is non-functional logic.... don't use it.
			NMEA_ParseBuf(nmea_raw, &nmea_idx);
			snprintf((char*) console_buf2, sizeof(console_buf2),
					"Received GPS data with lat (microdeg) %d and lon (microdeg) %d\r\n",
					data_RMC.lat_microdeg, data_RMC.lon_microdeg);
			tx_com(console_buf2, strlen((char const *) console_buf2));
			new_data_ready = 0;
		}
		HAL_Delay(10);
	}
}

static void tx_com(uint8_t *tx_buffer, uint16_t len) {
	HAL_UART_Transmit(&huart2, tx_buffer, len, 1000);
}

void HAL_UARTEx_RxEventCallback(UART_HandleTypeDef *huart, uint16_t Size) {
	nmea_idx = Size;
	HAL_GPIO_TogglePin(GPIO_LED2_GPIO_Port, GPIO_LED2_Pin);
	HAL_UARTEx_ReceiveToIdle_DMA(&huart1, nmea_raw, sizeof(nmea_raw));
	new_data_ready += 1;
}

void HAL_UART_RxCpltCallback(UART_HandleTypeDef *huart) {
	HAL_GPIO_TogglePin(GPIO_LED2_GPIO_Port, GPIO_LED2_Pin);
}
