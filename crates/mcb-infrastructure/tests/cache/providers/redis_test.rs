//! Redis Cache Provider Tests
//!
//! Note: These tests require a Redis server to be running.

use mcb_application::ports::providers::cache::CacheEntryConfig;
use mcb_application::ports::providers::cache::CacheProvider;
use mcb_providers::cache::RedisCacheProvider;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestValue {
    data: String,
    number: i32,
}

#[tokio::test]
async fn test_redis_provider_basic_operations() {
    let provider = RedisCacheProvider::with_host_port("localhost", 6379).unwrap();

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
}

#[test]
fn test_redis_provider_creation() {
    // Test with valid connection string
    let result = RedisCacheProvider::new("redis://localhost:6379");
    assert!(result.is_ok());

    // Test with invalid connection string
    let result = RedisCacheProvider::new("invalid://url");
    assert!(result.is_err());
}

#[test]
fn test_redis_provider_addresses() {
    let provider = RedisCacheProvider::new("redis://localhost:6379").unwrap();
    // Note: server_address returns a placeholder since connection_info.addr is private
    assert!(!provider.server_address().is_empty());
    assert!(!provider.is_tls());
}
