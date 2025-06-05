/**
 * imu.h
 *
 * Created on: 2025-MAY-10
 *     Author: Anni
 *   Modified: 2025-MAY-10
 *
 * Common prototypes for using all IMU's
 */

#ifndef IMU_H
#define IMU_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

typedef enum {
  IMU_NOT_AVAIL = 0,
  IMU_ASM330LHH = 1,
  // IMU_BMX160 = 2,
} ImuType_T;

typedef enum {
  IMU_STATUS_OK = 0,
  IMU_STATUS_ERR = 1,
  IMU_STATUS_SETUP_FAILURE = 2,
  // Future error types?
} ImuStatus_E;

/**
 * imu data struct
 * pack the data so that there is no space between fields
 * https://www.reddit.com/r/embedded/comments/1byxt04/attribute_packed_in_embedded_systems_pros_and_cons/
 *
 * TODO: eventually decide if we prever "gyro" or "angular rate"
 */
// typedef struct __attribute__((packed)) {
typedef struct {
  uint32_t timestamp;
  int16_t accel_x_mg;  // milli-g
  int16_t accel_y_mg;
  int16_t accel_z_mg;
  int16_t gyro_x_mdps;  // milli-degree-per-second
  int16_t gyro_y_mdps;
  int16_t gyro_z_mdps;
  int16_t mag_x_microT;  // micro-tesla
  int16_t mag_y_microT;
  int16_t mag_z_microT;
  bool accel_data_valid;
  bool gyro_data_valid;
  bool mag_data_valid;
} ImuData_S;

ImuStatus_E setup_imus(const ImuType_T *const imu_type, const size_t num_imus);

ImuStatus_E poll_imus(const ImuType_T *const imu_type, const size_t num_imus,
                      ImuData_S *imu_data);

#endif  // IMU_H
