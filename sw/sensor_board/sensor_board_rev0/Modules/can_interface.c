/*
 * can_interface.h
 *
 *  Created on: Mar 5, 2025
 *      Author: Hardy Yu
 */

#include "can_interface.h"
#include "string.h"
#include "can.h"
#include "core_armv8mml.h"

extern CAN_HandleTypeDef hcan;
static CAN_RxHeaderTypeDef rxHeader; //CAN Bus Transmit Header
static uint8_t canRX[8] = {0,0,0,0,0,0,0,0};  //CAN Bus Receive Buffer

void can_start_basic(void){


	CAN_TxHeaderTypeDef txHeader; //CAN Bus Receive Header
	uint32_t canMailbox; //CAN Bus Mail box variable
	txHeader.DLC = 8;
	txHeader.IDE = CAN_ID_STD;
	txHeader.RTR = CAN_RTR_DATA;
	txHeader.StdId = 0x030;
	txHeader.ExtId = 0x02;
	txHeader.TransmitGlobalTime = DISABLE;

	can_set_filter();
	HAL_CAN_Start(&hcan);
	HAL_CAN_ActivateNotification(&hcan, CAN_IT_RX_FIFO0_MSG_PENDING);

	while (1)
	{
		/* USER CODE END WHILE */

		/* USER CODE BEGIN 3 */
		uint8_t csend[] = {0x01,0x02,0x03,0x04,0x05,0x06,0x07,0x08};
		HAL_StatusTypeDef ret = HAL_CAN_AddTxMessage(&hcan,&txHeader,csend,&canMailbox);

		HAL_GPIO_TogglePin(GPIOB, GPIO_PIN_5);
		printf("Hardy\n");

		HAL_Delay(100);

	}

}

void HAL_CAN_RxFifo0MsgPendingCallback(CAN_HandleTypeDef *hcan1)
{
	HAL_CAN_GetRxMessage(&hcan, CAN_RX_FIFO0, &rxHeader, canRX);

	// do something specifically works for the board you are testing
	HAL_GPIO_TogglePin(GPIOB, GPIO_PIN_4);
}


//int16_t canardSTM32Recieve(CAN_HandleTypeDef *hcan, uint32_t RxLocation, CanardCANFrame *const rx_frame) {
//	if (rx_frame == NULL) {
//		return -CANARD_ERROR_INVALID_ARGUMENT;
//	}
//
//	CAN_RxHeaderTypeDef RxHeader;
//	uint8_t RxData[8];
//
//	if (HAL_CAN_GetRxMessage(hcan, RxLocation, &RxHeader, RxData) == HAL_OK) {
//
//		// Process ID to canard format
//		if (RxHeader.IDE == CAN_ID_EXT) { // canard will only process the message if it is extended ID
//			rx_frame->id = RxHeader.ExtId;
//			rx_frame->id |= CANARD_CAN_FRAME_EFF;
//		} else {
//			rx_frame->id = RxHeader.StdId;
//		}
//
//		if (RxHeader.RTR == CAN_RTR_REMOTE) { // canard won't process the message if it is a remote frame
//			rx_frame->id |= CANARD_CAN_FRAME_RTR;
//		}
//
//		rx_frame->data_len = RxHeader.DLC;
//		memcpy(rx_frame->data, RxData, RxHeader.DLC);
//
//		// assume a single interface
//		rx_frame->iface_id = 0;
//
//		return 1;
//	}
//
//	// Either no CAN msg to be read, or an error that can be read from hfdcan->ErrorCode
//	return 0;
//}
//
//int16_t canardSTM32Transmit(CAN_HandleTypeDef *hcan, const CanardCANFrame* const tx_frame) {
//	if (tx_frame == NULL) {
//		return -CANARD_ERROR_INVALID_ARGUMENT;
//	}
//
//	if (tx_frame->id & CANARD_CAN_FRAME_ERR) {
//		return -CANARD_ERROR_INVALID_ARGUMENT; // unsupported frame format
//	}
//
//	CAN_TxHeaderTypeDef TxHeader;
//	uint8_t TxData[8];
//	uint32_t TxMailbox;
//
//	// Process canard id to STM FDCAN header format
//	if (tx_frame->id & CANARD_CAN_FRAME_EFF) {
//		TxHeader.IDE = CAN_ID_EXT;
//		TxHeader.ExtId = tx_frame->id & CANARD_CAN_EXT_ID_MASK;
//	} else {
//		TxHeader.IDE = CAN_ID_STD;
//		TxHeader.StdId = tx_frame->id & CANARD_CAN_STD_ID_MASK;
//	}
//
//	TxHeader.DLC = tx_frame->data_len;
//
//	if (tx_frame->id & CANARD_CAN_FRAME_RTR) {
//		TxHeader.RTR = CAN_RTR_REMOTE;
//	} else {
//		TxHeader.RTR = CAN_RTR_DATA;
//	}
//
//	TxHeader.TransmitGlobalTime = DISABLE;
//	memcpy(TxData, tx_frame->data, TxHeader.DLC);
//
//	if (HAL_CAN_AddTxMessage(hcan, &TxHeader, TxData, &TxMailbox) == HAL_OK) {
//		return 1;
//	}
//
//	return 0;
//}


inline void can_set_filter(void){
    CAN_FilterTypeDef filter;
    filter.FilterBank = 0;
	filter.FilterMode = CAN_FILTERMODE_IDMASK;
	filter.FilterFIFOAssignment = CAN_RX_FIFO0;
	filter.FilterIdHigh = 0;
	filter.FilterIdLow = 0;
	filter.FilterMaskIdHigh = 0;
	filter.FilterMaskIdLow = 0;
	filter.FilterScale = CAN_FILTERSCALE_32BIT;
	filter.FilterActivation = ENABLE;
	filter.SlaveStartFilterBank = 14;

	HAL_CAN_ConfigFilter(&hcan, &filter);
}

int _write(int file, char *ptr, int len)
{
 (void)file;
 int DataIdx;

 for (DataIdx = 0; DataIdx < len; DataIdx++)
 {
   ITM_SendChar(*ptr++);
 }
 return len;
}
