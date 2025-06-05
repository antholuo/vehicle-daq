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
#include "gps.h"
#include "imu.h"
#include "sensor_data.h"

typedef enum {
  COMMS_STATUS_OK = 0,
  COMMS_STATUS_ERR = 1,
} CanCommsStatus_E;

typedef void (*IMUReceptionFunc)(const struct uavcan_equipment_ahrs_SensorIMU *,
                                 uint8_t);
typedef void (*GPSReceptionFunc)(const struct uavcan_equipment_gnss_SensorGPS *,
                                 uint8_t);

extern IMUReceptionFunc imu_reception_f_ptr;
extern GPSReceptionFunc gps_reception_f_ptr;

void can_pack_ImuData(const ImuData_S *src,
                      struct uavcan_equipment_ahrs_SensorIMU *dst);

/* DroneCAN IMU format converts to pb format */
void protobuf_pack_ImuData(const struct uavcan_equipment_ahrs_SensorIMU *src,
                           struct imu_data_t *dst);

/* broadcast this node's IMU data on can bus */
CanCommsStatus_E can_send_ImuData(
    struct uavcan_equipment_ahrs_SensorIMU raw_imu);

/* handling imu data received from other can node */
void can_receive_ImuData(CanardInstance *ins, CanardRxTransfer *transfer);

/* Packing gps data to DroneCAN format */
void can_pack_GpsData(const GpsData_S *src,
                      struct uavcan_equipment_gnss_SensorGPS *dst);

/* broadcast this node's GPS data on can bus */
CanCommsStatus_E can_send_GpsData(
    struct uavcan_equipment_gnss_SensorGPS gps_data);

/* DroneCAN GPS format converts to pb format */
void protobuf_pack_GpsData(const struct uavcan_equipment_gnss_SensorGPS *src,
                           struct gps_data_t *dst);

/* handling gps data received from other can node */
void can_receive_GpsData(CanardInstance *ins, CanardRxTransfer *transfer);

#endif /* CAN_SENSORS_H_ */
