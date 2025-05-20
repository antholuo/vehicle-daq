/*
 * can_sensors.h
 *
 *  Created on: Mar 19, 2025
 *      Author: Hardy Yu
 */
#ifndef CAN_SENSORS_H_
#define CAN_SENSORS_H_

#include <stdint.h>
#include "canard.h"
#include "dronecan_msgs.h"
#include "imu.h"

typedef enum {
    COMMS_STATUS_OK = 0,
    COMMS_STATUS_ERR = 1,
} CAN_COMMS_STATUS_E;

void
convertImuToDroneCAN (const ImuData_S *src,
                    // const uint8_t imu_id,
                    struct uavcan_equipment_ahrs_SensorIMU *dst);

/* broadcast this node's IMU data on can bus */
CAN_COMMS_STATUS_E can_send_ImuData(struct uavcan_equipment_ahrs_SensorIMU raw_imu);

/* handling imu data received from other can node */
void handle_ImuData(CanardInstance *ins, CanardRxTransfer *transfer);

#endif /* CAN_SENSORS_H_ */
