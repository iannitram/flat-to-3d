//! Operating-system facts used by the engine.

/// The current user id on Unix (zero on platforms without Unix ids).
pub fn current_uid() -> u32 {
    #[cfg(unix)]
    {
        unsafe extern "C" {
            fn geteuid() -> u32;
        }
        // SAFETY: geteuid has no preconditions and does not retain pointers.
        unsafe { geteuid() }
    }
    #[cfg(not(unix))]
    {
        0
    }
}

/// Seconds since the Unix epoch, or zero if the clock predates it.
pub fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}
