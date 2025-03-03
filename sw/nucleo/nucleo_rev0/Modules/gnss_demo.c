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


extern UART_HandleTypeDef huartt1; // Where we are receiving the raw GNSS data
extern UART_HandleTypeDef huart2; // where we send the pretty data (our computer)

uint8_t console_buf2[100];
uint8_t nmea_raw[2000];	// Encoded NMEA data
uint8_t nmea_idx = 0;

static void tx_com(uint8_t *tx_buffer, uint16_t len);

void run_gnss_demo() {

	HAL_UARTEx_ReceiveToIdle_DMA(&huart1, nmea_raw, sizeof(nmea_raw));

	while (1) {
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
}

void HAL_UART_RxCpltCallback(UART_HandleTypeDef *huart)
{
	HAL_GPIO_TogglePin(GPIO_LED2_GPIO_Port, GPIO_LED2_Pin);
}
