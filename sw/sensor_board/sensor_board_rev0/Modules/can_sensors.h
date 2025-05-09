/*
 * can_sensors.h
 *
 *  Created on: Mar 19, 2025
 *      Author: Hardy Yu
 */
#ifndef CAN_SENSORS_H_
#define CAN_SENSORS_H_

#include <stdint.h>
#include "dronecan_msgs.h"
#include "imu.h"
#include "gps.h"


/* IMU data conversion to dronecan format */
struct uavcan_equipment_ahrs_SensorIMU
raw_imu_transform_dronecan (imuRawData_S data);

/* broadcast this node's IMU data on can bus */
void send_RawIMU(struct uavcan_equipment_ahrs_SensorIMU raw_imu);

/* IMU data conversion to dronecan format */
struct uavcan_equipment_gnss_SensorGPS
raw_gps_transform_dronecan (GpsData_T data);

/* broadcast GPS's data on CAN bus */
void send_RawGPS(struct uavcan_equipment_gnss_SensorGPS raw_gps);


void imu_setup(void);
void can_imu_loop(void);

/* The TODOs */
void gps_setup(void);
void can_gps_loop(void);


#endif /* CAN_SENSORS_H_ */
