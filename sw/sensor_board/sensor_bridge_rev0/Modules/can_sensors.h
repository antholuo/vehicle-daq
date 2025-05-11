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


#define IMU_PROTOBUF_SIZE 64
#define CQ_DEPTH	4
typedef struct {
    uint8_t buffer[CQ_DEPTH][IMU_PROTOBUF_SIZE];  // Queue storage
    uint16_t lengths[CQ_DEPTH];          // Lengths of valid data
    uint8_t head;  // Points to the next message to send
    uint8_t tail;  // Points to the next free slot
    uint8_t count; // Number of valid items
} CircularQueue;


/* IMU data conversion to dronecan format */
struct uavcan_equipment_ahrs_SensorIMU
raw_imu_transform_dronecan (imuRawData_S data);

/* DroneCAN IMU format converts to pd format */
void imu_dronecan_transform_pb (struct uavcan_equipment_ahrs_SensorIMU dronecan_data, struct raw_imu_data_t *pd_data);

/* handling raw imu data received from other can node */
void handle_RawIMU(CanardInstance *ins, CanardRxTransfer *transfer);

/* run before while loop, prepare for handling can data */
void can_sensor_reception_setup(void);

/* function to call inside while(1) */
void can_sensor_reception_loop(void);


/* INNER FUNCTIONS FOR CICRULAR QUEUE ! */
void cq_init(CircularQueue *q);

int cq_enqueue(CircularQueue *q, const uint8_t *data, uint16_t len);

int cq_dequeue(CircularQueue *q, uint8_t **data, uint16_t *len);

#endif /* CAN_SENSORS_H_ */
