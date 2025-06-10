import os
import re
import shutil

import matplotlib.pyplot as plt
import numpy as np

DATAFILE = "/Users/anthony/Downloads/first_car_data.txt"
OUTPATH = "./out/"


class SensorData:
    dump = 0
    timestamp = 0
    gps_update = 0
    latitude = 0
    longitude = 0
    altitude_m = 0
    altitude_ft = 0
    speed_kmph = 0
    accuracy_m = 0
    accel_x_g = 0
    accel_y_g = 0
    accel_z_g = 0
    accel_valid = False
    gyro_valid = False

    def __init__(self, dump):
        self.dump = dump

    def add_imu_data(self, ts_ln, vld_ln, xl_ln, gy_ln):
        # Timestamp line: "Timestamp: 15178774"
        ts_match = re.search(r"Timestamp:\s*(\d+)", ts_ln)
        if ts_match:
            self.timestamp = int(ts_match.group(1))

        # Accel line: "Accel: 145, 76, 981"
        accel_match = re.search(r"Accel:\s*(-?\d+),\s*(-?\d+),\s*(-?\d+)", xl_ln)
        if accel_match:
            self.accel_x_g = -int(accel_match.group(2))
            self.accel_y_g = -int(accel_match.group(1))
            self.accel_z_g = int(accel_match.group(3))

        # Gyro line: "Gyro:  3920, -128, 0"
        gyro_match = re.search(r"Gyro:\s*(-?\d+),\s*(-?\d+),\s*(-?\d+)", gy_ln)
        if gyro_match:
            self.gyro_x = -int(gyro_match.group(2))
            self.gyro_y = -int(gyro_match.group(1))
            self.gyro_z = int(gyro_match.group(3))

        # Validity flags if you want to track them (optional)
        # vld_ln: "Accel is valid: False, Gyro is valid: False"
        accel_valid = "Accel is valid: True" in vld_ln
        gyro_valid = "Gyro is valid: True" in vld_ln
        self.accel_valid = accel_valid
        self.gyro_valid = gyro_valid

    def add_gps_data(self, latlon_ln, alt_ln, speed_ln):
        import re

        # Lat/Lon (microdegrees): 43473545, -80540025
        latlon_match = re.search(
            r"Lat/Lon \(microdegrees\):\s*(-?\d+),\s*(-?\d+)", latlon_ln
        )
        if latlon_match:
            self.latitude = int(latlon_match.group(1)) / 1e6
            self.longitude = int(latlon_match.group(2)) / 1e6

        # Altitude (mm): 0
        alt_match = re.search(r"Altitude \(mm\):\s*(-?\d+)", alt_ln)
        if alt_match:
            self.altitude_m = int(alt_match.group(1)) / 1000.0
            self.altitude_ft = self.altitude_m * 3.28084

        # Speed (knots): 154 → km/h
        speed_match = re.search(r"Speed \(knots\):\s*(\d+)", speed_ln)
        if speed_match:
            knots = int(speed_match.group(1))
            self.speed_kmph = knots * 1.852  # convert to km/h


class SensorNode:
    id = 0
    sensor_data = []

    def __init__(self, id):
        self.id = id

    def append_data(self, sensor_data):
        self.sensor_data.append(sensor_data)

    def update_last_with_gps(self, gps_data):
        last_data = self.sensor_data[-1]
        last_data.gps_update = 1
        last_data.latitude = gps_data.latitude
        last_data.longitude = gps_data.longitude
        last_data.altitude_m = gps_data.altitude_m
        last_data.altitude_ft = gps_data.altitude_ft
        last_data.speed_kmph = gps_data.speed_kmph

    def write_to_csv(self, file):
        file.write(
            "Time, Lap, GPS_Update, Latitude, Longitude, ALtitude (m), Altitude (ft), Speed (Km/h), Heading, Accuracy (m), Accel X, Accel Y, Accel Z \n"
        )
        for data in self.sensor_data:
            outline = (
                str(data.timestamp)
                + ","
                + "0"
                + ","
                + str(data.gps_update)
                + ","
                + str(data.latitude)
                + ","
                + str(data.longitude)
                + ","
                + str(data.altitude_m)
                + ","
                + str(data.altitude_ft)
                + ","
                + str(data.speed_kmph)
                + ","
                + "0"
                + ","
                + "0"
                + ","
                + str(data.accel_x_g)
                + ","
                + str(data.accel_y_g)
                + ","
                + str(data.accel_z_g)
                + "\n"
            )
            file.write(outline)


def moving_average(data, window_size):
    """Calculates the moving average of a 1D array.
    """
    window = np.ones(window_size) / window_size
    return np.convolve(data, window, mode='same')

def parse_datafile_to_blocks(datafile):
    blocks = []
    current_block = []

    for line in datafile:
        line = line.rstrip("\n")
        if line:
            current_block.append(line)
        elif current_block:
            blocks.append(current_block)
            current_block = []

    if current_block:
        blocks.append(current_block)

    return blocks


def parse_data(datafile, outpath):
    blocks = parse_datafile_to_blocks(datafile)
    sensor_nodes = {}  # create one dataset per unique IMU

    for block in blocks:
        # grab the ID and collect the correct sensor node class
        id = re.search(r"CAN ID:\s*(\d+)", block[0]).group(1)
        if id in sensor_nodes:
            sensor_node = sensor_nodes[id]
        else:
            sensor_node = SensorNode(int(id))
            sensor_nodes[id] = sensor_node

        if block[2] == "IMU Message:":
            # create new IMU data
            sensor_data = SensorData(block[1])
            sensor_data.add_imu_data(block[3], block[4], block[5], block[6])

            if sensor_data.accel_valid and sensor_data.gyro_valid:
                sensor_node.append_data(sensor_data)
        elif block[2] == "GPS Message:":
            # append GPS data to BOTH prev IMU data's
            sensor_data = SensorData(block[1])
            sensor_data.add_gps_data(block[5], block[6], block[7])

            for ids in sensor_nodes:
                sensor_nodes[ids].update_last_with_gps(sensor_data)
        else:
            print("Couldn't parse datatype: ", block[2])

    shutil.rmtree("out")
    os.mkdir("out/")
    for ids in sensor_nodes:
        path = "out/" + str(sensor_nodes[ids].id) + ".csv"
        with open(path, "x") as file:
            sensor_nodes[ids].write_to_csv(file)

    return sensor_nodes

def plot_accel_xy(plotdata):
    id33_data = plotdata["33"]
    print(id33_data)

    lat = []
    lon = []

    x_pts = []
    y_pts_xl_x = []
    y_pts_xl_y = []
    reference = id33_data.sensor_data[0].timestamp;
    for data in id33_data.sensor_data:
        x_pts.append(data.timestamp - reference)
        y_pts_xl_x.append(data.accel_x_g)
        y_pts_xl_y.append(data.accel_y_g)
        if data.gps_update:
            lat.append(data.latitude)
            lon.append(data.longitude)

    x_pts = np.array(x_pts)
    y_pts_xl_x = np.array(y_pts_xl_x)
    y_pts_xl_y = np.array(y_pts_xl_y)

    y_x_filt = moving_average(y_pts_xl_x, 100);
    y_y_filt = moving_average(y_pts_xl_y, 100);

    # plt.plot(x_pts, y_pts_xl_x, label="xl_x")
    # plt.plot(x_pts, y_pts_xl_y, label="xl_y")

    plt.plot(y_x_filt, label="xl_x_filt")
    plt.plot(y_y_filt, label="xl_y_filt")
    
    plt.legend(loc='upper left')

    plt.figure()

    # create second plot (gps data)
    plt.scatter(x=lon, y=lat)
    plt.xlabel("longitude")
    plt.ylabel("latitude")
    plt.title("GPS Data")
    plt.show()

if __name__ == "__main__":
    plotdata = None
    with open(DATAFILE, "r") as datafile:
        plotdata = parse_data(datafile, OUTPATH)

    plot_accel_xy(plotdata)
