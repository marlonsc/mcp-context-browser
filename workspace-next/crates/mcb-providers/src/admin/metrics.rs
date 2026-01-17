//! Performance Metrics Implementation
//!
//! Atomic counter-based performance metrics tracking for the MCP server.
//! Implements the `PerformanceMetricsInterface` port from mcb-domain.

use mcb_domain::ports::admin::{PerformanceMetricsData, PerformanceMetricsInterface};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// Atomic performance metrics tracker
///
/// Thread-safe implementation of PerformanceMetricsInterface using atomic operations.
/// Tracks queries, response times, cache hits, and active connections.
#[derive(shaku::Component)]
#[shaku(interface = mcb_domain::ports::admin::PerformanceMetricsInterface)]
pub struct AtomicPerformanceMetrics {
    /// Server start time for uptime calculation
    start_time: Instant,

    /// Total number of queries processed
    total_queries: AtomicU64,

    /// Number of successful queries
    successful_queries: AtomicU64,

    /// Number of failed queries
    failed_queries: AtomicU64,

    /// Sum of all response times in milliseconds
    total_response_time_ms: AtomicU64,

    /// Number of cache hits
    cache_hits: AtomicU64,

    /// Current active connection count
    active_connections: AtomicU32,
}

impl AtomicPerformanceMetrics {
    /// Create a new performance metrics tracker
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            total_queries: AtomicU64::new(0),
            successful_queries: AtomicU64::new(0),
            failed_queries: AtomicU64::new(0),
            total_response_time_ms: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            active_connections: AtomicU32::new(0),
        }
    }

    /// Create as Arc for sharing
    pub fn new_shared() -> Arc<Self> {
        Arc::new(Self::new())
    }
}

impl Default for AtomicPerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceMetricsInterface for AtomicPerformanceMetrics {
    fn uptime_secs(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    fn record_query(&self, response_time_ms: u64, success: bool, cache_hit: bool) {
        self.total_queries.fetch_add(1, Ordering::Relaxed);
        self.total_response_time_ms
            .fetch_add(response_time_ms, Ordering::Relaxed);

        if success {
            self.successful_queries.fetch_add(1, Ordering::Relaxed);
        } else {
            self.failed_queries.fetch_add(1, Ordering::Relaxed);
        }

        if cache_hit {
            self.cache_hits.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn update_active_connections(&self, delta: i64) {
        if delta >= 0 {
            self.active_connections
                .fetch_add(delta as u32, Ordering::Relaxed);
        } else {
            let abs_delta = (-delta) as u32;
            self.active_connections.fetch_sub(
                abs_delta.min(self.active_connections.load(Ordering::Relaxed)),
                Ordering::Relaxed,
            );
        }
    }

    fn get_performance_metrics(&self) -> PerformanceMetricsData {
        let total = self.total_queries.load(Ordering::Relaxed);
        let successful = self.successful_queries.load(Ordering::Relaxed);
        let failed = self.failed_queries.load(Ordering::Relaxed);
        let total_time = self.total_response_time_ms.load(Ordering::Relaxed);
        let cache_hits = self.cache_hits.load(Ordering::Relaxed);

        let average_response_time_ms = if total > 0 {
            total_time as f64 / total as f64
        } else {
            0.0
        };

        let cache_hit_rate = if total > 0 {
            cache_hits as f64 / total as f64
        } else {
            0.0
        };

        PerformanceMetricsData {
            total_queries: total,
            successful_queries: successful,
            failed_queries: failed,
            average_response_time_ms,
            cache_hit_rate,
            active_connections: self.active_connections.load(Ordering::Relaxed),
            uptime_seconds: self.uptime_secs(),
        }
    }
}
