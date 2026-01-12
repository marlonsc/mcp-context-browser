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

// Activity Feed Configuration
pub const DEFAULT_MAX_ACTIVITIES: usize = 100;
pub const DEFAULT_ACTIVITY_RETENTION_DAYS: u32 = 30;
pub const DEFAULT_ACTIVITY_BUFFER_SIZE: usize = 1000;

// Configuration History
pub const DEFAULT_MAX_HISTORY_ENTRIES: usize = 1000;
pub const DEFAULT_HISTORY_RETENTION_DAYS: u32 = 90;
pub const DEFAULT_CONFIG_QUERY_LIMIT: usize = 100;

// Logging Configuration
pub const DEFAULT_LOG_BUFFER_SIZE: usize = 1000;
pub const DEFAULT_LOG_RETENTION_DAYS: u32 = 7;
pub const DEFAULT_LOG_QUERY_LIMIT: usize = 100;

// Backup Configuration
pub const DEFAULT_BACKUP_RETENTION_DAYS: u32 = 30;
pub const DEFAULT_BACKUP_COMPRESSION_LEVEL: u32 = 6;
pub const DEFAULT_MAX_BACKUPS: usize = 10;

// Route Discovery Configuration
pub const DEFAULT_ROUTE_RATE_LIMIT_HEALTH: u32 = 100; // requests per minute
pub const DEFAULT_ROUTE_RATE_LIMIT_ADMIN: u32 = 100; // requests per minute
pub const DEFAULT_ROUTE_RATE_LIMIT_INDEXING: u32 = 10; // requests per minute
pub const DEFAULT_ROUTE_RATE_LIMIT_SEARCH: u32 = 10; // requests per minute
pub const DEFAULT_ROUTE_RATE_LIMIT_SHUTDOWN: u32 = 60; // seconds cooldown
pub const DEFAULT_ROUTE_RATE_LIMIT_RELOAD: u32 = 30; // seconds cooldown
pub const DEFAULT_ROUTE_RATE_LIMIT_BACKUP: u32 = 60; // seconds cooldown
pub const DEFAULT_ROUTE_RATE_LIMIT_RESTORE: u32 = 10; // requests per minute

// Maintenance Operations
pub const DEFAULT_CLEANUP_BATCH_SIZE: usize = 100;
pub const DEFAULT_CLEANUP_RETENTION_DAYS: u32 = 30;
pub const DEFAULT_INDEX_REBUILD_TIMEOUT_SECS: u64 = 3600; // 1 hour
pub const DEFAULT_CACHE_CLEAR_TIMEOUT_SECS: u64 = 300; // 5 minutes

// Performance Testing
pub const DEFAULT_PERF_TEST_DURATION_SECS: u32 = 30;
pub const DEFAULT_PERF_TEST_CONCURRENCY: u32 = 4;
pub const DEFAULT_PERF_TEST_TIMEOUT_MS: u64 = 5000;

// Directory Configuration
pub const DEFAULT_BACKUPS_DIR: &str = "./backups";
pub const DEFAULT_DATA_DIR: &str = "./data";
pub const DEFAULT_EXPORTS_DIR: &str = "./exports";

// Time Constants
pub const SECONDS_PER_DAY: u64 = 86400;

// Byte Conversion Constants
pub const BYTES_PER_KILOBYTE: u64 = 1024;
pub const BYTES_PER_MEGABYTE: u64 = 1024 * 1024;
pub const BYTES_PER_GIGABYTE: u64 = 1024 * 1024 * 1024;

// Helper function to read environment variables with defaults
pub fn get_env_usize(key: &str, default: usize) -> usize {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

pub fn get_env_u32(key: &str, default: u32) -> u32 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

pub fn get_env_u64(key: &str, default: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_env_usize_uses_default_when_not_set() {
        let result = get_env_usize("NONEXISTENT_ADMIN_VAR_12345", 100);
        assert_eq!(result, 100);
    }

    #[test]
    fn test_get_env_usize_parses_valid_value() {
        std::env::set_var("TEST_ADMIN_USIZE", "250");
        let result = get_env_usize("TEST_ADMIN_USIZE", 100);
        assert_eq!(result, 250);
        std::env::remove_var("TEST_ADMIN_USIZE");
    }

    #[test]
    fn test_get_env_usize_uses_default_on_invalid_value() {
        std::env::set_var("TEST_ADMIN_USIZE_INVALID", "not_a_number");
        let result = get_env_usize("TEST_ADMIN_USIZE_INVALID", 100);
        assert_eq!(result, 100);
        std::env::remove_var("TEST_ADMIN_USIZE_INVALID");
    }

    #[test]
    fn test_get_env_usize_handles_empty_string() {
        std::env::set_var("TEST_ADMIN_USIZE_EMPTY", "");
        let result = get_env_usize("TEST_ADMIN_USIZE_EMPTY", 100);
        assert_eq!(result, 100);
        std::env::remove_var("TEST_ADMIN_USIZE_EMPTY");
    }

    #[test]
    fn test_get_env_u32_uses_default_when_not_set() {
        let result = get_env_u32("NONEXISTENT_ADMIN_VAR_U32_12345", 100);
        assert_eq!(result, 100);
    }

    #[test]
    fn test_get_env_u32_parses_valid_value() {
        std::env::set_var("TEST_ADMIN_U32", "500");
        let result = get_env_u32("TEST_ADMIN_U32", 100);
        assert_eq!(result, 500);
        std::env::remove_var("TEST_ADMIN_U32");
    }

    #[test]
    fn test_get_env_u32_uses_default_on_invalid_value() {
        std::env::set_var("TEST_ADMIN_U32_INVALID", "abc123");
        let result = get_env_u32("TEST_ADMIN_U32_INVALID", 100);
        assert_eq!(result, 100);
        std::env::remove_var("TEST_ADMIN_U32_INVALID");
    }

    #[test]
    fn test_get_env_u64_uses_default_when_not_set() {
        let result = get_env_u64("NONEXISTENT_ADMIN_VAR_U64_12345", 3600);
        assert_eq!(result, 3600);
    }

    #[test]
    fn test_get_env_u64_parses_valid_value() {
        std::env::set_var("TEST_ADMIN_U64", "7200");
        let result = get_env_u64("TEST_ADMIN_U64", 3600);
        assert_eq!(result, 7200);
        std::env::remove_var("TEST_ADMIN_U64");
    }

    #[test]
    fn test_get_env_u64_uses_default_on_invalid_value() {
        std::env::set_var("TEST_ADMIN_U64_INVALID", "xyz");
        let result = get_env_u64("TEST_ADMIN_U64_INVALID", 3600);
        assert_eq!(result, 3600);
        std::env::remove_var("TEST_ADMIN_U64_INVALID");
    }

    #[test]
    fn test_byte_conversion_constants() {
        assert_eq!(BYTES_PER_KILOBYTE, 1024);
        assert_eq!(BYTES_PER_MEGABYTE, 1024 * 1024);
        assert_eq!(BYTES_PER_GIGABYTE, 1024 * 1024 * 1024);
    }

    #[test]
    fn test_seconds_per_day_constant() {
        assert_eq!(SECONDS_PER_DAY, 86400);
    }

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn test_activity_defaults() {
        assert!(DEFAULT_MAX_ACTIVITIES > 0);
        assert!(DEFAULT_ACTIVITY_RETENTION_DAYS > 0);
        assert!(DEFAULT_ACTIVITY_BUFFER_SIZE > 0);
    }

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn test_backup_defaults() {
        assert!(DEFAULT_BACKUP_RETENTION_DAYS > 0);
        assert!(DEFAULT_BACKUP_COMPRESSION_LEVEL >= 1 && DEFAULT_BACKUP_COMPRESSION_LEVEL <= 9);
        assert!(DEFAULT_MAX_BACKUPS > 0);
    }
}
