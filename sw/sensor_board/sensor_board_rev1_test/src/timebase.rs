use embassy_time::Instant;

// Program start timestamp in milliseconds since an arbitrary epoch (Instant::now()).
// This is safe because we only write once at startup (set_program_start)
// and read many times afterwards. Single-threaded embedded environment.
static mut PROGRAM_START_MS: u64 = 0;

/// Initialize the program start time (call once early during startup)
pub fn set_program_start() {
    // SAFETY: Safe because called once at startup before any other threads access it
    unsafe {
        PROGRAM_START_MS = Instant::now().as_millis();
    }
}

/// Returns the program start timestamp (milliseconds)
pub fn program_start_ms() -> u64 {
    // SAFETY: Safe for reading as-is in single-threaded context
    unsafe { PROGRAM_START_MS }
}

/// Returns elapsed seconds (as f32) since program start using Instant::now()
pub fn elapsed_seconds() -> f32 {
    let now_ms = Instant::now().as_millis();
    // SAFETY: Safe for reading as-is in single-threaded context
    let start = unsafe { PROGRAM_START_MS };
    let elapsed_ms = now_ms - start;
    (elapsed_ms as f32) / 1000.0
}
