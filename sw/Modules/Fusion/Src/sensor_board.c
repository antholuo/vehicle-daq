/**
 * sensor_board.c
 *
 * Created on: 2025-MAY-10
 *     Author: Anni
 *
 * Contains primary sensor_board control loop and setup
 */

#include "sensor_board.h"
#include "imu.h"
#include "gps.h"
#include "fusion_datatypes.h"
#include "blinky.h" // Only for testing

#include "gpio.h"
#include "tim.h"
#include "usart.h"

#include "can_sensors.h"
#include "can_interface.h"

////////
// Private function prototypes

#define NUM_IMUS 1
#define NUM_GNSS 1
// /* TODO: intruduce this macro in build system instead, plus making this configurable in build */
// #define IMU_ID   10

extern UART_HandleTypeDef huart1;

static const ImuType_T imu_types[NUM_IMUS] = {IMU_ASM330LHH};
static ImuData_S imu_data[NUM_IMUS];
static const GpsType_T gps_types[NUM_GNSS] = {GPS_NEO_M8N};
static GpsData_S gps_data[NUM_GNSS];
bool flag_100hz;

static bool setup_peripherals() {
    bool returnVal = true;

    // Setup sensors
    returnVal &= setup_imus(imu_types, NUM_IMUS);

    /* __HAL_ENABLE_IT(&huart1, UART_IT_IDLE); // is this it? */
    gnss_start_rx(gps_types, NUM_GNSS);

    return returnVal;
}

static bool poll_peripherals() {
    poll_imus(imu_types, NUM_IMUS, imu_data);

    /* disabling GPS for now, will add it back later */
    // (void)gnss_parse_data_if_available(gps_types, NUM_GNSS, gps_data);
    return true;
}

void run_sensor_board() {
    (void)setup_peripherals();
    setup_comms();

    // Start task timers
    HAL_TIM_Base_Start_IT(&htim16);
    HAL_TIM_Base_Start_IT(&htim17);

    uint8_t blink_cnt = 0;
    while(1) {
        poll_peripherals();
        loop_comms();

        // TODO: check gps data availability, transmit gps data on CAN
        if (flag_100hz) {
            struct uavcan_equipment_ahrs_SensorIMU can_imu_pkt;
            can_pack_ImuData(&imu_data[0], &can_imu_pkt);
            (void)can_send_ImuData(can_imu_pkt);
            flag_100hz = false;
            if (blink_cnt == 10){
                HAL_GPIO_TogglePin(GPIO_LED1_GPIO_Port, GPIO_LED1_Pin);
                blink_cnt = 0;
            }
            blink_cnt++;
        }
    }

    //
}

////////
// Callbacks

void task_100hz() {
    // MotionDataRaw_S motion_data;

    flag_100hz = true;
}

void task_800hz() {
    static uint8_t count = 0;
    if (count++ > 99) {
        /* HAL_GPIO_TogglePin(GPIO_LED2_GPIO_Port, GPIO_LED2_Pin); */
        count = 0;
    }
}

void HAL_UARTEx_RxEventCallback(UART_HandleTypeDef *huart, uint16_t Size) {
    /* HAL_GPIO_TogglePin(GPIO_LED1_GPIO_Port, GPIO_LED1_Pin); */
    if (gnss_process_incoming_data(huart, Size)) {
        return;
    } else {
        // Empty for now, but check other UART messages here
    }
}

void HAL_UART_RxCpltCallback(UART_HandleTypeDef *huart) {
    /* HAL_GPIO_TogglePin(GPIO_LED2_GPIO_Port, GPIO_LED2_Pin); */
}
// Restarts the DMA reception on UART3 whenever a framing error occurs
void HAL_UART_ErrorCallback(UART_HandleTypeDef *huart)
{
	if(huart->Instance == USART1)
	{
        gnss_process_incoming_data(huart, 0);
	}
}
