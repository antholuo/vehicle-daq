/**
 * m8n.c
 *
 * Created on: 2025-MAY-12
 *     Author: Anni
 *   Modified: 2025-MAY-12
 *
 * GPS Driver Code for UBLox M8N gps
 * (and probably others, this is pretty generic)
 */

#include "m8n.h"
#include "nmea.h"
#include "gps.h"

#include "usart.h"

#include <stdbool.h>
#include <string.h>

// Size of the internal UART buffer for receiving messages.
// Must be larger than the largest message we can expect to receive over UART
// Messages typically ~ 500 bytes long. BUT IN CASE !!!
#define GPS_UART_BUFFER_SIZE (1500)

// TODO: have the uart handle passed in on init
extern UART_HandleTypeDef huart1; // GNSS uart

extern RmcData_T data_RMC; // RMC & GGA data defined in nmea.c
extern GgaData_T data_GGA;

uint8_t   nmea_raw[GPS_UART_BUFFER_SIZE];
/* uint8_t   nmea_cp[GPS_UART_BUFFER_SIZE]; */
uint16_t  nmea_raw_idx;
int8_t    new_data_ready = -5; // ignore the first N packets

void gnss_m8n_start_rx() {
    HAL_UARTEx_ReceiveToIdle_DMA(&huart1, nmea_raw, sizeof(nmea_raw));
}

bool gnss_m8n_is_data_available() {
    return (new_data_ready > 0);
}

void gnss_m8n_parse_data(GpsData_S *gps_data) {
    NMEA_ParseBuf(nmea_raw, &nmea_raw_idx);
    gps_data->nmea_time = data_RMC.nmea_time;
    gps_data->lat_microdeg = data_RMC.lat_microdeg;
    gps_data->lon_microdeg = data_RMC.lon_microdeg;
    gps_data->altitude_mm = data_GGA.altitude_mm;
    gps_data->speed_mkts = data_RMC.speed_mkts;
    gps_data->course_deg = data_RMC.course_deg;
    gps_data->heading_valid = false; // M8N has no compass
    gps_data->num_sats = data_GGA.num_sats;
    gps_data->fix_status = data_GGA.quality;
    gps_data->data_valid = data_RMC.data_valid;
}

void gnss_m8n_process_incoming_data(uint16_t size) {
    nmea_raw_idx = size;
    new_data_ready += 1;
    /* memcpy(nmea_cp, nmea_raw, size); */
    HAL_UARTEx_ReceiveToIdle_DMA(&huart1, nmea_raw, sizeof(nmea_raw));
}
