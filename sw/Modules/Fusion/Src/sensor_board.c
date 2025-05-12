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
#include "blinky.h" // Only for testing

#include "gpio.h"
#include "tim.h"

////////
// Private function prototypes

#define NUM_IMUS 1
#define NUM_GNSS 1
static const ImuType_T imu_types[NUM_IMUS] = {IMU_ASM330LHH};
static ImuData_S imu_data[NUM_IMUS];
static GpsData_S gps_data[NUM_GNSS];

static bool setup_peripherals() {
    bool returnVal = true;

    // Setup sensors
    returnVal &= setup_imus(imu_types, NUM_IMUS);

    start_gnss_rx();

    return returnVal;
}

static bool poll_peripherals() {
    poll_imus(imu_types, NUM_IMUS, imu_data);

    if (new_gnss_data_available()) {
        parse_gnss_data(&gps_data[0]); // only 1 gps for now
    }
    return true;
}

void run_sensor_board() {
    (void)setup_peripherals();

    // Start task timers
    HAL_TIM_Base_Start_IT(&htim16);
    HAL_TIM_Base_Start_IT(&htim17);

    while(1) {
        poll_peripherals();
    }

    //
}

////////
// Callbacks

void task_100hz() {
    static uint8_t count = 0;
    if (count++ > 99) {
        HAL_GPIO_TogglePin(GPIO_LED1_GPIO_Port, GPIO_LED1_Pin);
        count = 0;
    }
}

void task_800hz() {
    static uint8_t count = 0;
    if (count++ > 99) {
        HAL_GPIO_TogglePin(GPIO_LED2_GPIO_Port, GPIO_LED2_Pin);
        count = 0;
    }
}

void HAL_UARTEx_RxEventCallback(UART_HandleTypeDef *huart, uint16_t Size) {
    if (huart->Instance == M8N_UART_INSTANCE) {
        process_incoming_gnss_data(Size);
    }
}
