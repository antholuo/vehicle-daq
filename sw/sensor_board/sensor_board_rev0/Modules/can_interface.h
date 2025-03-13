/*
 * can_interface.h
 *
 *  Created on: Mar 5, 2025
 *      Author: Hardy Yu
 */

#ifndef CAN_INTERFACE_H_
#define CAN_INTERFACE_H_

#include "can.h"

#ifdef __cplusplus
extern "C" {
#endif

/* start the can bus and transmit/receive routine */
void can_start_basic(void);

/**
  * @author Roni Kant
  * @brief  Process CAN message from RxLocation FIFO into rx_frame
  * @param  hcan pointer to an CAN_HandleTypeDef structure that contains
  *         the configuration information for the specified FDCAN.
  * @param  RxLocation Location of the received message to be read.
  *         This parameter can be a value of @arg CAN_receive_FIFO_number.
  * @param  rx_frame pointer to a CanardCANFrame structure where the received CAN message will be
  * 		stored.
  * @retval ret == 1: OK, ret < 0: CANARD_ERROR, ret == 0: Check hcan->ErrorCode
  */
// int16_t canardSTM32Recieve(CAN_HandleTypeDef *hcan, uint32_t RxLocation, CanardCANFrame *const rx_frame);

/**
  * @author Roni Kant
  * @brief  Process tx_frame CAN message into a Tx mailbox and transmit it
  * @param  hcan pointer to an CAN_HandleTypeDef structure that contains
  *         the configuration information for the specified FDCAN.
  * @param  tx_frame pointer to a CanardCANFrame structure that contains the CAN message to
  * 		transmit.
  * @retval ret == 1: OK, ret < 0: CANARD_ERROR, ret == 0: Check hcan->ErrorCode
  */
// int16_t canardSTM32Transmit(CAN_HandleTypeDef *hcan, const CanardCANFrame* const tx_frame);

#ifdef __cplusplus
}
#endif

#endif /* CAN_INTERFACE_H_ */
