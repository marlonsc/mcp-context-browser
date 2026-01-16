//! Null Cache Provider Tests

use mcb_infrastructure::cache::config::CacheEntryConfig;
use mcb_infrastructure::cache::providers::NullCacheProvider;
use mcb_infrastructure::cache::CacheProvider;

#[tokio::test]
async fn test_null_provider_operations() {
    let provider = NullCacheProvider::new();

    // Test get_json (should always return None)
    let result = provider.get_json("test_key").await.unwrap();
    assert!(result.is_none());

    // Test set_json (should succeed)
    let value_json = "\"test_value\"";
    assert!(provider
        .set_json("test_key", value_json, CacheEntryConfig::default())
        .await
        .is_ok());

    // Test exists (should always return false)
    assert!(!provider.exists("test_key").await.unwrap());

    // Test delete (should return false)
    assert!(!provider.delete("test_key").await.unwrap());

    // Test clear (should succeed)
    assert!(provider.clear().await.is_ok());

    // Test stats (should be empty)
    let stats = provider.stats().await.unwrap();
    assert_eq!(stats.hits, 0);
    assert_eq!(stats.misses, 0);
    assert_eq!(stats.entries, 0);

    // Test size (should be 0)
    assert_eq!(provider.size().await.unwrap(), 0);
}

#[test]
fn test_null_provider_default() {
    let provider = NullCacheProvider::default();
    // Verify default provider has correct debug representation
    assert!(format!("{:?}", provider).contains("NullCacheProvider"));
}
