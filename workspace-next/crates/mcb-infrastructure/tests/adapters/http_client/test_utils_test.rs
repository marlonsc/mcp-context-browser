//! HTTP Client Test Utilities Tests
//!
//! Tests for the null HTTP client pool used in testing.

use mcb_infrastructure::adapters::http_client::test_utils::NullHttpClientPool;
use mcb_infrastructure::adapters::http_client::HttpClientProvider;

#[test]
fn test_null_pool_not_enabled() {
    let pool = NullHttpClientPool::new();
    assert!(!pool.is_enabled());
}

#[test]
fn test_null_pool_default() {
    let pool = NullHttpClientPool::default();
    assert!(!pool.is_enabled());
}

#[test]
fn test_null_pool_has_client() {
    let pool = NullHttpClientPool::new();
    // Verify that calling client() doesn't panic and returns a valid client
    let _client = pool.client();
    // Assert pool state remains consistent
    assert!(!pool.is_enabled());
}
