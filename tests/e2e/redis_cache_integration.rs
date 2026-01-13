//! Redis Cache Provider Integration Tests
//!
//! Tests the RedisCacheProvider against a real local Redis instance.
//! Requires Redis running on localhost:6379 (see docker-compose.yml)
//!
//! Run with: cargo test --test '*' -- redis_cache_integration --nocapture

use mcp_context_browser::infrastructure::cache::{
    CacheConfig, CacheBackendConfig, CacheNamespacesConfig, create_cache_provider, CacheProvider,
};
use std::time::Duration;

/// Get Redis URL from environment or default to localhost
fn get_redis_url() -> String {
    std::env::var("REDIS_URL")
        .or_else(|_| std::env::var("MCP_CACHE__URL"))
        .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string())
}

/// Check if Redis is available
async fn is_redis_available() -> bool {
    let url = get_redis_url();
    match redis::aio::Connection::open(url).await {
        Ok(mut conn) => {
            use redis::AsyncCommands;
            let _: Result<(), _> = conn.ping().await;
            true
        }
        Err(_) => false,
    }
}

/// Helper to skip test if Redis is not available
macro_rules! skip_if_no_redis {
    () => {
        if !is_redis_available().await {
            eprintln!("⚠️  Skipping test: Redis not available on localhost:6379");
            eprintln!("    Start Redis with: docker-compose up -d redis");
            return;
        }
    };
}

#[tokio::test]
async fn test_redis_cache_provider_creation() {
    skip_if_no_redis!();

    let config = CacheConfig {
        enabled: true,
        backend: CacheBackendConfig::Redis {
            url: get_redis_url(),
            pool_size: 5,
            default_ttl_seconds: 3600,
        },
        namespaces: CacheNamespacesConfig::default(),
    };

    let provider = create_cache_provider(&config)
        .await
        .expect("Failed to create Redis cache provider");

    assert_eq!(provider.backend_type(), "redis");
    println!("✅ Redis cache provider created successfully");
}

#[tokio::test]
async fn test_redis_cache_set_and_get() {
    skip_if_no_redis!();

    let config = CacheConfig {
        enabled: true,
        backend: CacheBackendConfig::Redis {
            url: get_redis_url(),
            pool_size: 5,
            default_ttl_seconds: 3600,
        },
        namespaces: CacheNamespacesConfig::default(),
    };

    let cache = create_cache_provider(&config)
        .await
        .expect("Failed to create cache provider");

    let ttl = Duration::from_secs(60);
    let key = format!("test_redis_set_get_{}", chrono::Utc::now().timestamp_millis());
    let value = "test_value_redis".as_bytes().to_vec();

    // Set value
    cache
        .set("redis_test_ns", &key, value.clone(), ttl)
        .await
        .expect("Failed to set cache value");

    // Get value
    let retrieved = cache
        .get("redis_test_ns", &key)
        .await
        .expect("Failed to get cache value");

    assert!(retrieved.is_some(), "Expected value in Redis cache");
    assert_eq!(retrieved.unwrap(), value);

    // Cleanup
    let _ = cache.delete("redis_test_ns", &key).await;
    println!("✅ Redis set/get operations work correctly");
}

#[tokio::test]
async fn test_redis_cache_delete() {
    skip_if_no_redis!();

    let config = CacheConfig {
        enabled: true,
        backend: CacheBackendConfig::Redis {
            url: get_redis_url(),
            pool_size: 5,
            default_ttl_seconds: 3600,
        },
        namespaces: CacheNamespacesConfig::default(),
    };

    let cache = create_cache_provider(&config)
        .await
        .expect("Failed to create cache provider");

    let ttl = Duration::from_secs(60);
    let key = format!("test_redis_delete_{}", chrono::Utc::now().timestamp_millis());
    let value = "delete_me_redis".as_bytes().to_vec();

    // Set value
    cache
        .set("redis_test_ns", &key, value, ttl)
        .await
        .expect("Failed to set value");

    // Verify it exists
    let before_delete = cache
        .get("redis_test_ns", &key)
        .await
        .expect("Failed to get value");
    assert!(before_delete.is_some(), "Value should exist before deletion");

    // Delete it
    cache
        .delete("redis_test_ns", &key)
        .await
        .expect("Failed to delete value");

    // Verify it's gone
    let after_delete = cache
        .get("redis_test_ns", &key)
        .await
        .expect("Failed to get value after deletion");
    assert!(after_delete.is_none(), "Value should not exist after deletion");

    println!("✅ Redis delete operation works correctly");
}

#[tokio::test]
async fn test_redis_cache_clear_namespace() {
    skip_if_no_redis!();

    let config = CacheConfig {
        enabled: true,
        backend: CacheBackendConfig::Redis {
            url: get_redis_url(),
            pool_size: 5,
            default_ttl_seconds: 3600,
        },
        namespaces: CacheNamespacesConfig::default(),
    };

    let cache = create_cache_provider(&config)
        .await
        .expect("Failed to create cache provider");

    let ttl = Duration::from_secs(60);
    let ts = chrono::Utc::now().timestamp_millis();

    // Set values in different namespaces
    cache
        .set("redis_ns1", &format!("key1_{}", ts), "value1".as_bytes().to_vec(), ttl)
        .await
        .expect("Failed to set value 1");

    cache
        .set("redis_ns1", &format!("key2_{}", ts), "value2".as_bytes().to_vec(), ttl)
        .await
        .expect("Failed to set value 2");

    cache
        .set("redis_ns2", &format!("key1_{}", ts), "value3".as_bytes().to_vec(), ttl)
        .await
        .expect("Failed to set value 3");

    // Clear only redis_ns1
    cache
        .clear(Some("redis_ns1"))
        .await
        .expect("Failed to clear namespace");

    // Verify redis_ns1 is cleared but redis_ns2 still has data
    let ns1_key1 = cache
        .get("redis_ns1", &format!("key1_{}", ts))
        .await
        .expect("Failed to get value");
    let ns1_key2 = cache
        .get("redis_ns1", &format!("key2_{}", ts))
        .await
        .expect("Failed to get value");

    assert!(ns1_key1.is_none(), "redis_ns1 should be cleared");
    assert!(ns1_key2.is_none(), "redis_ns1 should be cleared");

    let ns2_key1 = cache
        .get("redis_ns2", &format!("key1_{}", ts))
        .await
        .expect("Failed to get value");
    assert!(ns2_key1.is_some(), "redis_ns2 should still have data");

    // Cleanup
    let _ = cache.clear(Some("redis_ns2")).await;
    println!("✅ Redis clear namespace operation works correctly");
}

#[tokio::test]
async fn test_redis_cache_exists() {
    skip_if_no_redis!();

    let config = CacheConfig {
        enabled: true,
        backend: CacheBackendConfig::Redis {
            url: get_redis_url(),
            pool_size: 5,
            default_ttl_seconds: 3600,
        },
        namespaces: CacheNamespacesConfig::default(),
    };

    let cache = create_cache_provider(&config)
        .await
        .expect("Failed to create cache provider");

    let ttl = Duration::from_secs(60);
    let key = format!("test_redis_exists_{}", chrono::Utc::now().timestamp_millis());
    let value = "exists_test".as_bytes().to_vec();

    // Set a value
    cache
        .set("redis_test_ns", &key, value, ttl)
        .await
        .expect("Failed to set value");

    // Test exists for existing key
    let exists = cache
        .exists("redis_test_ns", &key)
        .await
        .expect("Failed to check existence");
    assert!(exists, "Key should exist");

    // Test exists for non-existent key
    let not_exists = cache
        .exists("redis_test_ns", "nonexistent_key_xyz")
        .await
        .expect("Failed to check non-existent key");
    assert!(!not_exists, "Non-existent key should not exist");

    // Cleanup
    let _ = cache.delete("redis_test_ns", &key).await;
    println!("✅ Redis exists operation works correctly");
}

#[tokio::test]
async fn test_redis_cache_ttl_expiration() {
    skip_if_no_redis!();

    let config = CacheConfig {
        enabled: true,
        backend: CacheBackendConfig::Redis {
            url: get_redis_url(),
            pool_size: 5,
            default_ttl_seconds: 3600,
        },
        namespaces: CacheNamespacesConfig::default(),
    };

    let cache = create_cache_provider(&config)
        .await
        .expect("Failed to create cache provider");

    let ttl = Duration::from_secs(2); // 2 second TTL
    let key = format!("test_redis_ttl_{}", chrono::Utc::now().timestamp_millis());
    let value = "expires_soon".as_bytes().to_vec();

    // Set value with short TTL
    cache
        .set("redis_test_ns", &key, value, ttl)
        .await
        .expect("Failed to set value");

    // Immediately retrieve - should exist
    let exists = cache
        .get("redis_test_ns", &key)
        .await
        .expect("Failed to get value");
    assert!(exists.is_some(), "Value should exist immediately after set");

    // Wait for expiration
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Try to retrieve - should be expired
    let expired = cache
        .get("redis_test_ns", &key)
        .await
        .expect("Failed to get expired value");
    assert!(expired.is_none(), "Value should have expired");

    println!("✅ Redis TTL expiration works correctly");
}

#[tokio::test]
async fn test_redis_cache_health_check() {
    skip_if_no_redis!();

    let config = CacheConfig {
        enabled: true,
        backend: CacheBackendConfig::Redis {
            url: get_redis_url(),
            pool_size: 5,
            default_ttl_seconds: 3600,
        },
        namespaces: CacheNamespacesConfig::default(),
    };

    let cache = create_cache_provider(&config)
        .await
        .expect("Failed to create cache provider");

    // Test health check
    let health = cache
        .health_check()
        .await
        .expect("Failed to check health");

    assert_eq!(
        health,
        mcp_context_browser::infrastructure::cache::HealthStatus::Healthy,
        "Redis should be healthy"
    );

    println!("✅ Redis health check works correctly");
}

#[tokio::test]
async fn test_redis_cache_concurrent_access() {
    skip_if_no_redis!();

    let config = CacheConfig {
        enabled: true,
        backend: CacheBackendConfig::Redis {
            url: "redis://127.0.0.1:6379".to_string(),
            pool_size: 10,
            default_ttl_seconds: 3600,
        },
        namespaces: CacheNamespacesConfig::default(),
    };

    let cache = std::sync::Arc::new(
        create_cache_provider(&config)
            .await
            .expect("Failed to create cache provider"),
    );

    let ttl = Duration::from_secs(60);
    let base_key = format!("test_redis_concurrent_{}", chrono::Utc::now().timestamp_millis());

    // Spawn multiple concurrent tasks
    let mut handles = vec![];

    for i in 0..10 {
        let cache_clone = std::sync::Arc::clone(&cache);
        let key = format!("{}_{}", base_key, i);

        let handle = tokio::spawn(async move {
            let value = format!("concurrent_value_{}", i).into_bytes();

            // Set
            cache_clone
                .set("redis_concurrent_ns", &key, value.clone(), ttl)
                .await
                .expect("Failed to set in concurrent task");

            // Get
            let retrieved = cache_clone
                .get("redis_concurrent_ns", &key)
                .await
                .expect("Failed to get in concurrent task");

            assert!(retrieved.is_some(), "Value should exist");
            assert_eq!(retrieved.unwrap(), value);

            i
        });

        handles.push(handle);
    }

    // Wait for all tasks
    for handle in handles {
        let result = handle.await.expect("Task panicked");
        println!("  ✓ Task {} completed", result);
    }

    // Cleanup
    let _ = cache.clear(Some("redis_concurrent_ns")).await;
    println!("✅ Redis concurrent access works correctly");
}

#[tokio::test]
async fn test_redis_connection_pooling() {
    skip_if_no_redis!();

    let config = CacheConfig {
        enabled: true,
        backend: CacheBackendConfig::Redis {
            url: "redis://127.0.0.1:6379".to_string(),
            pool_size: 3,
            default_ttl_seconds: 3600,
        },
        namespaces: CacheNamespacesConfig::default(),
    };

    let cache = create_cache_provider(&config)
        .await
        .expect("Failed to create cache provider");

    // Perform multiple operations to exercise connection pool
    for i in 0..20 {
        let key = format!("pool_test_{}", i);
        let value = format!("value_{}", i).into_bytes();
        let ttl = Duration::from_secs(60);

        cache
            .set("redis_pool_ns", &key, value.clone(), ttl)
            .await
            .expect("Failed to set in pool test");

        let retrieved = cache
            .get("redis_pool_ns", &key)
            .await
            .expect("Failed to get in pool test");

        assert_eq!(retrieved.unwrap(), value);
    }

    // Cleanup
    let _ = cache.clear(Some("redis_pool_ns")).await;
    println!("✅ Redis connection pooling works correctly");
}
