//! Tests for HTTP client pool
//!
//! Tests for the HTTP client adapter module including connection pooling,
//! configuration, and the null HTTP client pool for testing.

use mcp_context_browser::adapters::http_client::test_utils::NullHttpClientPool;
use mcp_context_browser::adapters::http_client::{
    HttpClientConfig, HttpClientPool, HttpClientProvider,
};
use std::time::Duration;

#[test]
fn test_http_client_config_default() {
    let config = HttpClientConfig::default();
    assert_eq!(config.max_idle_per_host, 10);
    assert_eq!(config.idle_timeout, Duration::from_secs(90));
    assert_eq!(config.keepalive, Duration::from_secs(60));
    assert_eq!(config.timeout, Duration::from_secs(30));
    assert!(config.user_agent.contains("MCP-Context-Browser"));
}

#[tokio::test]
async fn test_http_client_pool_creation() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = HttpClientConfig {
        max_idle_per_host: 5,
        idle_timeout: Duration::from_secs(60),
        keepalive: Duration::from_secs(30),
        timeout: Duration::from_secs(10),
        user_agent: "Test-Agent/1.0".to_string(),
    };

    let pool = HttpClientPool::with_config(config.clone())?;
    assert_eq!(pool.config().max_idle_per_host, 5);
    assert_eq!(pool.config().user_agent, "Test-Agent/1.0");
    Ok(())
}

#[tokio::test]
async fn test_http_client_pool_via_trait() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = HttpClientConfig::default();
    let pool = HttpClientPool::with_config(config)?;
    let client = pool.client();

    // Test that we can create a request (doesn't need to succeed)
    let request = client.get("http://httpbin.org/status/200").build()?;
    assert_eq!(request.method(), "GET");
    Ok(())
}

#[test]
fn test_null_http_client_pool() {
    let pool = NullHttpClientPool::new();
    assert!(!pool.is_enabled());
    assert_eq!(pool.config().max_idle_per_host, 10);
}

#[test]
fn test_http_client_provider_trait() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let pool = HttpClientPool::new()?;
    let provider: &dyn HttpClientProvider = &pool;
    assert!(provider.is_enabled());
    Ok(())
}
