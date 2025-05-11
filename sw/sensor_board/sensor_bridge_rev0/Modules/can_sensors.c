/*
 * can_sensors.c
 *
 *  Created on: Mar 19, 2025
 *      Author: Hardy Yu
 */

#include "can_sensors.h"
#include "spi.h"
#include "usart.h"

/* IMU static variables */
extern SPI_HandleTypeDef hspi1;		// ASM330 SPI

/* time stamp mark*/
// static uint64_t next_10hz_service_at;

/* protobuf */
extern UART_HandleTypeDef huart2;	// pb comm

static uint8_t imu_pb[IMU_PROTOBUF_SIZE];
static struct raw_imu_data_t *pb_imu_p;
static CircularQueue imu_tx_queue;
static uint8_t dma_busy;

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

void
imu_dronecan_transform_pb (struct uavcan_equipment_ahrs_SensorIMU dronecan_data, struct raw_imu_data_t *pb_data){
	if (pb_data == NULL){
		return;
	}
	pb_data->timestamp = (int32_t)dronecan_data.timestamp;
	pb_data->accel_x = dronecan_data.accelerometer_latest[0];
	pb_data->accel_y = dronecan_data.accelerometer_latest[1];
	pb_data->accel_z = dronecan_data.accelerometer_latest[2];
	pb_data->gyro_x = dronecan_data.rate_gyro_latest[0];
	pb_data->gyro_y = dronecan_data.rate_gyro_latest[1];
	pb_data->gyro_z = dronecan_data.rate_gyro_latest[2];

	// information lost during the CAN comm, do we really need them?
	// seems like they can be processed before sent to rpi
	pb_data->gda = false;
	pb_data->xlda = false;
}

void handle_RawIMU(CanardInstance *ins, CanardRxTransfer *transfer){
	// try using static memory instead of interrupt stack for this variable
	struct uavcan_equipment_ahrs_SensorIMU rawIMU;

	if (uavcan_equipment_ahrs_SensorIMU_decode(transfer, &rawIMU)) {
		return;
	}

	// preparing for the pb data
	imu_dronecan_transform_pb(rawIMU, pb_imu_p);
	uint8_t encoded[IMU_PROTOBUF_SIZE];
	int pb_size = raw_imu_data_encode(pb_imu_p, encoded, IMU_PROTOBUF_SIZE);

	// then we add it to the queue, it will be transmit by DMA in main loop
	cq_enqueue(&imu_tx_queue, encoded, pb_size);

	/* toggle a LED when rx call back is trigger, for debugging */
	HAL_GPIO_TogglePin(GPIOB, GPIO_PIN_4);
	return;
}

void can_sensor_reception_setup(void){
	pb_imu_p = raw_imu_data_new(&imu_pb, IMU_PROTOBUF_SIZE);
	cq_init(&imu_tx_queue);
}

void can_sensor_reception_loop(void){
	uint8_t *imu_encoded_q  = NULL;
	uint16_t len;
	if (dma_busy == 0) {
		if (cq_dequeue(&imu_tx_queue, &imu_encoded_q, &len)){
			HAL_StatusTypeDef ret = HAL_UART_Transmit_DMA(&huart2, imu_encoded_q, len);
			if (ret == HAL_OK){
				dma_busy = 1;
			}
		}
	}
}


void cq_init(CircularQueue *q) {
    q->head = 0;
    q->tail = 0;
    q->count = 0;
}

int cq_enqueue(CircularQueue *q, const uint8_t *data, uint16_t len) {
    if (q->count >= CQ_DEPTH || len > IMU_PROTOBUF_SIZE)
        return 0;

    memcpy(q->buffer[q->tail], data, len);
    q->lengths[q->tail] = len;
    q->tail = (q->tail + 1) % CQ_DEPTH;
    q->count++;
    return 1;
}

int cq_dequeue(CircularQueue *q, uint8_t **data, uint16_t *len) {
    if (q->count == 0)
        return 0;

    *data = q->buffer[q->head];
    *len = q->lengths[q->head];
    q->head = (q->head + 1) % CQ_DEPTH;
    q->count--;
    return 1;
}

void HAL_UART_TxCpltCallback(UART_HandleTypeDef *huart) {
    if (huart == &huart2) {
		HAL_GPIO_TogglePin(GPIOB, GPIO_PIN_5);
        dma_busy = 0; // ready for next one
    }
}

