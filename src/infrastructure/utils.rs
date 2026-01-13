//! Utility functions and helpers
//!
//! This module demonstrates clean code principles and TDD refactoring.
//! Contains well-tested utility functions that follow SOLID principles.

use std::collections::HashMap;

/// Validation result for input data
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationResult<T> {
    Valid(T),
    Invalid(String),
}

/// Generic validation trait for clean code
pub trait Validatable {
    fn validate(&self) -> ValidationResult<Self>
    where
        Self: Clone;
}

/// String validation utilities - demonstrates Extract Method refactoring
pub struct StringValidator;

impl StringValidator {
    /// Validate string is not empty - extracted from multiple locations
    pub fn validate_not_empty(s: &str) -> ValidationResult<String> {
        if s.trim().is_empty() {
            ValidationResult::Invalid("String cannot be empty".to_string())
        } else {
            ValidationResult::Valid(s.to_string())
        }
    }

    /// Validate string length - extracted common validation logic
    pub fn validate_length(s: &str, min: usize, max: usize) -> ValidationResult<String> {
        let len = s.len();
        if len < min {
            ValidationResult::Invalid(format!("String too short: {} < {}", len, min))
        } else if len > max {
            ValidationResult::Invalid(format!("String too long: {} > {}", len, max))
        } else {
            ValidationResult::Valid(s.to_string())
        }
    }

    /// Validate string contains only alphanumeric characters - clean validation
    pub fn validate_alphanumeric(s: &str) -> ValidationResult<String> {
        if s.chars().all(|c| c.is_alphanumeric() || c == '_') {
            ValidationResult::Valid(s.to_string())
        } else {
            ValidationResult::Invalid("String contains invalid characters".to_string())
        }
    }
}

/// Collection utilities - demonstrates DRY principle
pub struct CollectionUtils;

impl CollectionUtils {
    /// Safe get with default - eliminates repetitive unwrap_or patterns
    pub fn get_or_default<T: Clone>(map: &HashMap<String, T>, key: &str, default: T) -> T {
        map.get(key).cloned().unwrap_or(default)
    }

    /// Safe get with custom default function - more flexible
    pub fn get_or_else<T: Clone, F>(map: &HashMap<String, T>, key: &str, default_fn: F) -> T
    where
        F: FnOnce() -> T,
    {
        map.get(key).cloned().unwrap_or_else(default_fn)
    }

    /// Check if collection is empty - extracted common check
    pub fn is_empty<T>(collection: &[T]) -> bool {
        collection.is_empty()
    }

    /// Safe indexing with bounds check - prevents panics
    pub fn get_safe<T: Clone>(slice: &[T], index: usize) -> Option<T> {
        slice.get(index).cloned()
    }
}

/// Error handling utilities - demonstrates consistent error patterns
pub struct ErrorUtils;

impl ErrorUtils {
    /// Create standardized error message - eliminates duplication
    pub fn format_error(context: &str, details: &str) -> String {
        format!("{}: {}", context, details)
    }

    /// Create validation error - consistent validation errors
    pub fn validation_error(field: &str, reason: &str) -> String {
        Self::format_error(&format!("Validation failed for {}", field), reason)
    }

    /// Create not found error - consistent not found errors
    pub fn not_found_error(resource: &str, identifier: &str) -> String {
        Self::format_error(
            &format!("{} not found", resource),
            &format!(
                "No {} found with identifier '{}'",
                resource.to_lowercase(),
                identifier
            ),
        )
    }
}

/// Configuration utilities - demonstrates parameter object pattern
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    pub min_length: usize,
    pub max_length: usize,
    pub allow_special_chars: bool,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            min_length: 1,
            max_length: 100,
            allow_special_chars: false,
        }
    }
}

impl ValidationConfig {
    /// Builder pattern for configuration - clean API
    pub fn new() -> Self {
        Self::default()
    }

    pub fn min_length(mut self, min: usize) -> Self {
        self.min_length = min;
        self
    }

    pub fn max_length(mut self, max: usize) -> Self {
        self.max_length = max;
        self
    }

    pub fn allow_special_chars(mut self, allow: bool) -> Self {
        self.allow_special_chars = allow;
        self
    }
}

/// Advanced validation with configuration - demonstrates strategy pattern
pub struct ConfigurableValidator {
    config: ValidationConfig,
}

impl ConfigurableValidator {
    pub fn new(config: ValidationConfig) -> Self {
        Self { config }
    }

    /// Validate string with configuration - flexible validation
    pub fn validate_string(&self, s: &str) -> ValidationResult<String> {
        // Check length
        match StringValidator::validate_length(s, self.config.min_length, self.config.max_length) {
            ValidationResult::Valid(_) => {}
            ValidationResult::Invalid(e) => return ValidationResult::Invalid(e),
        }

        // Check characters if special chars not allowed
        if !self.config.allow_special_chars {
            match StringValidator::validate_alphanumeric(s) {
                ValidationResult::Valid(_) => {}
                ValidationResult::Invalid(e) => return ValidationResult::Invalid(e),
            }
        }

        ValidationResult::Valid(s.to_string())
    }
}

// =============================================================================
// Formatting Utilities - Centralized formatting functions (DRY)
// =============================================================================

/// Byte size constants for formatting
pub mod size_constants {
    pub const BYTES_KB: u64 = 1024;
    pub const BYTES_MB: u64 = BYTES_KB * 1024;
    pub const BYTES_GB: u64 = BYTES_MB * 1024;
    pub const BYTES_TB: u64 = BYTES_GB * 1024;
}

/// Time constants for formatting
pub mod time_constants {
    pub const SECONDS_PER_MINUTE: u64 = 60;
    pub const SECONDS_PER_HOUR: u64 = 3600;
    pub const SECONDS_PER_DAY: u64 = 86400;
}

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

// =============================================================================
// Time Utilities - Centralized time functions (DRY)
// =============================================================================

/// Time utilities - eliminates `SystemTime::now().duration_since(UNIX_EPOCH)` pattern
pub struct TimeUtils;

impl TimeUtils {
    /// Get current Unix timestamp in seconds
    #[inline]
    pub fn now_unix_secs() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }

    /// Get current Unix timestamp in milliseconds
    #[inline]
    pub fn now_unix_millis() -> u128 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0)
    }

    /// Check if a timestamp has expired given a TTL in seconds
    #[inline]
    pub fn is_expired(timestamp: u64, ttl_secs: u64) -> bool {
        Self::now_unix_secs().saturating_sub(timestamp) > ttl_secs
    }

    /// Calculate age in seconds from a timestamp
    #[inline]
    pub fn age_secs(timestamp: u64) -> u64 {
        Self::now_unix_secs().saturating_sub(timestamp)
    }
}

// =============================================================================
// Status Constants - Centralized status strings (DRY)
// =============================================================================

/// Common status values used across admin interface
pub mod status {
    pub const HEALTHY: &str = "healthy";
    pub const DEGRADED: &str = "degraded";
    pub const CRITICAL: &str = "critical";
    pub const ACTIVE: &str = "active";
    pub const INACTIVE: &str = "inactive";
    pub const INDEXING: &str = "indexing";
    pub const IDLE: &str = "idle";
    pub const BUSY: &str = "busy";
    pub const UNKNOWN: &str = "unknown";
}

/// Activity level strings
pub mod activity_level {
    pub const SUCCESS: &str = "success";
    pub const WARNING: &str = "warning";
    pub const ERROR: &str = "error";
    pub const INFO: &str = "info";
}

// =============================================================================
// Health Check Utilities
// =============================================================================

/// Health check utilities for determining system status
pub struct HealthUtils;

impl HealthUtils {
    /// Determine health status based on CPU and memory usage
    pub fn compute_status(cpu_usage: f64, memory_usage: f64) -> &'static str {
        const HEALTHY_THRESHOLD: f64 = 85.0;
        match (
            cpu_usage < HEALTHY_THRESHOLD,
            memory_usage < HEALTHY_THRESHOLD,
        ) {
            (true, true) => status::HEALTHY,
            (false, false) => status::CRITICAL,
            _ => status::DEGRADED,
        }
    }
}

// =============================================================================
// String Utilities - Common string operations (DRY)
// =============================================================================

/// String utilities - common string operations
pub struct StringUtils;

impl StringUtils {
    /// Capitalize first letter of a string
    #[inline]
    pub fn capitalize_first(s: &str) -> String {
        let mut chars = s.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().chain(chars).collect(),
        }
    }

    /// Convert snake_case or space-separated string to Title Case
    ///
    /// # Examples
    /// - "open_ai" → "Open Ai"
    /// - "hello world" → "Hello World"
    /// - "my_api_key" → "My Api Key"
    pub fn to_title_case(s: &str) -> String {
        s.replace('_', " ")
            .split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().chain(chars).collect(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Format relative time from chrono DateTime (e.g., "Just now", "5m ago")
    pub fn format_relative_time(timestamp: chrono::DateTime<chrono::Utc>) -> String {
        let now = chrono::Utc::now();
        let diff = now.signed_duration_since(timestamp);
        let seconds = diff.num_seconds();

        if seconds < 60 {
            "Just now".to_string()
        } else if seconds < time_constants::SECONDS_PER_HOUR as i64 {
            format!("{}m ago", seconds / 60)
        } else if seconds < time_constants::SECONDS_PER_DAY as i64 {
            format!("{}h ago", seconds / time_constants::SECONDS_PER_HOUR as i64)
        } else {
            format!("{}d ago", seconds / time_constants::SECONDS_PER_DAY as i64)
        }
    }
}

// =============================================================================
// Async File Utilities - Replaces 10+ lines per call across codebase
// =============================================================================

use crate::domain::error::{Error, Result};
use std::path::Path;

/// Async file utilities for common I/O patterns
pub struct FileUtils;

impl FileUtils {
    /// Write JSON to file with proper error handling (replaces ~8 lines per use)
    ///
    /// Serializes value to JSON and writes atomically with descriptive error.
    pub async fn write_json<T: serde::Serialize, P: AsRef<Path>>(
        path: P,
        value: &T,
        context: &str,
    ) -> Result<()> {
        let content = serde_json::to_string_pretty(value)
            .map_err(|e| Error::internal(format!("Failed to serialize {}: {}", context, e)))?;
        tokio::fs::write(path.as_ref(), content)
            .await
            .map_err(|e| Error::io(format!("Failed to write {}: {}", context, e)))?;
        Ok(())
    }

    /// Read JSON from file with proper error handling (replaces ~8 lines per use)
    pub async fn read_json<T: serde::de::DeserializeOwned, P: AsRef<Path>>(
        path: P,
        context: &str,
    ) -> Result<T> {
        let content = tokio::fs::read_to_string(path.as_ref())
            .await
            .map_err(|e| Error::io(format!("Failed to read {}: {}", context, e)))?;
        serde_json::from_str(&content)
            .map_err(|e| Error::internal(format!("Failed to parse {}: {}", context, e)))
    }

    /// Ensure directory exists and write file (replaces ~12 lines per use)
    pub async fn ensure_dir_write<P: AsRef<Path>>(
        path: P,
        content: &[u8],
        context: &str,
    ) -> Result<()> {
        if let Some(parent) = path.as_ref().parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                Error::io(format!("Failed to create directory for {}: {}", context, e))
            })?;
        }
        tokio::fs::write(path.as_ref(), content)
            .await
            .map_err(|e| Error::io(format!("Failed to write {}: {}", context, e)))?;
        Ok(())
    }

    /// Ensure directory exists and write JSON (replaces ~15 lines per use)
    pub async fn ensure_dir_write_json<T: serde::Serialize, P: AsRef<Path>>(
        path: P,
        value: &T,
        context: &str,
    ) -> Result<()> {
        let content = serde_json::to_string_pretty(value)
            .map_err(|e| Error::internal(format!("Failed to serialize {}: {}", context, e)))?;
        Self::ensure_dir_write(path, content.as_bytes(), context).await
    }

    /// Check if path exists (async wrapper for std::path::Path::exists)
    pub async fn exists<P: AsRef<Path>>(path: P) -> bool {
        tokio::fs::metadata(path.as_ref()).await.is_ok()
    }

    /// Read file if exists, return None otherwise (replaces ~6 lines per use)
    pub async fn read_if_exists<P: AsRef<Path>>(path: P) -> Result<Option<Vec<u8>>> {
        match tokio::fs::read(path.as_ref()).await {
            Ok(content) => Ok(Some(content)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(Error::io(format!("Failed to read file: {}", e))),
        }
    }
}

// =============================================================================
// HTTP Handler Utilities - Replaces 46 occurrences of error mapping
// =============================================================================

/// Helper trait for converting Results to HTTP StatusCode errors
///
/// Replaces `.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?` pattern
pub trait IntoStatusCode<T> {
    /// Convert to StatusCode::INTERNAL_SERVER_ERROR on error
    fn to_500(self) -> std::result::Result<T, axum::http::StatusCode>;

    /// Convert to StatusCode::NOT_FOUND on error
    fn to_404(self) -> std::result::Result<T, axum::http::StatusCode>;

    /// Convert to StatusCode::BAD_REQUEST on error
    fn to_400(self) -> std::result::Result<T, axum::http::StatusCode>;
}

impl<T, E> IntoStatusCode<T> for std::result::Result<T, E> {
    #[inline]
    fn to_500(self) -> std::result::Result<T, axum::http::StatusCode> {
        self.map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)
    }

    #[inline]
    fn to_404(self) -> std::result::Result<T, axum::http::StatusCode> {
        self.map_err(|_| axum::http::StatusCode::NOT_FOUND)
    }

    #[inline]
    fn to_400(self) -> std::result::Result<T, axum::http::StatusCode> {
        self.map_err(|_| axum::http::StatusCode::BAD_REQUEST)
    }
}

// =============================================================================
// HTTP Response Utilities - Replaces ~8 lines per call across embedding providers
// =============================================================================

/// HTTP response utilities for API clients (embedding providers, etc.)
///
/// Consolidates the common pattern of checking response status and extracting errors.
pub struct HttpResponseUtils;

impl HttpResponseUtils {
    /// Check HTTP response and return error if not successful (saves ~8 lines per use)
    ///
    /// # Example
    /// ```ignore
    /// let response = client.post(url).send().await?;
    /// HttpResponseUtils::check_response(response, "OpenAI").await?;
    /// // vs. the old 8-line pattern:
    /// // if !response.status().is_success() {
    /// //     let status = response.status();
    /// //     let error_text = response.text().await.unwrap_or_default();
    /// //     return Err(Error::embedding(format!("API error {}: {}", status, error_text)));
    /// // }
    /// ```
    pub async fn check_response(
        response: reqwest::Response,
        provider_name: &str,
    ) -> Result<reqwest::Response> {
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::embedding(format!(
                "{} API error {}: {}",
                provider_name, status, error_text
            )));
        }
        Ok(response)
    }

    /// Parse JSON from response with provider-specific error (saves ~6 lines per use)
    pub async fn json_response<T: serde::de::DeserializeOwned>(
        response: reqwest::Response,
        provider_name: &str,
    ) -> Result<T> {
        response
            .json()
            .await
            .map_err(|e| Error::embedding(format!("{} response parse error: {}", provider_name, e)))
    }

    /// Combined: check response status and parse JSON (saves ~12 lines per use)
    pub async fn check_and_parse<T: serde::de::DeserializeOwned>(
        response: reqwest::Response,
        provider_name: &str,
    ) -> Result<T> {
        let response = Self::check_response(response, provider_name).await?;
        Self::json_response(response, provider_name).await
    }
}

// =============================================================================
// CSS Class Constants - DRY badge classes (11 occurrences consolidated)
// =============================================================================

/// Tailwind CSS badge classes for consistent UI styling
pub mod css {
    /// Badge background + text classes for different states
    pub mod badge {
        pub const SUCCESS: &str =
            "bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-300";
        pub const ERROR: &str = "bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-300";
        pub const WARNING: &str =
            "bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-300";
        pub const INFO: &str = "bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-300";
        pub const DEFAULT: &str = "bg-gray-100 text-gray-800 dark:bg-gray-900 dark:text-gray-300";
    }

    /// Indicator dot classes (single color)
    pub mod indicator {
        pub const SUCCESS: &str = "bg-green-500";
        pub const ERROR: &str = "bg-red-500";
        pub const WARNING: &str = "bg-yellow-500";
        pub const INFO: &str = "bg-blue-500";
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
}

// =============================================================================
// Retry Utilities - Async retry with exponential backoff (~15 lines per use)
// =============================================================================

use std::future::Future;
use std::time::Duration;

/// Configuration for retry behavior
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial delay between retries (doubles each attempt)
    pub initial_delay_ms: u64,
    /// Maximum delay cap (prevents exponential explosion)
    pub max_delay_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_ms: 500,
            max_delay_ms: 5000,
        }
    }
}

impl RetryConfig {
    /// Create with custom attempts
    pub fn with_attempts(mut self, attempts: u32) -> Self {
        self.max_attempts = attempts;
        self
    }

    /// Create with custom initial delay
    pub fn with_initial_delay(mut self, delay_ms: u64) -> Self {
        self.initial_delay_ms = delay_ms;
        self
    }

    /// Create with custom max delay
    pub fn with_max_delay(mut self, delay_ms: u64) -> Self {
        self.max_delay_ms = delay_ms;
        self
    }
}

/// Retry utilities - saves ~15 lines per retry pattern
pub struct RetryUtils;

impl RetryUtils {
    /// Retry an async operation with exponential backoff
    ///
    /// # Arguments
    /// * `config` - Retry configuration (attempts, delays)
    /// * `operation` - Async closure returning Result<T, E>
    /// * `should_retry` - Predicate on error to decide if retry should continue
    /// * `context` - Description for logging (e.g., "index creation")
    ///
    /// # Example
    /// ```ignore
    /// // Before: ~15 lines of retry logic
    /// // After: 6 lines
    /// RetryUtils::retry_with_backoff(
    ///     RetryConfig::default(),
    ///     || async { client.create_index(name).await },
    ///     |e| e.to_string().contains("NotFound"),
    ///     "index creation",
    /// ).await?;
    /// ```
    pub async fn retry_with_backoff<T, E, F, Fut, R>(
        config: RetryConfig,
        mut operation: F,
        should_retry: R,
        context: &str,
    ) -> std::result::Result<T, E>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = std::result::Result<T, E>>,
        R: Fn(&E) -> bool,
        E: std::fmt::Display,
    {
        let mut last_error = None;

        for attempt in 0..config.max_attempts {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if should_retry(&e) && attempt + 1 < config.max_attempts {
                        let delay = std::cmp::min(
                            config.initial_delay_ms * 2u64.pow(attempt),
                            config.max_delay_ms,
                        );
                        tracing::debug!(
                            "{} attempt {} failed ({}), retrying in {}ms...",
                            context,
                            attempt + 1,
                            e,
                            delay
                        );
                        tokio::time::sleep(Duration::from_millis(delay)).await;
                        last_error = Some(e);
                    } else {
                        return Err(e);
                    }
                }
            }
        }

        // Should not reach here, but return last error if we do
        Err(last_error.expect("retry loop should have at least one attempt"))
    }

    /// Simplified retry - always retries on any error
    ///
    /// # Example
    /// ```ignore
    /// RetryUtils::retry(3, 500, || async { fetch_data().await }, "data fetch").await?;
    /// ```
    pub async fn retry<T, E, F, Fut>(
        max_attempts: u32,
        initial_delay_ms: u64,
        operation: F,
        context: &str,
    ) -> std::result::Result<T, E>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = std::result::Result<T, E>>,
        E: std::fmt::Display,
    {
        Self::retry_with_backoff(
            RetryConfig::default()
                .with_attempts(max_attempts)
                .with_initial_delay(initial_delay_ms),
            operation,
            |_| true, // Always retry on error
            context,
        )
        .await
    }

    /// Calculate delay for a given attempt (useful for custom retry loops)
    #[inline]
    pub fn calculate_delay(attempt: u32, initial_ms: u64, max_ms: u64) -> Duration {
        let delay = std::cmp::min(initial_ms * 2u64.pow(attempt), max_ms);
        Duration::from_millis(delay)
    }
}

// =============================================================================
// JSON Value Extension - Replaces .get().and_then(|v| v.as_*()) pattern
// =============================================================================

/// Extension trait for serde_json::Value with convenient accessor methods
///
/// Replaces the verbose pattern:
/// ```ignore
/// meta.get("key").and_then(|v| v.as_str()).unwrap_or("default")
/// ```
/// With:
/// ```ignore
/// meta.str_or("key", "default")
/// ```
pub trait JsonExt {
    /// Get string value or default (replaces .get().and_then(as_str).unwrap_or)
    fn str_or<'a>(&'a self, key: &str, default: &'a str) -> &'a str;

    /// Get owned string value or default
    fn string_or(&self, key: &str, default: &str) -> String;

    /// Get i64 value or default
    fn i64_or(&self, key: &str, default: i64) -> i64;

    /// Get u64 value or default
    fn u64_or(&self, key: &str, default: u64) -> u64;

    /// Get f64 value or default
    fn f64_or(&self, key: &str, default: f64) -> f64;

    /// Get bool value or default
    fn bool_or(&self, key: &str, default: bool) -> bool;

    /// Get optional string (replaces .get().and_then(as_str))
    fn opt_str(&self, key: &str) -> Option<&str>;

    /// Get optional i64
    fn opt_i64(&self, key: &str) -> Option<i64>;

    /// Get optional u64
    fn opt_u64(&self, key: &str) -> Option<u64>;
}

impl JsonExt for serde_json::Value {
    #[inline]
    fn str_or<'a>(&'a self, key: &str, default: &'a str) -> &'a str {
        self.get(key).and_then(|v| v.as_str()).unwrap_or(default)
    }

    #[inline]
    fn string_or(&self, key: &str, default: &str) -> String {
        self.get(key)
            .and_then(|v| v.as_str())
            .unwrap_or(default)
            .to_string()
    }

    #[inline]
    fn i64_or(&self, key: &str, default: i64) -> i64 {
        self.get(key).and_then(|v| v.as_i64()).unwrap_or(default)
    }

    #[inline]
    fn u64_or(&self, key: &str, default: u64) -> u64 {
        self.get(key).and_then(|v| v.as_u64()).unwrap_or(default)
    }

    #[inline]
    fn f64_or(&self, key: &str, default: f64) -> f64 {
        self.get(key).and_then(|v| v.as_f64()).unwrap_or(default)
    }

    #[inline]
    fn bool_or(&self, key: &str, default: bool) -> bool {
        self.get(key).and_then(|v| v.as_bool()).unwrap_or(default)
    }

    #[inline]
    fn opt_str(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(|v| v.as_str())
    }

    #[inline]
    fn opt_i64(&self, key: &str) -> Option<i64> {
        self.get(key).and_then(|v| v.as_i64())
    }

    #[inline]
    fn opt_u64(&self, key: &str) -> Option<u64> {
        self.get(key).and_then(|v| v.as_u64())
    }
}

/// Extension trait for HashMap<String, serde_json::Value>
impl JsonExt for std::collections::HashMap<String, serde_json::Value> {
    #[inline]
    fn str_or<'a>(&'a self, key: &str, default: &'a str) -> &'a str {
        self.get(key).and_then(|v| v.as_str()).unwrap_or(default)
    }

    #[inline]
    fn string_or(&self, key: &str, default: &str) -> String {
        self.get(key)
            .and_then(|v| v.as_str())
            .unwrap_or(default)
            .to_string()
    }

    #[inline]
    fn i64_or(&self, key: &str, default: i64) -> i64 {
        self.get(key).and_then(|v| v.as_i64()).unwrap_or(default)
    }

    #[inline]
    fn u64_or(&self, key: &str, default: u64) -> u64 {
        self.get(key).and_then(|v| v.as_u64()).unwrap_or(default)
    }

    #[inline]
    fn f64_or(&self, key: &str, default: f64) -> f64 {
        self.get(key).and_then(|v| v.as_f64()).unwrap_or(default)
    }

    #[inline]
    fn bool_or(&self, key: &str, default: bool) -> bool {
        self.get(key).and_then(|v| v.as_bool()).unwrap_or(default)
    }

    #[inline]
    fn opt_str(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(|v| v.as_str())
    }

    #[inline]
    fn opt_i64(&self, key: &str) -> Option<i64> {
        self.get(key).and_then(|v| v.as_i64())
    }

    #[inline]
    fn opt_u64(&self, key: &str) -> Option<u64> {
        self.get(key).and_then(|v| v.as_u64())
    }
}

/// Extension trait for serde_json::Map<String, Value>
impl JsonExt for serde_json::Map<String, serde_json::Value> {
    #[inline]
    fn str_or<'a>(&'a self, key: &str, default: &'a str) -> &'a str {
        self.get(key).and_then(|v| v.as_str()).unwrap_or(default)
    }

    #[inline]
    fn string_or(&self, key: &str, default: &str) -> String {
        self.get(key)
            .and_then(|v| v.as_str())
            .unwrap_or(default)
            .to_string()
    }

    #[inline]
    fn i64_or(&self, key: &str, default: i64) -> i64 {
        self.get(key).and_then(|v| v.as_i64()).unwrap_or(default)
    }

    #[inline]
    fn u64_or(&self, key: &str, default: u64) -> u64 {
        self.get(key).and_then(|v| v.as_u64()).unwrap_or(default)
    }

    #[inline]
    fn f64_or(&self, key: &str, default: f64) -> f64 {
        self.get(key).and_then(|v| v.as_f64()).unwrap_or(default)
    }

    #[inline]
    fn bool_or(&self, key: &str, default: bool) -> bool {
        self.get(key).and_then(|v| v.as_bool()).unwrap_or(default)
    }

    #[inline]
    fn opt_str(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(|v| v.as_str())
    }

    #[inline]
    fn opt_i64(&self, key: &str) -> Option<i64> {
        self.get(key).and_then(|v| v.as_i64())
    }

    #[inline]
    fn opt_u64(&self, key: &str) -> Option<u64> {
        self.get(key).and_then(|v| v.as_u64())
    }
}
