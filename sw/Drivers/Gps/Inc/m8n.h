/**
 * m8n.h
 *
 * Created on: 2025-MAY-10
 *     Author: Anni
 *   Modified: 2025-MAY-10
 *
 * Contains common GPS functionality & Data structures for neo m8n gps
 */

#ifndef M8N_H
#define M8N_H

#include <stdbool.h>
#include <stdint.h>

#include "gps.h"
#include "nmea.h"

void gnss_m8n_start_rx();

bool gnss_m8n_is_data_available();

void gnss_m8n_parse_data(GpsData_S *gps_data);

void gnss_m8n_process_incoming_data(uint16_t size);

#endif  // M8N_H
