/// types.rs
/// Defines all of the common datatypes (IMU/GPS/FUSE/etc)

// =============================================================================
// Node Identification
// =============================================================================

/// Position where sensor node is installed on the car
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum CarPosition {
    FrontLeft = 0,
    FrontCenter = 1,
    FrontRight = 2,
    Left = 3,
    Center = 4,
    Right = 5,
    RearLeft = 6,
    RearCenter = 7,
    RearRight = 8,
    Roof = 9,
    /// Custom/unassigned position
    Custom = 255,
}

impl CarPosition {
    /// Convert from u8, returns Custom for unknown values
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => CarPosition::FrontLeft,
            1 => CarPosition::FrontCenter,
            2 => CarPosition::FrontRight,
            3 => CarPosition::Left,
            4 => CarPosition::Center,
            5 => CarPosition::Right,
            6 => CarPosition::RearLeft,
            7 => CarPosition::RearCenter,
            8 => CarPosition::RearRight,
            9 => CarPosition::Roof,
            _ => CarPosition::Custom,
        }
    }

    /// Convert to u8
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    /// Get human-readable name
    pub fn as_str(&self) -> &'static str {
        match self {
            CarPosition::FrontLeft => "FrontLeft",
            CarPosition::FrontCenter => "FrontCenter",
            CarPosition::FrontRight => "FrontRight",
            CarPosition::Left => "Left",
            CarPosition::Center => "Center",
            CarPosition::Right => "Right",
            CarPosition::RearLeft => "RearLeft",
            CarPosition::RearCenter => "RearCenter",
            CarPosition::RearRight => "RearRight",
            CarPosition::Roof => "Roof",
            CarPosition::Custom => "Custom",
        }
    }
}

/// Full node identifier combining car position and instance number
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeId {
    /// Position on the car where sensor is installed
    pub position: CarPosition,
    /// Instance number (0-7) for multiple nodes at same position
    pub instance: u8,
}

impl NodeId {
    /// Create a new NodeId
    pub const fn new(position: CarPosition, instance: u8) -> Self {
        Self { position, instance }
    }

    /// Serialize to 2 bytes: [position, instance]
    pub fn to_bytes(&self) -> [u8; 2] {
        [self.position.to_u8(), self.instance]
    }

    /// Deserialize from 2 bytes
    pub fn from_bytes(bytes: [u8; 2]) -> Self {
        Self {
            position: CarPosition::from_u8(bytes[0]),
            instance: bytes[1],
        }
    }
}

impl Default for NodeId {
    fn default() -> Self {
        Self {
            position: configured_position(),
            instance: 0,
        }
    }
}

/// Get the car position configured via Cargo features.
///
/// Set at compile time using one of: `pos_front_left`, `pos_center`, etc.
/// Defaults to `Custom` if no position feature is enabled.
///
/// Example build command:
/// ```bash
/// cargo build --bin rev1_board --features "pos_front_left"
/// ```
pub const fn configured_position() -> CarPosition {
    #[cfg(feature = "pos_front_left")]
    {
        return CarPosition::FrontLeft;
    }
    #[cfg(feature = "pos_front_center")]
    {
        return CarPosition::FrontCenter;
    }
    #[cfg(feature = "pos_front_right")]
    {
        return CarPosition::FrontRight;
    }
    #[cfg(feature = "pos_left")]
    {
        return CarPosition::Left;
    }
    #[cfg(feature = "pos_center")]
    {
        return CarPosition::Center;
    }
    #[cfg(feature = "pos_right")]
    {
        return CarPosition::Right;
    }
    #[cfg(feature = "pos_rear_left")]
    {
        return CarPosition::RearLeft;
    }
    #[cfg(feature = "pos_rear_center")]
    {
        return CarPosition::RearCenter;
    }
    #[cfg(feature = "pos_rear_right")]
    {
        return CarPosition::RearRight;
    }
    #[cfg(feature = "pos_roof")]
    {
        return CarPosition::Roof;
    }

    // Default when no position feature is set
    #[allow(unreachable_code)]
    CarPosition::Custom
}

// =============================================================================
// Sensor Data Types
// =============================================================================

/// IMU sensor data structure
#[derive(Debug, Clone, Copy)]
pub struct ImuData {
    /// Accelerometer X-axis in g
    pub accel_x: f32,
    /// Accelerometer Y-axis in g
    pub accel_y: f32,
    /// Accelerometer Z-axis in g
    pub accel_z: f32,
    /// Gyroscope X-axis in degrees/second
    pub gyro_x: f32,
    /// Gyroscope Y-axis in degrees/second
    pub gyro_y: f32,
    /// Gyroscope Z-axis in degrees/second
    pub gyro_z: f32,
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
