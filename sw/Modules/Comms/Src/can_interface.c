/*
 * can_interface.c
 *
 *  Created on: Mar 5, 2025
 *      Author: Hardy Yu
 */

#include "can_interface.h"
#include "string.h"

/* ATTENTION: NODE ID needs to be hard-coded differently between boards! */
/* TODO: Configurable NODE ID through build system */
#define NODE_ID 0x20


/* the master hal can handler*/
extern CAN_HandleTypeDef hcan;

/* global canard instance */
CanardInstance canard;

/* global node status info */
static struct uavcan_protocol_NodeStatus node_status;

/* canard memory pool */
#define CANARD_MEMOERY_POOL_BYTES 1024
static uint8_t memory_pool[CANARD_MEMOERY_POOL_BYTES];

/* an variable that keeps track of time */
static uint64_t next_1hz_service_at;

/* this is the functions that goes before the while(1) */
void setup_comms(void){
	/* Setup can filter */
	can_set_filter();
	/* Start can bus */
	HAL_CAN_Start(&hcan);
	/* Activate can rx call back */
#ifndef SENSOR_BOARD_CAN_RECEPTION_DISABLE
	HAL_CAN_ActivateNotification(&hcan, CAN_IT_RX_FIFO0_MSG_PENDING);
#endif

	/* Initialize canard library */
	canardInit(&canard, memory_pool, sizeof(memory_pool),
				onTransferReceived, shouldAcceptTransfer, NULL);

	/* Hardcodinig a node id for this can node */
	canard.node_id = NODE_ID;

	/* set a static variable to the current tick */
	next_1hz_service_at = HAL_GetTick();
}

/* this is the function that goes inside the while(1) */
void loop_comms(void){
	processCanardTxQueue(&hcan);

	const uint64_t ts = HAL_GetTick();

	if (ts >= next_1hz_service_at){
		next_1hz_service_at += 1000ULL;
		/* process 1 Hz Tasks */
		canardCleanupStaleTransfers(&canard, ts);
		send_NodeStatus();

		/* toggling an led light, could be commented*/
		HAL_GPIO_TogglePin(GPIO_LED2_GPIO_Port, GPIO_LED2_Pin);
	}
}

void HAL_CAN_RxFifo0MsgPendingCallback(CAN_HandleTypeDef *hcan) {
	// Receiving
	CanardCANFrame rx_frame;

	const uint64_t timestamp = HAL_GetTick() * 1000ULL;
	const int16_t rx_res = canardSTM32Recieve(hcan, CAN_RX_FIFO0, &rx_frame);

	if (rx_res < 0) {
		// printf("Receive error %d\n", rx_res);
		// error handling
	}
	else if (rx_res > 0)        // Success - process the frame
	{
		canardHandleRxFrame(&canard, &rx_frame, timestamp);
	}
}

void processCanardTxQueue(CAN_HandleTypeDef *hcan) {
	// Transmitting

	for (const CanardCANFrame *tx_frame ; (tx_frame = canardPeekTxQueue(&canard)) != NULL;) {
		const int16_t tx_res = canardSTM32Transmit(hcan, tx_frame);

		if (tx_res <= 0) {
			// try again later
		} else if (tx_res > 0) {
			// printf("Successfully transmitted message\n");
			canardPopTxQueue(&canard);
		} else {
			canardPopTxQueue(&canard);
		}
	}
}


int16_t canardSTM32Recieve(CAN_HandleTypeDef *hcan, uint32_t RxLocation, CanardCANFrame *const rx_frame) {
	if (rx_frame == NULL) {
		return -CANARD_ERROR_INVALID_ARGUMENT;
	}

	CAN_RxHeaderTypeDef RxHeader;
	uint8_t RxData[8];

	if (HAL_CAN_GetRxMessage(hcan, RxLocation, &RxHeader, RxData) == HAL_OK) {

		// Process ID to canard format
		if (RxHeader.IDE == CAN_ID_EXT) { // canard will only process the message if it is extended ID
			rx_frame->id = RxHeader.ExtId;
			rx_frame->id |= CANARD_CAN_FRAME_EFF;
		} else {
			rx_frame->id = RxHeader.StdId;
		}

		if (RxHeader.RTR == CAN_RTR_REMOTE) { // canard won't process the message if it is a remote frame
			rx_frame->id |= CANARD_CAN_FRAME_RTR;
		}

		rx_frame->data_len = RxHeader.DLC;
		memcpy(rx_frame->data, RxData, RxHeader.DLC);

		// assume a single interface
		rx_frame->iface_id = 0;

		return 1;
	}

	// Either no CAN msg to be read, or an error that can be read from hfdcan->ErrorCode
	return 0;
}

int16_t canardSTM32Transmit(CAN_HandleTypeDef *hcan, const CanardCANFrame* const tx_frame) {
	if (tx_frame == NULL) {
		return -CANARD_ERROR_INVALID_ARGUMENT;
	}

	if (tx_frame->id & CANARD_CAN_FRAME_ERR) {
		return -CANARD_ERROR_INVALID_ARGUMENT; // unsupported frame format
	}

	CAN_TxHeaderTypeDef TxHeader;
	uint8_t TxData[8];
	uint32_t TxMailbox;

	// Process canard id to STM FDCAN header format
	if (tx_frame->id & CANARD_CAN_FRAME_EFF) {
		TxHeader.IDE = CAN_ID_EXT;
		TxHeader.ExtId = tx_frame->id & CANARD_CAN_EXT_ID_MASK;
	} else {
		TxHeader.IDE = CAN_ID_STD;
		TxHeader.StdId = tx_frame->id & CANARD_CAN_STD_ID_MASK;
	}

	TxHeader.DLC = tx_frame->data_len;

	if (tx_frame->id & CANARD_CAN_FRAME_RTR) {
		TxHeader.RTR = CAN_RTR_REMOTE;
	} else {
		TxHeader.RTR = CAN_RTR_DATA;
	}

	TxHeader.TransmitGlobalTime = DISABLE;
	memcpy(TxData, tx_frame->data, TxHeader.DLC);

	// checking mailbox availability before adding message
	if ((CAN->TSR & CAN_TSR_TME0) || (CAN->TSR & CAN_TSR_TME1) || (CAN->TSR & CAN_TSR_TME2)) {
		if (HAL_CAN_AddTxMessage(hcan, &TxHeader, TxData, &TxMailbox) == HAL_OK) {
			return 1;
		}
	}
	return 0;
}



/* CANARD Util: a software can filter on which message to handle */
bool shouldAcceptTransfer(const CanardInstance *ins,
                                 uint64_t *out_data_type_signature,
                                 uint16_t data_type_id,
                                 CanardTransferType transfer_type,
                                 uint8_t source_node_id)
{
	if (transfer_type == CanardTransferTypeRequest) {
	// check if we want to handle a specific service request
		switch (data_type_id) {
		case UAVCAN_PROTOCOL_GETNODEINFO_ID: {
			*out_data_type_signature = UAVCAN_PROTOCOL_GETNODEINFO_REQUEST_SIGNATURE;
			return true;
		}
		}
	}
	if (transfer_type == CanardTransferTypeResponse) {
		// check if we want to handle a specific service request
		switch (data_type_id) {
		}
	}
	if (transfer_type == CanardTransferTypeBroadcast) {
		// see if we want to handle a specific broadcast packet
		switch (data_type_id) {
		case UAVCAN_EQUIPMENT_AHRS_SENSORIMU_ID: {
			*out_data_type_signature = UAVCAN_EQUIPMENT_AHRS_SENSORIMU_SIGNATURE;
			return true;
		}
		case UAVCAN_PROTOCOL_NODESTATUS_ID: {
			*out_data_type_signature = UAVCAN_PROTOCOL_NODESTATUS_SIGNATURE;
			return true;
		}
		}
	}
	// we don't want any other messages
	return false;
}

/* CANARD Util: CAN message handle coordinator */
void onTransferReceived(CanardInstance *ins, CanardRxTransfer *transfer) {
	// switch on data type ID to pass to the right handler function
	if (transfer->transfer_type == CanardTransferTypeRequest) {
		// check if we want to handle a specific service request
		switch (transfer->data_type_id) {
		case UAVCAN_PROTOCOL_GETNODEINFO_ID: {
			handle_GetNodeInfo(ins, transfer);
			break;
		}
		}
	}
	if (transfer->transfer_type == CanardTransferTypeResponse) {
		switch (transfer->data_type_id) {
			// we are not supporting this yet
		}
	}
	if (transfer->transfer_type == CanardTransferTypeBroadcast) {
		// check if we want to handle a specific broadcast message
		switch (transfer->data_type_id) {
		case UAVCAN_EQUIPMENT_AHRS_SENSORIMU_ID:{
			can_receive_ImuData(ins, transfer);
			break;
		}
		case UAVCAN_PROTOCOL_NODESTATUS_ID: {
			handle_NodeStatus(ins, transfer);
			break;
		}
		}
	}
}


void can_set_filter(void){
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


/*
  handle a GetNodeInfo request
*/
// TODO: All the data in here is temporary for testing. If actually need to send valid data, edit accordingly.
void handle_GetNodeInfo(CanardInstance *ins, CanardRxTransfer *transfer) {
	uint8_t buffer[UAVCAN_PROTOCOL_GETNODEINFO_RESPONSE_MAX_SIZE];
	struct uavcan_protocol_GetNodeInfoResponse pkt;

	memset(&pkt, 0, sizeof(pkt));

	node_status.uptime_sec = HAL_GetTick() / 1000ULL;
	pkt.status = node_status;

	// fill in your major and minor firmware version
	pkt.software_version.major = 1;
	pkt.software_version.minor = 0;
	pkt.software_version.optional_field_flags = 0;
	pkt.software_version.vcs_commit = 0; // should put git hash in here

	// should fill in hardware version
	pkt.hardware_version.major = 1;
	pkt.hardware_version.minor = 0;

	// just setting all 16 bytes to 1 for testing
	getUniqueID(pkt.hardware_version.unique_id);

	strncpy((char*)pkt.name.data, "SERVONode", sizeof(pkt.name.data));
	pkt.name.len = strnlen((char*)pkt.name.data, sizeof(pkt.name.data));

	uint16_t total_size = uavcan_protocol_GetNodeInfoResponse_encode(&pkt, buffer);

	canardRequestOrRespond(ins,
						   transfer->source_node_id,
						   UAVCAN_PROTOCOL_GETNODEINFO_SIGNATURE,
						   UAVCAN_PROTOCOL_GETNODEINFO_ID,
						   &transfer->transfer_id,
						   transfer->priority,
						   CanardResponse,
						   &buffer[0],
						   total_size);
}

/* basically, empty function... */
void handle_NodeStatus(CanardInstance *ins, CanardRxTransfer *transfer) {
	struct uavcan_protocol_NodeStatus nodeStatus;

	if (uavcan_protocol_NodeStatus_decode(transfer, &nodeStatus)) {
		return;
	}

	switch (nodeStatus.health) {
	case UAVCAN_PROTOCOL_NODESTATUS_HEALTH_OK:
		break;
	case UAVCAN_PROTOCOL_NODESTATUS_HEALTH_WARNING:
		break;
	case UAVCAN_PROTOCOL_NODESTATUS_HEALTH_ERROR:
		break;
	case UAVCAN_PROTOCOL_NODESTATUS_HEALTH_CRITICAL:
		break;
	default:
		break;
	}

	switch(nodeStatus.mode) {
	case UAVCAN_PROTOCOL_NODESTATUS_MODE_OPERATIONAL:
		break;
	case UAVCAN_PROTOCOL_NODESTATUS_MODE_INITIALIZATION:
		break;
	case UAVCAN_PROTOCOL_NODESTATUS_MODE_MAINTENANCE:
		break;
	case UAVCAN_PROTOCOL_NODESTATUS_MODE_SOFTWARE_UPDATE:
		break;
	case UAVCAN_PROTOCOL_NODESTATUS_MODE_OFFLINE:
		break;
	default:
		break;
	}
}

/*
  send the 1Hz NodeStatus message. This is what allows a node to show
  up in the DroneCAN GUI tool and in the flight controller logs
 */
void send_NodeStatus(void) {
    uint8_t buffer[UAVCAN_PROTOCOL_GETNODEINFO_RESPONSE_MAX_SIZE];

    node_status.uptime_sec = HAL_GetTick() / 1000UL;
    node_status.health = UAVCAN_PROTOCOL_NODESTATUS_HEALTH_OK;
    node_status.mode = UAVCAN_PROTOCOL_NODESTATUS_MODE_OPERATIONAL;
    node_status.sub_mode = 0;

    // put whatever you like in here for display in GUI
    node_status.vendor_specific_status_code = 1234;

    uint32_t len = uavcan_protocol_NodeStatus_encode(&node_status, buffer);

    // we need a static variable for the transfer ID. This is
    // incremeneted on each transfer, allowing for detection of packet
    // loss
    static uint8_t transfer_id;

    canardBroadcast(&canard,
                    UAVCAN_PROTOCOL_NODESTATUS_SIGNATURE,
                    UAVCAN_PROTOCOL_NODESTATUS_ID,
                    &transfer_id,
                    CANARD_TRANSFER_PRIORITY_LOW,
                    buffer,
                    len);
}



/*
  get a 16 byte unique ID for this node, this should be based on the CPU unique ID or other unique ID
 */
void getUniqueID(uint8_t id[16]){
	uint32_t HALUniqueIDs[3];
	// Make Unique ID out of the 96-bit STM32 UID and fill the rest with 0s
	memset(id, 0, 16);
	HALUniqueIDs[0] = HAL_GetUIDw0();
	HALUniqueIDs[1] = HAL_GetUIDw1();
	HALUniqueIDs[2] = HAL_GetUIDw2();
	memcpy(id, HALUniqueIDs, 12);
}

