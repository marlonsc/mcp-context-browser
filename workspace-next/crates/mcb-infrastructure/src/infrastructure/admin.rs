//! Admin Service Null Implementations
//!
//! Null implementations of admin port traits for Shaku DI testing.
//! Real implementations are in mcb-providers and created at runtime.

use mcb_application::ports::admin::{
    IndexingOperation, IndexingOperationsInterface, PerformanceMetricsData,
    PerformanceMetricsInterface,
};
use std::collections::HashMap;

/// Null implementation of PerformanceMetricsInterface for testing
#[derive(shaku::Component, Default)]
#[shaku(interface = PerformanceMetricsInterface)]
pub struct NullPerformanceMetrics;

impl PerformanceMetricsInterface for NullPerformanceMetrics {
    fn uptime_secs(&self) -> u64 {
        0
    }

    fn record_query(&self, _response_time_ms: u64, _success: bool, _cache_hit: bool) {}

    fn update_active_connections(&self, _delta: i64) {}

    fn get_performance_metrics(&self) -> PerformanceMetricsData {
        PerformanceMetricsData {
            total_queries: 0,
            successful_queries: 0,
            failed_queries: 0,
            average_response_time_ms: 0.0,
            cache_hit_rate: 0.0,
            active_connections: 0,
            uptime_seconds: 0,
        }
    }
}

/// Null implementation of IndexingOperationsInterface for testing
#[derive(shaku::Component, Default)]
#[shaku(interface = IndexingOperationsInterface)]
pub struct NullIndexingOperations;

impl IndexingOperationsInterface for NullIndexingOperations {
    fn get_operations(&self) -> HashMap<String, IndexingOperation> {
        HashMap::new()
    }
}
