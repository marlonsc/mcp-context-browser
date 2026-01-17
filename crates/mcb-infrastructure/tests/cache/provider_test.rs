//! Cache Provider Tests
//!
//! Tests for SharedCacheProvider and namespacing.

use mcb_application::ports::providers::cache::CacheEntryConfig;
use mcb_infrastructure::cache::provider::SharedCacheProvider;
use mcb_providers::cache::NullCacheProvider;
use std::time::Duration;

#[tokio::test]
async fn test_shared_cache_provider_basic_operations() {
    let provider = SharedCacheProvider::new(NullCacheProvider::new());

    // Test basic operations (NullCacheProvider always returns None/Ok)
    assert_eq!(provider.get::<String>("test").await.unwrap(), None);
    assert!(provider
        .set("test", &"value".to_string(), CacheEntryConfig::default())
        .await
        .is_ok());
    assert!(!provider.exists("test").await.unwrap());
    assert!(!provider.delete("test").await.unwrap());
    assert!(provider.clear().await.is_ok());
}

#[tokio::test]
async fn test_shared_cache_provider_namespacing() {
    let provider = SharedCacheProvider::new(NullCacheProvider::new());

    // Test namespaced operations
    let namespaced = provider.namespaced("test_ns");

    assert_eq!(namespaced.get::<String>("key").await.unwrap(), None);
    assert!(namespaced
        .set("key", &"value".to_string(), CacheEntryConfig::default())
        .await
        .is_ok());
    assert!(!namespaced.exists("key").await.unwrap());
}

#[tokio::test]
async fn test_cache_entry_config() {
    let config = CacheEntryConfig::new()
        .with_ttl(Duration::from_secs(300))
        .with_namespace("test");

    assert_eq!(config.effective_ttl(), Duration::from_secs(300));
    assert_eq!(config.effective_namespace(), "test");
}
