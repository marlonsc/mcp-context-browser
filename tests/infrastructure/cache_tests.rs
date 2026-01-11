//! Cache manager tests
//!
//! Tests migrated from src/infrastructure/cache/mod.rs

use mcp_context_browser::infrastructure::cache::{
    CacheConfig, CacheManager, CacheNamespacesConfig, CacheResult,
};

#[test]
fn test_cache_config_default() {
    let config = CacheConfig::default();
    assert!(config.enabled);
    assert_eq!(config.default_ttl_seconds, 3600);
    assert_eq!(config.max_size, 10000);
    assert!(config.redis_url.is_empty()); // Default to Local mode
}

#[tokio::test]
async fn test_cache_manager_creation() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let config = CacheConfig {
        enabled: false,
        ..Default::default()
    };

    let manager = CacheManager::new(config, None).await?;
    assert!(!manager.is_enabled());

    let stats = manager.get_stats().await;
    assert_eq!(stats.total_entries, 0);
    assert_eq!(stats.hits, 0);
    assert_eq!(stats.misses, 0);
    Ok(())
}

#[tokio::test]
async fn test_local_cache_operations() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let config = CacheConfig {
        enabled: true,
        redis_url: "".to_string(), // Explicitly empty for Local mode
        ..Default::default()
    };

    let manager = CacheManager::new(config, None).await?;

    // Test set and get
    manager
        .set("test", "key1", "value1".to_string())
        .await?;

    let result: CacheResult<String> = manager.get("test", "key1").await;
    assert!(result.is_hit());
    let data = result.data().ok_or("Expected data in cache hit")?;
    assert_eq!(data, "value1");

    // Test miss
    let result: CacheResult<String> = manager.get("test", "nonexistent").await;
    assert!(result.is_miss());

    // Test delete
    manager.delete("test", "key1").await?;
    let result: CacheResult<String> = manager.get("test", "key1").await;
    assert!(result.is_miss());

    // Check stats
    let stats = manager.get_stats().await;
    println!(
        "Stats after test: hits={}, misses={}, ratio={}",
        stats.hits, stats.misses, stats.hit_ratio
    );
    assert_eq!(stats.hits, 1);
    assert_eq!(stats.misses, 2); // Should be 2 misses: nonexistent + deleted key
    Ok(())
}

#[tokio::test]
async fn test_namespace_operations() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let config = CacheConfig {
        enabled: true,
        redis_url: "".to_string(),
        ..Default::default()
    };

    let manager = CacheManager::new(config, None).await?;

    // Set values in different namespaces
    manager
        .set("ns1", "key1", "value1".to_string())
        .await?;
    manager
        .set("ns2", "key1", "value2".to_string())
        .await?;

    // Get values
    let result1: CacheResult<String> = manager.get("ns1", "key1").await;
    let result2: CacheResult<String> = manager.get("ns2", "key1").await;

    assert!(result1.is_hit());
    assert!(result2.is_hit());
    let data1 = result1.data().ok_or("Expected data in ns1 cache hit")?;
    let data2 = result2.data().ok_or("Expected data in ns2 cache hit")?;
    assert_eq!(data1, "value1");
    assert_eq!(data2, "value2");

    // Clear namespace
    manager.clear_namespace("ns1").await?;

    manager.get_stats().await;

    let result1: CacheResult<String> = manager.get("ns1", "key1").await;
    let result2: CacheResult<String> = manager.get("ns2", "key1").await;

    assert!(result1.is_miss());
    assert!(result2.is_hit());
    Ok(())
}

#[tokio::test]
async fn test_cache_manager_fail_invalid_redis() {
    // Test with invalid Redis configuration
    // In the new Exclusive mode, this should FAIL on creation
    let config = CacheConfig {
        redis_url: "redis://invalid:6379".to_string(),
        default_ttl_seconds: 300,
        max_size: 100,
        enabled: true,
        namespaces: CacheNamespacesConfig::default(),
    };

    let result = CacheManager::new(config, None).await;
    assert!(result.is_err()); // Strict failure
}

#[tokio::test]
async fn test_cache_manager_handles_disabled_cache_operations() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let config = CacheConfig {
        redis_url: "".to_string(),
        default_ttl_seconds: 300,
        max_size: 0, // Disabled
        enabled: false,
        namespaces: CacheNamespacesConfig::default(),
    };

    let manager = CacheManager::new(config, None).await?;

    // Operations on disabled cache should not panic
    let set_result = manager.set("test", "key", "value".to_string()).await;
    assert!(set_result.is_ok());

    let get_result: CacheResult<String> = manager.get("test", "key").await;
    assert!(!matches!(get_result, CacheResult::Error(_)));
    Ok(())
}

#[tokio::test]
async fn test_cache_manager_handles_large_data_operations() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let config = CacheConfig::default(); // Local mode
    let manager = CacheManager::new(config, None).await?;

    // Test with large data
    let large_data = "x".repeat(1024 * 1024); // 1MB string

    let set_result = manager.set("test", "large_key", large_data.clone()).await;
    assert!(set_result.is_ok());

    let get_result: CacheResult<String> = manager.get("test", "large_key").await;
    match get_result {
        CacheResult::Hit(data) => assert_eq!(data, large_data),
        CacheResult::Miss => return Err("Expected cache hit".into()),
        CacheResult::Error(e) => return Err(format!("Expected no error, got: {}", e).into()),
    }
    Ok(())
}

#[tokio::test]
async fn test_namespace_limits() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut config = CacheConfig {
        enabled: true,
        redis_url: "".to_string(),
        ..Default::default()
    };
    // Configure metadata namespace with small limit
    config.namespaces.metadata.max_entries = 2;

    let manager = CacheManager::new(config, None).await?;

    // Add 3 items to metadata namespace
    manager.set("meta", "k1", "v1".to_string()).await?;
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;

    manager.set("meta", "k2", "v2".to_string()).await?;
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;

    manager.set("meta", "k3", "v3".to_string()).await?;

    // Stats should show total entries <= 2 (might be 2)
    let stats = manager.get_stats().await;
    assert_eq!(stats.total_entries, 2);
    assert!(stats.evictions >= 1);
    Ok(())
}
