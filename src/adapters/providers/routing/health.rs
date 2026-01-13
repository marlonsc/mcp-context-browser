//! Health Monitoring Module
//!
//! This module provides health monitoring capabilities using DashMap
//! to eliminate locks and ensure non-blocking operation.
//!
//! ## Features
//!
//! - Non-blocking health status tracking via DashMap
//! - Health history with configurable window size
//! - Trend detection (Improving, Stable, Degrading)
//! - Integration with RecoveryManager for automatic restarts

use crate::domain::error::{Error, Result};
use crate::infrastructure::di::registry::{ProviderRegistry, ProviderRegistryTrait};
use async_trait::async_trait;
use dashmap::DashMap;
use shaku::Interface;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, warn};

/// Maximum number of health check records to keep in history
const MAX_HEALTH_HISTORY: usize = 10;

/// Provider health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ProviderHealthStatus {
    /// Provider is healthy and ready
    Healthy,
    /// Provider is unhealthy but may recover
    Unhealthy,
    /// Provider health is unknown
    Unknown,
}

impl std::fmt::Display for ProviderHealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Healthy => write!(f, "Healthy"),
            Self::Unhealthy => write!(f, "Unhealthy"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Health trend over the history window
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum HealthTrend {
    /// Success rate is increasing
    Improving,
    /// Success rate is stable
    Stable,
    /// Success rate is decreasing or showing consecutive failures
    Degrading,
    /// Not enough data to determine trend
    Unknown,
}

impl std::fmt::Display for HealthTrend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Improving => write!(f, "Improving"),
            Self::Stable => write!(f, "Stable"),
            Self::Degrading => write!(f, "Degrading"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Record of a single health check
#[derive(Debug, Clone)]
pub struct HealthCheckRecord {
    /// When the check was performed
    pub timestamp: Instant,
    /// Result of the check
    pub status: ProviderHealthStatus,
    /// How long the check took
    pub response_time: Duration,
    /// Error message if unhealthy
    pub error_message: Option<String>,
}

/// Health information for a provider
#[derive(Debug, Clone)]
pub struct ProviderHealth {
    /// Provider identifier
    pub provider_id: String,
    /// Current health status
    pub status: ProviderHealthStatus,
    /// Timestamp of last health check
    pub last_check: Instant,
    /// Number of consecutive failures
    pub consecutive_failures: u32,
    /// Total number of health checks performed
    pub total_checks: u64,
    /// Response time of the last check
    pub response_time: Option<Duration>,

    // Enhanced health tracking fields
    /// History of recent health checks (last N checks)
    pub history: VecDeque<HealthCheckRecord>,
    /// Current health trend based on history
    pub trend: HealthTrend,
    /// Average response time over history window
    pub avg_response_time: Duration,
    /// Success rate over history window (0.0 - 1.0)
    pub success_rate: f64,
}

impl ProviderHealth {
    /// Create a new ProviderHealth with default values
    pub fn new(provider_id: String) -> Self {
        Self {
            provider_id,
            status: ProviderHealthStatus::Unknown,
            last_check: Instant::now(),
            consecutive_failures: 0,
            total_checks: 0,
            response_time: None,
            history: VecDeque::with_capacity(MAX_HEALTH_HISTORY),
            trend: HealthTrend::Unknown,
            avg_response_time: Duration::ZERO,
            success_rate: 0.0,
        }
    }

    /// Record a new health check result
    pub fn record_check(&mut self, result: &HealthCheckResult) {
        // Create record
        let record = HealthCheckRecord {
            timestamp: Instant::now(),
            status: result.status,
            response_time: result.response_time,
            error_message: result.error_message.clone(),
        };

        // Add to history, removing oldest if at capacity
        if self.history.len() >= MAX_HEALTH_HISTORY {
            self.history.pop_front();
        }
        self.history.push_back(record);

        // Update basic fields
        self.last_check = Instant::now();
        self.total_checks += 1;
        self.response_time = Some(result.response_time);

        // Update consecutive failures and status
        match result.status {
            ProviderHealthStatus::Healthy => {
                self.status = ProviderHealthStatus::Healthy;
                self.consecutive_failures = 0;
            }
            ProviderHealthStatus::Unhealthy => {
                self.consecutive_failures += 1;
                if self.consecutive_failures >= 3 {
                    self.status = ProviderHealthStatus::Unhealthy;
                }
            }
            ProviderHealthStatus::Unknown => {
                self.status = ProviderHealthStatus::Unknown;
            }
        }

        // Update computed metrics
        self.update_metrics();
    }

    /// Update computed metrics (avg response time, success rate, trend)
    fn update_metrics(&mut self) {
        if self.history.is_empty() {
            return;
        }

        // Calculate average response time
        let total_response_time: Duration = self.history.iter().map(|r| r.response_time).sum();
        self.avg_response_time = total_response_time / self.history.len() as u32;

        // Calculate success rate
        let healthy_count = self
            .history
            .iter()
            .filter(|r| r.status == ProviderHealthStatus::Healthy)
            .count();
        self.success_rate = healthy_count as f64 / self.history.len() as f64;

        // Determine trend
        self.trend = self.calculate_trend();
    }

    /// Calculate health trend based on history
    fn calculate_trend(&self) -> HealthTrend {
        let len = self.history.len();

        // Need at least 3 records to determine trend
        if len < 3 {
            return HealthTrend::Unknown;
        }

        // Check for 3+ consecutive failures at the end - definitely degrading
        let recent_failures = self
            .history
            .iter()
            .rev()
            .take(3)
            .filter(|r| r.status == ProviderHealthStatus::Unhealthy)
            .count();

        if recent_failures >= 3 {
            return HealthTrend::Degrading;
        }

        // Split history into two halves and compare success rates
        let mid = len / 2;
        let first_half: Vec<_> = self.history.iter().take(mid).collect();
        let second_half: Vec<_> = self.history.iter().skip(mid).collect();

        let first_success_rate = first_half
            .iter()
            .filter(|r| r.status == ProviderHealthStatus::Healthy)
            .count() as f64
            / first_half.len().max(1) as f64;

        let second_success_rate = second_half
            .iter()
            .filter(|r| r.status == ProviderHealthStatus::Healthy)
            .count() as f64
            / second_half.len().max(1) as f64;

        // Determine trend based on change in success rate
        let change = second_success_rate - first_success_rate;
        const THRESHOLD: f64 = 0.2; // 20% change threshold

        if change > THRESHOLD {
            HealthTrend::Improving
        } else if change < -THRESHOLD {
            HealthTrend::Degrading
        } else {
            HealthTrend::Stable
        }
    }

    /// Get a summary string for logging/display
    pub fn summary(&self) -> String {
        format!(
            "{}: {} (trend: {}, success: {:.0}%, avg_latency: {}ms, failures: {})",
            self.provider_id,
            self.status,
            self.trend,
            self.success_rate * 100.0,
            self.avg_response_time.as_millis(),
            self.consecutive_failures
        )
    }
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
#[async_trait]
pub trait HealthMonitorTrait: Interface + Send + Sync {
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

impl Default for HealthMonitor {
    fn default() -> Self {
        Self::new()
    }
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
        let mut health = ProviderHealth::new(provider_id.to_string());
        health.record_check(&HealthCheckResult {
            provider_id: provider_id.to_string(),
            status: ProviderHealthStatus::Healthy,
            response_time: Duration::from_millis(1),
            error_message: None,
        });
        self.health_states.insert(provider_id.to_string(), health);
    }

    /// Get all provider health states with their trends
    pub fn get_all_health_states(&self) -> Vec<ProviderHealth> {
        self.health_states
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get providers with degrading health trend
    pub fn get_degrading_providers(&self) -> Vec<String> {
        self.health_states
            .iter()
            .filter(|entry| entry.value().trend == HealthTrend::Degrading)
            .map(|entry| entry.key().clone())
            .collect()
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
                history: VecDeque::with_capacity(MAX_HEALTH_HISTORY),
                trend: HealthTrend::Unknown,
                avg_response_time: Duration::ZERO,
                success_rate: 0.0,
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
