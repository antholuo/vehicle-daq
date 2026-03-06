//! AHRS module: Madgwick orientation, velocity integration, synthesized GPS, ground track.
//!
//! Assumes first 10 seconds of data are with vehicle at rest (for gyro bias and velocity zeroing).

use ahrs::{Ahrs, Madgwick};
use nalgebra::{UnitQuaternion, Vector3};
use serde::Serialize;

const REST_PERIOD_US: u64 = 2 * 1_000_000; // 2 seconds (steady state for calibration)
const GRAVITY_MPS2: f64 = 9.80665;
const DEG2RAD: f64 = std::f64::consts::PI / 180.0;
const RAD2DEG: f64 = 180.0 / std::f64::consts::PI;
/// Meters per degree latitude (approximate at mid-latitudes)
const M_PER_DEG_LAT: f64 = 111_320.0;

/// Transform from sensor frame to NED body frame (X forward, Y right, Z down).
/// ASM330LHH on flat PCB: Z often points out of board (up). NED has Z down.
/// Negating Z aligns sensor with NED body frame.
#[inline]
fn sensor_to_ned_body(vx: f64, vy: f64, vz: f64) -> (f64, f64, f64) {
    (vx, vy, -vz)
}

/// One row of AHRS output CSV: synthesized GPS, vehicle heading, ground track, orientation, velocity.
#[derive(Debug, Clone, Serialize)]
pub struct AhrsOutputRow {
    pub timestamp_us: u64,
    /// Synthesized latitude (degrees)
    pub lat_synth: f64,
    /// Synthesized longitude (degrees)
    pub lon_synth: f64,
    /// Synthesized altitude (m)
    pub alt_synth: f64,
    /// Vehicle heading (yaw, where nose points) in degrees [0, 360)
    pub vehicle_heading_deg: f32,
    /// Ground track (direction of velocity) in degrees [0, 360); differs from heading when sliding
    pub ground_track_deg: f32,
    pub roll_deg: f32,
    pub pitch_deg: f32,
    pub yaw_deg: f32,
    pub v_north_mps: f64,
    pub v_east_mps: f64,
    pub v_down_mps: f64,
    /// Speed in m/s (horizontal)
    pub speed_mps: f64,
}

fn quat_to_euler_deg(q: &UnitQuaternion<f64>) -> (f32, f32, f32) {
    let (q_w, q_i, q_j, q_k) = (q.scalar(), q.i, q.j, q.k);
    let sinp = 2.0 * (q_w * q_j - q_k * q_i);
    let pitch_rad = if sinp.abs() >= 1.0 {
        std::f64::consts::FRAC_PI_2.copysign(sinp)
    } else {
        sinp.asin()
    };
    let roll_rad = (2.0 * (q_w * q_i + q_j * q_k)).atan2(1.0 - 2.0 * (q_i * q_i + q_j * q_j));
    let yaw_rad = (2.0 * (q_w * q_k + q_i * q_j)).atan2(1.0 - 2.0 * (q_j * q_j + q_k * q_k));
    (
        (roll_rad * RAD2DEG) as f32,
        (pitch_rad * RAD2DEG) as f32,
        (yaw_rad * RAD2DEG) as f32,
    )
}

/// Normalize angle in degrees to [0, 360).
fn wrap_deg_360(deg: f32) -> f32 {
    let mut d = deg % 360.0;
    if d < 0.0 {
        d += 360.0;
    }
    d
}

/// AHRS filter: Madgwick orientation + velocity/position integration in NED.
/// First REST_PERIOD_US (10 s) is treated as at-rest (gyro bias estimated, velocity zeroed at end).
pub struct AhrsFilter {
    madgwick: Madgwick<f64>,
    last_ts_us: u64,
    first_ts_us: Option<u64>,
    /// Velocity in NED (m/s)
    v_ned: [f64; 3],
    /// Position in NED from origin (m)
    p_ned: [f64; 3],
    origin_lat: f64,
    origin_lon: f64,
    origin_alt: f64,
    origin_set: bool,
    /// Gyro bias accumulated during rest period (rad/s)
    gyro_bias_sum: [f64; 3],
    gyro_bias_n: u32,
    /// After rest period we zero velocity once
    velocity_zeroed: bool,
}

impl AhrsFilter {
    /// Create a new AHRS filter. `sample_period_s` is nominal IMU period (e.g. 1/400).
    pub fn new(sample_period_s: f64, beta: f64) -> Self {
        Self {
            madgwick: Madgwick::new(sample_period_s, beta),
            last_ts_us: 0,
            first_ts_us: None,
            v_ned: [0.0; 3],
            p_ned: [0.0; 3],
            origin_lat: 0.0,
            origin_lon: 0.0,
            origin_alt: 0.0,
            origin_set: false,
            gyro_bias_sum: [0.0; 3],
            gyro_bias_n: 0,
            velocity_zeroed: false,
        }
    }

    /// Set origin for synthesized lat/lon/alt (e.g. from first GPS fix).
    pub fn set_origin(&mut self, lat: f64, lon: f64, alt: f64) {
        self.origin_lat = lat;
        self.origin_lon = lon;
        self.origin_alt = alt;
        self.origin_set = true;
    }

    /// Update with one IMU sample. Accel in g, gyro in deg/s.
    /// If `data_in_vehicle_frame` is true, accel/gyro are already in NED body frame (e.g. after
    /// board-to-vehicle calibration). Otherwise, applies sensor_to_ned_body (Z-up PCB fix).
    pub fn update_imu_with_frame(
        &mut self,
        timestamp_us: u64,
        accel_x_g: f32,
        accel_y_g: f32,
        accel_z_g: f32,
        gyro_x_dps: f32,
        gyro_y_dps: f32,
        gyro_z_dps: f32,
        data_in_vehicle_frame: bool,
    ) -> bool {
        let (ax_s, ay_s, az_s, gx_s, gy_s, gz_s) = if data_in_vehicle_frame {
            (
                accel_x_g as f64,
                accel_y_g as f64,
                accel_z_g as f64,
                gyro_x_dps as f64,
                gyro_y_dps as f64,
                gyro_z_dps as f64,
            )
        } else {
            let (ax, ay, az) = sensor_to_ned_body(
                accel_x_g as f64,
                accel_y_g as f64,
                accel_z_g as f64,
            );
            let (gx, gy, gz) = sensor_to_ned_body(
                gyro_x_dps as f64,
                gyro_y_dps as f64,
                gyro_z_dps as f64,
            );
            (ax, ay, az, gx, gy, gz)
        };
        self.update_imu_internal(timestamp_us, ax_s, ay_s, az_s, gx_s, gy_s, gz_s)
    }

    /// Update with one IMU sample (raw sensor data, applies Z-up PCB fix).
    pub fn update_imu(
        &mut self,
        timestamp_us: u64,
        accel_x_g: f32,
        accel_y_g: f32,
        accel_z_g: f32,
        gyro_x_dps: f32,
        gyro_y_dps: f32,
        gyro_z_dps: f32,
    ) -> bool {
        self.update_imu_with_frame(
            timestamp_us,
            accel_x_g,
            accel_y_g,
            accel_z_g,
            gyro_x_dps,
            gyro_y_dps,
            gyro_z_dps,
            false,
        )
    }

    fn update_imu_internal(
        &mut self,
        timestamp_us: u64,
        ax_s: f64,
        ay_s: f64,
        az_s: f64,
        gx_s: f64,
        gy_s: f64,
        gz_s: f64,
    ) -> bool {
        if self.first_ts_us.is_none() {
            self.first_ts_us = Some(timestamp_us);
        }
        let first_ts = self.first_ts_us.unwrap();
        let dt_s = (timestamp_us - self.last_ts_us) as f64 / 1_000_000.0;
        self.last_ts_us = timestamp_us;

        // Clamp dt for first sample or gaps
        let dt_s = if dt_s <= 0.0 || dt_s > 1.0 {
            1.0 / 400.0
        } else {
            dt_s.min(0.1)
        };

        let in_rest = timestamp_us < first_ts + REST_PERIOD_US;

        let mut gx = gx_s * DEG2RAD;
        let mut gy = gy_s * DEG2RAD;
        let mut gz = gz_s * DEG2RAD;
        if in_rest {
            self.gyro_bias_sum[0] += gx;
            self.gyro_bias_sum[1] += gy;
            self.gyro_bias_sum[2] += gz;
            self.gyro_bias_n += 1;
        } else {
            if self.gyro_bias_n > 0 {
                let n = self.gyro_bias_n as f64;
                gx -= self.gyro_bias_sum[0] / n;
                gy -= self.gyro_bias_sum[1] / n;
                gz -= self.gyro_bias_sum[2] / n;
            }
        }

        let gyro = Vector3::new(gx, gy, gz);
        // Accel: normalize to unit vector for Madgwick (direction only), use NED body frame
        let (ax, ay, az) = (ax_s, ay_s, az_s);
        let norm = (ax * ax + ay * ay + az * az).sqrt();
        let (ax, ay, az) = if norm > 1e-6 {
            (ax / norm, ay / norm, az / norm)
        } else {
            (0.0, 0.0, 1.0)
        };
        let accel = Vector3::new(ax, ay, az);

        // Use actual dt for integration - IMU rate varies (multi-node, ~10-50 Hz per node)
        *self.madgwick.sample_period_mut() = dt_s;
        if self.madgwick.update_imu(&gyro, &accel).is_err() {
            return false;
        }

        let q = &self.madgwick.quat;
        // Body accel in g (NED body frame) -> m/s^2
        let ab = Vector3::new(ax_s, ay_s, az_s);
        let a_body_mps2 = ab * GRAVITY_MPS2;
        // Body to NED: a_ned = q * a_body (Madgwick q is body w.r.t. NED)
        let a_ned_measured = q.transform_vector(&a_body_mps2);
        // Kinematic accel: a = f_measured + g_ned, g_ned = [0, 0, -9.81]
        let a_ned = a_ned_measured + Vector3::new(0.0, 0.0, -GRAVITY_MPS2);

        if in_rest {
            // Don't integrate velocity during rest
            return true;
        }

        if !self.velocity_zeroed {
            self.v_ned = [0.0; 3];
            self.velocity_zeroed = true;
        }

        // Integrate velocity
        self.v_ned[0] += a_ned.x * dt_s;
        self.v_ned[1] += a_ned.y * dt_s;
        self.v_ned[2] += a_ned.z * dt_s;

        // Integrate position (NED: North, East, Down)
        self.p_ned[0] += self.v_ned[0] * dt_s;
        self.p_ned[1] += self.v_ned[1] * dt_s;
        self.p_ned[2] += self.v_ned[2] * dt_s;

        true
    }

    /// Build current state as an output row. Call after update_imu when emitting at fixed rate.
    pub fn get_output_row(&self, timestamp_us: u64) -> AhrsOutputRow {
        let (roll_deg, pitch_deg, yaw_deg) = quat_to_euler_deg(&self.madgwick.quat);
        let vehicle_heading_deg = wrap_deg_360(yaw_deg);

        let v_n = self.v_ned[0];
        let v_e = self.v_ned[1];
        let _v_d = self.v_ned[2];
        let speed_mps = (v_n * v_n + v_e * v_e).sqrt();
        let ground_track_deg = if speed_mps < 1e-3 {
            vehicle_heading_deg
        } else {
            wrap_deg_360((-v_e.atan2(v_n) * RAD2DEG) as f32)
        };

        let (lat_synth, lon_synth, alt_synth) = if self.origin_set {
            let lat_rad = self.origin_lat * DEG2RAD;
            let m_per_deg_lon = M_PER_DEG_LAT * lat_rad.cos();
            let lat = self.origin_lat + (self.p_ned[0] / M_PER_DEG_LAT);
            let lon = self.origin_lon + (self.p_ned[1] / m_per_deg_lon);
            let alt = self.origin_alt - self.p_ned[2]; // NED: down positive
            (lat, lon, alt)
        } else {
            (0.0, 0.0, 0.0)
        };

        AhrsOutputRow {
            timestamp_us,
            lat_synth,
            lon_synth,
            alt_synth,
            vehicle_heading_deg,
            ground_track_deg,
            roll_deg,
            pitch_deg,
            yaw_deg,
            v_north_mps: self.v_ned[0],
            v_east_mps: self.v_ned[1],
            v_down_mps: self.v_ned[2],
            speed_mps,
        }
    }

    /// Whether we have set an origin (so lat_synth/lon_synth are meaningful).
    pub fn has_origin(&self) -> bool {
        self.origin_set
    }

    /// Vehicle heading (yaw) in degrees [0, 360) from orientation only.
    /// Use for IMU-derived heading (e.g. for slide detection vs GPS heading).
    pub fn get_vehicle_heading_deg(&self) -> f32 {
        let (_, _, yaw_deg) = quat_to_euler_deg(&self.madgwick.quat);
        wrap_deg_360(yaw_deg)
    }

    /// Correct position and velocity from a GPS fix to prevent inertial drift.
    /// Resets p_ned and v_ned to match GPS. Requires origin to be set.
    pub fn correct_from_gps(
        &mut self,
        lat: f64,
        lon: f64,
        alt: f64,
        speed_kts: f32,
        heading_deg: f32,
    ) {
        if !self.origin_set {
            return;
        }
        let lat_rad = self.origin_lat * DEG2RAD;
        let m_per_deg_lon = M_PER_DEG_LAT * lat_rad.cos();
        self.p_ned[0] = (lat - self.origin_lat) * M_PER_DEG_LAT;
        self.p_ned[1] = (lon - self.origin_lon) * m_per_deg_lon;
        self.p_ned[2] = self.origin_alt - alt;

        let speed_mps = speed_kts as f64 * 0.514444;
        let h_rad = heading_deg as f64 * DEG2RAD;
        self.v_ned[0] = speed_mps * h_rad.cos();
        self.v_ned[1] = speed_mps * h_rad.sin();
    }

    /// Whether we're past the rest period (velocity integration active).
    pub fn past_rest_period(&self, timestamp_us: u64) -> bool {
        self.first_ts_us
            .map(|t| timestamp_us >= t + REST_PERIOD_US)
            .unwrap_or(false)
    }
}
