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

#include "asm330lhh.h"

ImuStatus_E setup_imus(const ImuType_T *const imu_type, const size_t num_imus) {
  ImuStatus_E retVal = IMU_STATUS_OK;
  for (size_t i = 0; i < num_imus; ++i) {
    switch (imu_type[i]) {
      case IMU_ASM330LHH:
        if (setup_imu_asm330lhh() != IMU_STATUS_OK) {
          retVal = IMU_STATUS_ERR;
        }
        break;
      default:
        // throw an error?
        break;
    }
  }

  return retVal;
}

ImuStatus_E poll_imus(const ImuType_T *const imu_type, const size_t num_imus,
                      ImuData_S *imu_data) {
  for (size_t i = 0; i < num_imus; ++i) {
    switch (imu_type[i]) {
      case IMU_ASM330LHH:
        poll_imu_asm330lhh(&imu_data[i]);
        break;
      default:
        // throw an error?
        break;
    }
  }
  return false;
}
