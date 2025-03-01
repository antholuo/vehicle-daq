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
typedef struct __attribute__((packed)) {
	asm330lhh_status_reg_t status_reg;
	uint32_t timestamp;
	axis3bit16_t acceleration;
	axis3bit16_t angular_rate;
} imuRawData_S;

#endif /* IMU_H_ */
