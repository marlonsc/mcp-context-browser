//! Moka Cache Provider Tests

use mcb_application::ports::providers::cache::CacheEntryConfig;
use mcb_application::ports::providers::cache::CacheProvider;
use mcb_providers::cache::MokaCacheProvider;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestValue {
    data: String,
    number: i32,
}

#[tokio::test]
async fn test_moka_provider_basic_operations() {
    let provider = MokaCacheProvider::new();

    let value = TestValue {
        data: "test data".to_string(),
        number: 42,
    };

    // Test set_json and get_json
    let json = serde_json::to_string(&value).unwrap();
    provider
        .set_json("test_key", &json, CacheEntryConfig::default())
        .await
        .unwrap();

    let retrieved_json = provider.get_json("test_key").await.unwrap();
    let retrieved: Option<TestValue> = retrieved_json.map(|j| serde_json::from_str(&j).unwrap());
    assert_eq!(retrieved, Some(value));

    // Test exists
    assert!(provider.exists("test_key").await.unwrap());

    // Test delete
    assert!(provider.delete("test_key").await.unwrap());
    assert!(!provider.exists("test_key").await.unwrap());

    // Test get_json after delete
    let retrieved_json = provider.get_json("test_key").await.unwrap();
    assert!(retrieved_json.is_none());
}

#[tokio::test]
async fn test_moka_provider_nonexistent_key() {
    let provider = MokaCacheProvider::new();

    let retrieved = provider.get_json("nonexistent").await.unwrap();
    assert!(retrieved.is_none());

    assert!(!provider.exists("nonexistent").await.unwrap());
    assert!(!provider.delete("nonexistent").await.unwrap());
}

#[tokio::test]
async fn test_moka_provider_clear() {
    let provider = MokaCacheProvider::new();

    // Add some entries
    provider
        .set_json("key1", "\"value1\"", CacheEntryConfig::default())
        .await
        .unwrap();
    provider
        .set_json("key2", "\"value2\"", CacheEntryConfig::default())
        .await
        .unwrap();

    assert_eq!(provider.size().await.unwrap(), 2);

    // Clear cache
    provider.clear().await.unwrap();

    assert_eq!(provider.size().await.unwrap(), 0);
    assert!(!provider.exists("key1").await.unwrap());
    assert!(!provider.exists("key2").await.unwrap());
}

#[tokio::test]
async fn test_moka_provider_stats() {
    let provider = MokaCacheProvider::new();

    provider
        .set_json("key1", "\"value1\"", CacheEntryConfig::default())
        .await
        .unwrap();
    provider
        .set_json("key2", "\"value2\"", CacheEntryConfig::default())
        .await
        .unwrap();

    let stats = provider.stats().await.unwrap();
    assert_eq!(stats.entries, 2);
}
