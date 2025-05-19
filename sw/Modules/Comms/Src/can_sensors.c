/*
 * can_sensors.c
 *
 *  Created on: Mar 19, 2025
 *      Author: Hardy Yu
 */

#include "can_sensors.h"
#include "spi.h"

/* IMU static variables */
extern SPI_HandleTypeDef hspi1;		// ASM330 SPI
static uint32_t timestamp;

/* time stamp mark*/
static uint64_t next_10hz_service_at;

void
convertImuToDroneCAN (const ImuData_S *src,
					// const uint8_t imu_id,
					struct uavcan_equipment_ahrs_SensorIMU *dst){
	// dst->imu_id = imu_id;
    dst->timestamp = src->timestamp;
    dst->accelerometer_latest[0] = src->accel_x_mg;
    dst->rate_gyro_latest[0] = src->gyro_x_mdps;
    // dst->magnetometer_latest[0] = src->mag_x_microT;
    dst->accelerometer_latest[1] = src->accel_y_mg;
    dst->rate_gyro_latest[1] = src->gyro_y_mdps;
    // dst->magnetometer_latest[1] = src->mag_y_microT;
    dst->accelerometer_latest[2] = src->accel_z_mg;
    dst->rate_gyro_latest[2] = src->gyro_z_mdps;
    // dst->magnetometer_latest[2] = src->mag_z_microT;
    dst->accel_data_valid = src->accel_data_valid;
    dst->gyro_data_valid = src->gyro_data_valid;
    // dst->mag_data_valid = src->mag_data_valid;
}