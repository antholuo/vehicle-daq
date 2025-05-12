/**
 * gps.h
 *
 * Created on: 2025-MAY-10
 *     Author: Anni
 *   Modified: 2025-MAY-10
 *
 * Contains common GPS functionality & Data structures
 */

#ifndef GPS_H
#define GPS_H

#include "nmea.h"

#include <stdbool.h>
#include <stdint.h>

// This is quite large, but the size of the internal UART buffer for receiving messages
// must be larger than the largest message we can expect to receieve over UART
#define GPS_UART_BUFFER_SIZE (800);

typedef struct {
    NmeaTime_T time_utc;
    int32_t lat_microdeg;   // latitude, microdegrees. Sign indicates direction (+n, -s)
    int32_t lon_microdeg;   // longitude, microdegrees. Sign indicates direction (+e, -w)
    int16_t altitude_mm;    // altitude, millimeters. Sign indicates above/below MSL
    uint8_t speed_mkts;     // groundspeed, microknots (default RMC speed)
    uint16_t course_deg;    // Track, in degrees, 0-360
    uint16_t heading_deg;   // Heading, in degrees.
    bool heading_valid;     // Only if GPS has MAG
    uint8_t num_sats;       // Number of satellites in view (max 22)
    uint8_t fix_status;     // 0 = No gps, 1 = GPS Fix, 2 = DGSP Fix, 3 = Estimated / Dead Reckoning
    bool data_valid;
} GpsData_S;

void start_gnss_rx();

bool new_gnss_data_available();

void parse_gnss_data(GpsData_S *gps_data);

void process_incoming_gnss_data(uint16_t size);

#endif // GPS_H
