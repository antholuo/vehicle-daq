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

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

typedef enum {
    IMU_NOT_AVAIL = 0,
    IMU_ASM330LHH = 1,
    // IMU_BMX160 = 2,
} imuType_T;

/**
* imu data struct
* pack the data so that there is no space between fields
* https://www.reddit.com/r/embedded/comments/1byxt04/attribute_packed_in_embedded_systems_pros_and_cons/
*/
typedef struct __attribute__((packed)) {
    int32_t timestamp;
    int32_t accel_x_mg;   // milli-g
    int32_t accel_y_mg;
    int32_t accel_z_mg;
    int32_t gyro_x_mdps;  // milli-degree-per-second
    int32_t gyro_y_mdps;
    int32_t gyro_z_mdps;
    int32_t mag_x_microT; // micro-tesla
    int32_t mag_y_microT;
    int32_t mag_z_microT;
    bool accel_data_valid;
    bool gyro_data_valid;
    bool mag_data_valid;
} imuData_S;

bool setup_imus(const imuType_T * const imu_type, size_t num_imus);

bool poll_imus(imuType_T *imu_type, size_t num_imus, imuData_S *imu_data);

#endif // IMU_H

