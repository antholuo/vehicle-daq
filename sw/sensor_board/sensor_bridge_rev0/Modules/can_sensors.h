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
#include "imu_pb.h"
#include "canard.h"


/* IMU data conversion to dronecan format */
struct uavcan_equipment_ahrs_SensorIMU
raw_imu_transform_dronecan (imuRawData_S data);

/* DroneCAN IMU format converts to pd format */
void imu_dronecan_transform_pb (struct uavcan_equipment_ahrs_SensorIMU *dronecan_data, struct raw_imu_data_t *pd_data);

/* handling raw imu data received from other can node */
void handle_RawIMU(CanardInstance *ins, CanardRxTransfer *transfer);

void

#endif /* CAN_SENSORS_H_ */
