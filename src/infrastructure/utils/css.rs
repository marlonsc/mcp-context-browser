//! CSS Class Constants - DRY badge classes (11 occurrences consolidated) (DRY)
//!
//! Provides consistent Tailwind CSS classes for UI components

/// Tailwind CSS badge classes for consistent UI styling
pub mod badge {
    /// Success state badge styling
    pub const SUCCESS: &str = "bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-300";
    /// Error state badge styling
    pub const ERROR: &str = "bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-300";
    /// Warning state badge styling
    pub const WARNING: &str =
        "bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-300";
    /// Info state badge styling
    pub const INFO: &str = "bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-300";
    /// Default/unknown state badge styling
    pub const DEFAULT: &str = "bg-gray-100 text-gray-800 dark:bg-gray-900 dark:text-gray-300";
}

/// Indicator dot classes (single color)
pub mod indicator {
    /// Success state indicator styling
    pub const SUCCESS: &str = "bg-green-500";
    /// Error state indicator styling
    pub const ERROR: &str = "bg-red-500";
    /// Warning state indicator styling
    pub const WARNING: &str = "bg-yellow-500";
    /// Info state indicator styling
    pub const INFO: &str = "bg-blue-500";
    /// Default/unknown state indicator styling
    pub const DEFAULT: &str = "bg-gray-500";
}

/// Get badge class for status string (provider, index, health)
#[inline]
pub fn badge_for_status(status: &str) -> &'static str {
    match status {
        "available" | "active" | "healthy" | "success" | "ready" => badge::SUCCESS,
        "unavailable" | "error" | "failed" | "critical" | "unhealthy" => badge::ERROR,
        "starting" | "initializing" | "indexing" | "warning" | "degraded" => badge::WARNING,
        "info" | "pending" => badge::INFO,
        _ => badge::DEFAULT,
    }
}

/// Get badge class for activity/log level
#[inline]
pub fn badge_for_level(level: &str) -> &'static str {
    match level.to_lowercase().as_str() {
        "success" => badge::SUCCESS,
        "error" => badge::ERROR,
        "warning" | "warn" => badge::WARNING,
        "info" | "debug" => badge::INFO,
        _ => badge::DEFAULT,
    }
}

/// Get indicator dot class for status string
#[inline]
pub fn indicator_for_status(status: &str) -> &'static str {
    match status {
        "available" | "active" | "healthy" | "success" | "ready" => indicator::SUCCESS,
        "unavailable" | "error" | "failed" | "critical" | "unhealthy" => indicator::ERROR,
        "starting" | "initializing" | "indexing" | "warning" | "degraded" => indicator::WARNING,
        "info" | "pending" => indicator::INFO,
        _ => indicator::DEFAULT,
    }
}

/// Get indicator dot class for activity/log level
#[inline]
pub fn indicator_for_level(level: &str) -> &'static str {
    match level.to_lowercase().as_str() {
        "success" => indicator::SUCCESS,
        "error" => indicator::ERROR,
        "warning" | "warn" => indicator::WARNING,
        "info" | "debug" => indicator::INFO,
        _ => indicator::DEFAULT,
    }
}
