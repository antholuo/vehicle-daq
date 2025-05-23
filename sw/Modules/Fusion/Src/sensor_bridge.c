
#include "sensor_bridge.h"
#include "can_interface.h"
#include "circular_queue.h"

#include <stdint.h>

extern IMUReceptionFunc     imu_reception_f_ptr;
extern UART_HandleTypeDef   huart2;

#define IMU_CQ_CAPACITY 3
#define IMU_CQ_SLOT_SIZE_BYTE 72
/* 2 bytes - Magic Number Header, 1 bytes - CAN ID, 2 bytes - pb length */
#define IMU_RPROTOBUF_HEADER_LENGTH_BYTE 5
#define IMU_PROTOBUF_MAGIC_HEADER_1     0xAA
#define IMU_PROTOBUF_MAGIC_HEADER_2     0x55

static CircularQueue imu_tx_queue;
static uint8_t imu_tx_queue_data_block[IMU_CQ_SLOT_SIZE_BYTE * IMU_CQ_CAPACITY];
static uint16_t imu_tx_queue_data_lengths[IMU_CQ_CAPACITY];

static uint8_t dma_busy;

static uint8_t blink_cnt1 = 0;
static uint8_t blink_cnt2 = 0;

void run_sensor_bridge(){
    can_sensor_reception_setup();
    setup_comms();

    while(1){
        can_sensor_reception_loop();
    }
}

void can_sensor_reception_setup(){
    (void)cq_init(&imu_tx_queue,
            imu_tx_queue_data_block,
            imu_tx_queue_data_lengths,
            IMU_CQ_SLOT_SIZE_BYTE,
            IMU_CQ_CAPACITY);
	imu_reception_f_ptr = can_sensor_bridge_imu_handler;
}

void can_sensor_reception_loop(){
    uint8_t imu_encoded_q[IMU_CQ_SLOT_SIZE_BYTE];
	uint16_t len;
	if (dma_busy == 0) {
		if (cq_pop(&imu_tx_queue, imu_encoded_q, &len) == CQ_OK){
			HAL_StatusTypeDef ret = HAL_UART_Transmit_DMA(&huart2, imu_encoded_q, len);
			if (ret == HAL_OK){
				dma_busy = 1;

				/* debugging feature, delete if no longer needed */
				if (blink_cnt2 >= 10){
					blink_cnt2 = 0;
					HAL_GPIO_TogglePin(GPIO_LED2_GPIO_Port, GPIO_LED2_Pin);
				}
				blink_cnt2++;
			}
		}
	}
}

void can_sensor_bridge_imu_handler(const struct uavcan_equipment_ahrs_SensorIMU* can_imu_p, uint8_t can_id){
    uint8_t encoded[IMU_CQ_SLOT_SIZE_BYTE - IMU_RPROTOBUF_HEADER_LENGTH_BYTE];
    uint8_t encode_w_header[IMU_CQ_SLOT_SIZE_BYTE];

    // preparing for the pb data
    struct imu_data_t *pb_imu_p = imu_data_new(encoded, IMU_CQ_SLOT_SIZE_BYTE - IMU_RPROTOBUF_HEADER_LENGTH_BYTE);
	protobuf_pack_ImuData(can_imu_p, pb_imu_p);

	int pb_size = imu_data_encode(pb_imu_p, encoded, IMU_CQ_SLOT_SIZE_BYTE - IMU_RPROTOBUF_HEADER_LENGTH_BYTE);
	
	// magic header for protobuf message beginning detection
	encode_w_header[0] = IMU_PROTOBUF_MAGIC_HEADER_1;
	encode_w_header[1] = IMU_PROTOBUF_MAGIC_HEADER_2;
    encode_w_header[2] = can_id;
	encode_w_header[3] = pb_size & 0xFF;
	encode_w_header[4] = (pb_size >> 8) & 0xFF;
	memcpy(encode_w_header + IMU_RPROTOBUF_HEADER_LENGTH_BYTE, encoded, pb_size);

	// then we add it to the queue, it will be transmit by DMA in main loop
	(void)cq_push(&imu_tx_queue, encode_w_header, pb_size + 4);

	/* debugging feature, delete if no longer needed */
	if (blink_cnt1 >= 10){
		blink_cnt1 = 0;
		HAL_GPIO_TogglePin(GPIO_LED1_GPIO_Port, GPIO_LED1_Pin);
	}
	blink_cnt1++;

}

void HAL_UART_TxCpltCallback(UART_HandleTypeDef *huart) {
    if (huart == &huart2) {
        dma_busy = 0; // ready for next one
    }
}