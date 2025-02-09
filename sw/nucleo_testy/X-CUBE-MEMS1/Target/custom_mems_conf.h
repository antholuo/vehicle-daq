/**
  ******************************************************************************
  * @file    custom_mems_conf.h
  * @author  MEMS Software Solutions Team
  * @brief   This file contains definitions of the MEMS components bus interfaces for custom boards
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

/* Define to prevent recursive inclusion -------------------------------------*/
#ifndef CUSTOM_MEMS_CONF_H
#define CUSTOM_MEMS_CONF_H

#ifdef __cplusplus
extern "C" {
#endif

/* Includes ------------------------------------------------------------------*/
#include "stm32f0xx_hal.h"
#include "stm32f0xx_nucleo_bus.h"
#include "stm32f0xx_nucleo_errno.h"

/* USER CODE BEGIN 1 */

/* USER CODE END 1 */

#define USE_CUSTOM_MOTION_SENSOR_ASM330LHH_0      0U

#define CUSTOM_ASM330LHH_0_SPI_Init BSP_SPI1_Init
#define CUSTOM_ASM330LHH_0_SPI_DeInit BSP_SPI1_DeInit
#define CUSTOM_ASM330LHH_0_SPI_Send BSP_SPI1_Send
#define CUSTOM_ASM330LHH_0_SPI_Recv BSP_SPI1_Recv

#define CUSTOM_ASM330LHH_0_CS_PORT GPIOF
#define CUSTOM_ASM330LHH_0_CS_PIN GPIO_PIN_1

#ifdef __cplusplus
}
#endif

#endif /* CUSTOM_MEMS_CONF_H*/
