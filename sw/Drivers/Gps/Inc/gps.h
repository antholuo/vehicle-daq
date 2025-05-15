/**
 * gps.h
 *
 * Created on: 2025-MAY-12
 *     Author: Anni
 *
 * Contains common GPS structs.
 */

#ifndef GPS_H
#define GPS_H

#include "nmea.h"  // UTC TIME

#include "usart.h"

#include <stdbool.h>
#include <stdint.h>

typedef enum {
    GPS_NOT_AVAIL = 0,
    GPS_NEO_M8N       = 1,
} GpsType_T;

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

bool gnss_process_incoming_data(UART_HandleTypeDef *huart, uint16_t size);

void gnss_start_rx(const GpsType_T * const gps_type, const size_t num_gps);

// returns true if data was parsed
bool gnss_parse_data_if_available(const GpsType_T * const gps_type, const size_t num_gps, GpsData_S *gps_data);

#endif // GPS_H
