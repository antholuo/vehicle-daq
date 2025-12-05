/// types.rs
/// Defines all of the common datatypes (IMU/GPS/FUSE/etc)

pub struct GpsTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
    pub millis: u16,
}

pub struct GpsData {
    pub lat: f64,
    pub lon: f64,
    pub alt: f32,
    pub speed_kts: f32,
    pub heading: u16,
    pub utc_time: GpsTime,
}
