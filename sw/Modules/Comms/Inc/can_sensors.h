/*
 * can_sensors.h
 *
 *  Created on: Mar 19, 2025
 *      Author: Hardy Yu
 */
#ifndef CAN_SENSORS_H_
#define CAN_SENSORS_H_

#include "canard.h"
#include "dronecan_msgs.h"

#include "imu.h"

#include <stdint.h>



typedef enum {
    COMMS_STATUS_OK = 0,
    COMMS_STATUS_ERR = 1,
} CanCommsStatus_E;

void can_pack_ImuData (const ImuData_S *src,
                           struct uavcan_equipment_ahrs_SensorIMU *dst);

/* broadcast this node's IMU data on can bus */
CanCommsStatus_E can_send_ImuData(struct uavcan_equipment_ahrs_SensorIMU raw_imu);

/* handling imu data received from other can node */
void can_receive_ImuData(CanardInstance *ins, CanardRxTransfer *transfer);

#endif /* CAN_SENSORS_H_ */
