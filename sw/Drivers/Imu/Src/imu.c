/**
 * imu.c
 *
 * Created on: 2025-MAY-10
 *     Author: Anni
 *   Modified: 2025-MAY-10
 *
 * Common functions for using all IMU's
 */

#include "imu.h"

bool setup_imus(const imuType_T * const imu_type, size_t num_imus) {
    bool returnVal = true;
    for (size_t i = 0; i < num_imus; ++i) {
        switch(imu_type[i]) {
        case IMU_ASM330LHH:
            /* returnVal &= setup_imu_asm330lhh(); */
            break;
        default:
            // throw an error?
            break;
        }
    }

    return returnVal;
}


bool poll_imus(imuType_T *imu_type, size_t num_imus, imuData_S *imu_data) {
    return false;
}
