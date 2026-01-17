//! Admin Service Null Implementations
//!
//! Null implementations of admin port traits for Shaku DI testing.
//! Real implementations are in mcb-providers and created at runtime.

use mcb_application::ports::admin::{
    IndexingOperation, IndexingOperationsInterface, PerformanceMetricsData,
    PerformanceMetricsInterface,
};

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
            uptime_secs: 0,
            total_queries: 0,
            successful_queries: 0,
            failed_queries: 0,
            average_response_time_ms: 0.0,
            cache_hits: 0,
            cache_hit_rate: 0.0,
            active_connections: 0,
        }
    }
}

/// Null implementation of IndexingOperationsInterface for testing
#[derive(shaku::Component, Default)]
#[shaku(interface = IndexingOperationsInterface)]
pub struct NullIndexingOperations;

impl IndexingOperationsInterface for NullIndexingOperations {
    fn start_operation(&self, _collection: &str, _total_files: u64) -> IndexingOperation {
        IndexingOperation {
            id: "null-operation".to_string(),
            collection: String::new(),
            start_time: std::time::Instant::now(),
            total_files: 0,
            processed_files: std::sync::atomic::AtomicU64::new(0),
            status: std::sync::atomic::AtomicU8::new(0),
        }
    }

    fn complete_operation(&self, _operation: &IndexingOperation) {}

    fn fail_operation(&self, _operation: &IndexingOperation, _error: &str) {}

    fn get_active_operations(&self) -> Vec<IndexingOperation> {
        Vec::new()
    }

    fn clear_completed(&self) {}
}
