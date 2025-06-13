import serial
import os
import time
import csv
import json
from datetime import datetime
from sensor_data_pb2 import ImuData, GpsData

# Magic headers
IMU_MAGIC_HEADER = b'\xAA\x55'
GPS_MAGIC_HEADER = b'\xBB\x77'

MAX_MESSAGE_SIZE = 256
UART_DEVICE = '/dev/ttyAMA4'
UART_BAUDRATE = 230400
LOG_ROTATE_INTERVAL = 600  # seconds
LOG_DIR = "/home/hardy/sensor_logs"

CSV_HEADER = [
    "node_id", "sensor_type", "imu_ts", "gps_ts", "rtc_ts",
    "gps_update", "imu_update", "lat", "lon", "altitude_m", "speed_kmph",
    "heading", "accuracy", "accel_x", "accel_y", "accel_z", "gyro_x", "gyro_y", "gyro_z"
]

os.makedirs(LOG_DIR, exist_ok=True)

def get_timestamp_string():
    try:
        return datetime.now().strftime("%Y-%m-%d_%H-%M-%S")
    except Exception:
        return f"uptime_{int(time.monotonic())}"

def get_new_logfile():
    timestamp = get_timestamp_string()
    path = os.path.join(LOG_DIR, f"log_{timestamp}.csv")
    f = open(path, "w", newline='')
    writer = csv.DictWriter(f, fieldnames=CSV_HEADER)
    writer.writeheader()
    print(f"Starting new log file: {path}")
    return f, writer, time.monotonic()

def read_n_bytes(ser, n):
    bytes_read = bytearray()
    while len(bytes_read) < n:
        chunk = ser.read(n - len(bytes_read))
        if not chunk:
            return None
        bytes_read.extend(chunk)
    return bytes(bytes_read)

def find_magic_header(ser):
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
    logfile, writer, last_rotation_time = get_new_logfile()

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
                logfile, writer, last_rotation_time = get_new_logfile()

            row = {key: "" for key in CSV_HEADER}
            row["node_id"] = can_id
            row["rtc_ts"] = int(now_monotonic)

            if msg_type == 'imu':
                imu_data = ImuData()
                imu_data.ParseFromString(encoded_message)
                if imu_data.accel_data_valid or imu_data.gyro_data_valid:
                    row["sensor_type"] = "IMU"
                    row["imu_ts"] = imu_data.timestamp
                    row["imu_update"] = 1
                    if imu_data.accel_data_valid:
                        row["accel_x"] = imu_data.accel_x_mg
                        row["accel_y"] = imu_data.accel_y_mg
                        row["accel_z"] = imu_data.accel_z_mg
                    if imu_data.gyro_data_valid:
                        row["gyro_x"] = imu_data.gyro_x_mdps
                        row["gyro_y"] = imu_data.gyro_y_mdps
                        row["gyro_z"] = imu_data.gyro_z_mdps
                    writer.writerow(row)

            elif msg_type == 'gps':
                gps_data = GpsData()
                gps_data.ParseFromString(encoded_message)
                if gps_data.data_valid:
                    row["sensor_type"] = "GPS"
                    row["gps_update"] = 1
                    row["gps_ts"] = f"{gps_data.time.hours:02}:{gps_data.time.minutes:02}:{gps_data.time.seconds:02}"
                    row["lat"] = gps_data.lat_microdeg / 1e6
                    row["lon"] = gps_data.lon_microdeg / 1e6
                    row["altitude_m"] = gps_data.altitude_mm / 1000.0
                    row["speed_kmph"] = gps_data.speed_mkts * 1.852 if gps_data.HasField("speed_mkts") else -1
                    row["heading"] = gps_data.course_deg if gps_data.HasField("course_deg") else -1
                    row["accuracy"] = gps_data.hdop if gps_data.HasField("hdop") else -1
                    writer.writerow(row)

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
