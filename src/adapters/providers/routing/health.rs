//! Health Monitoring Module
//!
//! This module provides health monitoring capabilities using DashMap
//! to eliminate locks and ensure non-blocking operation.

use crate::domain::error::{Error, Result};
use crate::infrastructure::di::registry::{ProviderRegistry, ProviderRegistryTrait};
use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, warn};

/// Provider health status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderHealthStatus {
    /// Provider is healthy and ready
    Healthy,
    /// Provider is unhealthy but may recover
    Unhealthy,
    /// Provider health is unknown
    Unknown,
}

/// Health information for a provider
#[derive(Debug, Clone)]
pub struct ProviderHealth {
    pub provider_id: String,
    pub status: ProviderHealthStatus,
    pub last_check: Instant,
    pub consecutive_failures: u32,
    pub total_checks: u64,
    pub response_time: Option<Duration>,
}

/// Health check result
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    pub provider_id: String,
    pub status: ProviderHealthStatus,
    pub response_time: Duration,
    pub error_message: Option<String>,
}

/// Trait for provider health checkers
#[async_trait::async_trait]
pub trait ProviderHealthChecker: Send + Sync {
    /// Perform a health check for a specific provider
    async fn check_health(&self, provider_id: &str) -> Result<HealthCheckResult>;
}

/// Trait for health monitoring
#[async_trait::async_trait]
pub trait HealthMonitorTrait: Send + Sync {
    async fn is_healthy(&self, provider_id: &str) -> bool;
    async fn get_health(&self, provider_id: &str) -> Option<ProviderHealth>;
    async fn record_result(&self, result: HealthCheckResult);
    async fn list_healthy_providers(&self) -> Vec<String>;
    async fn check_provider(&self, provider_id: &str) -> Result<()>;
}

/// Real provider health checker that performs actual health checks
pub struct RealProviderHealthChecker {
    registry: Arc<crate::infrastructure::di::registry::ProviderRegistry>,
    timeout: Duration,
}

impl RealProviderHealthChecker {
    /// Create a new real provider health checker
    pub fn new(registry: Arc<crate::infrastructure::di::registry::ProviderRegistry>) -> Self {
        Self {
            registry,
            timeout: Duration::from_secs(10), // Default timeout
        }
    }

    /// Create with custom timeout
    pub fn with_timeout(
        registry: Arc<crate::infrastructure::di::registry::ProviderRegistry>,
        timeout: Duration,
    ) -> Self {
        Self { registry, timeout }
    }

    /// Check health of an embedding provider
    async fn check_embedding_provider(&self, provider_id: &str) -> Result<HealthCheckResult> {
        let start_time = Instant::now();

        match self.registry.get_embedding_provider(provider_id) {
            Ok(provider) => {
                // Perform a lightweight health check - try to get dimensions
                // This is a minimal operation that verifies the provider is accessible
                match tokio::time::timeout(self.timeout, async {
                    let _ = provider.dimensions();
                    Ok::<(), Error>(())
                })
                .await
                {
                    Ok(Ok(_)) => {
                        let response_time = start_time.elapsed();
                        Ok(HealthCheckResult {
                            provider_id: provider_id.to_string(),
                            status: ProviderHealthStatus::Healthy,
                            response_time,
                            error_message: None,
                        })
                    }
                    Ok(Err(e)) => {
                        let response_time = start_time.elapsed();
                        Ok(HealthCheckResult {
                            provider_id: provider_id.to_string(),
                            status: ProviderHealthStatus::Unhealthy,
                            response_time,
                            error_message: Some(format!("Provider error: {}", e)),
                        })
                    }
                    Err(_) => {
                        let response_time = start_time.elapsed();
                        Ok(HealthCheckResult {
                            provider_id: provider_id.to_string(),
                            status: ProviderHealthStatus::Unhealthy,
                            response_time,
                            error_message: Some("Health check timed out".to_string()),
                        })
                    }
                }
            }
            Err(e) => Err(Error::not_found(format!(
                "Provider {} not found in registry: {}",
                provider_id, e
            ))),
        }
    }

    /// Check health of a vector store provider
    async fn check_vector_store_provider(&self, provider_id: &str) -> Result<HealthCheckResult> {
        let start_time = Instant::now();

        match self.registry.get_vector_store_provider(provider_id) {
            Ok(provider) => {
                // Perform a lightweight health check - check if a reserved collection name exists
                // This is a safe operation that verifies connectivity
                match tokio::time::timeout(
                    self.timeout,
                    provider.collection_exists("__health_check__"),
                )
                .await
                {
                    Ok(Ok(_)) => {
                        let response_time = start_time.elapsed();
                        Ok(HealthCheckResult {
                            provider_id: provider_id.to_string(),
                            status: ProviderHealthStatus::Healthy,
                            response_time,
                            error_message: None,
                        })
                    }
                    Ok(Err(e)) => {
                        let response_time = start_time.elapsed();
                        Ok(HealthCheckResult {
                            provider_id: provider_id.to_string(),
                            status: ProviderHealthStatus::Unhealthy,
                            response_time,
                            error_message: Some(format!("Vector store error: {}", e)),
                        })
                    }
                    Err(_) => {
                        let response_time = start_time.elapsed();
                        Ok(HealthCheckResult {
                            provider_id: provider_id.to_string(),
                            status: ProviderHealthStatus::Unhealthy,
                            response_time,
                            error_message: Some("Health check timed out".to_string()),
                        })
                    }
                }
            }
            Err(e) => Err(Error::not_found(format!(
                "Provider {} not found in registry: {}",
                provider_id, e
            ))),
        }
    }
}

#[async_trait::async_trait]
impl ProviderHealthChecker for RealProviderHealthChecker {
    async fn check_health(&self, provider_id: &str) -> Result<HealthCheckResult> {
        // Try embedding provider first, then vector store
        if let Ok(result) = self.check_embedding_provider(provider_id).await {
            return Ok(result);
        }

        if let Ok(result) = self.check_vector_store_provider(provider_id).await {
            return Ok(result);
        }

        Err(Error::not_found(format!(
            "Provider {} not found in any registry",
            provider_id
        )))
    }
}

/// Health monitor coordinating health checks and tracking status
pub struct HealthMonitor {
    health_states: DashMap<String, ProviderHealth>,
    checker: Option<Arc<dyn ProviderHealthChecker>>,
}

impl HealthMonitor {
    /// Create a new health monitor
    pub fn new() -> Self {
        Self {
            health_states: DashMap::new(),
            checker: None,
        }
    }

    /// Create with a specific checker
    pub fn with_checker(checker: Arc<dyn ProviderHealthChecker>) -> Self {
        Self {
            health_states: DashMap::new(),
            checker: Some(checker),
        }
    }

    /// Create with a registry (uses RealProviderHealthChecker)
    pub fn with_registry(registry: Arc<ProviderRegistry>) -> Self {
        let checker = Arc::new(RealProviderHealthChecker::new(registry));
        Self::with_checker(checker)
    }

    /// Trigger a health check for a provider
    pub async fn check_provider(&self, provider_id: &str) -> Result<()> {
        if let Some(checker) = &self.checker {
            let result = checker.check_health(provider_id).await?;
            self.record_result(result).await;
            Ok(())
        } else {
            // If no checker, we can't really check, but we can return current status or unknown
            // For now, let's return an error if no checker is configured but check is requested
            Err(Error::generic("No health checker configured"))
        }
    }

    /// Mark a provider as healthy (for testing or manual override)
    pub fn mark_healthy(&self, provider_id: &str) {
        self.health_states.insert(
            provider_id.to_string(),
            ProviderHealth {
                provider_id: provider_id.to_string(),
                status: ProviderHealthStatus::Healthy,
                last_check: std::time::Instant::now(),
                consecutive_failures: 0,
                total_checks: 1,
                response_time: Some(std::time::Duration::from_millis(1)),
            },
        );
    }
}

#[async_trait::async_trait]
impl HealthMonitorTrait for HealthMonitor {
    /// Check if a provider is considered healthy
    /// Returns false if provider is unknown (fail-safe: unknown providers don't receive traffic)
    async fn is_healthy(&self, provider_id: &str) -> bool {
        self.health_states
            .get(provider_id)
            .map(|h| h.status == ProviderHealthStatus::Healthy)
            .unwrap_or(false) // Fail-safe: assume unhealthy if unknown
    }

    /// Get detailed health information for a provider
    async fn get_health(&self, provider_id: &str) -> Option<ProviderHealth> {
        self.health_states.get(provider_id).map(|h| h.clone())
    }

    /// Record a health check result
    async fn record_result(&self, result: HealthCheckResult) {
        let mut health = self
            .health_states
            .entry(result.provider_id.clone())
            .or_insert_with(|| ProviderHealth {
                provider_id: result.provider_id.clone(),
                status: ProviderHealthStatus::Unknown,
                last_check: Instant::now(),
                consecutive_failures: 0,
                total_checks: 0,
                response_time: None,
            });

        health.last_check = Instant::now();
        health.total_checks += 1;
        health.response_time = Some(result.response_time);

        match result.status {
            ProviderHealthStatus::Healthy => {
                health.status = ProviderHealthStatus::Healthy;
                health.consecutive_failures = 0;
                debug!("Provider {} is healthy", result.provider_id);
            }
            ProviderHealthStatus::Unhealthy => {
                health.consecutive_failures += 1;
                if health.consecutive_failures >= 3 {
                    health.status = ProviderHealthStatus::Unhealthy;
                    warn!(
                        "Provider {} marked as unhealthy after {} failures",
                        result.provider_id, health.consecutive_failures
                    );
                }
            }
            ProviderHealthStatus::Unknown => {
                health.status = ProviderHealthStatus::Unknown;
            }
        }
    }

    /// List all currently healthy provider IDs
    async fn list_healthy_providers(&self) -> Vec<String> {
        self.health_states
            .iter()
            .filter(|h| h.status == ProviderHealthStatus::Healthy)
            .map(|h| h.key().clone())
            .collect()
    }

    /// Perform a health check for a specific provider and record the result
    async fn check_provider(&self, provider_id: &str) -> Result<()> {
        if let Some(checker) = &self.checker {
            let result = checker.check_health(provider_id).await?;
            self.record_result(result).await;
            Ok(())
        } else {
            Err(Error::generic("No health checker configured"))
        }
    }
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_monitor_creation() {
        let monitor = HealthMonitor::new();
        // Unknown providers are considered unhealthy (fail-safe behavior)
        assert!(!monitor.is_healthy("any").await);
    }

    #[tokio::test]
    async fn test_provider_health_check_unregistered() {
        let registry = Arc::new(crate::infrastructure::di::registry::ProviderRegistry::new());
        let checker = RealProviderHealthChecker::new(registry);
        let result = checker.check_health("non-existent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_healthy_providers() {
        let monitor = HealthMonitor::new();
        monitor
            .record_result(HealthCheckResult {
                provider_id: "p1".to_string(),
                status: ProviderHealthStatus::Healthy,
                response_time: Duration::from_millis(10),
                error_message: None,
            })
            .await;

        let healthy = monitor.list_healthy_providers().await;
        assert_eq!(healthy.len(), 1);
        assert_eq!(healthy[0], "p1");
    }

    #[tokio::test]
    async fn test_real_provider_health_checker() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let registry = Arc::new(crate::infrastructure::di::registry::ProviderRegistry::new());
        let mock_provider =
            Arc::new(crate::adapters::providers::embedding::null::NullEmbeddingProvider::new());
        registry.register_embedding_provider("mock".to_string(), mock_provider)?;

        let checker = RealProviderHealthChecker::new(registry);
        let result = checker.check_health("mock").await?;
        assert_eq!(result.status, ProviderHealthStatus::Healthy);
        assert_eq!(result.provider_id, "mock");
        Ok(())
    }
}
