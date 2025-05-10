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

typedef struct __attribute__((packed)) {
    NmeaTime_T time_utc;
    int32_t lat_microdeg;   // latitude, microdegrees. Sign indicates direction (+n, -s)
    int32_t lon_microdeg;   // longitude, microdegrees. Sign indicates direction (+e, -w)
    int16_t altitude_m;     // altitude, meters. Sign indicates above/below MSL
    uint8_t speed_kts;      // groundspeed, knots (default RMC speed)
    uint16_t heading_deg;   // in degrees, 0-360
    uint8_t num_sats;       // Number of satellites in view (max 22)
    uint8_t fix_status;     // 0 = No gps, 1 = GPS Fix, 2 = DGSP Fix, 3 = Estimated / Dead Reckoning
    bool data_valid;
} gpsData_S;

#endif // GPS_H
