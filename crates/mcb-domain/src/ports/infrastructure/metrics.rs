//! System Metrics Collector Port
//!
//! Defines the contract for collecting system metrics.

use crate::error::Result;
use async_trait::async_trait;

/// System metrics data
#[derive(Debug, Clone, Default)]
pub struct SystemMetrics {
    /// CPU usage percentage
    pub cpu_percent: f64,
    /// Memory usage percentage
    pub memory_percent: f64,
    /// Disk usage percentage
    pub disk_percent: f64,
}

/// System metrics collector interface
#[async_trait]
pub trait SystemMetricsCollectorInterface: Send + Sync {
    /// Collect current system metrics
    async fn collect(&self) -> Result<SystemMetrics>;

    /// Get CPU usage percentage
    fn cpu_usage(&self) -> f64;

    /// Get memory usage percentage
    fn memory_usage(&self) -> f64;
}
