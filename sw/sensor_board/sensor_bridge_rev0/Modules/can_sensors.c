/*
 * can_sensors.c
 *
 *  Created on: Mar 19, 2025
 *      Author: Hardy Yu
 */

#include "can_sensors.h"
#include "spi.h"

/* IMU static variables */
extern SPI_HandleTypeDef hspi1;		// ASM330 SPI
static stmdev_ctx_t dev_ctx;
static asm330lhh_reg_t reg;
static uint32_t timestamp;

/* time stamp mark*/
static uint64_t next_10hz_service_at;

/* protobuf */
#define PROTOBUF_SIZE 254
static uint8_t pb[PROTOBUF_SIZE];
static uint8_t encode[PROTOBUF_SIZE];

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
imu_dronecan_transform_pb (struct uavcan_equipment_ahrs_SensorIMU *dronecan_data, struct raw_imu_data_t *pb_data){
	pb_data->timestamp = dronecan_data->timestamp;
	pb_data->accel_x = dronecan_data->accelerometer_latest[0];
	pb_data->accel_y = dronecan_data->accelerometer_latest[0];
	pb_data->accel_z = dronecan_data->accelerometer_latest[0];
	pb_data->gyro_x = dronecan_data->rate_gyro_latest[0];
	pb_data->gyro_y = dronecan_data->rate_gyro_latest[0];
	pb_data->gyro_z = dronecan_data->rate_gyro_latest[0];

	// information lost during the CAN comm, do we really need them?
	// seems like they can be processed before sent to rpi
	pb_data->gda = false;
	pb_data->xlda = false;
}

void handle_RawIMU(CanardInstance *ins, CanardRxTransfer *transfer){
	struct uavcan_equipment_ahrs_SensorIMU rawIMU;

	if (uavcan_equipment_ahrs_SensorIMU_decode(transfer, &rawIMU)) {
		return;
	}

	// preparing for the pb data
	struct raw_imu_data_t *pb_data_p = raw_imu_data_new(&pb, UAVCAN_EQUIPMENT_AHRS_SENSORIMU_MAX_SIZE);
	imu_dronecan_transform_pb(&rawIMU, pb_data_p);
	size_t pb_size = raw_imu_data_encode(pb_data_p, encode, UAVCAN_EQUIPMENT_AHRS_SENSORIMU_MAX_SIZE);

	// then we transmit it through UART

	/* toggle a LED when rx call back is trigger, for debugging */
	HAL_GPIO_TogglePin(GPIOB, GPIO_PIN_4);
	return;
}
