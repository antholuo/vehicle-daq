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
#define PROTOBUF_SIZE 512
static uint8_t pb[PROTOBUF_SIZE];
static uint8_t encoded[PROTOBUF_SIZE];
static struct raw_imu_data_t *pb_imu_p;

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
	 int pb_size = raw_imu_data_encode(pb_imu_p, &encoded[0], PROTOBUF_SIZE);

	 // then we transmit it through UART
	 HAL_UART_Transmit(&huart2, &encoded[0], (uint16_t)pb_size, 100);

	/* toggle a LED when rx call back is trigger, for debugging */
	HAL_GPIO_TogglePin(GPIOB, GPIO_PIN_4);
	return;
}

void can_sensor_receiption_setup(void){
	pb_imu_p = raw_imu_data_new(&pb, PROTOBUF_SIZE);
}
