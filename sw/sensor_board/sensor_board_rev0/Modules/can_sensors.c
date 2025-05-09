/*
 * can_sensors.c
 *
 *  Created on: Mar 19, 2025
 *      Author: Hardy Yu
 */

#include "can_sensors.h"
#include "spi.h"
#include "canard.h"

/* IMU static variables */
extern SPI_HandleTypeDef hspi1;		// ASM330 SPI
extern CanardInstance canard;		// from can_interface.c
static stmdev_ctx_t dev_ctx;
static asm330lhh_reg_t reg;
static uint32_t timestamp;

/* time stamp mark*/
static uint64_t next_10hz_service_at;

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

void send_RawIMU(struct uavcan_equipment_ahrs_SensorIMU raw_imu){
	uint8_t buffer[UAVCAN_EQUIPMENT_AHRS_SENSORIMU_MAX_SIZE];

	uint32_t len = uavcan_equipment_ahrs_SensorIMU_encode(&raw_imu, buffer);

	static uint8_t transfer_id;

    canardBroadcast(&canard,
					UAVCAN_EQUIPMENT_AHRS_SENSORIMU_SIGNATURE,
                    UAVCAN_EQUIPMENT_AHRS_SENSORIMU_ID,
                    &transfer_id,
                    CANARD_TRANSFER_PRIORITY_LOW,
                    buffer,
                    len);
}

struct uavcan_equipment_gnss_SensorGPS
raw_gps_transform_dronecan (GpsData_T data){
	struct uavcan_equipment_gnss_SensorGPS dronecan_data;

	/* TODO: How to make the GPS's ts aligned with IMU's ts? */
	dronecan_data.timestamp = 0;

	/* really rough, MVP sorta */
	dronecan_data.hours = data.time_utc.hours;
	dronecan_data.minutes = data.time_utc.minutes;
	dronecan_data.seconds = data.time_utc.seconds;
	dronecan_data.lat_microdeg = data.lat_microdeg;
	dronecan_data.lon_microdeg = data.lon_microdeg;
	dronecan_data.altitude_m = data.altitude_m;
	dronecan_data.speed_kts = data.speed_kts;
	dronecan_data.heading_deg = data.heading_deg;
	dronecan_data.num_sats = data.num_sats;
	dronecan_data.fix_status = data.fix_status;

	return dronecan_data;
}

void send_RawGPS(struct uavcan_equipment_gnss_SensorGPS raw_gps){
	uint8_t buffer[UAVCAN_EQUIPMENT_GNSS_SENSORGPS_MAX_SIZE];

	uint32_t len = uavcan_equipment_gnss_SensorGPS_encode(&raw_gps, buffer);

	static uint8_t transfer_id;

	canardBroadcast(&canard,
					UAVCAN_EQUIPMENT_GNSS_SENSORGPS_SIGNATURE,
					UAVCAN_EQUIPMENT_GNSS_SENSORGPS_ID,
					&transfer_id,
					CANARD_TRANSFER_PRIORITY_LOW,
					buffer,
					len);
}

/* a lot of copy paste from anni's code, just for testing */
void imu_setup(void){
    // IMU Setup
	dev_ctx.write_reg = platform_write;
	dev_ctx.read_reg = platform_read;
	dev_ctx.mdelay = platform_delay;
	dev_ctx.handle = &hspi1;

	// Wait IMU boot time.
	HAL_Delay(100);

	do {
        asm330lhh_device_id_get(&dev_ctx, &asm330_wai);
    } while (asm330_wai != ASM330LHH_ID);

    // Restore Default Configuration
	asm330lhh_reset_set(&dev_ctx, PROPERTY_ENABLE);
	do {
		asm330lhh_reset_get(&dev_ctx, &rst);
	} while (rst);

    // Configure IMU
	asm330lhh_device_conf_set(&dev_ctx, PROPERTY_ENABLE); 			// Set device configuration (no clue what this does)
	asm330lhh_block_data_update_set(&dev_ctx, PROPERTY_ENABLE);		// Enable block data update
	asm330lhh_xl_data_rate_set(&dev_ctx, ASM330LHH_XL_ODR_417Hz);	// TODO: determine the correct output rates
	asm330lhh_gy_data_rate_set(&dev_ctx, ASM330LHH_GY_ODR_417Hz);
	asm330lhh_xl_full_scale_set(&dev_ctx, ASM330LHH_2g);			// TODO: determine the correct scaling
	asm330lhh_gy_full_scale_set(&dev_ctx, ASM330LHH_2000dps);
	asm330lhh_timestamp_set(&dev_ctx, PROPERTY_ENABLE);				// Enable timestamping

	/* Configure filtering chain(No aux interface)
	 * Accelerometer - LPF1 + LPF2 path
	 */
	asm330lhh_xl_hp_path_on_out_set(&dev_ctx, ASM330LHH_LP_ODR_DIV_100);
	asm330lhh_xl_filter_lp2_set(&dev_ctx, PROPERTY_ENABLE);

    /* time stamp setup */
    next_10hz_service_at = HAL_GetTick();
}

void can_imu_loop(void){
	const uint64_t ts = HAL_GetTick();

    if (ts < next_10hz_service_at){
        return;
    }

    next_10hz_service_at += 100ULL;

    imuRawData_S raw_data;
    asm330lhh_status_reg_get(&dev_ctx, &raw_data.status_reg);

    if (raw_data.status_reg.xlda || raw_data.status_reg.gda) {
        asm330lhh_timestamp_raw_get(&dev_ctx, &timestamp);
        raw_data.timestamp = timestamp;
    } else {
        return;
    }

    if (raw_data.status_reg.xlda) {
        int16_t raw_acceleration[3];
        asm330lhh_acceleration_raw_get(&dev_ctx, raw_acceleration);
        raw_data.acceleration.i16bit[0] = raw_acceleration[0];
        raw_data.acceleration.i16bit[1] = raw_acceleration[1];
        raw_data.acceleration.i16bit[2] = raw_acceleration[2];
#if DO_FP
        acceleration_mg[0] = asm330lhh_from_fs2g_to_mg(
                raw_data.acceleration.i16bit[0]);
        acceleration_mg[1] = asm330lhh_from_fs2g_to_mg(
                raw_data.acceleration.i16bit[1]);
        acceleration_mg[2] = asm330lhh_from_fs2g_to_mg(
                raw_data.acceleration.i16bit[2]);

        acceleration_g[0] = acceleration_mg[0] / 1000;
        acceleration_g[1] = acceleration_mg[1] / 1000;
        acceleration_g[2] = acceleration_mg[2] / 1000;
#endif
    } else {
        return;
    }

    if (raw_data.status_reg.gda) {
    	int16_t raw_angular_rate[3];
        asm330lhh_angular_rate_raw_get(&dev_ctx, raw_angular_rate);
        raw_data.angular_rate.i16bit[0] = raw_angular_rate[0];
		raw_data.angular_rate.i16bit[1] = raw_angular_rate[1];
		raw_data.angular_rate.i16bit[2] = raw_angular_rate[2];
#if DO_FP
        angular_rate_mdps[0] = asm330lhh_from_fs2000dps_to_mdps(
                raw_data.angular_rate.i16bit[0]);
        angular_rate_mdps[1] = asm330lhh_from_fs2000dps_to_mdps(
                raw_data.angular_rate.i16bit[1]);
        angular_rate_mdps[2] = asm330lhh_from_fs2000dps_to_mdps(
                raw_data.angular_rate.i16bit[2]);

        angular_rate_dps[0] = angular_rate_mdps[0] / 1000;
        angular_rate_dps[1] = angular_rate_mdps[1] / 1000;
        angular_rate_dps[2] = angular_rate_mdps[2] / 1000;
#endif
    } else {
        return;
    }

    /* CAN package transmission */
    struct uavcan_equipment_ahrs_SensorIMU can_imu_pkt = raw_imu_transform_dronecan(raw_data);
    send_RawIMU(can_imu_pkt);

	/* toggling an led light, could be commented*/
	HAL_GPIO_TogglePin(GPIOB, GPIO_PIN_5);
}
