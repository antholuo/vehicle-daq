
#include "sensor_bridge.h"

#include <stdint.h>

#include "can_interface.h"
#include "circular_queue.h"

extern IMUReceptionFunc imu_reception_f_ptr;
extern GPSReceptionFunc gps_reception_f_ptr;
extern UART_HandleTypeDef huart2;

/* ------------- IMU DEFINES -------------*/
#define IMU_CQ_CAPACITY 3
#define IMU_CQ_SLOT_SIZE_BYTE 88
/* 2 bytes - Magic Number Header, 1 bytes - CAN ID, 2 bytes - pb length */
#define IMU_RPROTOBUF_HEADER_LENGTH_BYTE 5
#define IMU_PROTOBUF_MAGIC_HEADER_1 0xAA
#define IMU_PROTOBUF_MAGIC_HEADER_2 0x55

/* ------------- GPS DEFINES -------------*/
#define GPS_CQ_CAPACITY 2
#define GPS_CQ_SLOT_SIZE_BYTE 128
/* 2 bytes - Magic Number Header, 1 bytes - CAN ID, 2 bytes - pb length */
#define GPS_RPROTOBUF_HEADER_LENGTH_BYTE 5
#define GPS_PROTOBUF_MAGIC_HEADER_1 0xBB
#define GPS_PROTOBUF_MAGIC_HEADER_2 0x77

/* ------------- IMU GLOBAL VARIABLES -------------*/
static CircularQueue imu_tx_queue;
static uint8_t imu_tx_queue_data_block[IMU_CQ_SLOT_SIZE_BYTE * IMU_CQ_CAPACITY];
static uint16_t imu_tx_queue_data_lengths[IMU_CQ_CAPACITY];

/* ------------- GPS GLOBAL VARIABLES -------------*/
static CircularQueue gps_tx_queue;
static uint8_t gps_tx_queue_data_block[GPS_CQ_SLOT_SIZE_BYTE * GPS_CQ_CAPACITY];
static uint16_t gps_tx_queue_data_lengths[GPS_CQ_CAPACITY];

/* ------------- OTHER STUFF -------------*/
static volatile uint8_t dma_busy;

static uint8_t blink_cnt1 = 0;
static uint8_t blink_cnt2 = 0;

void run_sensor_bridge() {
  can_sensor_reception_setup();
  setup_comms();

  uint32_t cnt = 0;
  while (1) {
    can_sensor_reception_loop();
    if (cnt > 500000) {
      cnt = 0;
      HAL_GPIO_TogglePin(GPIO_LED1_GPIO_Port, GPIO_LED1_Pin);
    }
    cnt++;
  }
}

void can_sensor_reception_setup() {
  (void)cq_init(&imu_tx_queue, imu_tx_queue_data_block,
                imu_tx_queue_data_lengths, IMU_CQ_SLOT_SIZE_BYTE,
                IMU_CQ_CAPACITY);
  (void)cq_init(&gps_tx_queue, gps_tx_queue_data_block,
                gps_tx_queue_data_lengths, GPS_CQ_SLOT_SIZE_BYTE,
                GPS_CQ_CAPACITY);
  imu_reception_f_ptr = can_sensor_bridge_imu_handler;
  gps_reception_f_ptr = can_sensor_bridge_gps_handler;
}

void can_sensor_reception_loop() {
  uint8_t imu_encoded_q[IMU_CQ_SLOT_SIZE_BYTE];
  uint8_t gps_encoded_q[GPS_CQ_SLOT_SIZE_BYTE];
  uint16_t imu_len;
  uint16_t gps_len;
  static uint8_t last_sent_type = 0;  // 0 for IMU, 1 for GPS

  /* Long Round-Robin method for ensuring the fairness between imu and gps */
  if (dma_busy == 0) {
    if (last_sent_type == 0) {
      // Try IMU first
      if (cq_pop(&imu_tx_queue, imu_encoded_q, &imu_len) == CQ_OK) {
        HAL_StatusTypeDef ret =
            HAL_UART_Transmit_DMA(&huart2, imu_encoded_q, imu_len);
        if (ret == HAL_OK) {
          dma_busy = 1;
          last_sent_type = 1;  // Next time try GPS first

          // Debugging feature
          if (blink_cnt2 >= 10) {
            blink_cnt2 = 0;
            HAL_GPIO_TogglePin(GPIO_LED2_GPIO_Port, GPIO_LED2_Pin);
          }
          blink_cnt2++;
          return;
        }
      }
      // Try GPS only if IMU wasn't available
      if (cq_pop(&gps_tx_queue, gps_encoded_q, &gps_len) == CQ_OK) {
        HAL_StatusTypeDef ret =
            HAL_UART_Transmit_DMA(&huart2, gps_encoded_q, gps_len);
        if (ret == HAL_OK) {
          dma_busy = 1;
          last_sent_type = 0;  // Next time try IMU first
          return;
        }
      }
    } else {
      // Try GPS first
      if (cq_pop(&gps_tx_queue, gps_encoded_q, &gps_len) == CQ_OK) {
        HAL_StatusTypeDef ret =
            HAL_UART_Transmit_DMA(&huart2, gps_encoded_q, gps_len);
        if (ret == HAL_OK) {
          dma_busy = 1;
          last_sent_type = 0;  // Next time try IMU first
          return;
        }
      }
      // Try IMU only if GPS wasn't available
      if (cq_pop(&imu_tx_queue, imu_encoded_q, &imu_len) == CQ_OK) {
        HAL_StatusTypeDef ret =
            HAL_UART_Transmit_DMA(&huart2, imu_encoded_q, imu_len);
        if (ret == HAL_OK) {
          dma_busy = 1;
          last_sent_type = 1;  // Next time try GPS first

          // Debugging feature
          if (blink_cnt2 >= 10) {
            blink_cnt2 = 0;
            HAL_GPIO_TogglePin(GPIO_LED2_GPIO_Port, GPIO_LED2_Pin);
          }
          blink_cnt2++;
          return;
        }
      }
    }
  }
}

void can_sensor_bridge_imu_handler(
    const struct uavcan_equipment_ahrs_SensorIMU *can_imu_p, uint8_t can_id) {
  uint8_t encoded[IMU_CQ_SLOT_SIZE_BYTE - IMU_RPROTOBUF_HEADER_LENGTH_BYTE];
  uint8_t encode_w_header[IMU_CQ_SLOT_SIZE_BYTE];

  // preparing for the pb data
  struct imu_data_t *pb_imu_p = imu_data_new(
      encoded, IMU_CQ_SLOT_SIZE_BYTE - IMU_RPROTOBUF_HEADER_LENGTH_BYTE);
  protobuf_pack_ImuData(can_imu_p, pb_imu_p);

  int pb_size =
      imu_data_encode(pb_imu_p, encoded,
                      IMU_CQ_SLOT_SIZE_BYTE - IMU_RPROTOBUF_HEADER_LENGTH_BYTE);

  // magic header for protobuf message beginning detection
  encode_w_header[0] = IMU_PROTOBUF_MAGIC_HEADER_1;
  encode_w_header[1] = IMU_PROTOBUF_MAGIC_HEADER_2;
  encode_w_header[2] = can_id;
  encode_w_header[3] = pb_size & 0xFF;
  encode_w_header[4] = (pb_size >> 8) & 0xFF;
  memcpy(encode_w_header + IMU_RPROTOBUF_HEADER_LENGTH_BYTE, encoded, pb_size);

  // then we add it to the queue, it will be transmit by DMA in main loop
  (void)cq_push(&imu_tx_queue, encode_w_header,
                pb_size + IMU_RPROTOBUF_HEADER_LENGTH_BYTE);

  /* debugging feature, delete if no longer needed */
  if (blink_cnt1 >= 10) {
    blink_cnt1 = 0;
    HAL_GPIO_TogglePin(GPIO_LED1_GPIO_Port, GPIO_LED1_Pin);
  }
  blink_cnt1++;
}

void can_sensor_bridge_gps_handler(
    const struct uavcan_equipment_gnss_SensorGPS *can_gps_p, uint8_t can_id) {
  uint8_t encoded[GPS_CQ_SLOT_SIZE_BYTE - GPS_RPROTOBUF_HEADER_LENGTH_BYTE];
  uint8_t encode_w_header[GPS_CQ_SLOT_SIZE_BYTE];

  // preparing for the pb data
  struct gps_data_t *pb_gps_p = gps_data_new(
      encoded, GPS_CQ_SLOT_SIZE_BYTE - GPS_RPROTOBUF_HEADER_LENGTH_BYTE);
  (void)gps_data_time_alloc(pb_gps_p);
  (void)gps_data_date_alloc(pb_gps_p);
  protobuf_pack_GpsData(can_gps_p, pb_gps_p);

  int pb_size =
      gps_data_encode(pb_gps_p, encoded,
                      GPS_CQ_SLOT_SIZE_BYTE - GPS_RPROTOBUF_HEADER_LENGTH_BYTE);

  // magic header for protobuf message beginning detection
  encode_w_header[0] = GPS_PROTOBUF_MAGIC_HEADER_1;
  encode_w_header[1] = GPS_PROTOBUF_MAGIC_HEADER_2;
  encode_w_header[2] = can_id;
  encode_w_header[3] = pb_size & 0xFF;
  encode_w_header[4] = (pb_size >> 8) & 0xFF;
  memcpy(encode_w_header + GPS_RPROTOBUF_HEADER_LENGTH_BYTE, encoded, pb_size);

  // then we add it to the queue, it will be transmit by DMA in main loop
  (void)cq_push(&gps_tx_queue, encode_w_header,
                pb_size + GPS_RPROTOBUF_HEADER_LENGTH_BYTE);
}

void HAL_UART_TxCpltCallback(UART_HandleTypeDef *huart) {
  if (huart == &huart2) {
    dma_busy = 0;  // ready for next one
  }
}

void HAL_UART_ErrorCallback(UART_HandleTypeDef *huart) {
  if (huart == &huart2) {
    HAL_UART_DMAStop(huart);
    dma_busy = 0;
  }
}