/*
 * imu.c
 *
 *  Created on: Feb 22, 2025
 *      Author: Anthony Luo
 */

#include "imu.h"
#include "gpio.h"
#include "spi.h"

extern SPI_HandleTypeDef hspi1;

void run_imu_basic() {
	uint8_t count = 0;
	uint8_t tx[2];
	uint8_t rx[2];

	tx[0] = 0x00; // pad second half of tx with 0's
	tx[1] = 0x0F | 0x80; // 0x0F is WHOAMI, pad with 80 to make r/w bit high at the start

	while(1) {
		  count++;
		  if (count == 10) {
			  count = 0;
			  HAL_GPIO_TogglePin(GPIO_LED2_GPIO_Port, GPIO_LED2_Pin);
		  }
		  HAL_GPIO_TogglePin(GPIO_LED1_GPIO_Port, GPIO_LED1_Pin);

		  // Send SPI Transaction
		  HAL_SPI_TransmitReceive(&hspi1, tx, rx, 1, 1000);

		  if (rx[0] == 0x6B) {
			  while(1) {
				  HAL_GPIO_WritePin(GPIO_LED1_GPIO_Port, GPIO_LED1_Pin, GPIO_PIN_SET);
				  HAL_GPIO_TogglePin(GPIO_LED2_GPIO_Port, GPIO_LED2_Pin);
				  HAL_Delay(100);
			  }
		  }
		  HAL_Delay(100);
	}
}
