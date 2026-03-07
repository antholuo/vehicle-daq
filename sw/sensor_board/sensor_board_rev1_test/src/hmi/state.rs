use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::Instant;

/// Bridge capture LED phase: 0 = Idle, 1 = RequestSent (orange), 2 = Success (blue 0.5s), 3 = Timeout (rainbow 1s)
pub const CAPTURE_PHASE_IDLE: u8 = 0;
pub const CAPTURE_PHASE_REQUEST_SENT: u8 = 1;
pub const CAPTURE_PHASE_SUCCESS: u8 = 2;
pub const CAPTURE_PHASE_TIMEOUT: u8 = 3;

pub struct HmiStateData {
    pub last_imu_timestamp: Option<Instant>,
    pub gps_fix: bool,
    pub last_gps_timestamp: Option<Instant>,
    pub last_gps_rx_timestamp: Option<Instant>,
    /// When true, floating node shows solid orange 50ms then rapid orange blink (acquiring GPS)
    pub acquiring_gps_capture: bool,
    /// Bridge: track capture armed (double blink rate)
    pub track_armed: bool,
    /// Bridge: capture LED phase (Idle / RequestSent / Success / Timeout)
    pub capture_led_phase: u8,
    /// When to clear Success/Timeout phase (return to Idle)
    pub capture_phase_deadline: Option<Instant>,
}

impl Default for HmiStateData {
    fn default() -> Self {
        Self {
            last_imu_timestamp: None,
            gps_fix: false,
            last_gps_timestamp: None,
            last_gps_rx_timestamp: None,
            acquiring_gps_capture: false,
            track_armed: false,
            capture_led_phase: CAPTURE_PHASE_IDLE,
            capture_phase_deadline: None,
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
            track_armed: false,
            capture_led_phase: CAPTURE_PHASE_IDLE,
            capture_phase_deadline: None,
        }))
    }
}

// Public global HMI state that tasks can update/read
pub static HMI_STATE: HmiState = HmiState::new();
