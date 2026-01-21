//! Health Monitoring for Provider Routing
//!
//! Tracks provider health status based on success/failure reports.

use async_trait::async_trait;
use dashmap::DashMap;
use mcb_domain::ports::infrastructure::routing::ProviderHealthStatus;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Instant;

/// Health data for a single provider
#[derive(Debug)]
pub struct ProviderHealthData {
    /// Current health status
    pub status: ProviderHealthStatus,
    /// Last health check time
    pub last_check: Instant,
    /// Consecutive failure count
    pub failure_count: AtomicU32,
    /// Total success count
    pub success_count: AtomicU32,
}

impl Default for ProviderHealthData {
    fn default() -> Self {
        Self {
            status: ProviderHealthStatus::Healthy,
            last_check: Instant::now(),
            failure_count: AtomicU32::new(0),
            success_count: AtomicU32::new(0),
        }
    }
}

/// Health monitor interface for DI
///
/// # Example
///
/// ```ignore
/// // Get health status of a provider
/// let status = monitor.get_health("ollama-embedding");
/// match status {
///     ProviderHealthStatus::Healthy => println!("Provider is healthy"),
///     ProviderHealthStatus::Degraded => println!("Provider is degraded"),
///     ProviderHealthStatus::Unhealthy => println!("Provider is unhealthy"),
/// }
///
/// // Record operation results
/// monitor.record_success("ollama-embedding");
/// monitor.record_failure("milvus-store");
///
/// // Get all health statuses
/// let all_health = monitor.get_all_health();
/// for (provider, status) in all_health {
///     println!("{}: {:?}", provider, status);
/// }
/// ```
#[async_trait]
pub trait HealthMonitor: Send + Sync {
    /// Get health status for a provider
    fn get_health(&self, provider_id: &str) -> ProviderHealthStatus;

    /// Record a successful operation
    fn record_success(&self, provider_id: &str);

    /// Record a failed operation
    fn record_failure(&self, provider_id: &str);

    /// Get all provider health statuses
    fn get_all_health(&self) -> std::collections::HashMap<String, ProviderHealthStatus>;
}

/// In-memory health monitor implementation
///
/// Tracks provider health using concurrent hash maps and atomic counters.
/// Uses configurable failure threshold to determine health status.
pub struct InMemoryHealthMonitor {
    /// Health data per provider
    health_data: DashMap<String, ProviderHealthData>,
    /// Number of consecutive failures before marking unhealthy
    failure_threshold: u32,
    /// Number of consecutive failures before marking degraded
    degraded_threshold: u32,
}

impl InMemoryHealthMonitor {
    /// Create a new health monitor with default thresholds
    pub fn new() -> Self {
        Self {
            health_data: DashMap::new(),
            failure_threshold: 5,
            degraded_threshold: 2,
        }
    }

    /// Create with custom thresholds
    pub fn with_thresholds(degraded_threshold: u32, failure_threshold: u32) -> Self {
        Self {
            health_data: DashMap::new(),
            failure_threshold,
            degraded_threshold,
        }
    }

    /// Get or create health data for a provider
    fn get_or_create(
        &self,
        provider_id: &str,
    ) -> dashmap::mapref::one::RefMut<'_, String, ProviderHealthData> {
        self.health_data.entry(provider_id.to_string()).or_default()
    }

    /// Calculate status based on failure count
    fn calculate_status(&self, failure_count: u32) -> ProviderHealthStatus {
        if failure_count >= self.failure_threshold {
            ProviderHealthStatus::Unhealthy
        } else if failure_count >= self.degraded_threshold {
            ProviderHealthStatus::Degraded
        } else {
            ProviderHealthStatus::Healthy
        }
    }
}

impl Default for InMemoryHealthMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl HealthMonitor for InMemoryHealthMonitor {
    fn get_health(&self, provider_id: &str) -> ProviderHealthStatus {
        self.health_data
            .get(provider_id)
            .map(|data| data.status)
            .unwrap_or(ProviderHealthStatus::Healthy)
    }

    fn record_success(&self, provider_id: &str) {
        let mut data = self.get_or_create(provider_id);
        // Reset failure count on success
        data.failure_count.store(0, Ordering::SeqCst);
        data.success_count.fetch_add(1, Ordering::SeqCst);
        data.status = ProviderHealthStatus::Healthy;
        data.last_check = Instant::now();
    }

    fn record_failure(&self, provider_id: &str) {
        let mut data = self.get_or_create(provider_id);
        let failures = data.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
        data.status = self.calculate_status(failures);
        data.last_check = Instant::now();
    }

    fn get_all_health(&self) -> std::collections::HashMap<String, ProviderHealthStatus> {
        self.health_data
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().status))
            .collect()
    }
}

/// Null health monitor for testing
///
/// Always reports all providers as healthy.
pub struct NullHealthMonitor;

impl NullHealthMonitor {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NullHealthMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl HealthMonitor for NullHealthMonitor {
    fn get_health(&self, _provider_id: &str) -> ProviderHealthStatus {
        ProviderHealthStatus::Healthy
    }

    fn record_success(&self, _provider_id: &str) {
        // No-op
    }

    fn record_failure(&self, _provider_id: &str) {
        // No-op
    }

    fn get_all_health(&self) -> std::collections::HashMap<String, ProviderHealthStatus> {
        std::collections::HashMap::new()
    }
}
