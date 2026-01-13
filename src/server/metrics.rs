//! Real-time performance metrics tracking for MCP server
//!
//! Provides interfaces and implementations for tracking server performance
//! metrics including queries, response times, cache hits, and active connections.

use crate::infrastructure::service_helpers::UptimeTracker;
use shaku::Interface;
use std::sync::atomic::{AtomicU64, Ordering};

/// Real-time performance metrics tracking interface
pub trait PerformanceMetricsInterface: Interface + Send + Sync {
    /// Get server uptime in seconds
    fn uptime_secs(&self) -> u64;
    fn record_query(&self, response_time_ms: u64, success: bool, cache_hit: bool);
    fn update_active_connections(&self, delta: i64);
    fn get_performance_metrics(&self) -> crate::admin::service::PerformanceMetricsData;
}

/// Real-time performance metrics tracking
#[derive(Debug, shaku::Component)]
#[shaku(interface = PerformanceMetricsInterface)]
pub struct McpPerformanceMetrics {
    /// Total queries processed
    #[shaku(default = AtomicU64::new(0))]
    pub total_queries: AtomicU64,
    /// Successful queries
    #[shaku(default = AtomicU64::new(0))]
    pub successful_queries: AtomicU64,
    /// Failed queries
    #[shaku(default = AtomicU64::new(0))]
    pub failed_queries: AtomicU64,
    /// Response time accumulator (in milliseconds)
    #[shaku(default = AtomicU64::new(0))]
    pub response_time_sum: AtomicU64,
    /// Cache hits
    #[shaku(default = AtomicU64::new(0))]
    pub cache_hits: AtomicU64,
    /// Cache misses
    #[shaku(default = AtomicU64::new(0))]
    pub cache_misses: AtomicU64,
    /// Active connections
    #[shaku(default = AtomicU64::new(0))]
    pub active_connections: AtomicU64,
    /// Server uptime tracker
    #[shaku(default = UptimeTracker::start())]
    pub uptime: UptimeTracker,
}

impl PerformanceMetricsInterface for McpPerformanceMetrics {
    fn uptime_secs(&self) -> u64 {
        self.uptime.elapsed_secs()
    }

    fn record_query(&self, response_time_ms: u64, success: bool, cache_hit: bool) {
        self.total_queries.fetch_add(1, Ordering::Relaxed);

        if success {
            self.successful_queries.fetch_add(1, Ordering::Relaxed);
        } else {
            self.failed_queries.fetch_add(1, Ordering::Relaxed);
        }

        self.response_time_sum
            .fetch_add(response_time_ms, Ordering::Relaxed);

        if cache_hit {
            self.cache_hits.fetch_add(1, Ordering::Relaxed);
        } else {
            self.cache_misses.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn update_active_connections(&self, delta: i64) {
        if delta > 0 {
            self.active_connections
                .fetch_add(delta as u64, Ordering::Relaxed);
        } else {
            let current = self.active_connections.load(Ordering::Relaxed);
            let new_value = current.saturating_sub((-delta) as u64);
            self.active_connections.store(new_value, Ordering::Relaxed);
        }
    }

    fn get_performance_metrics(&self) -> crate::admin::service::PerformanceMetricsData {
        let total_queries = self.total_queries.load(Ordering::Relaxed);
        let successful_queries = self.successful_queries.load(Ordering::Relaxed);
        let failed_queries = self.failed_queries.load(Ordering::Relaxed);
        let response_time_sum = self.response_time_sum.load(Ordering::Relaxed);
        let cache_hits = self.cache_hits.load(Ordering::Relaxed);
        let cache_misses = self.cache_misses.load(Ordering::Relaxed);

        let average_response_time_ms = if total_queries > 0 {
            response_time_sum as f64 / total_queries as f64
        } else {
            0.0
        };

        let total_cache_requests = cache_hits + cache_misses;
        let cache_hit_rate = if total_cache_requests > 0 {
            cache_hits as f64 / total_cache_requests as f64
        } else {
            0.0
        };

        let uptime_seconds = self.uptime.elapsed_secs();

        crate::admin::service::PerformanceMetricsData {
            total_queries,
            successful_queries,
            failed_queries,
            average_response_time_ms,
            cache_hit_rate,
            active_connections: self.active_connections.load(Ordering::Relaxed) as u32,
            uptime_seconds,
        }
    }
}

impl Default for McpPerformanceMetrics {
    fn default() -> Self {
        Self {
            total_queries: AtomicU64::new(0),
            successful_queries: AtomicU64::new(0),
            failed_queries: AtomicU64::new(0),
            response_time_sum: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            active_connections: AtomicU64::new(0),
            uptime: UptimeTracker::start(),
        }
    }
}
