/**
 * sensor_board.c
 *
 * Created on: 2025-MAY-10
 *     Author: Anni
 *
 * Contains primary sensor_board control loop and setup
 */

#include "imu.h"
#include "sensor_board.h"
#include "blinky.h" // Only for testing

#include "gpio.h"
#include "tim.h"

////////
// Private function prototypes

#define NUM_IMUS 1
static const ImuType_T imu_types[NUM_IMUS] = {IMU_ASM330LHH};
static ImuData_S imu_data[NUM_IMUS];

static bool setup_peripherals() {
    bool returnVal = true;

    // Setup sensors
    returnVal &= setup_imus(imu_types, NUM_IMUS);

    return returnVal;
}

static bool poll_peripherals() {
    poll_imus(imu_types, NUM_IMUS, imu_data);

    return true;
}

void run_sensor_board() {
    (void)setup_peripherals();

    // Start task timers
    HAL_TIM_Base_Start_IT(&htim16);
    HAL_TIM_Base_Start_IT(&htim17);

    while(1) {
        poll_peripherals();
        /* blinky(); */
    }

    //
}

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

