//! System Metrics Collector Port
//!
//! Defines the contract for collecting system metrics.

use async_trait::async_trait;
use mcb_domain::error::Result;
use shaku::Interface;

/// System metrics data
#[derive(Debug, Clone, Default)]
pub struct SystemMetrics {
    pub cpu_percent: f64,
    pub memory_percent: f64,
    pub disk_percent: f64,
}

/// System metrics collector interface
#[async_trait]
pub trait SystemMetricsCollectorInterface: Interface + Send + Sync {
    /// Collect current system metrics
    async fn collect(&self) -> Result<SystemMetrics>;

    /// Get CPU usage percentage
    fn cpu_usage(&self) -> f64;

    /// Get memory usage percentage
    fn memory_usage(&self) -> f64;
}
