/*
 * can_sensors.c
 *
 *  Created on: Mar 19, 2025
 *      Author: Hardy Yu
 */

#include "can_sensors.h"


/* global imu reception handler function allowing configuration */
IMUReceptionFunc imu_reception_f_ptr = NULL;

/* global gps reception handler function allowing configuration */
GPSReceptionFunc gps_reception_f_ptr = NULL;

/* using canard instance */

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


void protobuf_pack_ImuData (const struct uavcan_equipment_ahrs_SensorIMU *src, 
                            struct imu_data_t *dst){
	if (src == NULL || dst == NULL){
		return;
	}
	dst->timestamp = (int32_t)src->timestamp;
	dst->accel_x_mg = (int32_t)src->accelerometer_latest[0];
	dst->accel_y_mg = (int32_t)src->accelerometer_latest[1];
	dst->accel_z_mg = (int32_t)src->accelerometer_latest[2];
	dst->gyro_x_mdps = (int32_t)src->rate_gyro_latest[0];
	dst->gyro_y_mdps = (int32_t)src->rate_gyro_latest[1];
	dst->gyro_z_mdps = (int32_t)src->rate_gyro_latest[2];
	dst->gyro_data_valid = src->accel_data_valid;
	dst->accel_data_valid = src->accel_data_valid;
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

	/* If the imu reception handler is defined elsewhere, then run the handler function */
    if (imu_reception_f_ptr != NULL){
        uint8_t can_id = 10; // fake data 
        imu_reception_f_ptr(&rawIMU, can_id);
    }

    
	return;
}

void can_pack_GpsData (const GpsData_S *src,
                       struct uavcan_equipment_gnss_SensorGPS *dst){
	dst->hours = src->nmea_time.time_utc.hours;
	dst->minutes = src->nmea_time.time_utc.minutes;
	dst->seconds = src->nmea_time.time_utc.seconds;
	dst->year = src->nmea_time.date_utc.year;
	dst->month = src->nmea_time.date_utc.month;
	dst->day = src->nmea_time.date_utc.day;
	dst->lat_microdeg = src->lat_microdeg;
	dst->lon_microdeg = src->lon_microdeg;
	dst->altitude_mm = src->altitude_mm;
	dst->speed_mkts = src->speed_mkts; 
	dst->course_deg = src->course_deg;
	dst->num_sats = src->num_sats;
	dst->fix_status = src->fix_status;
	dst->data_valid = src->data_valid;
}

/* DroneCAN GPS format converts to pb format */
void protobuf_pack_GpsData (const struct uavcan_equipment_gnss_SensorGPS *src, 
                            struct gps_data_t *dst){
	if (src == NULL || dst == NULL){
		return;
	}
	if (dst->time_p == NULL || dst->date_p == NULL){
		return;
	}
	dst->time_p->hours = (uint32_t)src->hours;
	dst->time_p->minutes = (uint32_t)src->minutes;
	dst->time_p->seconds = (uint32_t)src->seconds;
	dst->date_p->year = (uint32_t)src->year;
	dst->date_p->month = (uint32_t)src->month;
	dst->date_p->day = (uint32_t)src->day;
	dst->lat_microdeg = (int32_t)src->lat_microdeg;
	dst->lon_microdeg = (int32_t)src->lon_microdeg;
	dst->altitude_mm = (int32_t)src->altitude_mm;
	dst->speed_mkts = (int32_t)src->speed_mkts; 
	dst->course_deg = (int32_t)src->course_deg;
	dst->num_sats = (int32_t)src->num_sats;
	dst->quality = (int32_t)src->fix_status;
	dst->data_valid = src->data_valid;
}

/* broadcast this node's GPS data on can bus */
CanCommsStatus_E can_send_GpsData(struct uavcan_equipment_gnss_SensorGPS gps_data){
	uint8_t buffer[UAVCAN_EQUIPMENT_GNSS_SENSORGPS_MAX_SIZE];

	uint32_t len = uavcan_equipment_gnss_SensorGPS_encode(&gps_data, buffer);

	static uint8_t transfer_id;

    int16_t frame_num = canardBroadcast(&canard,
					UAVCAN_EQUIPMENT_GNSS_SENSORGPS_SIGNATURE,
                    UAVCAN_EQUIPMENT_GNSS_SENSORGPS_ID,
                    &transfer_id,
                    CANARD_TRANSFER_PRIORITY_LOW,
                    buffer,
                    len);
	if (frame_num <= 0 ){
		return COMMS_STATUS_ERR;
	}

    return COMMS_STATUS_OK;
}

void can_receive_GpsData(CanardInstance *ins, CanardRxTransfer *transfer){
	struct uavcan_equipment_gnss_SensorGPS gps_data;

	if (uavcan_equipment_gnss_SensorGPS_decode(transfer, &gps_data)) {
		return;
	}

	/* If the imu reception handler is defined elsewhere, then run the handler function */
    if (gps_reception_f_ptr != NULL){
        uint8_t can_id = 10; // fake data 
        gps_reception_f_ptr(&gps_data, can_id);
    }

    
	return;
}
