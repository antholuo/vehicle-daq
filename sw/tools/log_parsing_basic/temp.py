import re
import numpy as np
import pandas as pd

from ahrs.filters import Madgwick
from pyproj import Geod

# ------------------------------------------------------------
# 1. LOG PARSER
# ------------------------------------------------------------

def parse_log_file(path):
    imu_rows = []
    gps_pos_rows = []
    gps_vel_rows = []

    imu_re = re.compile(
        r"t=(?P<t>[0-9.]+)s.*?Accel: X=(?P<ax>[0-9.\-]+)g Y=(?P<ay>[0-9.\-]+)g Z=(?P<az>[0-9.\-]+)g "
        r"Gyro: X=(?P<gx>[0-9.\-]+)dps Y=(?P<gy>[0-9.\-]+)dps Z=(?P<gz>[0-9.\-]+)dps"
    )

    gga_re = re.compile(
        r"t=(?P<t>[0-9.]+)s.*?Lat=(?P<lat>[0-9.\-]+), Lon=(?P<lon>[0-9.\-]+), HDOP=(?P<hdop>[0-9.]+)"
    )

    vtg_re = re.compile(
        r"t=(?P<t>[0-9.]+)s.*?Speed=(?P<speed>[0-9.\-]+) knots.*?Heading:\s*(?P<head>[0-9.\-]+)"
    )

    with open(path, "r", errors="ignore") as f:
        for raw in f:
            # strip ANSI color codes if present
            line = re.sub(r"\x1b\[[0-9;]*m", "", raw)

            m = imu_re.search(line)
            if m:
                imu_rows.append({
                    "timestamp": float(m.group("t")),
                    "ax": float(m.group("ax")) * 9.80665,
                    "ay": float(m.group("ay")) * 9.80665,
                    "az": float(m.group("az")) * 9.80665,

                    "gx": np.radians(float(m.group("gx"))),
                    "gy": np.radians(float(m.group("gy"))),
                    "gz": np.radians(float(m.group("gz")))
                })
                continue

            m = gga_re.search(line)
            if m:
                gps_pos_rows.append({
                    "timestamp": float(m.group("t")),
                    "lat": float(m.group("lat")),
                    "lon": float(m.group("lon")),
                    "accuracy": float(m.group("hdop")) * 5.0
                })
                continue

            m = vtg_re.search(line)
            if m:
                gps_vel_rows.append({
                    "timestamp": float(m.group("t")),
                    "speed": float(m.group("speed")) * 0.514444,
                    "heading": float(m.group("head"))
                })
                continue

    imu_df = pd.DataFrame(imu_rows).sort_values("timestamp")
    gps_pos_df = pd.DataFrame(gps_pos_rows).sort_values("timestamp")
    gps_vel_df = pd.DataFrame(gps_vel_rows).sort_values("timestamp")

    # Merge nearest position to IMU
    if not gps_pos_df.empty:
        imu_pos = pd.merge_asof(
            imu_df, gps_pos_df,
            on="timestamp",
            direction="backward",
            tolerance=1.0  # 1s tolerance for position
        )
        imu_pos[["lat", "lon", "accuracy"]] = imu_pos[["lat", "lon", "accuracy"]].bfill().ffill()
    else:
        imu_pos = imu_df.copy()
        imu_pos["lat"] = np.nan
        imu_pos["lon"] = np.nan
        imu_pos["accuracy"] = 10.0

    # Merge nearest velocity to IMU
    if not gps_vel_df.empty:
        df = pd.merge_asof(
            imu_pos, gps_vel_df,
            on="timestamp",
            direction="backward",
            tolerance=1.0  # 1s tolerance for velocity
        )
        df["speed"] = df["speed"].fillna(0.0)
        df["heading"] = df["heading"].fillna(0.0)
    else:
        df = imu_pos.copy()
        df["speed"] = 0.0
        df["heading"] = 0.0

    df["alt"] = 0.0
    return df




# ------------------------------------------------------------
# 2. AHRS + POSITION FUSION
# ------------------------------------------------------------
def quat_to_rotation_matrix(q):
    w, x, y, z = q
    return np.array([
        [1-2*(y*y+z*z), 2*(x*y-w*z),   2*(x*z+w*y)],
        [2*(x*y+w*z),   1-2*(x*x+z*z), 2*(y*z-w*x)],
        [2*(x*z-w*y),   2*(y*z+w*x),   1-2*(x*x+y*y)]
    ])

def run_ahrs_and_resample(df):
    madgwick = Madgwick()
    geod = Geod(ellps="WGS84")

    df = df.sort_values("timestamp").reset_index(drop=True)

    t_start = df["timestamp"].iloc[0]
    t_end   = df["timestamp"].iloc[-1]
    t_out = np.arange(t_start, t_end, 0.1)

    interp = {}
    for col in ["ax","ay","az","gx","gy","gz","lat","lon","alt","accuracy"]:
        interp[col] = np.interp(t_out, df["timestamp"], df[col])

    q = np.array([1.0, 0.0, 0.0, 0.0])
    vel = np.array([0.0, 0.0, 0.0])

    last_lat = interp["lat"][0]
    last_lon = interp["lon"][0]
    last_alt = interp["alt"][0]

    out_rows = []

    for i in range(len(t_out)-1):
        dt = t_out[i+1] - t_out[i]

        ax, ay, az = interp["ax"][i], interp["ay"][i], interp["az"][i]
        gx, gy, gz = interp["gx"][i], interp["gy"][i], interp["gz"][i]

        q = madgwick.updateIMU(q, gyr=[gx, gy, gz], acc=[ax, ay, az])

        qw, qx, qy, qz = q
        yaw = np.arctan2(2*(qw*qz + qx*qy), 1 - 2*(qy*qy + qz*qz))
        heading_deg = np.degrees(yaw) % 360

        g = np.array([0, 0, 9.80665])
        acc_body = np.array([ax, ay, az])

        R = quat_to_rotation_matrix(q)
        acc_world = R @ acc_body - g

        vel = vel + acc_world * dt

        d_north = vel[0] * dt
        d_east  = vel[1] * dt

        lon_new, lat_new, _ = geod.fwd(
            last_lon, last_lat,
            heading_deg,
            np.sqrt(d_north**2 + d_east**2)
        )

        last_lat = lat_new
        last_lon = lon_new
        last_alt = interp["alt"][i]

        out_rows.append([
            t_out[i],
            last_lat,
            last_lon,
            last_alt,
            np.linalg.norm(vel),
            heading_deg,
            interp["accuracy"][i],
            ax, ay, az
        ])

    out_df = pd.DataFrame(out_rows, columns=[
        "timestamp","lat","lon","altitude",
        "speed","direction","accuracy",
        "xl_x","xl_y","xl_z"
    ])

    return out_df

# ------------------------------------------------------------
# 3. MAIN ENTRY POINT
# ------------------------------------------------------------

if __name__ == "__main__":
    import sys

    if len(sys.argv) < 3:
        print("Usage: python fused_pipeline.py input_log.txt output.csv")
        sys.exit(1)

    log_path = sys.argv[1]
    out_path = sys.argv[2]

    print("Parsing log...")
    df = parse_log_file(log_path)

    print("Running AHRS + fusion...")
    fused = run_ahrs_and_resample(df)

    print("Saving CSV...")
    fused.to_csv(out_path, index=False)

    print("Done.")
