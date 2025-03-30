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
static stmdev_ctx_t dev_ctx;
static asm330lhh_reg_t reg;
static uint32_t timestamp;

/* time stamp mark*/
static uint64_t next_10hz_service_at;

struct uavcan_equipment_ahrs_SensorIMU
raw_imu_transform_dronecan (imuRawData_S data){
    struct uavcan_equipment_ahrs_SensorIMU dronecan_data;

    dronecan_data.timestamp = data.timestamp;

    for (int i = 0; i < 3; i++){
        dronecan_data.accelerometer_latest[i] = data.acceleration.i16bit[i];
        dronecan_data.rate_gyro_latest[i] = data.angular_rate.i16bit[i];
    }
    
    return dronecan_data;
}
