/* USER CODE BEGIN Header */
/**
 ******************************************************************************
 * @file           : main.c
 * @brief          : Main program body
 ******************************************************************************
 * @attention
 *
 * Copyright (c) 2025 STMicroelectronics.
 * All rights reserved.
 *
 * This software is licensed under terms that can be found in the LICENSE file
 * in the root directory of this software component.
 * If no LICENSE file comes with this software, it is provided AS-IS.
 *
 ******************************************************************************
 */
/* USER CODE END Header */
/* Includes ------------------------------------------------------------------*/
#include "main.h"
#include "can.h"
#include "crc.h"
#include "spi.h"
#include "usart.h"
#include "gpio.h"

/* Private includes ----------------------------------------------------------*/
/* USER CODE BEGIN Includes */
#include <stdio.h>
#include <string.h>

#include "asm330lhh_reg.h"

/* USER CODE END Includes */

/* Private typedef -----------------------------------------------------------*/
/* USER CODE BEGIN PTD */

/* USER CODE END PTD */

/* Private define ------------------------------------------------------------*/
/* USER CODE BEGIN PD */

/* USER CODE END PD */

/* Private macro -------------------------------------------------------------*/
/* USER CODE BEGIN PM */
#define    BOOT_TIME            10 //ms
/* USER CODE END PM */

/* Private variables ---------------------------------------------------------*/

/* USER CODE BEGIN PV */

static int16_t data_raw_acceleration[3];
static int16_t data_raw_angular_rate[3];
static int16_t data_raw_temperature;
//static float_t acceleration_mg[3];
//static float_t angular_rate_mdps[3];
//static float_t temperature_degC;
static uint8_t whoamI, rst;
static uint8_t tx_buffer[200];



extern SPI_HandleTypeDef hspi1;
extern UART_HandleTypeDef haurt1;
extern UART_HandleTypeDef huart2;

/* USER CODE END PV */

/* Private function prototypes -----------------------------------------------*/
void SystemClock_Config(void);
/* USER CODE BEGIN PFP */

/** Please note that is MANDATORY: return 0 -> no Error.**/
static int32_t platform_write(void *handle, uint8_t reg, const uint8_t *bufp,
		uint16_t len);
static int32_t platform_read(void *handle, uint8_t reg, uint8_t *bufp,
		uint16_t len);

/** Optional (may be required by driver) **/
static void platform_delay(uint32_t millisec);

static void tx_com( uint8_t *tx_buffer, uint16_t len );

/* USER CODE END PFP */

/* Private user code ---------------------------------------------------------*/
/* USER CODE BEGIN 0 */

/* USER CODE END 0 */

/**
  * @brief  The application entry point.
  * @retval int
  */
int main(void)
{

  /* USER CODE BEGIN 1 */
//	if (STATE_SIZE < MotionFX_CM0P_GetStateSize()) {
//		Error_Handler();
//	}
  /* USER CODE END 1 */

  /* MCU Configuration--------------------------------------------------------*/

  /* Reset of all peripherals, Initializes the Flash interface and the Systick. */
  HAL_Init();

  /* USER CODE BEGIN Init */

  /* USER CODE END Init */

  /* Configure the system clock */
  SystemClock_Config();

  /* USER CODE BEGIN SysInit */

  /* USER CODE END SysInit */

  /* Initialize all configured peripherals */
  MX_GPIO_Init();
  MX_USART2_UART_Init();
  MX_CAN_Init();
  MX_USART1_UART_Init();
  MX_CRC_Init();
  MX_SPI1_Init();
  /* USER CODE BEGIN 2 */

	/* Initialize mems driver interface */
	stmdev_ctx_t dev_ctx;
	dev_ctx.write_reg = platform_write;
	dev_ctx.read_reg = platform_read;
	dev_ctx.mdelay = platform_delay;
	dev_ctx.handle = &hspi1;

	/* Wait sensor boot time */
	HAL_Delay(100);
	/* Check device ID */
	asm330lhh_device_id_get(&dev_ctx, &whoamI);

	HAL_Delay(100);

	/* Restore default configuration */
	asm330lhh_reset_set(&dev_ctx, PROPERTY_ENABLE);

	HAL_Delay(100);

	do {
		asm330lhh_reset_get(&dev_ctx, &rst);
		HAL_Delay(100);
	} while (rst);

	/* Start device configuration. */
	asm330lhh_device_conf_set(&dev_ctx, PROPERTY_ENABLE);
	/* Enable Block Data Update */
	asm330lhh_block_data_update_set(&dev_ctx, PROPERTY_ENABLE);
	/* Set Output Data Rate */
	asm330lhh_xl_data_rate_set(&dev_ctx, ASM330LHH_XL_ODR_52Hz);
	asm330lhh_gy_data_rate_set(&dev_ctx, ASM330LHH_GY_ODR_52Hz);
	/* Set full scale */
	asm330lhh_xl_full_scale_set(&dev_ctx, ASM330LHH_4g);
	asm330lhh_gy_full_scale_set(&dev_ctx, ASM330LHH_2000dps);


	/* Configure filtering chain(No aux interface)
	 * Accelerometer - LPF1 + LPF2 path
	 */
//	asm330lhh_xl_hp_path_on_out_set(&dev_ctx, ASM330LHH_LP_ODR_DIV_100);
//	asm330lhh_xl_filter_lp2_set(&dev_ctx, PROPERTY_ENABLE);

	// TESTING SECTION
//	uint8_t data[] = "\n";
  /* USER CODE END 2 */

  /* Infinite loop */
  /* USER CODE BEGIN WHILE */
	while (1) {
		uint8_t reg;
		// Toggle LED
		HAL_GPIO_TogglePin(GPIO_LED_GPIO_Port, GPIO_LED_Pin);

		/* Read output only if new xl value is available */
		asm330lhh_xl_flag_data_ready_get(&dev_ctx, &reg);

		if (reg) {
			/* Read acceleration field data */
			memset(data_raw_acceleration, 0x00, 3 * sizeof(int16_t));
			asm330lhh_acceleration_raw_get(&dev_ctx, data_raw_acceleration);
//			acceleration_mg[0] = asm330lhh_from_fs2g_to_mg(
//					data_raw_acceleration[0]);
//			acceleration_mg[1] = asm330lhh_from_fs2g_to_mg(
//					data_raw_acceleration[1]);
//			acceleration_mg[2] = asm330lhh_from_fs2g_to_mg(
//					data_raw_acceleration[2]);
			sprintf((char *)tx_buffer,
			              "AccelerationRaw :%d\t%d\t%d\r\n",
			              data_raw_acceleration[0], data_raw_acceleration[1], data_raw_acceleration[2]);
			tx_com(tx_buffer, strlen((char const *)tx_buffer));
		}

		HAL_Delay(500);


		asm330lhh_gy_flag_data_ready_get(&dev_ctx, &reg);

		if (reg) {
			/* Read angular rate field data */
			memset(data_raw_angular_rate, 0x00, 3 * sizeof(int16_t));
			asm330lhh_angular_rate_raw_get(&dev_ctx, data_raw_angular_rate);
//			angular_rate_mdps[0] = asm330lhh_from_fs2000dps_to_mdps(
//					data_raw_angular_rate[0]);
//			angular_rate_mdps[1] = asm330lhh_from_fs2000dps_to_mdps(
//					data_raw_angular_rate[1]);
//			angular_rate_mdps[2] = asm330lhh_from_fs2000dps_to_mdps(
//					data_raw_angular_rate[2]);
			sprintf((char *)tx_buffer,
			              "AngularRateRaw :%d\t%d\t%d\r\n",
			              data_raw_angular_rate[0], data_raw_angular_rate[1], data_raw_angular_rate[2]);
			tx_com(tx_buffer, strlen((char const *)tx_buffer));
		}

//		asm330lhh_temp_flag_data_ready_get(&dev_ctx, &reg);
//
//		if (reg) {
//			/* Read temperature data */
////			memset(&data_raw_temperature, 0x00, sizeof(int16_t));
//			asm330lhh_temperature_raw_get(&dev_ctx, &data_raw_temperature);
//		    sprintf((char *)tx_buffer,
//		              "TemperatureRaw: %d\r\n", data_raw_temperature);
//		    tx_com(tx_buffer, strlen((char const *)tx_buffer));
//		}

//		HAL_UART_Transmit(&huart1, acceleration_mg, sizeof(int16_t) * 3, 100);
    /* USER CODE END WHILE */

    /* USER CODE BEGIN 3 */
//		HAL_UART_Transmit(&huart2, data, 1, 1000);
//		HAL_UART_Transmit(&huart2, data, 1, 1000);
		HAL_Delay(500);
	}
  /* USER CODE END 3 */
}

/**
  * @brief System Clock Configuration
  * @retval None
  */
void SystemClock_Config(void)
{
  RCC_OscInitTypeDef RCC_OscInitStruct = {0};
  RCC_ClkInitTypeDef RCC_ClkInitStruct = {0};
  RCC_PeriphCLKInitTypeDef PeriphClkInit = {0};

  /** Initializes the RCC Oscillators according to the specified parameters
  * in the RCC_OscInitTypeDef structure.
  */
  RCC_OscInitStruct.OscillatorType = RCC_OSCILLATORTYPE_HSI48;
  RCC_OscInitStruct.HSI48State = RCC_HSI48_ON;
  RCC_OscInitStruct.PLL.PLLState = RCC_PLL_NONE;
  if (HAL_RCC_OscConfig(&RCC_OscInitStruct) != HAL_OK)
  {
    Error_Handler();
  }

  /** Initializes the CPU, AHB and APB buses clocks
  */
  RCC_ClkInitStruct.ClockType = RCC_CLOCKTYPE_HCLK|RCC_CLOCKTYPE_SYSCLK
                              |RCC_CLOCKTYPE_PCLK1;
  RCC_ClkInitStruct.SYSCLKSource = RCC_SYSCLKSOURCE_HSI48;
  RCC_ClkInitStruct.AHBCLKDivider = RCC_SYSCLK_DIV1;
  RCC_ClkInitStruct.APB1CLKDivider = RCC_HCLK_DIV1;

  if (HAL_RCC_ClockConfig(&RCC_ClkInitStruct, FLASH_LATENCY_1) != HAL_OK)
  {
    Error_Handler();
  }
  PeriphClkInit.PeriphClockSelection = RCC_PERIPHCLK_USART1;
  PeriphClkInit.Usart1ClockSelection = RCC_USART1CLKSOURCE_PCLK1;
  if (HAL_RCCEx_PeriphCLKConfig(&PeriphClkInit) != HAL_OK)
  {
    Error_Handler();
  }
}

/* USER CODE BEGIN 4 */

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

/*
 * @brief  Write generic device register (platform dependent)
 *
 * @param  tx_buffer     buffer to transmit
 * @param  len           number of byte to send
 *
 */
static void tx_com(uint8_t *tx_buffer, uint16_t len)
{
  HAL_UART_Transmit(&huart2, tx_buffer, len, 1000);
}

/* USER CODE END 4 */

/**
  * @brief  This function is executed in case of error occurrence.
  * @retval None
  */
void Error_Handler(void)
{
  /* USER CODE BEGIN Error_Handler_Debug */
	/* User can add his own implementation to report the HAL error return state */
	__disable_irq();
	while (1) {
	}
  /* USER CODE END Error_Handler_Debug */
}

#ifdef  USE_FULL_ASSERT
/**
  * @brief  Reports the name of the source file and the source line number
  *         where the assert_param error has occurred.
  * @param  file: pointer to the source file name
  * @param  line: assert_param error line source number
  * @retval None
  */
void assert_failed(uint8_t *file, uint32_t line)
{
  /* USER CODE BEGIN 6 */
  /* User can add his own implementation to report the file name and line number,
     ex: printf("Wrong parameters value: file %s on line %d\r\n", file, line) */
  /* USER CODE END 6 */
}
#endif /* USE_FULL_ASSERT */
