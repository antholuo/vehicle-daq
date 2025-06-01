
#pragma once
#include <stdbool.h>
#include <stdint.h>
#include <canard.h>




#define UAVCAN_EQUIPMENT_GNSS_SENSORGPS_MAX_SIZE 19
#define UAVCAN_EQUIPMENT_GNSS_SENSORGPS_SIGNATURE (0x3938EF4F89EEBB36ULL)

#define UAVCAN_EQUIPMENT_GNSS_SENSORGPS_ID 2101





#if defined(__cplusplus) && defined(DRONECAN_CXX_WRAPPERS)
class uavcan_equipment_gnss_SensorGPS_cxx_iface;
#endif


struct uavcan_equipment_gnss_SensorGPS {

#if defined(__cplusplus) && defined(DRONECAN_CXX_WRAPPERS)
    using cxx_iface = uavcan_equipment_gnss_SensorGPS_cxx_iface;
#endif




    uint8_t hours;



    uint8_t minutes;



    uint8_t seconds;



    int32_t lat_microdeg;



    int32_t lon_microdeg;



    int16_t altitude_mm;



    uint8_t speed_mkts;



    uint16_t course_deg;



    uint8_t num_sats;



    uint8_t fix_status;



    bool data_valid;



};

#ifdef __cplusplus
extern "C"
{
#endif

uint32_t uavcan_equipment_gnss_SensorGPS_encode(struct uavcan_equipment_gnss_SensorGPS* msg, uint8_t* buffer
#if CANARD_ENABLE_TAO_OPTION
    , bool tao
#endif
);
bool uavcan_equipment_gnss_SensorGPS_decode(const CanardRxTransfer* transfer, struct uavcan_equipment_gnss_SensorGPS* msg);

#if defined(CANARD_DSDLC_INTERNAL)

static inline void _uavcan_equipment_gnss_SensorGPS_encode(uint8_t* buffer, uint32_t* bit_ofs, struct uavcan_equipment_gnss_SensorGPS* msg, bool tao);
static inline bool _uavcan_equipment_gnss_SensorGPS_decode(const CanardRxTransfer* transfer, uint32_t* bit_ofs, struct uavcan_equipment_gnss_SensorGPS* msg, bool tao);
void _uavcan_equipment_gnss_SensorGPS_encode(uint8_t* buffer, uint32_t* bit_ofs, struct uavcan_equipment_gnss_SensorGPS* msg, bool tao) {

    (void)buffer;
    (void)bit_ofs;
    (void)msg;
    (void)tao;






    canardEncodeScalar(buffer, *bit_ofs, 8, &msg->hours);

    *bit_ofs += 8;






    canardEncodeScalar(buffer, *bit_ofs, 8, &msg->minutes);

    *bit_ofs += 8;






    canardEncodeScalar(buffer, *bit_ofs, 8, &msg->seconds);

    *bit_ofs += 8;






    canardEncodeScalar(buffer, *bit_ofs, 32, &msg->lat_microdeg);

    *bit_ofs += 32;






    canardEncodeScalar(buffer, *bit_ofs, 32, &msg->lon_microdeg);

    *bit_ofs += 32;






    canardEncodeScalar(buffer, *bit_ofs, 16, &msg->altitude_mm);

    *bit_ofs += 16;






    canardEncodeScalar(buffer, *bit_ofs, 8, &msg->speed_mkts);

    *bit_ofs += 8;






    canardEncodeScalar(buffer, *bit_ofs, 16, &msg->course_deg);

    *bit_ofs += 16;






    canardEncodeScalar(buffer, *bit_ofs, 8, &msg->num_sats);

    *bit_ofs += 8;






    canardEncodeScalar(buffer, *bit_ofs, 8, &msg->fix_status);

    *bit_ofs += 8;






    canardEncodeScalar(buffer, *bit_ofs, 1, &msg->data_valid);

    *bit_ofs += 1;





}

/*
 decode uavcan_equipment_gnss_SensorGPS, return true on failure, false on success
*/
bool _uavcan_equipment_gnss_SensorGPS_decode(const CanardRxTransfer* transfer, uint32_t* bit_ofs, struct uavcan_equipment_gnss_SensorGPS* msg, bool tao) {

    (void)transfer;
    (void)bit_ofs;
    (void)msg;
    (void)tao;





    canardDecodeScalar(transfer, *bit_ofs, 8, false, &msg->hours);

    *bit_ofs += 8;







    canardDecodeScalar(transfer, *bit_ofs, 8, false, &msg->minutes);

    *bit_ofs += 8;







    canardDecodeScalar(transfer, *bit_ofs, 8, false, &msg->seconds);

    *bit_ofs += 8;







    canardDecodeScalar(transfer, *bit_ofs, 32, true, &msg->lat_microdeg);

    *bit_ofs += 32;







    canardDecodeScalar(transfer, *bit_ofs, 32, true, &msg->lon_microdeg);

    *bit_ofs += 32;







    canardDecodeScalar(transfer, *bit_ofs, 16, true, &msg->altitude_mm);

    *bit_ofs += 16;







    canardDecodeScalar(transfer, *bit_ofs, 8, false, &msg->speed_mkts);

    *bit_ofs += 8;







    canardDecodeScalar(transfer, *bit_ofs, 16, false, &msg->course_deg);

    *bit_ofs += 16;







    canardDecodeScalar(transfer, *bit_ofs, 8, false, &msg->num_sats);

    *bit_ofs += 8;







    canardDecodeScalar(transfer, *bit_ofs, 8, false, &msg->fix_status);

    *bit_ofs += 8;







    canardDecodeScalar(transfer, *bit_ofs, 1, false, &msg->data_valid);

    *bit_ofs += 1;





    return false; /* success */

}
#endif
#ifdef CANARD_DSDLC_TEST_BUILD
struct uavcan_equipment_gnss_SensorGPS sample_uavcan_equipment_gnss_SensorGPS_msg(void);
#endif
#ifdef __cplusplus
} // extern "C"

#ifdef DRONECAN_CXX_WRAPPERS
#include <canard/cxx_wrappers.h>


BROADCAST_MESSAGE_CXX_IFACE(uavcan_equipment_gnss_SensorGPS, UAVCAN_EQUIPMENT_GNSS_SENSORGPS_ID, UAVCAN_EQUIPMENT_GNSS_SENSORGPS_SIGNATURE, UAVCAN_EQUIPMENT_GNSS_SENSORGPS_MAX_SIZE);


#endif
#endif
