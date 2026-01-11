//! Tests for metrics collection module

use mcp_context_browser::infrastructure::metrics::{
    system::SystemMetricsCollectorInterface, CpuMetrics, MemoryMetrics, SystemMetricsCollector,
};

#[tokio::test]
async fn test_system_metrics_collector_creation() {
    let _collector = SystemMetricsCollector::new();
    // Test passes if no panic occurs during creation
}

#[tokio::test]
async fn test_cpu_metrics_collection() -> Result<(), Box<dyn std::error::Error>> {
    let collector = SystemMetricsCollector::new();
    let metrics = collector.collect_cpu_metrics().await?;

    // Basic structure validation
    assert!(metrics.cores >= 1); // Should have at least 1 core
    assert!(metrics.usage >= 0.0 && metrics.usage <= 100.0); // Usage should be percentage
    assert!(!metrics.model.is_empty()); // Model should not be empty
    Ok(())
}

#[tokio::test]
async fn test_memory_metrics_collection() -> Result<(), Box<dyn std::error::Error>> {
    let collector = SystemMetricsCollector::new();
    let metrics = collector.collect_memory_metrics().await?;

    // Basic validation
    assert!(metrics.total > 0); // Should have some total memory
    assert!(metrics.used <= metrics.total); // Used should not exceed total
    assert!(metrics.free <= metrics.total); // Free should not exceed total
    assert!(metrics.usage_percent >= 0.0 && metrics.usage_percent <= 100.0); // Percentage validation
    Ok(())
}

#[test]
fn test_cpu_metrics_structure() {
    let metrics = CpuMetrics {
        usage: 50.0,
        cores: 4,
        model: "Test CPU".to_string(),
        speed: 3000,
    };

    assert_eq!(metrics.usage, 50.0);
    assert_eq!(metrics.cores, 4);
    assert_eq!(metrics.model, "Test CPU");
    assert_eq!(metrics.speed, 3000);
}

#[test]
fn test_memory_metrics_structure() {
    let metrics = MemoryMetrics {
        total: 16_000_000_000, // 16GB
        used: 8_000_000_000,   // 8GB
        free: 8_000_000_000,   // 8GB
        usage_percent: 50.0,
    };

    assert_eq!(metrics.total, 16_000_000_000);
    assert_eq!(metrics.used, 8_000_000_000);
    assert_eq!(metrics.free, 8_000_000_000);
    assert_eq!(metrics.usage_percent, 50.0);
}
