/*
 * imu.h
 *
 *  Created on: Feb 22, 2025
 *      Author: Anthony Luo
 */

#ifndef IMU_H_
#define IMU_H_

// Sets whether or not we want to do floating point operations on the micro.
// 1u = enable, 0u = disable
#define DO_FP 1u

#include <stdint.h>

#include "asm330lhh_reg.h"

typedef union {
  int16_t i16bit[3];
  uint8_t u8bit[6];
} axis3bit16_t;

// TODO: build a struct of cool data

static axis3bit16_t data_raw_acceleration;
static axis3bit16_t data_raw_angular_rate;
static uint8_t asm330_wai, rst;

#if DO_FP
// TODO: replace these and populate these on the rust micro
static float_t acceleration_mg[3];
static float_t acceleration_g[3];
static float_t angular_rate_mdps[3];
static float_t angular_rate_dps[3];
#endif DO_FP

static int32_t platform_write(void *handle, uint8_t reg, const uint8_t *bufp,
                              uint16_t len);
static int32_t platform_read(void *handle, uint8_t reg, uint8_t *bufp,
                             uint16_t len);
//static void tx_com( uint8_t *tx_buffer, uint16_t len );
static void platform_delay(uint32_t ms);

void run_imu_basic();

#endif /* IMU_H_ */
