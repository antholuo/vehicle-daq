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
static uint32_t timestamp;

/* time stamp mark*/
static uint64_t next_10hz_service_at;

struct uavcan_equipment_ahrs_SensorIMU
raw_imu_transform_dronecan (ImuData_S data){
    struct uavcan_equipment_ahrs_SensorIMU dronecan_data;

    dronecan_data.timestamp = data.timestamp;
    dronecan_data.accelerometer_latest[0] = data.accel_x_mg;
    dronecan_data.rate_gyro_latest[0] = data.gyro_x_mdps;
    dronecan_data.magnetometer_latest[0] = data.mag_x_microT;
    dronecan_data.accelerometer_latest[1] = data.accel_y_mg;
    dronecan_data.rate_gyro_latest[1] = data.gyro_y_mdps;
    dronecan_data.magnetometer_latest[1] = data.mag_y_microT;
    dronecan_data.accelerometer_latest[2] = data.accel_z_mg;
    dronecan_data.rate_gyro_latest[2] = data.gyro_z_mdps;
    dronecan_data.magnetometer_latest[2] = data.mag_z_microT;
    dronecan_data.accel_data_valid = data.accel_data_valid;
    dronecan_data.gyro_data_valid = data.gyro_data_valid;
    dronecan_data.mag_data_valid = data.mag_data_valid;
    
    return dronecan_data;
}

/* a lot of copy paste from anni's code, just for testing */
void imu_setup(void){
    // // IMU Setup
	// dev_ctx.write_reg = platform_write;
	// dev_ctx.read_reg = platform_read;
	// dev_ctx.mdelay = platform_delay;
	// dev_ctx.handle = &hspi1;

	// // Wait IMU boot time.
	// HAL_Delay(100);

	// do {
    //     asm330lhh_device_id_get(&dev_ctx, &asm330_wai);
    // } while (asm330_wai != ASM330LHH_ID);

    // // Restore Default Configuration
	// asm330lhh_reset_set(&dev_ctx, PROPERTY_ENABLE);
	// do {
	// 	asm330lhh_reset_get(&dev_ctx, &rst);
	// } while (rst);

    // // Configure IMU
	// asm330lhh_device_conf_set(&dev_ctx, PROPERTY_ENABLE); 			// Set device configuration (no clue what this does)
	// asm330lhh_block_data_update_set(&dev_ctx, PROPERTY_ENABLE);		// Enable block data update
	// asm330lhh_xl_data_rate_set(&dev_ctx, ASM330LHH_XL_ODR_417Hz);	// TODO: determine the correct output rates
	// asm330lhh_gy_data_rate_set(&dev_ctx, ASM330LHH_GY_ODR_417Hz);
	// asm330lhh_xl_full_scale_set(&dev_ctx, ASM330LHH_2g);			// TODO: determine the correct scaling
	// asm330lhh_gy_full_scale_set(&dev_ctx, ASM330LHH_2000dps);
	// asm330lhh_timestamp_set(&dev_ctx, PROPERTY_ENABLE);				// Enable timestamping

	// /* Configure filtering chain(No aux interface)
	//  * Accelerometer - LPF1 + LPF2 path
	//  */
	// asm330lhh_xl_hp_path_on_out_set(&dev_ctx, ASM330LHH_LP_ODR_DIV_100);
	// asm330lhh_xl_filter_lp2_set(&dev_ctx, PROPERTY_ENABLE);

    // /* time stamp setup */
    // next_10hz_service_at = HAL_GetTick();
}

void can_imu_loop(void){
// 	const uint64_t ts = HAL_GetTick();

//     if (ts < next_10hz_service_at){
//         return;
//     }

//     next_10hz_service_at += 100ULL;

//     ImuData_S raw_data;
//     asm330lhh_status_reg_get(&dev_ctx, &raw_data.status_reg);

//     if (raw_data.status_reg.xlda || raw_data.status_reg.gda) {
//         asm330lhh_timestamp_raw_get(&dev_ctx, &timestamp);
//         raw_data.timestamp = timestamp;
//     } else {
//         return;
//     }

//     if (raw_data.status_reg.xlda) {
//         int16_t raw_acceleration[3];
//         asm330lhh_acceleration_raw_get(&dev_ctx, raw_acceleration);
//         raw_data.acceleration.i16bit[0] = raw_acceleration[0];
//         raw_data.acceleration.i16bit[1] = raw_acceleration[1];
//         raw_data.acceleration.i16bit[2] = raw_acceleration[2];
// #if DO_FP
//         acceleration_mg[0] = asm330lhh_from_fs2g_to_mg(
//                 raw_data.acceleration.i16bit[0]);
//         acceleration_mg[1] = asm330lhh_from_fs2g_to_mg(
//                 raw_data.acceleration.i16bit[1]);
//         acceleration_mg[2] = asm330lhh_from_fs2g_to_mg(
//                 raw_data.acceleration.i16bit[2]);

//         acceleration_g[0] = acceleration_mg[0] / 1000;
//         acceleration_g[1] = acceleration_mg[1] / 1000;
//         acceleration_g[2] = acceleration_mg[2] / 1000;
// #endif
//     } else {
//         return;
//     }

//     if (raw_data.status_reg.gda) {
//     	int16_t raw_angular_rate[3];
//         asm330lhh_angular_rate_raw_get(&dev_ctx, raw_angular_rate);
//         raw_data.angular_rate.i16bit[0] = raw_angular_rate[0];
// 		raw_data.angular_rate.i16bit[1] = raw_angular_rate[1];
// 		raw_data.angular_rate.i16bit[2] = raw_angular_rate[2];
// #if DO_FP
//         angular_rate_mdps[0] = asm330lhh_from_fs2000dps_to_mdps(
//                 raw_data.angular_rate.i16bit[0]);
//         angular_rate_mdps[1] = asm330lhh_from_fs2000dps_to_mdps(
//                 raw_data.angular_rate.i16bit[1]);
//         angular_rate_mdps[2] = asm330lhh_from_fs2000dps_to_mdps(
//                 raw_data.angular_rate.i16bit[2]);

//         angular_rate_dps[0] = angular_rate_mdps[0] / 1000;
//         angular_rate_dps[1] = angular_rate_mdps[1] / 1000;
//         angular_rate_dps[2] = angular_rate_mdps[2] / 1000;
// #endif
//     } else {
//         return;
//     }

//     /* CAN package transmission */
//     struct uavcan_equipment_ahrs_SensorIMU can_imu_pkt = raw_imu_transform_dronecan(raw_data);
//     send_RawIMU(can_imu_pkt);

// 	/* toggling an led light, could be commented*/
// 	HAL_GPIO_TogglePin(GPIOB, GPIO_PIN_5);
}
