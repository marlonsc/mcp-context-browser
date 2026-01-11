//! Metrics module re-exports
//!
//! Provides access to system and performance metrics collectors.

pub mod http_server;
pub mod performance;
pub mod system;

pub use crate::domain::types::{
    CacheMetrics as DomainCacheMetrics, QueryPerformanceMetrics as DomainQueryPerformanceMetrics,
};
pub use http_server::{HealthResponse, MetricsApiServer};
pub use performance::{CacheMetrics, PerformanceMetrics, QueryPerformanceMetrics};
pub use system::{
    CpuMetrics, DiskMetrics, MemoryMetrics, NetworkMetrics, ProcessMetrics, SystemMetricsCollector,
};
