//! Health check endpoints and monitoring
//!
//! Provides health check endpoints and health monitoring capabilities
//! for system status assessment and monitoring.

use crate::logging::log_health_check;
use mcb_domain::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Health status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    /// Service is healthy and fully operational
    Up,
    /// Service is experiencing issues but still operational
    Degraded,
    /// Service is down and not operational
    Down,
}

impl HealthStatus {
    /// Check if the status indicates the service is healthy
    pub fn is_healthy(&self) -> bool {
        matches!(self, Self::Up)
    }

    /// Check if the service is operational (healthy or degraded)
    pub fn is_operational(&self) -> bool {
        matches!(self, Self::Up | Self::Degraded)
    }
}

/// Individual health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    /// Name of the health check
    pub name: String,
    /// Current status
    pub status: HealthStatus,
    /// Timestamp of last check
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Response time in milliseconds
    pub response_time_ms: u64,
    /// Optional error message
    pub error: Option<String>,
    /// Additional details
    pub details: Option<serde_json::Value>,
}

impl HealthCheck {
    /// Create a successful health check
    pub fn healthy<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Up,
            timestamp: chrono::Utc::now(),
            response_time_ms: 0,
            error: None,
            details: None,
        }
    }

    /// Create a failed health check
    pub fn failed<S: Into<String>>(name: S, error: Option<String>) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Down,
            timestamp: chrono::Utc::now(),
            response_time_ms: 0,
            error,
            details: None,
        }
    }

    /// Create a degraded health check
    pub fn degraded<S: Into<String>>(name: S, details: Option<String>) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Degraded,
            timestamp: chrono::Utc::now(),
            response_time_ms: 0,
            error: details,
            details: None,
        }
    }

    /// Set response time
    pub fn with_response_time(mut self, duration: Duration) -> Self {
        self.response_time_ms = duration.as_millis() as u64;
        self
    }

    /// Set additional details
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}

/// Overall health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Overall system status
    pub status: HealthStatus,
    /// Timestamp of the health check
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Total response time in milliseconds
    pub response_time_ms: u64,
    /// Individual health check results
    pub checks: HashMap<String, HealthCheck>,
    /// System information
    pub system: SystemInfo,
}

impl Default for HealthResponse {
    fn default() -> Self {
        Self::new()
    }
}

impl HealthResponse {
    /// Create a new health response
    pub fn new() -> Self {
        Self {
            status: HealthStatus::Up,
            timestamp: chrono::Utc::now(),
            response_time_ms: 0,
            checks: HashMap::new(),
            system: SystemInfo::default(),
        }
    }

    /// Add a health check result
    pub fn add_check(mut self, check: HealthCheck) -> Self {
        // Update overall status based on individual check
        if check.status == HealthStatus::Down {
            self.status = HealthStatus::Down;
        } else if check.status == HealthStatus::Degraded && self.status == HealthStatus::Up {
            self.status = HealthStatus::Degraded;
        }

        self.checks.insert(check.name.clone(), check);
        self
    }

    /// Set response time
    pub fn with_response_time(mut self, duration: Duration) -> Self {
        self.response_time_ms = duration.as_millis() as u64;
        self
    }

    /// Check if the overall system is healthy
    pub fn is_healthy(&self) -> bool {
        self.status.is_healthy()
    }
}

/// System information for health responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    /// System uptime in seconds
    pub uptime_seconds: u64,
    /// Memory usage in bytes
    pub memory_used_bytes: u64,
    /// Memory available in bytes
    pub memory_available_bytes: u64,
    /// CPU usage percentage
    pub cpu_usage_percent: f64,
    /// Number of active connections
    pub active_connections: u32,
    /// Version information
    pub version: String,
}

impl Default for SystemInfo {
    fn default() -> Self {
        Self {
            uptime_seconds: 0,
            memory_used_bytes: 0,
            memory_available_bytes: 0,
            cpu_usage_percent: 0.0,
            active_connections: 0,
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

/// Health check function trait
///
/// # Example
///
/// ```no_run
/// use mcb_infrastructure::health::{HealthChecker, HealthCheck};
/// use async_trait::async_trait;
///
/// struct DatabaseHealthChecker;
///
/// #[async_trait]
/// impl HealthChecker for DatabaseHealthChecker {
///     async fn check_health(&self) -> HealthCheck {
///         HealthCheck::healthy("database")
///     }
/// }
/// ```
#[async_trait::async_trait]
pub trait HealthChecker: Send + Sync {
    /// Perform a health check
    async fn check_health(&self) -> HealthCheck;
}

/// Health check registry
#[derive(Clone)]
pub struct HealthRegistry {
    checkers: Arc<RwLock<HashMap<String, Box<dyn HealthChecker>>>>,
}

impl HealthRegistry {
    /// Create a new health registry
    pub fn new() -> Self {
        Self {
            checkers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a health checker
    pub async fn register_checker<C>(&self, name: String, checker: C)
    where
        C: HealthChecker + 'static,
    {
        self.checkers.write().await.insert(name, Box::new(checker));
    }

    /// Unregister a health checker
    pub async fn unregister_checker(&self, name: &str) {
        self.checkers.write().await.remove(name);
    }

    /// Perform all registered health checks
    pub async fn perform_health_checks(&self) -> HealthResponse {
        let start_time = Instant::now();
        let checkers = self.checkers.read().await;

        let mut response = HealthResponse::new();

        for (name, checker) in checkers.iter() {
            let check = checker.check_health().await;
            log_health_check(name, check.status.is_healthy(), check.error.as_deref());
            response = response.add_check(check);
        }

        response.with_response_time(start_time.elapsed())
    }

    /// Get a list of registered health check names
    pub async fn list_checks(&self) -> Vec<String> {
        self.checkers.read().await.keys().cloned().collect()
    }
}

impl Default for HealthRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Built-in health checkers
pub mod checkers {
    use super::*;

    /// Database connectivity health checker
    pub struct DatabaseHealthChecker<F> {
        check_fn: F,
    }

    impl<F> DatabaseHealthChecker<F> {
        pub fn new(check_fn: F) -> Self
        where
            F: Fn() -> Result<()> + Send + Sync,
        {
            Self { check_fn }
        }
    }

    #[async_trait::async_trait]
    impl<F> HealthChecker for DatabaseHealthChecker<F>
    where
        F: Fn() -> Result<()> + Send + Sync,
    {
        async fn check_health(&self) -> HealthCheck {
            let start_time = Instant::now();

            match (self.check_fn)() {
                Ok(_) => HealthCheck::healthy("database").with_response_time(start_time.elapsed()),
                Err(e) => HealthCheck::failed("database", Some(e.to_string()))
                    .with_response_time(start_time.elapsed()),
            }
        }
    }

    /// External service health checker
    pub struct ServiceHealthChecker<F> {
        name: String,
        check_fn: F,
    }

    impl<F> ServiceHealthChecker<F> {
        pub fn new<S: Into<String>>(name: S, check_fn: F) -> Self
        where
            F: Fn() -> Result<()> + Send + Sync,
        {
            Self {
                name: name.into(),
                check_fn,
            }
        }
    }

    #[async_trait::async_trait]
    impl<F> HealthChecker for ServiceHealthChecker<F>
    where
        F: Fn() -> Result<()> + Send + Sync,
    {
        async fn check_health(&self) -> HealthCheck {
            let start_time = Instant::now();

            match (self.check_fn)() {
                Ok(_) => HealthCheck::healthy(&self.name).with_response_time(start_time.elapsed()),
                Err(e) => HealthCheck::failed(&self.name, Some(e.to_string()))
                    .with_response_time(start_time.elapsed()),
            }
        }
    }

    /// System resource health checker using real system metrics
    pub struct SystemHealthChecker {
        cpu_threshold_percent: f32,
        memory_threshold_percent: f64,
    }

    impl Default for SystemHealthChecker {
        fn default() -> Self {
            Self::new()
        }
    }

    impl SystemHealthChecker {
        /// Create a new system health checker with default thresholds
        ///
        /// Default thresholds: CPU > 90%, Memory > 90%
        pub fn new() -> Self {
            Self {
                cpu_threshold_percent: 90.0,
                memory_threshold_percent: 90.0,
            }
        }

        /// Create with custom thresholds
        pub fn with_thresholds(cpu_threshold: f32, memory_threshold: f64) -> Self {
            Self {
                cpu_threshold_percent: cpu_threshold,
                memory_threshold_percent: memory_threshold,
            }
        }
    }

    #[async_trait::async_trait]
    impl HealthChecker for SystemHealthChecker {
        async fn check_health(&self) -> HealthCheck {
            use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};

            let start_time = Instant::now();

            // Create system with specific refresh kinds for efficiency
            let mut sys = System::new_with_specifics(
                RefreshKind::nothing()
                    .with_cpu(CpuRefreshKind::everything())
                    .with_memory(MemoryRefreshKind::everything()),
            );

            // Refresh CPU metrics (requires two reads with a small delay)
            // Note: sysinfo::System is not Send, so we use spawn_blocking
            // to avoid blocking the async runtime
            sys.refresh_cpu_all();
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            sys.refresh_cpu_all();

            // Get CPU usage (global average)
            let cpu_usage = sys.global_cpu_usage();

            // Get memory metrics
            let total_memory = sys.total_memory();
            let used_memory = sys.used_memory();
            let memory_percent = if total_memory > 0 {
                (used_memory as f64 / total_memory as f64) * 100.0
            } else {
                0.0
            };

            // Determine health status based on thresholds
            let status = if cpu_usage > self.cpu_threshold_percent
                || memory_percent > self.memory_threshold_percent
            {
                HealthStatus::Degraded
            } else {
                HealthStatus::Up
            };

            HealthCheck {
                name: "system".to_string(),
                status,
                timestamp: chrono::Utc::now(),
                response_time_ms: start_time.elapsed().as_millis() as u64,
                error: None,
                details: Some(serde_json::json!({
                    "cpu_usage_percent": cpu_usage,
                    "memory_used_bytes": used_memory,
                    "memory_total_bytes": total_memory,
                    "memory_usage_percent": memory_percent,
                    "cpu_threshold_percent": self.cpu_threshold_percent,
                    "memory_threshold_percent": self.memory_threshold_percent,
                })),
            }
        }
    }
}

/// Health check utilities
pub struct HealthUtils;

impl HealthUtils {
    /// Create a simple health check response for HTTP endpoints
    pub async fn simple_health_response() -> HealthResponse {
        HealthResponse::new()
            .add_check(HealthCheck::healthy("service"))
            .with_response_time(Duration::from_millis(1))
    }

    /// Create a detailed health check with system information
    pub async fn detailed_health_response(registry: &HealthRegistry) -> HealthResponse {
        registry.perform_health_checks().await
    }

    /// Check if a health response indicates the system is ready for traffic
    pub fn is_ready(response: &HealthResponse) -> bool {
        response.status.is_operational()
    }

    /// Check if a health response indicates the system is alive
    pub fn is_alive(response: &HealthResponse) -> bool {
        response.status != HealthStatus::Down
    }
}
