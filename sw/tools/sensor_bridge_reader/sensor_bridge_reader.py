import serial
import os
import time
from datetime import datetime, timedelta
from sensor_data_pb2 import ImuData, GpsData

# Magic headers
IMU_MAGIC_HEADER = b'\xAA\x55'
GPS_MAGIC_HEADER = b'\xBB\x77'

MAX_MESSAGE_SIZE = 256
UART_DEVICE = '/dev/ttyAMA4'
UART_BAUDRATE = 230400
LOG_ROTATE_INTERVAL = 600  # seconds
LOG_DIR = "/home/hardy/sensor_logs"

os.makedirs(LOG_DIR, exist_ok=True)

def get_timestamp_string():
    try:
        # Try using real time
        return datetime.now().strftime("%Y-%m-%d_%H-%M-%S")
    except Exception:
        # Fallback to monotonic seconds since boot
        return f"uptime_{int(time.monotonic())}"

def get_new_logfile():
    timestamp = get_timestamp_string()
    path = os.path.join(LOG_DIR, f"log_{timestamp}.txt")
    print(f"Starting new log file: {path}")
    return open(path, "w"), time.monotonic()

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
    logfile, last_rotation_time = get_new_logfile()

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

            now_monotonic = time.monotonic()
            if now_monotonic - last_rotation_time > LOG_ROTATE_INTERVAL:
                logfile.close()
                logfile, last_rotation_time = get_new_logfile()

            try:
                now_str = datetime.now().isoformat()
            except Exception:
                now_str = f"monotonic_{int(now_monotonic)}"

            logfile.write(f"\n[{now_str}] CAN ID: {can_id}\n")
            logfile.write(f"Dump: {encoded_message.hex()}\n")

            if msg_type == 'imu':
                imu_data = ImuData()
                imu_data.ParseFromString(encoded_message)
                msg = (
                    f"IMU Message:\n"
                    f"  Timestamp: {imu_data.timestamp}\n"
                    f"  Accel is valid: {imu_data.accel_data_valid}, Gyro is valid: {imu_data.gyro_data_valid}\n"
                    f"  Accel: {imu_data.accel_x_mg}, {imu_data.accel_y_mg}, {imu_data.accel_z_mg}\n"
                    f"  Gyro:  {imu_data.gyro_x_mdps}, {imu_data.gyro_y_mdps}, {imu_data.gyro_z_mdps}\n"
                )
                logfile.write(msg)
            elif msg_type == 'gps':
                gps_data = GpsData()
                gps_data.ParseFromString(encoded_message)
                msg = (
                    f"GPS Message:\n"
                    f"  Time: {gps_data.time.hours}:{gps_data.time.minutes}:{gps_data.time.seconds}\n"
                    f"  Date: {gps_data.date.year}-{gps_data.date.month}-{gps_data.date.day}\n"
                    f"  Lat/Lon (microdegrees): {gps_data.lat_microdeg}, {gps_data.lon_microdeg}\n"
                    f"  Altitude (mm): {gps_data.altitude_mm}\n"
                    f"  Speed (knots): {gps_data.speed_mkts}\n"
                    f"  Course (deg): {gps_data.course_deg}\n"
                    f"  Quality: {gps_data.quality}\n"
                    f"  Num Satellites: {gps_data.num_sats}\n"
                    f"  Data Valid: {gps_data.data_valid}\n"
                )
                logfile.write(msg)

            logfile.flush()

        except KeyboardInterrupt:
            print("Exiting...")
            break
        except Exception as e:
            print("Error:", e)

    logfile.close()
    ser.close()

if __name__ == '__main__':
    main()
