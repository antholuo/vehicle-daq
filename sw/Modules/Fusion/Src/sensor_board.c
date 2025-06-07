/**
 * sensor_board.c
 *
 * Created on: 2025-MAY-10
 *     Author: Anni
 *
 * Contains primary sensor_board control loop and setup
 */

#include "sensor_board.h"

#include "blinky.h"  // Only for testing
#include "can_interface.h"
#include "can_sensors.h"
#include "fusion_datatypes.h"
#include "gpio.h"
#include "gps.h"
#include "imu.h"
#include "tim.h"
#include "usart.h"

////////
// Private function prototypes

#define NUM_IMUS 1
#define NUM_GNSS 1

/* this delay serves for staggering the CAN bus TX timing, default 0 */
#ifndef BOARD_INIT_DELAY_MS
#define BOARD_INIT_DELAY_MS 0
#endif

extern UART_HandleTypeDef huart1;

static const ImuType_T imu_types[NUM_IMUS] = {IMU_ASM330LHH};
static ImuData_S imu_data[NUM_IMUS];
static const GpsType_T gps_types[NUM_GNSS] = {GPS_NEO_M8N};
static GpsData_S gps_data[NUM_GNSS];
bool flag_100hz = false;
bool flag_new_gps = false;

static bool setup_peripherals() {
  bool returnVal = true;

  // Setup sensors
  returnVal &= setup_imus(imu_types, NUM_IMUS);

  // TODO: Have "setup_gnss" that sets messages wanted and baud rate wanted
  // setup_gnss(gps_types, NUM_GNSS);

  // no return type for GNSS, since no congfiguration (only rx) (for now)
  gnss_start_rx(gps_types, NUM_GNSS);

  return returnVal;
}

static void poll_peripherals() {
  (void)poll_imus(imu_types, NUM_IMUS, imu_data);

  (void)gnss_parse_data_if_available(gps_types, NUM_GNSS, gps_data);
}

void run_sensor_board() {
  HAL_Delay(BOARD_INIT_DELAY_MS);

  (void)setup_peripherals();
  setup_comms();

  // Start task timers
  // TODO: hide task timer configurations between DEFINE opts?
  HAL_TIM_Base_Start_IT(&htim16);
  HAL_TIM_Base_Start_IT(&htim17);

  uint8_t blink_cnt = 0;
  while (1) {
    poll_peripherals();
    if (flag_new_gps) {
      struct uavcan_equipment_gnss_SensorGPS can_gps_pkt;
      can_pack_GpsData(&gps_data[0], &can_gps_pkt);
      (void)can_send_GpsData(can_gps_pkt);
      flag_new_gps = false;
    }

    if (flag_100hz) {
      struct uavcan_equipment_ahrs_SensorIMU can_imu_pkt;
      can_pack_ImuData(&imu_data[0], &can_imu_pkt);
      (void)can_send_ImuData(can_imu_pkt);
      flag_100hz = false;
      if (blink_cnt == 10) {
        HAL_GPIO_TogglePin(GPIO_LED1_GPIO_Port, GPIO_LED1_Pin);
        blink_cnt = 0;
      }
      blink_cnt++;
    }

    loop_comms();
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
  HAL_GPIO_TogglePin(GPIO_LED2_GPIO_Port, GPIO_LED2_Pin);
  if (gnss_process_incoming_data(huart, Size)) {
    flag_new_gps = true;
    return;
  } else {
    // Empty for now, but check other UART messages here
  }
}

void HAL_UART_RxCpltCallback(UART_HandleTypeDef *huart) {
  HAL_GPIO_TogglePin(GPIO_LED1_GPIO_Port, GPIO_LED1_Pin);
}
// Restarts the DMA reception on UART3 whenever a framing error occurs
void HAL_UART_ErrorCallback(UART_HandleTypeDef *huart) {
  if (huart->Instance == USART1) {
    gnss_process_incoming_data(huart, 0);
    gnss_start_rx(gps_types, NUM_GNSS);
  }
}
