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

#define NUM_IMUS 1
static const ImuType_T imu_types[NUM_IMUS] = {IMU_ASM330LHH};

static bool setup_sensor_board() {
    bool returnVal = true;

    returnVal &= setup_imus(imu_types, NUM_IMUS);

    return returnVal;
}

void run_sensor_board() {
    (void)setup_sensor_board();

    while(1) {
        blinky();
    }

    //
}
