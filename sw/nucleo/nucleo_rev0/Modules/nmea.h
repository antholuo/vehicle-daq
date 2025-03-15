/*
 * nmea.h
 *
 *  Created on: Mar 2, 2025
 *      Author: antho
 *
 *  Library to parse a buffer of NMEA messges.
 */

#ifndef NMEA_H_
#define NMEA_H_

#include <stdint.h>
#include <stdbool.h>

#define MAX_SATELLITES 12	// maximum number of satellites the gps receiver can handle

// ENUM of possible NMEA messages
// Lightly derived from UBX-13003221
enum NmeaSentenceType_E {
	NMEA_NOT_FOUND = 1,		// No sentence found
	NMEA_UNKNOWN,           // Unsupported sentence found
	NMEA_INVALID,           // Sentence validation failed
	// Mixed GPS + GLONASS sentences
	NMEA_xxGGA,				// GGA = Global Positioning system fix data
	NMEA_xxGLL,				// GLL = Latitude & Longitude, with time of position fix & status
	NMEA_xxGNS,				// GNS = GNSS fix data, incl. Time, Pos, Num satellites, HDOP, age of differential data, etc.
	NMEA_xxGSA,				// GSA = GNSS DOP & Active Satellites (One set per GNSS system)
	NMEA_xxGSV,				// GSV = GNSS Satellites in View (One set per GNSS system)
	NMEA_xxRMC,				// RMC = Recommended Minimum data
	NMEA_xxTHS,				// THS = True Heading Status -- we probably don't have this since we don't hve a compass
	NMEA_xxVTG,				// VTG = Course over ground & ground speed.
	NMEA_xxZDA,				// ZDA = time and date (UTC)
};

typedef struct {
	uint8_t *start;		// pointer to first byte of data
	uint8_t *data;		// pointer to first term
	uint8_t *end;		// pointer to last byte
	uint8_t type;		// Sentence type
} NmeaSentence_T;

typedef struct {
	uint8_t hours;
	uint8_t minutes;
	uint8_t seconds;
} NmeaTime_T;

typedef struct {
	uint16_t year;
	uint8_t month;
	uint8_t day;
} NmeaDate_T;

// TODO: rest of the structs. We're going to bring this on ONE AT A TIME!!!! slow and steady...
typedef struct {
	NmeaTime_T time_utc;			// UTC Time
	int32_t lat_microdeg;		// latitude in microdegrees
	uint8_t lat_char;			// latitude direction indicator (N/S)
	int32_t lon_microdeg;		// longitude in microdegrees
	uint8_t lon_char;			// longitude indicator (E/W)
	uint8_t quality;			// 0 = no fix, 1 = autonomous Gnss, 2 = differential gnss, 4 = rtk fied, 5 = rtk float, 6 = estimated/dead reckoning
	uint8_t num_sats;			// Number of satellites used (0-12)
	uint32_t hdop;				// Horizontal Dilution of Precision
	int32_t altitude_m;			// Altitude above MSL
	uint8_t alt_unit;			// M, meters, altitude unit
	int32_t geoid_sep;			// Geoid separation (wtf)
	uint8_t sep_char;			// M, meters, geoid separation unit
	uint8_t diff_age;			// Age of differential corrections (Null when not using DGPS)
	uint8_t diff_station;		// ID of station providing differential corrections (null when not used)
	uint8_t checksum;			// Checksum.
} GgaData_T;

// Struct of recommended minimum data...
typedef struct {
	NmeaTime_T time_utc;		// UTC Time of fix
	bool data_valid;			// *Technically a character for data validity, but we only care about t/f
	int32_t lat_microdeg;		// latitude in microdegrees
	uint8_t lat_char;			// latitude direction indicator (N/S)
	int32_t lon_microdeg;		// longitude in microdegrees
	uint8_t lon_char;			// longitude direction indicator (E/W)
	uint32_t speed_mkts;		// Speed in (micro) knots?
	uint32_t course_deg;		// Track angle relative to north (degrees)
	NmeaDate_T utc_date;		// UTC Date
	// Ignoring magnetic variation
	uint8_t pos_mode;			// Position mode. A=autonomous, D=differential, E=estimated, R=coarse, S=simulator, N=not valid
} RmcData_T;

#endif /* NMEA_H_ */
