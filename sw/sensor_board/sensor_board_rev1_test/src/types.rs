/// types.rs
/// Defines all of the common datatypes (IMU/GPS/FUSE/etc)

/// IMU sensor data structure
#[derive(Debug, Clone, Copy)]
pub struct ImuData {
    // TODO: IMU data fields
}

/// GPS timestamp
#[derive(Debug, Clone, Copy)]
pub struct GpsTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
    pub millis: u16,
}

/// GPS sensor data structure
#[derive(Debug, Clone, Copy)]
pub struct GpsData {
    pub lat: f64,
    pub lon: f64,
    pub alt: f32,
    pub speed_kts: f32,
    pub heading: u16,
    pub utc_time: GpsTime,
}
