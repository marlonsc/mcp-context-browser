//! Performance metrics tracking for queries and cache operations
//!
//! Tracks query latency, cache hit/miss ratios, and other performance indicators.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, oneshot};

/// Performance metrics messages
pub enum PerformanceMessage {
    /// Record a query execution with latency and success status
    RecordQuery {
        /// Query execution time
        latency: Duration,
        /// Whether the query was successful
        success: bool
    },
    /// Record a cache hit
    RecordCacheHit,
    /// Record a cache miss
    RecordCacheMiss,
    /// Update the current cache size
    UpdateCacheSize(u64),
    /// Request current query performance metrics
    GetQueryPerformance(oneshot::Sender<QueryPerformanceMetrics>),
    /// Request current cache metrics
    GetCacheMetrics(oneshot::Sender<CacheMetrics>),
    /// Reset all metrics
    Reset,
    /// Clean old records older than the specified duration
    CleanOldRecords(Duration),
}

/// Handle for performance metrics actor
pub struct PerformanceMetrics {
    sender: mpsc::Sender<PerformanceMessage>,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceMetrics {
    /// Create a new performance metrics handle and start the actor
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(1000);
        let mut actor = PerformanceMetricsActor::new(rx);
        tokio::spawn(async move {
            actor.run().await;
        });
        Self { sender: tx }
    }

    /// Record a query performance measurement
    pub fn record_query(&self, latency: Duration, success: bool) {
        let _ = self
            .sender
            .try_send(PerformanceMessage::RecordQuery { latency, success });
    }

    /// Record a cache hit
    pub fn record_cache_hit(&self) {
        let _ = self.sender.try_send(PerformanceMessage::RecordCacheHit);
    }

    /// Record a cache miss
    pub fn record_cache_miss(&self) {
        let _ = self.sender.try_send(PerformanceMessage::RecordCacheMiss);
    }

    /// Update the current cache size
    pub fn update_cache_size(&self, size_bytes: u64) {
        let _ = self
            .sender
            .try_send(PerformanceMessage::UpdateCacheSize(size_bytes));
    }

    /// Get current query performance metrics
    pub async fn get_query_performance(&self) -> QueryPerformanceMetrics {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .sender
            .send(PerformanceMessage::GetQueryPerformance(tx))
            .await;
        rx.await.unwrap_or_default()
    }

    /// Get current cache performance metrics
    pub async fn get_cache_metrics(&self) -> CacheMetrics {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .sender
            .send(PerformanceMessage::GetCacheMetrics(tx))
            .await;
        rx.await.unwrap_or_default()
    }

    /// Reset all performance metrics
    pub fn reset(&self) {
        let _ = self.sender.try_send(PerformanceMessage::Reset);
    }

    /// Clean old performance records older than max_age
    pub fn clean_old_records(&self, max_age: Duration) {
        let _ = self
            .sender
            .try_send(PerformanceMessage::CleanOldRecords(max_age));
    }
}

/// Query performance metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

/// Performance metrics collector actor
struct PerformanceMetricsActor {
    receiver: mpsc::Receiver<PerformanceMessage>,
    query_records: VecDeque<QueryRecord>,
    cache_hits: u64,
    cache_misses: u64,
    cache_size: u64,
    max_history: usize,
}

impl PerformanceMetricsActor {
    fn new(receiver: mpsc::Receiver<PerformanceMessage>) -> Self {
        Self {
            receiver,
            query_records: VecDeque::new(),
            cache_hits: 0,
            cache_misses: 0,
            cache_size: 0,
            max_history: 1000,
        }
    }

    async fn run(&mut self) {
        while let Some(msg) = self.receiver.recv().await {
            match msg {
                PerformanceMessage::RecordQuery { latency, success } => {
                    let record = QueryRecord {
                        latency_ms: latency.as_millis() as f64,
                        success,
                        timestamp: Instant::now(),
                    };
                    self.query_records.push_back(record);
                    while self.query_records.len() > self.max_history {
                        self.query_records.pop_front();
                    }
                }
                PerformanceMessage::RecordCacheHit => {
                    self.cache_hits = self.cache_hits.saturating_add(1);
                }
                PerformanceMessage::RecordCacheMiss => {
                    self.cache_misses = self.cache_misses.saturating_add(1);
                }
                PerformanceMessage::UpdateCacheSize(size) => {
                    self.cache_size = size;
                }
                PerformanceMessage::GetQueryPerformance(tx) => {
                    let _ = tx.send(self.calculate_query_performance());
                }
                PerformanceMessage::GetCacheMetrics(tx) => {
                    let _ = tx.send(self.calculate_cache_metrics());
                }
                PerformanceMessage::Reset => {
                    self.query_records.clear();
                    self.cache_hits = 0;
                    self.cache_misses = 0;
                    self.cache_size = 0;
                }
                PerformanceMessage::CleanOldRecords(max_age) => {
                    let now = Instant::now();
                    self.query_records
                        .retain(|r| now.duration_since(r.timestamp) < max_age);
                }
            }
        }
    }

    fn calculate_query_performance(&self) -> QueryPerformanceMetrics {
        let total_queries = self.query_records.len() as u64;
        if total_queries == 0 {
            return QueryPerformanceMetrics::default();
        }

        let total_latency: f64 = self.query_records.iter().map(|r| r.latency_ms).sum();
        let average_latency = total_latency / total_queries as f64;

        let successful_queries = self.query_records.iter().filter(|r| r.success).count() as f64;
        let success_rate = (successful_queries / total_queries as f64) * 100.0;

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

    fn calculate_cache_metrics(&self) -> CacheMetrics {
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
}
