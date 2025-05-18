
#pragma once
#include <stdbool.h>
#include <stdint.h>
#include <canard.h>




#define UAVCAN_EQUIPMENT_AHRS_SENSORIMU_MAX_SIZE 23
#define UAVCAN_EQUIPMENT_AHRS_SENSORIMU_SIGNATURE (0x4B0725C6F79B19AFULL)

#define UAVCAN_EQUIPMENT_AHRS_SENSORIMU_ID 2100





#if defined(__cplusplus) && defined(DRONECAN_CXX_WRAPPERS)
class uavcan_equipment_ahrs_SensorIMU_cxx_iface;
#endif


struct uavcan_equipment_ahrs_SensorIMU {

#if defined(__cplusplus) && defined(DRONECAN_CXX_WRAPPERS)
    using cxx_iface = uavcan_equipment_ahrs_SensorIMU_cxx_iface;
#endif




    uint32_t timestamp;



    int16_t accelerometer_latest[3];



    int16_t rate_gyro_latest[3];



    int16_t magnetometer_latest[3];



    bool accel_data_valid;



    bool gyro_data_valid;



    bool mag_data_valid;



};

#ifdef __cplusplus
extern "C"
{
#endif

uint32_t uavcan_equipment_ahrs_SensorIMU_encode(struct uavcan_equipment_ahrs_SensorIMU* msg, uint8_t* buffer
#if CANARD_ENABLE_TAO_OPTION
    , bool tao
#endif
);
bool uavcan_equipment_ahrs_SensorIMU_decode(const CanardRxTransfer* transfer, struct uavcan_equipment_ahrs_SensorIMU* msg);

#if defined(CANARD_DSDLC_INTERNAL)

static inline void _uavcan_equipment_ahrs_SensorIMU_encode(uint8_t* buffer, uint32_t* bit_ofs, struct uavcan_equipment_ahrs_SensorIMU* msg, bool tao);
static inline bool _uavcan_equipment_ahrs_SensorIMU_decode(const CanardRxTransfer* transfer, uint32_t* bit_ofs, struct uavcan_equipment_ahrs_SensorIMU* msg, bool tao);
void _uavcan_equipment_ahrs_SensorIMU_encode(uint8_t* buffer, uint32_t* bit_ofs, struct uavcan_equipment_ahrs_SensorIMU* msg, bool tao) {

    (void)buffer;
    (void)bit_ofs;
    (void)msg;
    (void)tao;






    canardEncodeScalar(buffer, *bit_ofs, 32, &msg->timestamp);

    *bit_ofs += 32;






    for (size_t i=0; i < 3; i++) {




        canardEncodeScalar(buffer, *bit_ofs, 16, &msg->accelerometer_latest[i]);

        *bit_ofs += 16;


    }






    for (size_t i=0; i < 3; i++) {




        canardEncodeScalar(buffer, *bit_ofs, 16, &msg->rate_gyro_latest[i]);

        *bit_ofs += 16;


    }






    for (size_t i=0; i < 3; i++) {




        canardEncodeScalar(buffer, *bit_ofs, 16, &msg->magnetometer_latest[i]);

        *bit_ofs += 16;


    }






    canardEncodeScalar(buffer, *bit_ofs, 1, &msg->accel_data_valid);

    *bit_ofs += 1;






    canardEncodeScalar(buffer, *bit_ofs, 1, &msg->gyro_data_valid);

    *bit_ofs += 1;






    canardEncodeScalar(buffer, *bit_ofs, 1, &msg->mag_data_valid);

    *bit_ofs += 1;





}

/*
 decode uavcan_equipment_ahrs_SensorIMU, return true on failure, false on success
*/
bool _uavcan_equipment_ahrs_SensorIMU_decode(const CanardRxTransfer* transfer, uint32_t* bit_ofs, struct uavcan_equipment_ahrs_SensorIMU* msg, bool tao) {

    (void)transfer;
    (void)bit_ofs;
    (void)msg;
    (void)tao;





    canardDecodeScalar(transfer, *bit_ofs, 32, false, &msg->timestamp);

    *bit_ofs += 32;







    for (size_t i=0; i < 3; i++) {




        canardDecodeScalar(transfer, *bit_ofs, 16, true, &msg->accelerometer_latest[i]);

        *bit_ofs += 16;


    }








    for (size_t i=0; i < 3; i++) {




        canardDecodeScalar(transfer, *bit_ofs, 16, true, &msg->rate_gyro_latest[i]);

        *bit_ofs += 16;


    }








    for (size_t i=0; i < 3; i++) {




        canardDecodeScalar(transfer, *bit_ofs, 16, true, &msg->magnetometer_latest[i]);

        *bit_ofs += 16;


    }








    canardDecodeScalar(transfer, *bit_ofs, 1, false, &msg->accel_data_valid);

    *bit_ofs += 1;







    canardDecodeScalar(transfer, *bit_ofs, 1, false, &msg->gyro_data_valid);

    *bit_ofs += 1;







    canardDecodeScalar(transfer, *bit_ofs, 1, false, &msg->mag_data_valid);

    *bit_ofs += 1;





    return false; /* success */

}
#endif
#ifdef CANARD_DSDLC_TEST_BUILD
struct uavcan_equipment_ahrs_SensorIMU sample_uavcan_equipment_ahrs_SensorIMU_msg(void);
#endif
#ifdef __cplusplus
} // extern "C"

#ifdef DRONECAN_CXX_WRAPPERS
#include <canard/cxx_wrappers.h>


BROADCAST_MESSAGE_CXX_IFACE(uavcan_equipment_ahrs_SensorIMU, UAVCAN_EQUIPMENT_AHRS_SENSORIMU_ID, UAVCAN_EQUIPMENT_AHRS_SENSORIMU_SIGNATURE, UAVCAN_EQUIPMENT_AHRS_SENSORIMU_MAX_SIZE);


#endif
#endif
