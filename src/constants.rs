//! Constants used throughout the memo application

/// Permission mode for cache directory (owner read/write/execute only)
#[cfg(unix)]
pub const CACHE_DIR_PERMISSIONS: u32 = 0o700;

/// Permission mode for cache files (owner read/write only)
#[cfg(unix)]
pub const FILE_PERMISSIONS: u32 = 0o600;

/// Timeout in seconds for waiting to acquire lock on concurrent memoization
pub const LOCK_WAIT_TIMEOUT_SECS: u64 = 2;

/// Interval in milliseconds between lock acquisition retries
pub const LOCK_WAIT_INTERVAL_MS: u64 = 25;
