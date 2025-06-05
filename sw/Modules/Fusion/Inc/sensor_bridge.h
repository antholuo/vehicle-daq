/**
 * sensor_board.h
 *
 * Created on: 2025-MAY-10
 *     Author: Anni
 *
 * Prototypes for setting up and running the sensor board
 */

#ifndef FUSION_SENSOR_BRIDGE_H
#define FUSION_SENSOR_BRIDGE_H

#include "can_sensors.h"

#define PROTOBUF_UART_INSTANCE (USART2)

void run_sensor_bridge();

void can_sensor_reception_setup();

void can_sensor_reception_loop();

void can_sensor_bridge_imu_handler(
    const struct uavcan_equipment_ahrs_SensorIMU *can_imu_p, uint8_t can_id);

void can_sensor_bridge_gps_handler(
    const struct uavcan_equipment_gnss_SensorGPS *can_gps_p, uint8_t can_id);

#endif  // FUSION_SENSOR_BRIDGE_H
