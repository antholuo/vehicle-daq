import serial
from sensor_data_pb2 import ImuData, GpsData

# Magic headers
IMU_MAGIC_HEADER = b'\xAA\x55'
GPS_MAGIC_HEADER = b'\xBB\x77'

MAX_MESSAGE_SIZE = 256
UART_DEVICE = '/dev/ttyAMA4'
UART_BAUDRATE = 230400

def read_n_bytes(ser, n):
    bytes_read = bytearray()
    while len(bytes_read) < n:
        chunk = ser.read(n - len(bytes_read))
        if not chunk:
            return None
        bytes_read.extend(chunk)
    return bytes(bytes_read)

def find_magic_header(ser):
    """Find either the IMU or GPS magic header."""
    while True:
        first = ser.read(1)
        if not first:
            continue
        if first == IMU_MAGIC_HEADER[0:1]:
            second = ser.read(1)
            if second == IMU_MAGIC_HEADER[1:2]:
                return 'imu'
        elif first == GPS_MAGIC_HEADER[0:1]:
            second = ser.read(1)
            if second == GPS_MAGIC_HEADER[1:2]:
                return 'gps'

def main():
    ser = serial.Serial(UART_DEVICE, baudrate=UART_BAUDRATE, timeout=1)

    while True:
        try:
            msg_type = find_magic_header(ser)
            if not msg_type:
                continue

            can_id_bytes = read_n_bytes(ser, 1)
            if can_id_bytes is None:
                continue

            can_id = int.from_bytes(can_id_bytes, byteorder='little')

            size_bytes = read_n_bytes(ser, 2)
            if size_bytes is None:
                continue

            message_size = int.from_bytes(size_bytes, byteorder='little')
            if message_size == 0 or message_size > MAX_MESSAGE_SIZE:
                continue

            encoded_message = read_n_bytes(ser, message_size)
            if encoded_message is None or len(encoded_message) != message_size:
                print(f"Error: Expected {message_size} bytes but got {len(encoded_message)}")
                continue

            print(f"\nCAN ID: {can_id}")
            print(f"Dump: {encoded_message.hex()}")

            if msg_type == 'imu':
                imu_data = ImuData()
                imu_data.ParseFromString(encoded_message)
                print(f"IMU Message:")
                print(f"  Timestamp: {imu_data.timestamp}")
                print(f"  Accel is valid: {imu_data.accel_data_valid}, Gyro is valid: {imu_data.gyro_data_valid}")
                print(f"  Accel: {imu_data.accel_x_mg}, {imu_data.accel_y_mg}, {imu_data.accel_z_mg}")
                print(f"  Gyro:  {imu_data.gyro_x_mdps}, {imu_data.gyro_y_mdps}, {imu_data.gyro_z_mdps}")
            elif msg_type == 'gps':
                gps_data = GpsData()
                gps_data.ParseFromString(encoded_message)
                print(f"GPS Message:")
                print(f"  Time: {gps_data.time.hours}:{gps_data.time.minutes}:{gps_data.time.seconds}")
                print(f"  Date: {gps_data.date.year}-{gps_data.date.month}-{gps_data.date.day}")
                print(f"  Lat/Lon (microdegrees): {gps_data.lat_microdeg}, {gps_data.lon_microdeg}")
                print(f"  Altitude (mm): {gps_data.altitude_mm}")
                print(f"  Speed (knots): {gps_data.speed_mkts}")
                print(f"  Course (deg): {gps_data.course_deg}")
                print(f"  Quality: {gps_data.quality}")
                print(f"  Num Satellites: {gps_data.num_sats}")
                print(f"  Data Valid: {gps_data.data_valid}")

        except KeyboardInterrupt:
            break
        except Exception as e:
            print("Error:", e)

    ser.close()

if __name__ == '__main__':
    main()
