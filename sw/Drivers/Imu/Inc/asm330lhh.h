/**
 * asm330lhh.h
 *
 * Created on: 2025-MAY-10
 *     Author: Anni
 *   Modified: 2025-MAY-10
 *
 * Functions for using ASM330LHH IMU
 */

#ifndef ASM330LHH_H
#define ASM330LHH_H

#include "imu.h"

ImuStatus_E setup_imu_asm330lhh();

ImuStatus_E poll_imu_asm330lhh(ImuData_S *imu_data);

#endif  // ASM330LHH_H
