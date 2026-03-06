use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use embassy_time::Instant;

// Program start timestamp in milliseconds since an arbitrary epoch (Instant::now()).
// Using u32 to remain compatible with no_std targets; overflow after ~49 days.
static PROGRAM_START_MS: AtomicU32 = AtomicU32::new(0);

/// Initialize the program start time (call once early during startup)
pub fn set_program_start() {
    let now_ms = (Instant::now().as_millis() as u64) as u32;
    PROGRAM_START_MS.store(now_ms, Ordering::SeqCst);
}

/// Returns the program start timestamp (milliseconds)
pub fn program_start_ms() -> u32 {
    PROGRAM_START_MS.load(Ordering::SeqCst)
}

/// Returns elapsed seconds (as f32) since program start using Instant::now()
pub fn elapsed_seconds() -> f32 {
    let now_ms = (Instant::now().as_millis() as u64) as u32;
    let start = PROGRAM_START_MS.load(Ordering::SeqCst);
    let elapsed_ms = now_ms.wrapping_sub(start) as u32;
    (elapsed_ms as f32) / 1000.0
}

// =========================================================================
// Time synchronization
// =========================================================================
//
// The sync offset converts local monotonic time to the RPi's session-elapsed
// time.  Stored as two AtomicU32 halves (ESP32-C6 is 32-bit RISC-V, no
// native 64-bit atomics) representing a signed i64 microsecond offset:
//
//   synced_us = local_us as i64 + offset
//
// Reads/writes use SeqCst ordering; tiny races between the two halves are
// acceptable because apply_sync is called periodically and the offset
// changes slowly (drift correction).

static SYNC_OFFSET_LO: AtomicU32 = AtomicU32::new(0);
static SYNC_OFFSET_HI: AtomicU32 = AtomicU32::new(0);
static TIME_SYNCED: AtomicBool = AtomicBool::new(false);

fn store_offset(offset: i64) {
    let bits = offset as u64;
    SYNC_OFFSET_LO.store(bits as u32, Ordering::SeqCst);
    SYNC_OFFSET_HI.store((bits >> 32) as u32, Ordering::SeqCst);
    TIME_SYNCED.store(true, Ordering::SeqCst);
}

fn load_offset() -> i64 {
    let lo = SYNC_OFFSET_LO.load(Ordering::SeqCst) as u64;
    let hi = SYNC_OFFSET_HI.load(Ordering::SeqCst) as u64;
    ((hi << 32) | lo) as i64
}

/// Compute and store the sync offset from an RPi session timestamp.
///
/// Call this when a TimeSync message arrives.  The offset is:
///   `rpi_session_time_us - Instant::now().as_micros()`
///
/// Subsequent calls update the offset (drift correction).
pub fn apply_sync(rpi_session_time_us: u64) {
    let local_us = Instant::now().as_micros() as i64;
    let offset = rpi_session_time_us as i64 - local_us;
    store_offset(offset);
}

/// Return a synchronized timestamp in microseconds.
///
/// If time sync has been applied, returns `local_us + offset` (the RPi
/// session-elapsed time).  Otherwise falls back to raw `Instant::now()`.
pub fn synced_timestamp_us() -> u64 {
    if TIME_SYNCED.load(Ordering::SeqCst) {
        let local_us = Instant::now().as_micros() as i64;
        (local_us + load_offset()) as u64
    } else {
        Instant::now().as_micros()
    }
}

/// Returns `true` once at least one TimeSync has been applied.
pub fn is_time_synced() -> bool {
    TIME_SYNCED.load(Ordering::SeqCst)
}
