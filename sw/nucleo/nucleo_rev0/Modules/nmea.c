/*
 * nmea.c
 *
 *  Created on: Mar 2, 2025
 *      Author: antho
 *
 *  Library to parse a buffer of NMEA messages.
 *
 *  Format of messages taken from UBX-13003221 - R28
 */

// C Libs
#include "stdint.h"

// STM Files

// Our own files
#include "nmea.h"

/////////////////////////////
// Buffer variables!
/////////////////////////////
RmcData_T data_RMC;
GgaData_T data_GGA;

/////////////////////////////
// Helper functions
/////////////////////////////

// Parse a number in data buffer with specified length
// input:
//   buf - pointer to the pointer to the data buffer
//   len - number of digits to parse
// return: parsed value
// note: only positive values can be parsed
// note: buf pointer will point to the next byte after parsed number
uint32_t atoi_len(uint8_t **buf, uint8_t len) {
	uint32_t value = 0;

	do {
		if ((**buf < '0') || (**buf > '9'))
			return 0; // character is not a digit -> error
		value *= 10;
		value += *((*buf)++) - '0';
	} while (--len);

	return value;
}

// Parse a (potentially negative) number in data buffer with unknown length
// input:
//   buf - pointer to the pointer to the data buffer
// return: parsed value
// note: buf will point to the next byte after parsed value or to the next term if
//       a byte after parsed number is ','
// note: the function will parse the buffer contents until it hits a non-digit char,
//       therefore in case of long number the int32_t value can be overflowed
int32_t atoi_chr(uint8_t **buf) {
	int32_t neg = 1;
	int32_t value = 0;

	// Check if a number is negative
	if (**buf == '-') {
		neg = -1;
		(*buf)++;
	}

	// Parse buffer until first non-digit character, no overflow check for 'value',
	// therefore 32-bit value will overflow if number length is more than 10 digits
	while ((**buf > '0' - 1) && (**buf < '9' + 1)) {
		value *= 10;
		value += *((*buf)++) - '0';
	}

	// Shift the pointer to the next term if it points to the "," symbol
	if (**buf == ',')
		(*buf)++;

	return neg * value;
}

// Convert two HEX characters into 8-bit binary
// input:
//   buf - pointer to the data buffer
// return: binary value
uint8_t atoi_hex(uint8_t *buf) {
	uint8_t result, tmp;

	// If byte contains letter, it will be converted to lowercase, in case of digit the byte will remain intact
	tmp = (*buf++ | 0x20);
	// Add 0xD0 is same as subtract 0x30, same is for 0xA9
	result = (tmp <= '9') ? (tmp + 0xD0) : (tmp + 0xA9);
	// This is high nibble
	result <<= 4;
	// Repeat the same for low nibble
	tmp = (*buf | 0x20);
	result += (tmp <= '9') ? (tmp + 0xD0) : (tmp + 0xA9);

	return result;
}

// Parse float value from a GPS sentence
// input:
//   buf - pointer to the pointer to the data buffer
// return: parsed value
// note: buf will point to the next byte after parsed value or to the next term if
//       a byte after parsed number is ','
// note: float will be represented as integer, e.g. string '1234.567' will be parsed to 1234567
int32_t atoi_flt(uint8_t **buf) {
	int32_t value;
	int32_t neg = 1;

	// Parse integer part of a number
	value = atoi_chr(buf);
	if (value < 0) {
		value *= -1;
		neg = -1;
	}

	// Parse fractional part if it present
	if (*((*buf)++) == '.') {
		while ((**buf > '0' - 1) && (**buf < '9' + 1)) {
			value *= 10;
			value += *((*buf)++) - '0';
		}
	}

	// Shift the pointer to the next term if it points to the "," symbol
	if (**buf == ',')
		(*buf)++;

	return neg * value;
}

/////////////////////////////
// NMEA Functionss
/////////////////////////////

// Calculate the CRC value of NMEA sentence
// input:
//   str - pointer to the buffer containing sentence
// return: checksum of sentence in HEX format
// note: input sentence should begin with a '$' and end with '*' or a zero byte
uint8_t NMEA_CalcCRC(char *str) {
	uint8_t result = 0;

	if (*str++ == '$')
		while ((*str != '*') && (*str != '\0'))
			result ^= *str++;

	return result;
}

// Find end of the NMEA sentence
// input:
//   buf - pointer to a byte in the data buffer from where search will start
//   buf_end - pointer to the last byte where search must stop
// return: pointer to the last byte of NMEA sentence
// note: the result pointer will point to the last byte of a sentence or to the end
//       of the data buffer in case of sentence is incomplete
uint8_t* NMEA_FindTail(uint8_t *buf, uint8_t *buf_end) {
	do {
		if (*buf++ == '\r') {
			if (*buf == '\n') {
				break;
			}
		}
	} while (buf < buf_end);

	return buf;
}

// Find next term of NMEA sentence
// input:
//   buf - pointer to pointer to the data buffer
// note: buf pointer will point to the next term or to the sentence end ('*' character)
void NMEA_NextTerm(uint8_t **buf) {
	// Find next term or sentence end
	while ((**buf != ',') && (**buf != '*'))
		(*buf)++;

	// Point to the first character of next term
	if (**buf == ',')
		(*buf)++;
}

// Find NMEA sentence in buffer
// input:
//   sentence - pointer to the structure describing NMEA sentence
//   buf_start - pointer to the data buffer where search will start
//   buf_end - pointer to the end of the data buffer
// note: function modifies the 'sentence' structure
void NMEA_FindSentence(NmeaSentence_T *sentence, uint8_t *buf_start,
		uint8_t *buf_end) {
	uint32_t *ptr = (uint32_t*) buf_start;
	uint32_t hdr;

	// Populate the sentence structure with initial values
	sentence->start = (uint8_t*) ptr;
	sentence->data = sentence->start;
	sentence->end = buf_end;
	sentence->type = NMEA_NOT_FOUND;

	// Further parsing does not make sense if there is no space left in buffer for NMEA sentence
	if (buf_end - (uint8_t*) ptr < 10)
		return;

	do {
		hdr = *ptr << 8;
		if (hdr == 0x50472400 || hdr == 0x4c472400 || hdr == 0x4e472400) {
			// $GPxxx            $GLxxx               $GNxxx

			// Point to the first byte of sentence
			sentence->start = (uint8_t*) ptr;

			// Point to the 4th byte of sentence
			ptr = (uint32_t*) ((uint8_t*) ptr + 3);
			hdr = *ptr << 8;

			// Point to the first term of sentence
			sentence->data = (uint8_t*) ptr + 4;

			// Find sentence tail
			sentence->end = NMEA_FindTail(sentence->start, buf_end);

			// Determine a sentence type
			switch (hdr) {
			case 0x4c4c4700:
				sentence->type = NMEA_xxGLL;
				return;
			case 0x434d5200:
				sentence->type = NMEA_xxRMC;
				return;
			case 0x47545600:
				sentence->type = NMEA_xxVTG;
				return;
			case 0x41474700:
				sentence->type = NMEA_xxGGA;
				return;
			case 0x41534700:
				sentence->type = NMEA_xxGSA;
				return;
			case 0x56534700:
				sentence->type = NMEA_xxGSV;
				return;
			case 0x41445a00:
				sentence->type = NMEA_xxZDA;
				return;
			default:
				// Unsupported GPS sentence
				sentence->type = NMEA_UNKNOWN;
				return;
			}
		} else {
			// Random message of unknown / unsupported type
			sentence->type = NMEA_UNKNOWN;

			return;
		}

		// Proceed to next byte in the buffer
		ptr = (uint32_t*) ((uint8_t*) ptr + 1);
	} while (ptr < (uint32_t*) buf_end);
}

// Parse time from NMEA sentence (format: HHMMSS.XXX)
// input:
//   buf - pointer to the pointer to the data buffer
//   time - pointer to structure where time will be stored
// note: the buf pointer will point to the next term
void NMEA_ParseTime(uint8_t **buf, NmeaTime_T *time) {
	if (**buf != ',') {
		// Hours
		time->hours = atoi_len(buf, 2);

		// Minutes
		time->minutes = atoi_len(buf, 2);

		// Seconds
		time->seconds = atoi_len(buf, 2);

		// ... Milliseconds are ignored
		NMEA_NextTerm(buf);
	} else {
		(*buf)++;
	}
}

// Parse latitude or longitude term ONLY ONE
// input:
//   buf - pointer to the pointer to the data buffer
//   deg_len - length of the degrees value (2 for latitude, 3 for longitude)
//   value - pointer to the coordinate variable
//   char_value - pointer to the coordinate character variable
// note: buf will point to the next term
void NMEA_ParseLatLon(uint8_t **buf, uint8_t deg_len, int32_t *value,
		uint8_t *char_value) {
	uint32_t f_deg = 0; // coordinate degrees fractional part
	uint8_t f_len = 0; // fractional part length

	// Coordinate
	if (**buf != ',') {
		// Degrees
		*value = atoi_len(buf, deg_len) * 60;

		// Minutes integer part
		*value += atoi_len(buf, 2);

		// Skip decimal dot
		(*buf)++;

		// Minutes fractional part, it length deends on GPS receiver
		// Parse no more than 4 digits (~22cm precision?)
		while (f_len++ < 4) {
			f_deg *= 10;
			f_deg += *((*buf)++) - '0';
			if ((**buf < '0') || (**buf > '9'))
				break; // character is not a digit -> end of number
		}

		// If a fractional part consists less than 4 digits, need to adjust a
		// fractional part and scale factor up to 4 digits
		if (f_len < 4) {
			while (f_len++ < 4)
				f_deg *= 10;
		}

		// If a fractional part consists more than 4 digits, need to scan up to the end of the number
		if (**buf != ',') {
			while ((**buf > '0' - 1) && (**buf < '9' + 1))
				(*buf)++;
		}

		// Point to next NMEA term
		(*buf)++;

		// Calculate a 'micro-degrees' value
		// value of '10000' - the scaling factor, deending on the length of the fractional part
		*value = (((*value * 10000) + f_deg) * 10) / 6;
	} else {
		// Point to next NMEA term
		(*buf)++;

		// No coordinates in sentence, bail out
		*value = 0;
	}

	// Coordinate character
	if (**buf != ',') {
		*char_value = **buf;
		*buf += 2;

		// In case of 'S' latitude or 'W' longitude the degrees value must be negative
		if ((*char_value == 'W') || (*char_value == 'S'))
			*value *= -1;
	} else {
		*char_value = 'X';
		(*buf)++;
	}
}

// Parse a xxRMC message, and put the output into the RMC data buffer
// input:
//   buf - pointer to the data buffer
void NMEA_ParseRmc(uint8_t *ptr) {
	// Time
	NMEA_ParseTime(&ptr, &data_RMC.time_utc);

	// validity
	if (*ptr != ',') {
		if (*ptr++ == 'A') {
			data_RMC.data_valid = true;
		} else {
			data_RMC.data_valid = false;
		}
	}
	ptr++;	// Move to the next elem

	// Parse Latitude
	NMEA_ParseLatLon(&ptr, 2, &data_RMC.lat_microdeg, &data_RMC.lat_char);

	// Parse Longitude
	NMEA_ParseLatLon(&ptr, 3, &data_RMC.lon_microdeg, &data_RMC.lon_char);

	// Horizontal speed (kts, but atoi turns it into /1000 (micro knot))
	if (ptr[0] != ',') {
		data_RMC.speed_mkts = atoi_flt(&ptr);
	} else {
		ptr++;
	}

	// Course
	if (ptr[0] != ',') {
		data_RMC.course_deg = atoi_flt(&ptr);
	} else {
		ptr++;
	}

	// Date of fix
	if (*ptr != ',') {
		// Day
		data_RMC.utc_date.day = atoi_len(&ptr, 2);

		// Month
		data_RMC.utc_date.month = atoi_len(&ptr, 2);

		// Year (two digits)
		data_RMC.utc_date.year = atoi_len(&ptr, 2);
		// Some receivers report date year as 70 or 80 when their internal clock has
		// not yet synchronized with the satellites
		// Yep, this trick wouldn't work after 2069 year ^_^
		if (data_RMC.utc_date.year > 69) {
			// Assume what year is less than 2000
			data_RMC.utc_date.year += 1900;
		} else {
			// Assume what year is greater than 2000
			// Copy fix_date to date and fix_time to time in case of the $GPZDA sentence are disabled
			data_RMC.utc_date.year += 2000;
		}
	}
	ptr++;

	// Magnetic variation (Ignored)
	NMEA_NextTerm(&ptr);	// MV
	NMEA_NextTerm(&ptr);	// MV dir

	// Mode indicator (NMEA 0183 v2.3 or never)
	if ((*ptr != ',') && (*ptr != '*')) {
		data_RMC.pos_mode = *ptr;
	}
}

void NMEA_ParseGga(uint8_t *ptr) {
	// Time
	NMEA_ParseTime(&ptr, &data_GGA.time_utc);

	// LatLon
	NMEA_ParseLatLon(&ptr, 2, &data_GGA.lat_microdeg, &data_GGA.lat_char);
	NMEA_ParseLatLon(&ptr, 3, &data_GGA.lon_microdeg, &data_GGA.lon_char);

	// Quality
	if (ptr[0] != ',') {
		data_GGA.quality = atoi_len(&ptr, 1);
	} else {
		ptr++;
	}

	// num sats
	if (ptr[0] != ',') {
		data_GGA.num_sats = atoi_len(&ptr, 2);
	} else {
		ptr++;
	}

	// HDOP
	if (ptr[0] != ',') {
		data_GGA.hdop_scaled = atoi_flt(&ptr) * 1000;
	} else {
		ptr++;
	}

	// Alt
	if (ptr[0] != ',') {
		data_GGA.altitude_mm = atoi_flt(&ptr) * 1000;
	} else {
		ptr++;
	}
	NMEA_NextTerm(&ptr);	// Alt Unit (always M)

	// Geoid separation
	if (ptr[0] != ',') {
		data_GGA.geoid_sep_scaled = atoi_flt(&ptr) * 1000;
	} else {
		ptr++;
	}

	// Age of differential corrections
	if (ptr[0] != ',') {
		data_GGA.diff_age = atoi_flt(&ptr);
	} else {
		ptr++;
	}
	// Differential station ID
	NMEA_NextTerm(&ptr);

}

void NMEA_ParseSentence(NmeaSentence_T *sentence) {
	uint8_t *ptr = sentence->data;

	switch (sentence->type) {
	case NMEA_xxRMC:
		// This is the case we care the most about, since it gives us basically all the data we need.
		// This typically comes through as a GN message.
		NMEA_ParseRmc(ptr);
		break;
	case NMEA_xxZDA:
		// This is second  most important, since it tells us the local time.
		break;
	default:
		break;
	}
}

// Parse NMEA sentences in specified data buffer
// input:
//   buf - pointer to the buffer with GPS data
//   length - pointer to the variable with number of bytes in the data buffer
// Be careful not to modify the buf pointer
void NMEA_ParseBuf(uint8_t *buf, uint16_t *length) {
	uint8_t *buf_end = buf + *length;
	uint8_t *ptr = buf;

	while (ptr < buf_end) {
		// Find the sentence
		if (ptr[0] == '$') { // Found the start of a sentence
			ptr += 3; // Skip past the GP / GL series
			uint32_t hdr = ptr[0] << 16 | ptr[1] << 8 | ptr[2];

			// TODO: make the headers defines somewhere
			switch (hdr) {// Not sure why this was easier than doing a strcmp...but here we are
			case 0x474741:  // GGA == 0x47 47 41
				// GGA Message
				NMEA_ParseGga(ptr);
				break;
			case 0x524d43: // RMC == 0x52 4D 43
				ptr += 4; // Skip past RMC, to point to whatever is AFTER the comma
				NMEA_ParseRmc(ptr);
				break;
			default:
				break;
			}
		}
		// Dumb increment to the next location
		ptr++;
	}
}
