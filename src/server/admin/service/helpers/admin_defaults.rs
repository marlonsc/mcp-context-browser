//! Admin Operations Default Configuration
//!
//! Centralized default values for admin service operations.
//! All values are configurable via environment variables.
//!
//! ## Environment Variable Mapping
//!
//! Each constant can be overridden via environment variables using the pattern:
//! `ADMIN_<CONSTANT_NAME>` (e.g., `ADMIN_MAX_ACTIVITIES=200` to override `DEFAULT_MAX_ACTIVITIES`).
//!
//! ### Activity Feed Configuration
//! - `ADMIN_MAX_ACTIVITIES` - Default: 100 (max activities in memory)
//! - `ADMIN_ACTIVITY_RETENTION_DAYS` - Default: 30 (days to keep activity records)
//! - `ADMIN_ACTIVITY_BUFFER_SIZE` - Default: 1000 (activity buffer capacity)
//!
//! ### Configuration History
//! - `ADMIN_MAX_HISTORY_ENTRIES` - Default: 1000 (max config history entries)
//! - `ADMIN_HISTORY_RETENTION_DAYS` - Default: 90 (days to keep config history)
//! - `ADMIN_CONFIG_QUERY_LIMIT` - Default: 100 (max history entries per query)
//!
//! ### Logging Configuration
//! - `ADMIN_LOG_BUFFER_SIZE` - Default: 1000 (log buffer capacity)
//! - `ADMIN_LOG_RETENTION_DAYS` - Default: 7 (days to keep logs)
//! - `ADMIN_LOG_QUERY_LIMIT` - Default: 100 (max log entries per query)
//!
//! ### Backup Configuration
//! - `ADMIN_BACKUP_RETENTION_DAYS` - Default: 30 (days to keep backups)
//! - `ADMIN_BACKUP_COMPRESSION_LEVEL` - Default: 6 (gzip compression 1-9)
//! - `ADMIN_MAX_BACKUPS` - Default: 10 (max backup files to retain)
//!
//! ### Route Discovery Configuration
//! - `ADMIN_ROUTE_RATE_LIMIT_HEALTH` - Default: 100 (req/min for health endpoints)
//! - `ADMIN_ROUTE_RATE_LIMIT_ADMIN` - Default: 100 (req/min for admin endpoints)
//! - `ADMIN_ROUTE_RATE_LIMIT_INDEXING` - Default: 10 (req/min for indexing)
//! - `ADMIN_ROUTE_RATE_LIMIT_SEARCH` - Default: 10 (req/min for search)
//! - `ADMIN_ROUTE_RATE_LIMIT_SHUTDOWN` - Default: 60 (seconds cooldown for shutdown)
//! - `ADMIN_ROUTE_RATE_LIMIT_RELOAD` - Default: 30 (seconds cooldown for reload)
//! - `ADMIN_ROUTE_RATE_LIMIT_BACKUP` - Default: 60 (seconds cooldown for backup)
//! - `ADMIN_ROUTE_RATE_LIMIT_RESTORE` - Default: 10 (req/min for restore)
//!
//! ### Maintenance Operations
//! - `ADMIN_CLEANUP_BATCH_SIZE` - Default: 100 (items to process per cleanup batch)
//! - `ADMIN_CLEANUP_RETENTION_DAYS` - Default: 30 (days before cleanup targets)
//! - `ADMIN_INDEX_REBUILD_TIMEOUT_SECS` - Default: 3600 (1 hour timeout)
//! - `ADMIN_CACHE_CLEAR_TIMEOUT_SECS` - Default: 300 (5 minute timeout)
//!
//! ### Performance Testing
//! - `ADMIN_PERF_TEST_DURATION_SECS` - Default: 30 (test duration in seconds)
//! - `ADMIN_PERF_TEST_CONCURRENCY` - Default: 4 (concurrent test threads)
//! - `ADMIN_PERF_TEST_TIMEOUT_MS` - Default: 5000 (per-request timeout in ms)
//!
//! ### Directory Configuration
//! - `ADMIN_BACKUPS_DIR` - Default: "./backups" (backup storage directory)
//! - `ADMIN_DATA_DIR` - Default: "./data" (data directory)
//! - `ADMIN_EXPORTS_DIR` - Default: "./exports" (log export directory)
//!
//! ## Example Usage
//!
//! ```bash
//! # Override activity buffer to 200 items
//! export ADMIN_MAX_ACTIVITIES=200
//! # Override backup retention to 60 days
//! export ADMIN_BACKUP_RETENTION_DAYS=60
//! # Start application with custom values
//! ./mcp-context-browser
//! ```

use crate::infrastructure::constants::{
    ADMIN_ACTIVITY_BUFFER_SIZE, ADMIN_ACTIVITY_RETENTION_DAYS, ADMIN_BACKUPS_DIR,
    ADMIN_BACKUP_COMPRESSION_LEVEL, ADMIN_BACKUP_RETENTION_DAYS, ADMIN_CONFIG_QUERY_LIMIT,
    ADMIN_DATA_DIR, ADMIN_EXPORTS_DIR, ADMIN_HISTORY_RETENTION_DAYS, ADMIN_LOG_BUFFER_SIZE,
    ADMIN_LOG_QUERY_LIMIT, ADMIN_LOG_RETENTION_DAYS, ADMIN_MAX_ACTIVITIES, ADMIN_MAX_BACKUPS,
    ADMIN_MAX_HISTORY_ENTRIES, CACHE_CLEAR_TIMEOUT_SECS, CLEANUP_BATCH_SIZE,
    CLEANUP_RETENTION_DAYS, INDEX_REBUILD_TIMEOUT_SECS, PERF_TEST_CONCURRENCY,
    PERF_TEST_DURATION_SECS, PERF_TEST_TIMEOUT_MS, RATE_LIMIT_ADMIN, RATE_LIMIT_BACKUP_COOLDOWN,
    RATE_LIMIT_HEALTH, RATE_LIMIT_INDEXING, RATE_LIMIT_RELOAD_COOLDOWN, RATE_LIMIT_RESTORE,
    RATE_LIMIT_SEARCH, RATE_LIMIT_SHUTDOWN_COOLDOWN,
};

// Activity Feed Configuration
/// Maximum number of activities to keep in memory
pub const DEFAULT_MAX_ACTIVITIES: usize = ADMIN_MAX_ACTIVITIES;
/// Number of days to retain activity entries
pub const DEFAULT_ACTIVITY_RETENTION_DAYS: u32 = ADMIN_ACTIVITY_RETENTION_DAYS;
/// Size of the activity buffer for in-memory storage
pub const DEFAULT_ACTIVITY_BUFFER_SIZE: usize = ADMIN_ACTIVITY_BUFFER_SIZE;

// Configuration History
/// Maximum number of configuration history entries to keep
pub const DEFAULT_MAX_HISTORY_ENTRIES: usize = ADMIN_MAX_HISTORY_ENTRIES;
/// Number of days to retain configuration history
pub const DEFAULT_HISTORY_RETENTION_DAYS: u32 = ADMIN_HISTORY_RETENTION_DAYS;
/// Maximum number of configuration entries to return in queries
pub const DEFAULT_CONFIG_QUERY_LIMIT: usize = ADMIN_CONFIG_QUERY_LIMIT;

// Logging Configuration
/// Size of the in-memory log buffer
pub const DEFAULT_LOG_BUFFER_SIZE: usize = ADMIN_LOG_BUFFER_SIZE;
/// Number of days to retain log entries
pub const DEFAULT_LOG_RETENTION_DAYS: u32 = ADMIN_LOG_RETENTION_DAYS;
/// Maximum number of log entries to return in queries
pub const DEFAULT_LOG_QUERY_LIMIT: usize = ADMIN_LOG_QUERY_LIMIT;

// Backup Configuration
/// Number of days to retain backup files
pub const DEFAULT_BACKUP_RETENTION_DAYS: u32 = ADMIN_BACKUP_RETENTION_DAYS;
/// Compression level for backup files (1-9, higher = more compression)
pub const DEFAULT_BACKUP_COMPRESSION_LEVEL: u32 = ADMIN_BACKUP_COMPRESSION_LEVEL;
/// Maximum number of backup files to keep
pub const DEFAULT_MAX_BACKUPS: usize = ADMIN_MAX_BACKUPS;

// Route Discovery Configuration
/// Rate limit for health check endpoints (requests per minute)
pub const DEFAULT_ROUTE_RATE_LIMIT_HEALTH: u32 = RATE_LIMIT_HEALTH;
/// Rate limit for admin endpoints (requests per minute)
pub const DEFAULT_ROUTE_RATE_LIMIT_ADMIN: u32 = RATE_LIMIT_ADMIN;
/// Rate limit for indexing endpoints (requests per minute)
pub const DEFAULT_ROUTE_RATE_LIMIT_INDEXING: u32 = RATE_LIMIT_INDEXING;
/// Rate limit for search endpoints (requests per minute)
pub const DEFAULT_ROUTE_RATE_LIMIT_SEARCH: u32 = RATE_LIMIT_SEARCH;
/// Cooldown period for shutdown operations (seconds)
pub const DEFAULT_ROUTE_RATE_LIMIT_SHUTDOWN: u32 = RATE_LIMIT_SHUTDOWN_COOLDOWN;
/// Cooldown period for reload operations (seconds)
pub const DEFAULT_ROUTE_RATE_LIMIT_RELOAD: u32 = RATE_LIMIT_RELOAD_COOLDOWN;
/// Cooldown period for backup operations (seconds)
pub const DEFAULT_ROUTE_RATE_LIMIT_BACKUP: u32 = RATE_LIMIT_BACKUP_COOLDOWN;
/// Rate limit for restore operations (requests per minute)
pub const DEFAULT_ROUTE_RATE_LIMIT_RESTORE: u32 = RATE_LIMIT_RESTORE;

// Maintenance Operations
/// Number of items to process in each cleanup batch
pub const DEFAULT_CLEANUP_BATCH_SIZE: usize = CLEANUP_BATCH_SIZE;
/// Number of days to retain data before cleanup
pub const DEFAULT_CLEANUP_RETENTION_DAYS: u32 = CLEANUP_RETENTION_DAYS;
/// Timeout for index rebuild operations (seconds)
pub const DEFAULT_INDEX_REBUILD_TIMEOUT_SECS: u64 = INDEX_REBUILD_TIMEOUT_SECS;
/// Timeout for cache clear operations (seconds)
pub const DEFAULT_CACHE_CLEAR_TIMEOUT_SECS: u64 = CACHE_CLEAR_TIMEOUT_SECS;

// Performance Testing
/// Default duration for performance tests in seconds
pub const DEFAULT_PERF_TEST_DURATION_SECS: u32 = PERF_TEST_DURATION_SECS;
/// Default concurrency level for performance tests
pub const DEFAULT_PERF_TEST_CONCURRENCY: u32 = PERF_TEST_CONCURRENCY;
/// Default timeout for performance test requests in milliseconds
pub const DEFAULT_PERF_TEST_TIMEOUT_MS: u64 = PERF_TEST_TIMEOUT_MS;

// Directory Configuration
/// Default directory for storing backup files
pub const DEFAULT_BACKUPS_DIR: &str = ADMIN_BACKUPS_DIR;
/// Default directory for storing application data
pub const DEFAULT_DATA_DIR: &str = ADMIN_DATA_DIR;
/// Default directory for storing exported data
pub const DEFAULT_EXPORTS_DIR: &str = ADMIN_EXPORTS_DIR;

// Indexing Configuration
/// Supported file extensions for code indexing
/// These 12 languages are supported: Rust, Python, JavaScript, TypeScript, Go, Java, C, C++, C#, Ruby, PHP, Swift, Kotlin
pub const SUPPORTED_FILE_EXTENSIONS: &[&str] = &[
    ".rs",    // Rust
    ".py",    // Python
    ".js",    // JavaScript
    ".ts",    // TypeScript
    ".go",    // Go
    ".java",  // Java
    ".c",     // C
    ".cpp",   // C++
    ".cc",    // C++ (alternative)
    ".cxx",   // C++ (alternative)
    ".cs",    // C#
    ".rb",    // Ruby
    ".php",   // PHP
    ".swift", // Swift
    ".kt",    // Kotlin
    ".h",     // C/C++ header
    ".hpp",   // C++ header
];

/// Default exclude patterns for indexing
pub const DEFAULT_EXCLUDE_PATTERNS: &[&str] = &[
    "node_modules",
    "target",
    ".git",
    "dist",
    "build",
    ".venv",
    "venv",
    "__pycache__",
];

/// Helper to get supported extensions as Vec<String>
pub fn supported_extensions() -> Vec<String> {
    SUPPORTED_FILE_EXTENSIONS
        .iter()
        .map(|s| s.to_string())
        .collect()
}

/// Helper to get default exclude patterns as Vec<String>
pub fn default_exclude_patterns() -> Vec<String> {
    DEFAULT_EXCLUDE_PATTERNS
        .iter()
        .map(|s| s.to_string())
        .collect()
}

// Time Constants
/// Number of seconds in a standard day (24 hours)
pub const SECONDS_PER_DAY: u64 = 86400;

// Byte Conversion Constants
/// Number of bytes in one kilobyte (1024)
pub const BYTES_PER_KILOBYTE: u64 = 1024;
/// Number of bytes in one megabyte (1024^2)
pub const BYTES_PER_MEGABYTE: u64 = 1024 * 1024;
/// Number of bytes in one gigabyte (1024^3)
pub const BYTES_PER_GIGABYTE: u64 = 1024 * 1024 * 1024;

/// Read environment variable as usize with default fallback
pub fn get_env_usize(key: &str, default: usize) -> usize {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

/// Read environment variable as u32 with default fallback
pub fn get_env_u32(key: &str, default: u32) -> u32 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

/// Read environment variable as u64 with default fallback
pub fn get_env_u64(key: &str, default: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

// Compile-time verification of constant values (assertions_on_constants is intentional here)
#[allow(clippy::assertions_on_constants)]
const _: () = {
    assert!(
        DEFAULT_MAX_ACTIVITIES > 0,
        "MAX_ACTIVITIES must be positive"
    );
    assert!(
        DEFAULT_ACTIVITY_RETENTION_DAYS > 0,
        "RETENTION_DAYS must be positive"
    );
    assert!(
        DEFAULT_ACTIVITY_BUFFER_SIZE > 0,
        "BUFFER_SIZE must be positive"
    );
    assert!(
        DEFAULT_BACKUP_RETENTION_DAYS > 0,
        "BACKUP_RETENTION_DAYS must be positive"
    );
    assert!(
        DEFAULT_BACKUP_COMPRESSION_LEVEL >= 1 && DEFAULT_BACKUP_COMPRESSION_LEVEL <= 9,
        "COMPRESSION_LEVEL must be 1-9"
    );
    assert!(DEFAULT_MAX_BACKUPS > 0, "MAX_BACKUPS must be positive");
    assert!(SECONDS_PER_DAY == 86400, "SECONDS_PER_DAY must equal 86400");
};
