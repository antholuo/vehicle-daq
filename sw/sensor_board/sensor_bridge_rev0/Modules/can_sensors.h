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


/* IMU data conversion to dronecan format
 * TODO: USE custome imu message instead of dronecan's
 */
struct uavcan_equipment_ahrs_SensorIMU
raw_imu_transform_dronecan (imuRawData_S data);


#endif /* CAN_SENSORS_H_ */
