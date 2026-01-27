use core::sync::atomic::{AtomicU32, Ordering};
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
