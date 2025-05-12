/**
 * gps.c
 *
 * Created on: 2025-MAY-12
 *     Author: Anni
 *
 * Contains common GPS code (starting GNSS, processing incoming UART messages, etc)-
 */

#include "gps.h"
#include "m8n.h"
#include "sensor_board.h" // M8N_UART_INSTANCE

void gnss_start_rx(const GpsType_T * const gps_type, const size_t num_gps) {
    for (size_t i = 0; i < num_gps; ++i) {
        switch(gps_type[i]) {
        case GPS_NEO_M8N:
            gnss_m8n_start_rx();
            break;
        default:
            break;
        }
    }
}

bool gnss_parse_data_if_available(const GpsType_T * const gps_type, const size_t num_gps, GpsData_S *gps_data) {
    bool retval = false;

    for (size_t i = 0; i < num_gps; ++i) {
        switch(gps_type[i]) {
        case GPS_NEO_M8N:
            if(gnss_m8n_is_data_available()) {
                retval |= true;
                gnss_m8n_parse_data(&gps_data[i]);
            }
            break;
        default:
            break;
        }
    }

    return retval;
}

bool gnss_process_incoming_data(UART_HandleTypeDef *huart, uint16_t size) {
    bool retval = false;
    if (huart->Instance == M8N_UART_INSTANCE) {
        retval = true;
        gnss_m8n_process_incoming_data(size);
    }

    return retval;
}
