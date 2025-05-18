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

void
convertImuToDroneCAN (const ImuData_S *src, struct uavcan_equipment_ahrs_SensorIMU *dst);

void imu_setup(void);
void can_imu_loop(void);



#endif /* CAN_SENSORS_H_ */
