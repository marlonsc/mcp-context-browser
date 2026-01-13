//! Performance metrics tests
//!
//! Tests migrated from src/infrastructure/metrics/performance.rs

use mcp_context_browser::infrastructure::metrics::performance::PerformanceMetrics;
use std::time::Duration;

#[tokio::test]
async fn test_record_query() {
    let metrics = PerformanceMetrics::new();

    metrics.record_query(Duration::from_millis(100), true);
    metrics.record_query(Duration::from_millis(200), false);

    // Give some time for the actor to process
    tokio::time::sleep(Duration::from_millis(100)).await;

    let perf = metrics.get_query_performance().await;
    assert_eq!(perf.total_queries, 2);
    assert_eq!(perf.average_latency, 150.0); // (100 + 200) / 2
    assert_eq!(perf.success_rate, 50.0); // 1/2 * 100
}

#[tokio::test]
async fn test_cache_metrics() {
    let metrics = PerformanceMetrics::new();

    metrics.record_cache_hit();
    metrics.record_cache_hit();
    metrics.record_cache_miss();
    metrics.update_cache_size(1024);

    // Give some time for the actor to process
    tokio::time::sleep(Duration::from_millis(100)).await;

    let cache = metrics.get_cache_metrics().await;
    assert_eq!(cache.hits, 2);
    assert_eq!(cache.misses, 1);
    assert_eq!(cache.hit_rate, 66.66666666666666); // 2/3 * 100
    assert_eq!(cache.size, 1024);
}

#[tokio::test]
async fn test_empty_metrics() {
    let metrics = PerformanceMetrics::new();

    // Give some time for the actor to process
    tokio::time::sleep(Duration::from_millis(100)).await;

    let perf = metrics.get_query_performance().await;
    assert_eq!(perf.total_queries, 0);
    assert_eq!(perf.average_latency, 0.0);
    assert_eq!(perf.p99_latency, 0.0);
    assert_eq!(perf.success_rate, 0.0);

    let cache = metrics.get_cache_metrics().await;
    assert_eq!(cache.hits, 0);
    assert_eq!(cache.misses, 0);
    assert_eq!(cache.hit_rate, 0.0);
    assert_eq!(cache.size, 0);
}

#[tokio::test]
async fn test_reset() {
    let metrics = PerformanceMetrics::new();

    metrics.record_query(Duration::from_millis(100), true);
    metrics.record_cache_hit();
    metrics.update_cache_size(1024);

    // Wait for processing
    tokio::time::sleep(Duration::from_millis(100)).await;

    metrics.reset();

    // Give some time for the actor to process reset
    tokio::time::sleep(Duration::from_millis(100)).await;

    let perf = metrics.get_query_performance().await;
    assert_eq!(perf.total_queries, 0);

    let cache = metrics.get_cache_metrics().await;
    assert_eq!(cache.hits, 0);
    assert_eq!(cache.size, 0);
}
