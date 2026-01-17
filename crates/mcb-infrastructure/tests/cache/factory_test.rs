//! Cache Factory Tests

use mcb_application::ports::providers::cache::CacheEntryConfig;
use mcb_infrastructure::cache::factory::CacheProviderFactory;
use mcb_infrastructure::config::data::CacheConfig;

#[tokio::test]
async fn test_factory_null_provider() {
    let provider = CacheProviderFactory::create_null();

    // Test basic operations
    let value = "value".to_string();
    assert!(provider
        .set("test", &value, CacheEntryConfig::default())
        .await
        .is_ok());
    let result: Option<String> = provider.get("test").await.unwrap();
    assert!(result.is_none()); // Null provider always returns None
}

#[tokio::test]
async fn test_factory_moka_provider() {
    let provider = CacheProviderFactory::create_moka(1024 * 1024); // 1MB

    // Test basic operations
    let value = "value".to_string();
    assert!(provider
        .set("test", &value, CacheEntryConfig::default())
        .await
        .is_ok());
    let result: Option<String> = provider.get("test").await.unwrap();
    assert_eq!(result, Some("value".to_string()));
}

#[tokio::test]
async fn test_factory_redis_provider() {
    let provider = CacheProviderFactory::create_redis("redis://localhost:6379")
        .await
        .unwrap();

    // Test basic operations
    let value = "value".to_string();
    assert!(provider
        .set("test", &value, CacheEntryConfig::default())
        .await
        .is_ok());
    let result: Option<String> = provider.get("test").await.unwrap();
    assert_eq!(result, Some("value".to_string()));
}

#[tokio::test]
async fn test_factory_from_config_disabled() {
    let config = CacheConfig {
        enabled: false,
        ..Default::default()
    };

    let provider = CacheProviderFactory::create_from_config(&config)
        .await
        .unwrap();

    // Should be null provider when disabled
    let result: Option<String> = provider.get("test").await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_factory_from_config_moka() {
    let config = CacheConfig {
        enabled: true,
        provider: mcb_infrastructure::config::data::CacheProvider::Moka,
        max_size: 1024 * 1024,
        namespace: "test".to_string(),
        ..Default::default()
    };

    let provider = CacheProviderFactory::create_from_config(&config)
        .await
        .unwrap();

    // Test that it works and has namespace
    let value = "value".to_string();
    assert!(provider
        .set("key", &value, CacheEntryConfig::default())
        .await
        .is_ok());
    let result: Option<String> = provider.get("key").await.unwrap();
    assert_eq!(result, Some("value".to_string()));
}
