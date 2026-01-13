//! Cache Provider Tests
//!
//! Tests for new CacheProvider trait-based implementation
//! Tests use MokaCacheProvider (local) which is the default

use mcp_context_browser::infrastructure::cache::{
    CacheConfig, CacheBackendConfig, CacheNamespacesConfig, create_cache_provider,
    CacheProvider,
};
use std::time::Duration;

#[test]
fn test_cache_config_default() {
    let config = CacheConfig::default();
    assert!(config.enabled);
    assert!(config.backend.is_local()); // Default to local Moka
}

#[test]
fn test_cache_backend_local_default() {
    let backend = CacheBackendConfig::default();
    assert!(backend.is_local());
    assert!(!backend.is_redis());
    assert_eq!(backend.backend_type(), "local");
}

#[tokio::test]
async fn test_moka_cache_provider_creation() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let config = CacheConfig {
        enabled: true,
        backend: CacheBackendConfig::Local {
            max_entries: 1000,
            default_ttl_seconds: 3600,
        },
        namespaces: CacheNamespacesConfig::default(),
    };

    let provider = create_cache_provider(&config).await?;
    assert_eq!(provider.backend_type(), "moka");
    Ok(())
}

#[tokio::test]
async fn test_null_cache_provider_when_disabled() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let config = CacheConfig {
        enabled: false,
        ..Default::default()
    };

    let provider = create_cache_provider(&config).await?;
    assert_eq!(provider.backend_type(), "null");
    Ok(())
}

#[tokio::test]
async fn test_cache_set_and_get() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let config = CacheConfig {
        enabled: true,
        backend: CacheBackendConfig::Local {
            max_entries: 1000,
            default_ttl_seconds: 3600,
        },
        namespaces: CacheNamespacesConfig::default(),
    };

    let cache = create_cache_provider(&config).await?;
    let ttl = Duration::from_secs(3600);

    // Test set
    let value = "test_value".as_bytes().to_vec();
    cache.set("test_ns", "test_key", value.clone(), ttl).await?;

    // Test get
    let retrieved = cache.get("test_ns", "test_key").await?;
    assert!(retrieved.is_some(), "Expected value to be in cache");
    assert_eq!(retrieved.unwrap(), value);

    Ok(())
}

#[tokio::test]
async fn test_cache_miss() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let config = CacheConfig::default();
    let cache = create_cache_provider(&config).await?;

    // Try to get a key that doesn't exist
    let retrieved = cache.get("nonexistent_ns", "nonexistent_key").await?;
    assert!(retrieved.is_none(), "Expected cache miss for non-existent key");

    Ok(())
}

#[tokio::test]
async fn test_cache_delete() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let config = CacheConfig::default();
    let cache = create_cache_provider(&config).await?;
    let ttl = Duration::from_secs(3600);

    // Set a value
    let value = "delete_me".as_bytes().to_vec();
    cache.set("test_ns", "delete_key", value, ttl).await?;

    // Verify it's there
    let retrieved = cache.get("test_ns", "delete_key").await?;
    assert!(retrieved.is_some());

    // Delete it
    cache.delete("test_ns", "delete_key").await?;

    // Verify it's gone
    let retrieved = cache.get("test_ns", "delete_key").await?;
    assert!(retrieved.is_none(), "Expected cache miss after deletion");

    Ok(())
}

#[tokio::test]
async fn test_cache_clear_namespace() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let config = CacheConfig::default();
    let cache = create_cache_provider(&config).await?;
    let ttl = Duration::from_secs(3600);

    // Set values in two namespaces
    cache.set("ns1", "key1", "value1".as_bytes().to_vec(), ttl).await?;
    cache.set("ns1", "key2", "value2".as_bytes().to_vec(), ttl).await?;
    cache.set("ns2", "key1", "value3".as_bytes().to_vec(), ttl).await?;

    // Clear ns1
    cache.clear(Some("ns1")).await?;

    // Verify ns1 is empty, ns2 still has data
    assert!(cache.get("ns1", "key1").await?.is_none());
    assert!(cache.get("ns1", "key2").await?.is_none());
    assert!(cache.get("ns2", "key1").await?.is_some());

    Ok(())
}

#[tokio::test]
async fn test_cache_exists() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let config = CacheConfig::default();
    let cache = create_cache_provider(&config).await?;
    let ttl = Duration::from_secs(3600);

    // Set a value
    cache.set("test_ns", "exists_key", "data".as_bytes().to_vec(), ttl).await?;

    // Test exists
    assert!(cache.exists("test_ns", "exists_key").await?);
    assert!(!cache.exists("test_ns", "nonexistent").await?);

    Ok(())
}

#[tokio::test]
async fn test_cache_get_stats() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let config = CacheConfig::default();
    let cache = create_cache_provider(&config).await?;
    let ttl = Duration::from_secs(3600);

    // Perform some cache operations
    let value = "stats_test".as_bytes().to_vec();
    cache.set("stats_ns", "key1", value, ttl).await?;
    let _ = cache.get("stats_ns", "key1").await?; // cache hit
    let _ = cache.get("stats_ns", "nonexistent").await?; // cache miss

    // Get stats
    let stats = cache.get_stats("stats_ns").await?;
    assert_eq!(stats.total_entries, 1);
    assert!(stats.hits > 0, "Expected at least one cache hit");

    Ok(())
}

#[tokio::test]
async fn test_cache_health_check() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let config = CacheConfig::default();
    let cache = create_cache_provider(&config).await?;

    // Test health check
    let health = cache.health_check().await?;
    assert_eq!(health, mcp_context_browser::infrastructure::cache::HealthStatus::Healthy);

    Ok(())
}
