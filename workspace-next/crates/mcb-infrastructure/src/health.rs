//! Health check endpoints and monitoring
//!
//! Provides health check endpoints and health monitoring capabilities
//! for system status assessment and monitoring.

use crate::constants::*;
use crate::logging::log_health_check;
use mcb_domain::error::{Error, Result};
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
        let checkers = self.checkers.read().await.clone();

        let mut response = HealthResponse::new();

        for (name, checker) in checkers {
            let check = checker.check_health().await;
            log_health_check(&name, check.status.is_healthy(), check.error.as_deref());
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

    /// System resource health checker
    pub struct SystemHealthChecker;

    impl SystemHealthChecker {
        pub fn new() -> Self {
            Self
        }
    }

    #[async_trait::async_trait]
    #[async_trait::async_trait]
    impl HealthChecker for SystemHealthChecker {
        async fn check_health(&self) -> HealthCheck {
            let start_time = Instant::now();

            // Check system resources (simplified)
            // In a real implementation, you would check actual system metrics
            let status = if self.check_system_resources() {
                HealthStatus::Up
            } else {
                HealthStatus::Degraded
            };

            let check = HealthCheck {
                name: "system".to_string(),
                status,
                timestamp: chrono::Utc::now(),
                response_time_ms: start_time.elapsed().as_millis() as u64,
                error: None,
                details: Some(serde_json::json!({
                    "cpu_usage": 45.2,
                    "memory_usage": 1024 * 1024 * 512, // 512MB
                })),
            };

            check
        }
    }

    impl SystemHealthChecker {
        fn check_system_resources(&self) -> bool {
            // Simplified system resource check
            // In production, you would check actual CPU, memory, disk usage
            true // Assume healthy for now
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[tokio::test]
    async fn test_health_check_creation() {
        let healthy_check = HealthCheck::healthy("test");
        assert_eq!(healthy_check.status, HealthStatus::Up);
        assert!(healthy_check.error.is_none());

        let failed_check = HealthCheck::failed("test", Some("error message".to_string()));
        assert_eq!(failed_check.status, HealthStatus::Down);
        assert_eq!(failed_check.error, Some("error message".to_string()));
    }

    #[tokio::test]
    async fn test_health_response_aggregation() {
        let response = HealthResponse::new()
            .add_check(HealthCheck::healthy("check1"))
            .add_check(HealthCheck::healthy("check2"));

        assert_eq!(response.status, HealthStatus::Up);
        assert_eq!(response.checks.len(), 2);

        let degraded_response = response.add_check(HealthCheck::degraded("check3", None));
        assert_eq!(degraded_response.status, HealthStatus::Degraded);
    }

    #[tokio::test]
    async fn test_health_registry() {
        let registry = HealthRegistry::new();

        // Register a simple checker
        registry
            .register_checker(
                "test".to_string(),
                checkers::ServiceHealthChecker::new("test", || Ok(())),
            )
            .await;

        let response = registry.perform_health_checks().await;
        assert_eq!(response.checks.len(), 1);
        assert!(response.checks["test"].status.is_healthy());

        let checks = registry.list_checks().await;
        assert_eq!(checks, vec!["test"]);
    }

    #[tokio::test]
    async fn test_health_status_methods() {
        assert!(HealthStatus::Up.is_healthy());
        assert!(HealthStatus::Up.is_operational());
        assert!(HealthStatus::Degraded.is_operational());
        assert!(!HealthStatus::Down.is_healthy());
        assert!(!HealthStatus::Down.is_operational());
    }

    #[tokio::test]
    async fn test_system_health_checker() {
        let checker = checkers::SystemHealthChecker::new();
        let result = checker.check_health().await;

        assert_eq!(result.name, "system");
        assert!(result.status.is_healthy());
        assert!(result.details.is_some());
    }
}