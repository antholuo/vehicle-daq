/**
 * fusion_datatypes.h
 *
 * Created on: 2025-MAY-10
 *     Author: Anni
 *   Modified: 2025-MAY-10
 *
 * Contains datatypes that we'll use for passing data back and forth
 */

#ifndef FUSION_DATATYPES_H
#define FUSION_DATATYPES_H

#include "imu.h"
#include "gps.h"

typedef struct __attribute__((packed)) {
    ImuData_S imu_data;
    GpsData_S gps_data;
} MotionDataRaw_S;

#endif // FUSION_DATATYPES_H
