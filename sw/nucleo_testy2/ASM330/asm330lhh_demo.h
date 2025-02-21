/*
 * asm330lhh_demo.h
 *
 *  Created on: Feb 12, 2025
 *      Author: antho
 */

#ifndef ASM330LHH_DEMO_H_
#define ASM330LHH_DEMO_H_

#include "asm330lhh_reg.h"

typedef struct {
	int16_t xl_x;
	int16_t xl_y;
	int16_t xl_z;
	int16_t gy_x;
	int16_t gy_y;
	int16_t gy_z;
//	uint32_t ts;
//	asm330lhh_status_reg_t sr;
} RawImuDataPkg_S;

void transmit_hello_world();

void start_demo();

#endif /* ASM330LHH_DEMO_H_ */
