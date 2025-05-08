/*
 * gps.h
 *
 *  Created on: Mar 13, 2025
 *      Author: antho
 */

#ifndef GPS_H_
#define GPS_H_

#include <stdint.h>
#include "usart.h"
#include "nmea.h"

// Size of the internal UART buffer for receiving messages.
// Must be larger than the largest message we can expect to receive over UART
#define GPS_UART_BUFFER_SIZE 800

// TODO: add other fields: HDOP, VDOP, diff age (dgps)
typedef struct {
	NmeaTime_T time_utc;
	int32_t lat_microdeg;	// latitude, microdegrees. Sign indicates direction (+n, -s)
	int32_t lon_microdeg;	// longitude, microdegrees. Sign indicates direction (+e, -w)
	int16_t altitude_m;		// altitude, meters. Sign indicates above/below MSL
	uint8_t speed_kts;		// groundspeed, knots (default RMC speed)
	uint16_t heading_deg;	// in degrees, 0-360
	uint8_t num_sats;		// Number of satellites in view (max 22)
	uint8_t fix_status;		// 0 = No gps, 1 = GPS Fix, 2 = DGSP Fix, 3 = Estimated / Dead Reckoning
} GpsData_T;

void run_gnss_demo();

#endif /* GPS_H_ */
