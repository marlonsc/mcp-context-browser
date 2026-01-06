//! Basic system metrics collection for MCP Context Browser v0.0.3
//!
//! Simple metrics collection without HTTP server for initial implementation.

use serde::{Deserialize, Serialize};
use sysinfo::System;

pub mod performance;

pub use performance::{PerformanceMetrics, QueryPerformanceMetrics, CacheMetrics};

/// CPU usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuMetrics {
    /// CPU usage percentage (0-100)
    pub usage: f32,
    /// Number of CPU cores
    pub cores: usize,
    /// CPU model name
    pub model: String,
}

/// Memory usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetrics {
    /// Total memory in bytes
    pub total: u64,
    /// Used memory in bytes
    pub used: u64,
    /// Free memory in bytes
    pub free: u64,
    /// Memory usage percentage (0-100)
    pub usage_percent: f32,
}

/// System metrics collector
pub struct SystemMetricsCollector {
    system: System,
}

impl SystemMetricsCollector {
    /// Create a new system metrics collector
    pub fn new() -> Self {
        let mut system = System::new();
        system.refresh_all();

        Self { system }
    }

    /// Collect CPU metrics
    pub fn collect_cpu_metrics(&mut self) -> CpuMetrics {
        self.system.refresh_cpu();

        let cpus = self.system.cpus();
        let cores = cpus.len();

        // For sysinfo 0.30, CPU usage is available
        let usage = if cores > 0 {
            cpus.iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() / cores as f32
        } else {
            0.0
        };

        let model = if cores > 0 {
            cpus[0].brand().to_string()
        } else {
            "Unknown".to_string()
        };

        CpuMetrics {
            usage,
            cores,
            model,
        }
    }

    /// Collect memory metrics
    pub fn collect_memory_metrics(&mut self) -> MemoryMetrics {
        self.system.refresh_memory();

        let total = self.system.total_memory();
        let used = self.system.used_memory();
        let free = total.saturating_sub(used);
        let usage_percent = if total > 0 { (used as f32 / total as f32) * 100.0 } else { 0.0 };

        MemoryMetrics {
            total,
            used,
            free,
            usage_percent,
        }
    }
}

impl Default for SystemMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}