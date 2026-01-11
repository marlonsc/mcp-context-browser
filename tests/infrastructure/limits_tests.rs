//! Resource limits tests
//!
//! Tests migrated from src/infrastructure/limits/mod.rs

use mcp_context_browser::infrastructure::limits::{
    NullResourceLimits, OperationLimits, ResourceLimits, ResourceLimitsConfig,
    ResourceLimitsProvider,
};
use std::time::Duration;

#[test]
fn test_resource_limits_config_default() {
    let config = ResourceLimitsConfig::default();
    assert!(config.enabled);
    assert_eq!(config.memory.max_usage_percent, 85.0);
    assert_eq!(config.cpu.max_usage_percent, 80.0);
    assert_eq!(config.disk.max_usage_percent, 90.0);
}

#[test]
fn test_operation_limits_config() {
    let limits = OperationLimits::default();
    assert_eq!(limits.max_concurrent_indexing, 3);
    assert_eq!(limits.max_concurrent_search, 10);
    assert_eq!(limits.max_concurrent_embedding, 5);
}

#[tokio::test]
async fn test_resource_limits_creation(
) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = ResourceLimitsConfig::default();
    let limits = ResourceLimits::new(config);

    assert!(limits.is_enabled());

    let stats = limits.get_stats().await?;
    assert!(stats.timestamp > 0);
    assert!(stats.memory.total > 0);
    assert!(stats.cpu.cores > 0);
    Ok(())
}

#[tokio::test]
async fn test_operation_permits(
) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = ResourceLimitsConfig::default();
    let limits = ResourceLimits::new(config);

    let _permit1 = limits.acquire_operation_permit("indexing").await?;
    let _permit2 = limits.acquire_operation_permit("search").await?;

    let stats = limits.get_stats().await?;
    assert_eq!(stats.operations.active_indexing, 1);
    assert_eq!(stats.operations.active_search, 1);

    drop(_permit1);
    drop(_permit2);

    tokio::time::sleep(Duration::from_millis(10)).await;

    let stats = limits.get_stats().await?;
    assert_eq!(stats.operations.active_indexing, 0);
    assert_eq!(stats.operations.active_search, 0);
    Ok(())
}

#[tokio::test]
async fn test_disabled_limits(
) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = ResourceLimitsConfig {
        enabled: false,
        ..Default::default()
    };
    let limits = ResourceLimits::new(config);

    assert!(!limits.is_enabled());

    limits.check_operation_allowed("indexing").await?;
    let _permit = limits.acquire_operation_permit("search").await?;
    Ok(())
}

#[tokio::test]
async fn test_resource_limits_provider_trait(
) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let limits = ResourceLimits::new(ResourceLimitsConfig::default());
    let provider: &dyn ResourceLimitsProvider = &limits;
    assert!(provider.is_enabled());

    let stats = provider.get_stats().await?;
    assert!(stats.memory.total > 0);
    Ok(())
}

#[tokio::test]
async fn test_null_resource_limits(
) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let null_limits = NullResourceLimits::new();
    assert!(!null_limits.is_enabled());

    null_limits.check_operation_allowed("indexing").await?;

    let stats = null_limits.get_stats().await?;
    assert_eq!(stats.memory.total, 0);
    assert_eq!(stats.cpu.cores, 0);
    Ok(())
}
