//! System Metrics Adapter
//!
//! Null implementation of the system metrics port for testing.

use async_trait::async_trait;
use mcb_domain::error::Result;
use mcb_domain::ports::infrastructure::{SystemMetrics, SystemMetricsCollectorInterface};

/// Null implementation for testing
#[derive(shaku::Component)]
#[shaku(interface = SystemMetricsCollectorInterface)]
pub struct NullSystemMetricsCollector;

impl NullSystemMetricsCollector {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NullSystemMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SystemMetricsCollectorInterface for NullSystemMetricsCollector {
    async fn collect(&self) -> Result<SystemMetrics> {
        Ok(SystemMetrics::default())
    }
    fn cpu_usage(&self) -> f64 {
        0.0
    }
    fn memory_usage(&self) -> f64 {
        0.0
    }
}
