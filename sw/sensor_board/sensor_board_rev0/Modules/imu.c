/*
 * imu.c
 *
 *  Created on: Feb 22, 2025
 *      Author: Anthony Luo
 */

#include "imu.h"
#include "gpio.h"
#include "spi.h"

#include "asm330lhh_reg.h"

extern SPI_HandleTypeDef hspi1;

void run_imu_basic() {
	uint8_t count = 0;

	// IMU DEFINES
	stmdev_ctx_t dev_ctx;

	dev_ctx.write_reg = platform_write;
	dev_ctx.read_reg = platform_read;
	dev_ctx.mdelay = platform_delay;
	dev_ctx.handle = &hspi1;

	while(1) {
		  count++;
		  if (count == 10) {
			  count = 0;
			  HAL_GPIO_TogglePin(GPIO_LED2_GPIO_Port, GPIO_LED2_Pin);
		  }
		  HAL_GPIO_TogglePin(GPIO_LED1_GPIO_Port, GPIO_LED1_Pin);

		  // Use the ASM330LHH PID driver to get device ID
		  asm330lhh_device_id_get(&dev_ctx, &whoamI);

		  if (whoamI == ASM330LHH_ID) {
			  while(1) {
				  HAL_GPIO_WritePin(GPIO_LED1_GPIO_Port, GPIO_LED1_Pin, GPIO_PIN_SET);
				  HAL_GPIO_TogglePin(GPIO_LED2_GPIO_Port, GPIO_LED2_Pin);
				  HAL_Delay(100);
			  }
		  }
		  HAL_Delay(100);
	}
}

static int32_t platform_write(void *handle, uint8_t reg, const uint8_t *bufp,
                              uint16_t len)
{
  HAL_SPI_Transmit(handle, &reg, 1, 1000);
  HAL_SPI_Transmit(handle, (uint8_t*) bufp, len, 1000);
  return 0;
}

static int32_t platform_read(void *handle, uint8_t reg, uint8_t *bufp,
                             uint16_t len)
{
  reg |= 0x80;
  HAL_SPI_Transmit(handle, &reg, 1, 1000);
  HAL_SPI_Receive(handle, bufp, len, 1000);
  return 0;
}

static void platform_delay(uint32_t ms)
{
  HAL_Delay(ms);
}
