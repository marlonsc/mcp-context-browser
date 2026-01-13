//! Service layer helpers - consolidates repeated patterns across admin and application services
//!
//! This module provides reusable patterns for:
//! - Event publishing with error handling
//! - Metrics collection with safe defaults
//! - Operation timing instrumentation
//! - Retry logic with exponential backoff
//! - Configuration validation
//!
//! Goal: Reduce 300+ lines of duplicated code across services

use std::time::Instant;
use std::time::Duration;
use anyhow::{Context, Result};

/// Timing instrumentation helper - tracks operation elapsed time
///
/// Eliminates 8+ manual `Instant::now()` patterns across services
///
/// # Example
/// ```ignore
/// let timer = TimedOperation::start();
/// perform_operation().await?;
/// println!("Elapsed: {}ms", timer.elapsed_ms());
/// ```
pub struct TimedOperation {
    start: Instant,
}

impl TimedOperation {
    /// Start a new timed operation
    pub fn start() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    /// Get elapsed time in milliseconds
    pub fn elapsed_ms(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }

    /// Get elapsed time in seconds
    pub fn elapsed_secs(&self) -> f64 {
        self.start.elapsed().as_secs_f64()
    }

    /// Get remaining time before deadline (returns None if already exceeded)
    pub fn remaining(&self, deadline: Duration) -> Option<Duration> {
        deadline.checked_sub(self.start.elapsed())
    }
}

/// Safe metrics collection wrapper - provides fallback defaults
///
/// Eliminates 6+ `unwrap_or_default()` patterns across health checks
///
/// # Example
/// ```ignore
/// let cpu = SafeMetrics::collect(
///     || system_collector.collect_cpu_metrics().await,
///     || Ok(CpuMetrics::default())
/// ).await;
/// ```
pub struct SafeMetrics;

impl SafeMetrics {
    /// Collect metrics with fallback default
    pub async fn collect<F, T, E>(f: F, fallback: T) -> T
    where
        F: std::future::Future<Output = std::result::Result<T, E>>,
    {
        match f.await {
            Ok(metrics) => metrics,
            Err(_) => fallback,
        }
    }

    /// Collect metrics with logging on failure
    pub async fn collect_or_log<F, T, E>(f: F, fallback: T, operation: &str) -> T
    where
        F: std::future::Future<Output = std::result::Result<T, E>>,
        E: std::fmt::Display,
    {
        match f.await {
            Ok(metrics) => metrics,
            Err(e) => {
                tracing::warn!("Failed to collect {}: {}", operation, e);
                fallback
            }
        }
    }
}

/// Retry helper with exponential backoff
///
/// Eliminates hardcoded retry loops in provider health checks
///
/// # Example
/// ```ignore
/// let result = RetryHelper::with_backoff(
///     || async { provider.health_check().await },
///     3,
/// ).await?;
/// ```
pub struct RetryHelper;

impl RetryHelper {
    /// Execute operation with exponential backoff retry
    pub async fn with_backoff<F, Fut, T, E>(
        mut operation: F,
        max_retries: u32,
    ) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = std::result::Result<T, E>>,
        E: std::fmt::Display,
    {
        let mut retries = 0;
        let mut delay_ms = 100;

        loop {
            match operation().await {
                Ok(value) => return Ok(value),
                Err(e) => {
                    retries += 1;
                    if retries > max_retries {
                        return Err(anyhow::anyhow!("Max retries ({}) exceeded: {}", max_retries, e));
                    }
                    tracing::debug!("Operation failed (attempt {}), retrying in {}ms: {}", retries, delay_ms, e);
                    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                    delay_ms = (delay_ms * 2).min(5000); // Exponential backoff, capped at 5s
                }
            }
        }
    }

    /// Execute operation with timeout
    pub async fn with_timeout<F, Fut, T>(
        operation: F,
        timeout: Duration,
    ) -> Result<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        tokio::time::timeout(timeout, operation())
            .await
            .context("Operation timed out")?
    }

    /// Execute operation with timeout and retries
    pub async fn with_timeout_and_retries<F, Fut, T, E>(
        mut operation: F,
        max_retries: u32,
        timeout: Duration,
    ) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = std::result::Result<T, E>>,
        E: std::fmt::Display + 'static,
    {
        let mut retries = 0;
        let mut delay_ms = 100;
        let deadline = Instant::now() + timeout;

        loop {
            // Check if we've exceeded total timeout
            if Instant::now() > deadline {
                return Err(anyhow::anyhow!("Total timeout of {:?} exceeded", timeout));
            }

            // Attempt operation with remaining timeout
            let remaining = deadline.saturating_duration_since(Instant::now());
            match tokio::time::timeout(remaining, operation()).await {
                Ok(Ok(value)) => return Ok(value),
                Ok(Err(e)) => {
                    retries += 1;
                    if retries > max_retries {
                        return Err(anyhow::anyhow!("Max retries ({}) exceeded: {}", max_retries, e));
                    }
                    tracing::debug!("Operation failed (attempt {}), retrying in {}ms: {}", retries, delay_ms, e);
                    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                    delay_ms = (delay_ms * 2).min(5000);
                }
                Err(_) => {
                    return Err(anyhow::anyhow!("Operation timed out after {:?}", timeout));
                }
            }
        }
    }
}

/// Configuration validation builder - declarative validation rules
///
/// Eliminates 80+ lines of manual validation match statements
///
/// # Example
/// ```ignore
/// ValidationBuilder::new("database connection pool")
///     .check_range("pool_size", pool_size, 1..=100, "must be 1-100")?
///     .check_positive("timeout_ms", timeout_ms, "must be positive")?
///     .check_string_not_empty("username", &username)?
///     .finalize()
/// ```
pub struct ValidationBuilder {
    name: String,
    warnings: Vec<String>,
    errors: Vec<String>,
}

impl ValidationBuilder {
    /// Create a new validation builder
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Check value is within range
    pub fn check_range<T: std::cmp::PartialOrd + std::fmt::Display>(
        mut self,
        field: &str,
        value: T,
        min: T,
        max: T,
        message: &str,
    ) -> Self {
        if value < min || value > max {
            self.errors.push(format!("{}: {} ({})", field, message, value));
        }
        self
    }

    /// Check value is positive
    pub fn check_positive(mut self, field: &str, value: i64, message: &str) -> Self {
        if value <= 0 {
            self.errors.push(format!("{}: {} (value: {})", field, message, value));
        }
        self
    }

    /// Check string is not empty
    pub fn check_string_not_empty(mut self, field: &str, value: &str) -> Self {
        if value.trim().is_empty() {
            self.errors.push(format!("{}: must not be empty", field));
        }
        self
    }

    /// Add a warning message
    pub fn warn(mut self, message: impl Into<String>) -> Self {
        self.warnings.push(message.into());
        self
    }

    /// Add an error message
    pub fn error(mut self, message: impl Into<String>) -> Self {
        self.errors.push(message.into());
        self
    }

    /// Finalize validation and return result
    pub fn finalize(self) -> Result<Vec<String>> {
        if !self.errors.is_empty() {
            return Err(anyhow::anyhow!(
                "Validation failed for {}: {}",
                self.name,
                self.errors.join(", ")
            ));
        }
        Ok(self.warnings)
    }
}

/// Iterator helpers - cleaner functional chains
///
/// Eliminates manual vec filtering patterns
///
/// # Example
/// ```ignore
/// use itertools::Itertools;
/// entries.into_iter()
///     .filter(|e| e.matches_filter(filter))
///     .take(limit)
///     .collect::<Vec<_>>()
/// ```
pub struct IteratorHelpers;

impl IteratorHelpers {
    /// Collect iterator with size limit
    pub fn take_collect<T, I>(iter: I, limit: usize) -> Vec<T>
    where
        I: IntoIterator<Item = T>,
    {
        iter.into_iter().take(limit).collect()
    }

    /// Filter and collect
    pub fn filter_collect<T, I, F>(iter: I, predicate: F) -> Vec<T>
    where
        I: IntoIterator<Item = T>,
        F: FnMut(&T) -> bool,
    {
        iter.into_iter().filter(predicate).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timed_operation() {
        let timer = TimedOperation::start();
        std::thread::sleep(Duration::from_millis(10));
        assert!(timer.elapsed_ms() >= 10);
    }

    #[test]
    fn test_timed_operation_remaining() {
        let timer = TimedOperation::start();
        let remaining = timer.remaining(Duration::from_secs(10));
        assert!(remaining.is_some());
        assert!(remaining.unwrap() > Duration::from_secs(9));
    }

    #[tokio::test]
    async fn test_safe_metrics_collect_success() {
        let result: i32 = SafeMetrics::collect::<_, i32, String>(
            async { Ok::<i32, String>(42) },
            0,
        ).await;
        assert_eq!(result, 42);
    }

    #[tokio::test]
    async fn test_safe_metrics_collect_failure() {
        let result: i32 = SafeMetrics::collect::<_, i32, String>(
            async { Err::<i32, String>("error".to_string()) },
            99,
        ).await;
        assert_eq!(result, 99);
    }

    #[test]
    fn test_validation_builder_range_ok() {
        let result = ValidationBuilder::new("test")
            .check_range("value", 50, 1, 100, "must be 1-100")
            .finalize();
        assert!(result.is_ok());
    }

    #[test]
    fn test_validation_builder_range_fail() {
        let result = ValidationBuilder::new("test")
            .check_range("value", 150, 1, 100, "must be 1-100")
            .finalize();
        assert!(result.is_err());
    }

    #[test]
    fn test_validation_builder_positive() {
        let result = ValidationBuilder::new("test")
            .check_positive("timeout", -1, "must be positive")
            .finalize();
        assert!(result.is_err());
    }

    #[test]
    fn test_validation_builder_string() {
        let result = ValidationBuilder::new("test")
            .check_string_not_empty("username", "")
            .finalize();
        assert!(result.is_err());
    }

    #[test]
    fn test_iterator_helpers_take_collect() {
        let vec = vec![1, 2, 3, 4, 5];
        let result = IteratorHelpers::take_collect(vec, 3);
        assert_eq!(result, vec![1, 2, 3]);
    }
}
