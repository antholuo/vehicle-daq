use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::Instant;

pub struct HmiStateData {
    pub last_imu_timestamp: Option<Instant>,
    pub gps_fix: bool,
    pub last_gps_timestamp: Option<Instant>,
    pub last_gps_rx_timestamp: Option<Instant>,
    /// When true, floating node shows solid orange 50ms then rapid orange blink (acquiring GPS)
    pub acquiring_gps_capture: bool,
}

impl Default for HmiStateData {
    fn default() -> Self {
        Self {
            last_imu_timestamp: None,
            gps_fix: false,
            last_gps_timestamp: None,
            last_gps_rx_timestamp: None,
            acquiring_gps_capture: false,
        }
    }
}

pub struct HmiState(pub Mutex<CriticalSectionRawMutex, HmiStateData>);

impl HmiState {
    pub const fn new() -> Self {
        Self(Mutex::new(HmiStateData {
            last_imu_timestamp: None,
            gps_fix: false,
            last_gps_timestamp: None,
            last_gps_rx_timestamp: None,
            acquiring_gps_capture: false,
        }))
    }
}

// Public global HMI state that tasks can update/read
pub static HMI_STATE: HmiState = HmiState::new();
