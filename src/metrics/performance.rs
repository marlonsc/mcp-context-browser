//! Performance metrics tracking for queries and cache operations
//!
//! Tracks query latency, cache hit/miss ratios, and other performance indicators.

use crate::core::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Query performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryPerformanceMetrics {
    /// Total number of queries processed
    pub total_queries: u64,
    /// Average query latency in milliseconds
    pub average_latency: f64,
    /// 99th percentile latency in milliseconds
    pub p99_latency: f64,
    /// Query success rate (0-100)
    pub success_rate: f64,
}

/// Cache performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetrics {
    /// Total cache hits
    pub hits: u64,
    /// Total cache misses
    pub misses: u64,
    /// Cache hit rate percentage (0-100)
    pub hit_rate: f64,
    /// Current cache size in bytes
    pub size: u64,
}

/// Individual query record for latency tracking
#[derive(Debug, Clone)]
struct QueryRecord {
    latency_ms: f64,
    success: bool,
    timestamp: Instant,
}

/// Performance metrics collector
pub struct PerformanceMetrics {
    query_records: VecDeque<QueryRecord>,
    cache_hits: u64,
    cache_misses: u64,
    cache_size: u64,
    max_history: usize,
}

impl PerformanceMetrics {
    /// Create a new performance metrics collector
    pub fn new() -> Self {
        Self {
            query_records: VecDeque::new(),
            cache_hits: 0,
            cache_misses: 0,
            cache_size: 0,
            max_history: 1000, // Keep last 1000 queries
        }
    }

    /// Record a query execution
    pub fn record_query(&mut self, latency: Duration, success: bool) {
        let record = QueryRecord {
            latency_ms: latency.as_millis() as f64,
            success,
            timestamp: Instant::now(),
        };

        self.query_records.push_back(record);

        // Maintain max history size
        while self.query_records.len() > self.max_history {
            self.query_records.pop_front();
        }
    }

    /// Record a cache hit
    pub fn record_cache_hit(&mut self) {
        self.cache_hits = self.cache_hits.saturating_add(1);
    }

    /// Record a cache miss
    pub fn record_cache_miss(&mut self) {
        self.cache_misses = self.cache_misses.saturating_add(1);
    }

    /// Update current cache size
    pub fn update_cache_size(&mut self, size_bytes: u64) {
        self.cache_size = size_bytes;
    }

    /// Get query performance metrics
    pub fn get_query_performance(&self) -> QueryPerformanceMetrics {
        let total_queries = self.query_records.len() as u64;

        if total_queries == 0 {
            return QueryPerformanceMetrics {
                total_queries: 0,
                average_latency: 0.0,
                p99_latency: 0.0,
                success_rate: 0.0,
            };
        }

        // Calculate average latency
        let total_latency: f64 = self.query_records.iter().map(|r| r.latency_ms).sum();
        let average_latency = total_latency / total_queries as f64;

        // Calculate success rate
        let successful_queries = self.query_records.iter().filter(|r| r.success).count() as f64;
        let success_rate = (successful_queries / total_queries as f64) * 100.0;

        // Calculate P99 latency (99th percentile)
        let mut latencies: Vec<f64> = self.query_records.iter().map(|r| r.latency_ms).collect();
        latencies.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let p99_index = ((latencies.len() as f64 * 0.99).ceil() as usize).saturating_sub(1);
        let p99_index = p99_index.min(latencies.len().saturating_sub(1));
        let p99_latency = latencies.get(p99_index).copied().unwrap_or(0.0);

        QueryPerformanceMetrics {
            total_queries,
            average_latency,
            p99_latency,
            success_rate,
        }
    }

    /// Get cache performance metrics
    pub fn get_cache_metrics(&self) -> CacheMetrics {
        let total_operations = self.cache_hits + self.cache_misses;
        let hit_rate = if total_operations > 0 {
            (self.cache_hits as f64 / total_operations as f64) * 100.0
        } else {
            0.0
        };

        CacheMetrics {
            hits: self.cache_hits,
            misses: self.cache_misses,
            hit_rate,
            size: self.cache_size,
        }
    }

    /// Reset all metrics
    pub fn reset(&mut self) {
        self.query_records.clear();
        self.cache_hits = 0;
        self.cache_misses = 0;
        self.cache_size = 0;
    }

    /// Get total number of queries in history
    pub fn query_count(&self) -> usize {
        self.query_records.len()
    }

    /// Clean old records (older than specified duration)
    pub fn clean_old_records(&mut self, max_age: Duration) {
        let now = Instant::now();
        self.query_records.retain(|record| {
            now.duration_since(record.timestamp) < max_age
        });
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_new_performance_metrics() {
        let metrics = PerformanceMetrics::new();
        assert_eq!(metrics.query_count(), 0);
        assert_eq!(metrics.cache_hits, 0);
        assert_eq!(metrics.cache_misses, 0);
        assert_eq!(metrics.cache_size, 0);
    }

    #[test]
    fn test_record_query() {
        let mut metrics = PerformanceMetrics::new();

        metrics.record_query(Duration::from_millis(100), true);
        metrics.record_query(Duration::from_millis(200), false);

        assert_eq!(metrics.query_count(), 2);

        let perf = metrics.get_query_performance();
        assert_eq!(perf.total_queries, 2);
        assert_eq!(perf.average_latency, 150.0); // (100 + 200) / 2
        assert_eq!(perf.success_rate, 50.0); // 1/2 * 100
    }

    #[test]
    fn test_cache_metrics() {
        let mut metrics = PerformanceMetrics::new();

        metrics.record_cache_hit();
        metrics.record_cache_hit();
        metrics.record_cache_miss();
        metrics.update_cache_size(1024);

        let cache = metrics.get_cache_metrics();
        assert_eq!(cache.hits, 2);
        assert_eq!(cache.misses, 1);
        assert_eq!(cache.hit_rate, 66.66666666666666); // 2/3 * 100
        assert_eq!(cache.size, 1024);
    }

    #[test]
    fn test_empty_metrics() {
        let metrics = PerformanceMetrics::new();

        let perf = metrics.get_query_performance();
        assert_eq!(perf.total_queries, 0);
        assert_eq!(perf.average_latency, 0.0);
        assert_eq!(perf.p99_latency, 0.0);
        assert_eq!(perf.success_rate, 0.0);

        let cache = metrics.get_cache_metrics();
        assert_eq!(cache.hits, 0);
        assert_eq!(cache.misses, 0);
        assert_eq!(cache.hit_rate, 0.0);
        assert_eq!(cache.size, 0);
    }

    #[test]
    fn test_reset() {
        let mut metrics = PerformanceMetrics::new();

        metrics.record_query(Duration::from_millis(100), true);
        metrics.record_cache_hit();
        metrics.update_cache_size(1024);

        metrics.reset();

        assert_eq!(metrics.query_count(), 0);
        assert_eq!(metrics.cache_hits, 0);
        assert_eq!(metrics.cache_size, 0);
    }
}