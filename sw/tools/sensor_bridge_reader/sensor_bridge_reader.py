import serial
from sensor_data_pb2 import ImuData

MAGIC_HEADER = b'\xAA\x55'
MAX_MESSAGE_SIZE = 256

# Note! This script needs specific configuration to run properly
# This is GPIO12 - TX, GPIO 13 - RX, on raspberry pi 5
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
    """Continuously reads bytes until it finds the magic header."""
    while True:
        first = ser.read(1)
        if first != b'\xAA':
            continue
        second = ser.read(1)
        if second == b'\x55':
            return True
        # If second doesn't match, loop again

def main():
    ser = serial.Serial(UART_DEVICE, baudrate=UART_BAUDRATE, timeout=1)

    while True:
        try:
            # Step 1: Wait for the magic header
            found = find_magic_header(ser)
            if not found:
                continue

            can_id_bytes = read_n_bytes(ser, 1)
            if can_id_bytes is None:
                continue
            
            can_id = int.from_bytes(can_id_bytes, byteorder='little')

            # Step 3: Read the next two bytes (message size)
            size_bytes = read_n_bytes(ser, 2)
            if size_bytes is None:
                continue

            message_size = int.from_bytes(size_bytes, byteorder='little')
            if message_size == 0 or message_size > MAX_MESSAGE_SIZE:
                continue

            # Step 4: Read the protobuf payload
            encoded_message = read_n_bytes(ser, message_size)
            if encoded_message is None:
                continue

            if len(encoded_message) != message_size:
                print(f"Error: Expected {message_size} bytes but got {len(encoded_message)}")
                continue
            
            print(f"Dump: {encoded_message.hex()}")
            # Step 5: Decode and print
            imu_data = ImuData()
            imu_data.ParseFromString(encoded_message)

            print(f"CAN ID: {can_id}")
            print(f"Accel is valid: {imu_data.accel_data_valid}, Gyro is valid: {imu_data.gyro_data_valid}")
            print(f"Accel: {imu_data.accel_x_mg}, {imu_data.accel_y_mg}, {imu_data.accel_z_mg}")
            print(f"Gyro:  {imu_data.gyro_x_mdps}, {imu_data.gyro_y_mdps}, {imu_data.gyro_z_mdps}")
            print(f"Timestamp: {imu_data.timestamp}")

        except KeyboardInterrupt:
            break
        except Exception as e:
            print("Error:", e)

    ser.close()

if __name__ == '__main__':
    main()
