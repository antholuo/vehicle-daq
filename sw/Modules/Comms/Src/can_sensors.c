/*
 * can_sensors.c
 *
 *  Created on: Mar 19, 2025
 *      Author: Hardy Yu
 */

#include "can_sensors.h"

extern CanardInstance canard;

void can_pack_ImuData (const ImuData_S *src,
					       struct uavcan_equipment_ahrs_SensorIMU *dst){
    /* magnetometer data is unsupported unfortunately */
    dst->timestamp = src->timestamp;
    dst->accelerometer_latest[0] = src->accel_x_mg;
    dst->rate_gyro_latest[0] = src->gyro_x_mdps;
    dst->accelerometer_latest[1] = src->accel_y_mg;
    dst->rate_gyro_latest[1] = src->gyro_y_mdps;
    dst->accelerometer_latest[2] = src->accel_z_mg;
    dst->rate_gyro_latest[2] = src->gyro_z_mdps;
    dst->accel_data_valid = src->accel_data_valid;
    dst->gyro_data_valid = src->gyro_data_valid;
}

CanCommsStatus_E can_send_ImuData(struct uavcan_equipment_ahrs_SensorIMU raw_imu){
	uint8_t buffer[UAVCAN_EQUIPMENT_AHRS_SENSORIMU_MAX_SIZE];

	uint32_t len = uavcan_equipment_ahrs_SensorIMU_encode(&raw_imu, buffer);

	static uint8_t transfer_id;

    int16_t frame_num = canardBroadcast(&canard,
					UAVCAN_EQUIPMENT_AHRS_SENSORIMU_SIGNATURE,
                    UAVCAN_EQUIPMENT_AHRS_SENSORIMU_ID,
                    &transfer_id,
                    CANARD_TRANSFER_PRIORITY_LOW,
                    buffer,
                    len);
	if (frame_num <= 0 ){
		return COMMS_STATUS_ERR;
	}

    return COMMS_STATUS_OK;
}

void can_receive_ImuData(CanardInstance *ins, CanardRxTransfer *transfer){
	struct uavcan_equipment_ahrs_SensorIMU rawIMU;

	if (uavcan_equipment_ahrs_SensorIMU_decode(transfer, &rawIMU)) {
		return;
	}

	/* TODO: add formal IMU package handling */
    
	return;
}
