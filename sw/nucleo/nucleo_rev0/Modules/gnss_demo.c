/*
 * gnss_demo.c
 *
 *  Created on: Mar 1, 2025
 *      Author: antho
 */

#include "gnss.h"

#include "usart.h"

extern UART_HandleTypeDef huartt1; // Where we are receiving the raw GNSS data
extern GNSS_StateHandle GNSS_Handle;

void run_gnss_demo() {

	GNSS_Init(&GNSS_Handle, &huart1);

	HAL_Delay(1000);

	GNSS_LoadConfig(&GNSS_Handle);

	uint32_t Timer = HAL_GetTick();

	while (1) {
		if ((HAL_GetTick() - Timer > 1000)) {
			GNSS_GetUniqID(&GNSS_Handle);
			GNSS_ParseBuffer(&GNSS_Handle);
			HAL_Delay(250);
			GNSS_GetPVTData(&GNSS_Handle);
			GNSS_ParseBuffer(&GNSS_Handle);

			// Ignore the printfs for now.
//			snprintf("Day: %d-%d-%d \r\n", GNSS_Handle.day, GNSS_Handle.month,
//					GNSS_Handle.year);
//			snprintf("Time: %d:%d:%d \r\n", GNSS_Handle.hour, GNSS_Handle.min,
//					GNSS_Handle.sec);
//			snprintf("Status of fix: %d \r\n", GNSS_Handle.fixType);
////		printf("Latitude: %f \r\n", GNSS_Handle.fLat);
////		printf("Longitude: %f \r\n", (float) GNSS_Handle.lon / 10000000.0);
//			snprintf("Height above ellipsoid: %d \r\n", GNSS_Handle.height);
//			snprintf("Height above mean sea level: %d \r\n", GNSS_Handle.hMSL);
//			printf("Ground Speed (2-D): %d \r\n", GNSS_Handle.gSpeed);
//			printf("Unique ID: %04X %04X %04X %04X %04X \n\r",
//					GNSS_Handle.uniqueID[0], GNSS_Handle.uniqueID[1],
//					GNSS_Handle.uniqueID[2], GNSS_Handle.uniqueID[3],
//					GNSS_Handle.uniqueID[4], GNSS_Handle.uniqueID[5]);
			// We can break on this point to view the gps data.
			Timer = HAL_GetTick();
		}
	}
}
