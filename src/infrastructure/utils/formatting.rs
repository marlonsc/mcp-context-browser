//! Formatting utilities - Centralized formatting functions (DRY)
//!
//! Eliminates duplication across view_models.rs and builders.rs

/// Byte size constants for formatting
pub mod size_constants {
    /// Number of bytes in one kilobyte (1024)
    pub const BYTES_KB: u64 = 1024;
    /// Number of bytes in one megabyte (1024²)
    pub const BYTES_MB: u64 = BYTES_KB * 1024;
    /// Number of bytes in one gigabyte (1024³)
    pub const BYTES_GB: u64 = BYTES_MB * 1024;
    /// Number of bytes in one terabyte (1024⁴)
    pub const BYTES_TB: u64 = BYTES_GB * 1024;
}

/// Time constants for formatting
pub mod time_constants {
    /// Number of seconds in one minute
    pub const SECONDS_PER_MINUTE: u64 = 60;
    /// Number of seconds in one hour
    pub const SECONDS_PER_HOUR: u64 = 3600;
    /// Number of seconds in one day (24 hours)
    pub const SECONDS_PER_DAY: u64 = 86400;
}

use super::time::TimeUtils;

/// Formatting utilities - eliminates duplication across view_models.rs and builders.rs
pub struct FormattingUtils;

impl FormattingUtils {
    /// Format a number with thousands separator (e.g., 1234567 -> "1,234,567")
    pub fn format_number(n: u64) -> String {
        let s = n.to_string();
        let mut result = String::with_capacity(s.len() + s.len() / 3);
        for (i, c) in s.chars().rev().enumerate() {
            if i > 0 && i % 3 == 0 {
                result.insert(0, ',');
            }
            result.insert(0, c);
        }
        result
    }

    /// Format bytes in human-readable form (e.g., 1536 -> "1.5 KB")
    pub fn format_bytes(bytes: u64) -> String {
        use size_constants::*;
        if bytes >= BYTES_TB {
            format!("{:.1} TB", bytes as f64 / BYTES_TB as f64)
        } else if bytes >= BYTES_GB {
            format!("{:.1} GB", bytes as f64 / BYTES_GB as f64)
        } else if bytes >= BYTES_MB {
            format!("{:.1} MB", bytes as f64 / BYTES_MB as f64)
        } else if bytes >= BYTES_KB {
            format!("{:.1} KB", bytes as f64 / BYTES_KB as f64)
        } else {
            format!("{} B", bytes)
        }
    }

    /// Format duration in human-readable form (e.g., 3665 -> "1h 1m 5s")
    pub fn format_duration(seconds: u64) -> String {
        use time_constants::*;
        if seconds < SECONDS_PER_MINUTE {
            return format!("{}s", seconds);
        }
        let hours = seconds / SECONDS_PER_HOUR;
        let minutes = (seconds % SECONDS_PER_HOUR) / SECONDS_PER_MINUTE;
        let secs = seconds % SECONDS_PER_MINUTE;

        if hours > 0 {
            format!("{}h {}m {}s", hours, minutes, secs)
        } else {
            format!("{}m {}s", minutes, secs)
        }
    }

    /// Format age from Unix timestamp (e.g., "2 days ago", "Today")
    pub fn format_age(timestamp: u64) -> String {
        let now = TimeUtils::now_unix_secs();
        if timestamp == 0 {
            return "Unknown".to_string();
        }

        let age_seconds = now.saturating_sub(timestamp);
        let days = age_seconds / time_constants::SECONDS_PER_DAY;

        if days == 0 {
            "Today".to_string()
        } else if days == 1 {
            "1 day ago".to_string()
        } else if days < 7 {
            format!("{} days ago", days)
        } else if days < 30 {
            let weeks = days / 7;
            format!("{} week{} ago", weeks, if weeks == 1 { "" } else { "s" })
        } else if days < 365 {
            let months = days / 30;
            format!("{} month{} ago", months, if months == 1 { "" } else { "s" })
        } else {
            let years = days / 365;
            format!("{} year{} ago", years, if years == 1 { "" } else { "s" })
        }
    }

    /// Format percentage (e.g., 0.756 -> "75.6%")
    pub fn format_percentage(value: f64) -> String {
        format!("{:.1}%", value * 100.0)
    }

    /// Format percentage from pre-computed value (e.g., 75.6 -> "75.6%")
    pub fn format_percentage_raw(value: f64) -> String {
        format!("{:.1}%", value)
    }
}
