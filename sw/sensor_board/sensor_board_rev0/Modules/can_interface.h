/*
 * can_interface.h
 *
 *  Created on: Mar 5, 2025
 *      Author: Hardy Yu
 */

#ifndef CAN_INTERFACE_H_
#define CAN_INTERFACE_H_

#include "can.h"
#include "canard.h"
#include "dronecan_msgs.h"
#include "can_sensors.h"

#ifdef __cplusplus
extern "C" {
#endif

/* start the can bus and transmit/receive routine */
void can_start_basic(void);

/* the main can loop that handles transmission and initiate receive callback */
void can_main(void);

/* this is the functions that goes before the while(1) */
void can_main_setup(void);

/* this is the function that goes inside the while(1) */
void can_main_loop(void);

/* setup the can filter config */
void can_set_filter(void);

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
int16_t canardSTM32Recieve(CAN_HandleTypeDef *hcan, uint32_t RxLocation, CanardCANFrame *const rx_frame);

/**
  * @author Roni Kant
  * @brief  Process tx_frame CAN message into a Tx mailbox and transmit it
  * @param  hcan pointer to an CAN_HandleTypeDef structure that contains
  *         the configuration information for the specified FDCAN.
  * @param  tx_frame pointer to a CanardCANFrame structure that contains the CAN message to
  * 		transmit.
  * @retval ret == 1: OK, ret < 0: CANARD_ERROR, ret == 0: Check hcan->ErrorCode
  */
int16_t canardSTM32Transmit(CAN_HandleTypeDef *hcan, const CanardCANFrame* const tx_frame);

/* handling get node info request from other can node */
void handle_GetNodeInfo(CanardInstance *ins, CanardRxTransfer *transfer);

/* handling the node state notification from other can node */
void handle_NotifyState(CanardInstance *ins, CanardRxTransfer *transfer);

/* handling node status info from other can node */
void handle_NodeStatus(CanardInstance *ins, CanardRxTransfer *transfer);

/* handling raw imu data received from other can node */
void handle_RawIMU(CanardInstance *ins, CanardRxTransfer *transfer);

/* broadcast this node's status on can bus */
void send_NodeStatus(void);

/* CANARD Util: a software can filter on which message to handle */
bool shouldAcceptTransfer(const CanardInstance *ins,
                          uint64_t *out_data_type_signature,
                          uint16_t data_type_id,
                          CanardTransferType transfer_type,
                          uint8_t source_node_id);

/* CANARD Util: CAN message handle coordinator */
void onTransferReceived(CanardInstance *ins, CanardRxTransfer *transfer);

/* transmit tx data in the mailbox when CAN bus is clear */
void processCanardTxQueue(CAN_HandleTypeDef *hcan);

/*
  get a 16 byte unique ID for this node, this should be based on the CPU unique ID or other unique ID
 */
void getUniqueID(uint8_t id[16]);

#ifdef __cplusplus
}
#endif

#endif /* CAN_INTERFACE_H_ */
