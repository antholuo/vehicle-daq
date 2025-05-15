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

////////
// Private function prototypes

#define NUM_IMUS 1
#define NUM_GNSS 1

extern UART_HandleTypeDef huart1;

static const ImuType_T imu_types[NUM_IMUS] = {IMU_ASM330LHH};
static ImuData_S imu_data[NUM_IMUS];
static const GpsType_T gps_types[NUM_GNSS] = {GPS_NEO_M8N};
static GpsData_S gps_data[NUM_GNSS];
bool flag_send_motion_data;

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

    (void)gnss_parse_data_if_available(gps_types, NUM_GNSS, gps_data);
    return true;
}

void run_sensor_board() {
    (void)setup_peripherals();

    // Start task timers
    HAL_TIM_Base_Start_IT(&htim16);
    HAL_TIM_Base_Start_IT(&htim17);

    while(1) {
        poll_peripherals();

        // TODO: check motion_data available, send CAN message
        if (flag_send_motion_data) {
            flag_send_motion_data = false;
        }
    }

    //
}

////////
// Callbacks

void task_100hz() {
    MotionDataRaw_S motion_data;
    static uint8_t count = 0;
    if (count++ > 99) {
        /* HAL_GPIO_TogglePin(GPIO_LED1_GPIO_Port, GPIO_LED1_Pin); */
        count = 0;
    }

    // TODO: set a flag that allows us to send a CAN message inside of `run_sensor_board()`
    flag_send_motion_data = true;
}

void task_800hz() {
    static uint8_t count = 0;
    if (count++ > 99) {
        /* HAL_GPIO_TogglePin(GPIO_LED2_GPIO_Port, GPIO_LED2_Pin); */
        count = 0;
    }
}

void HAL_UARTEx_RxEventCallback(UART_HandleTypeDef *huart, uint16_t Size) {
    HAL_GPIO_TogglePin(GPIO_LED1_GPIO_Port, GPIO_LED1_Pin);
    if (gnss_process_incoming_data(huart, Size)) {
        return;
    } else {
        // Empty for now, but check other UART messages here
    }
}

void HAL_UART_RxCpltCallback(UART_HandleTypeDef *huart) {
    HAL_GPIO_TogglePin(GPIO_LED2_GPIO_Port, GPIO_LED2_Pin);
}
// Restarts the DMA reception on UART3 whenever a framing error occurs
void HAL_UART_ErrorCallback(UART_HandleTypeDef *huart)
{
	if(huart->Instance == USART1)
	{
        gnss_process_incoming_data(huart, 0);
	}
}
