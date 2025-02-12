/*
 * asm330lhh_demo.c
 *
 * Created on: 2025-FEB-11 by Anni
 * This is a file which we will use to demonstrate the functionality of the ASM330LHH sensor
 */

#include "asm330lhh_demo.h"
#include "gpio.h"
#include "spi.h"
#include "usart.h"

#include <stdio.h>
#include <string.h>

// forward declare a bunch of our component specific functions

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

static void tx_com(uint8_t *tx_buffer, uint16_t len)
{
 HAL_UART_Transmit(&huart2, tx_buffer, len, 1000);
}


// Some data structs that we'll need
static int16_t data_raw_acceleration[3];
static int16_t data_raw_angular_rate[3];
static int16_t data_raw_temperature;
static uint8_t rst;

static uint8_t tx_buffer[200];

extern SPI_HandleTypeDef hspi1;
extern UART_HandleTypeDef haurt1;
extern UART_HandleTypeDef huart2;


void startAsm330lhhDemo(void)
{
    // Initialize the ASM 330 

    stmdev_ctx_t dev_ctx;
	dev_ctx.handle = &hspi1;

    HAL_Delay(100); // Wait boot time

    asm330lhh_reset_get(&dev_ctx, &rst); // reset the device to defaults

    HAL_Delay(100); // Wait device response

    do {
        // asm330lhh_reset
        // TODO: fix this shit
    }


    // This is a placeholder for the demo code
    while (1) {
        HAL_GPIO_TogglePin(GPIO_LED_GPIO_Port, GPIO_LED_Pin);

        HAL_Delay(1000);
    }
}