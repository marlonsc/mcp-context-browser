//! HTTP Client Pool
//!
//! Shared HTTP client pool for all providers to optimize connection reuse
//! and reduce latency through connection pooling.
//!
//! ## Architecture
//!
//! This module uses dependency injection via the `HttpClientProvider` trait
//! to enable testability and flexibility. Pass `Arc<dyn HttpClientProvider>`
//! through constructors instead of using global state.

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

/// Trait for HTTP client pool operations (enables DI and testing)
pub trait HttpClientProvider: Send + Sync {
    /// Get a reference to the underlying reqwest Client
    fn client(&self) -> &Client;

    /// Get the configuration
    fn config(&self) -> &HttpClientConfig;

    /// Create a new client with custom timeout for specific operations
    fn client_with_timeout(
        &self,
        timeout: Duration,
    ) -> Result<Client, Box<dyn std::error::Error + Send + Sync>>;

    /// Check if the client pool is enabled
    fn is_enabled(&self) -> bool;
}

/// Type alias for shared HTTP client provider
pub type SharedHttpClient = Arc<dyn HttpClientProvider>;

/// HTTP client pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpClientConfig {
    /// Maximum idle connections per host
    pub max_idle_per_host: usize,
    /// Idle connection timeout
    pub idle_timeout: Duration,
    /// TCP keep-alive duration
    pub keepalive: Duration,
    /// Total timeout for requests
    pub timeout: Duration,
    /// User agent string
    pub user_agent: String,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            max_idle_per_host: 10,
            idle_timeout: Duration::from_secs(90),
            keepalive: Duration::from_secs(60),
            timeout: Duration::from_secs(30),
            user_agent: format!("MCP-Context-Browser/{}", env!("CARGO_PKG_VERSION")),
        }
    }
}

/// Thread-safe HTTP client pool
#[derive(Clone)]
pub struct HttpClientPool {
    client: Client,
    config: HttpClientConfig,
}

impl HttpClientPool {
    /// Create a new HTTP client pool with default configuration
    pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Self::with_config(HttpClientConfig::default())
    }

    /// Create a new HTTP client pool with custom configuration
    pub fn with_config(
        config: HttpClientConfig,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let client = Client::builder()
            .pool_max_idle_per_host(config.max_idle_per_host)
            .pool_idle_timeout(config.idle_timeout)
            .tcp_keepalive(config.keepalive)
            .timeout(config.timeout)
            .user_agent(&config.user_agent)
            .build()?;

        Ok(Self { client, config })
    }

    /// Get a reference to the underlying reqwest Client
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Get the configuration
    pub fn config(&self) -> &HttpClientConfig {
        &self.config
    }

    /// Create a new client with custom timeout for specific operations
    pub fn client_with_timeout(
        &self,
        timeout: Duration,
    ) -> Result<Client, Box<dyn std::error::Error + Send + Sync>> {
        let client = Client::builder()
            .pool_max_idle_per_host(self.config.max_idle_per_host)
            .pool_idle_timeout(self.config.idle_timeout)
            .tcp_keepalive(self.config.keepalive)
            .timeout(timeout)
            .user_agent(&self.config.user_agent)
            .build()?;

        Ok(client)
    }
}

/// Implement the provider trait for HttpClientPool
impl HttpClientProvider for HttpClientPool {
    fn client(&self) -> &Client {
        HttpClientPool::client(self)
    }

    fn config(&self) -> &HttpClientConfig {
        HttpClientPool::config(self)
    }

    fn client_with_timeout(
        &self,
        timeout: Duration,
    ) -> Result<Client, Box<dyn std::error::Error + Send + Sync>> {
        HttpClientPool::client_with_timeout(self, timeout)
    }

    fn is_enabled(&self) -> bool {
        true
    }
}

/// Null HTTP client pool for testing (always returns errors)
#[derive(Clone)]
pub struct NullHttpClientPool {
    config: HttpClientConfig,
    client: Client,
}

impl Default for NullHttpClientPool {
    fn default() -> Self {
        Self::new()
    }
}

impl NullHttpClientPool {
    /// Create a new null HTTP client pool for testing
    pub fn new() -> Self {
        // Create a minimal client for the null implementation
        // Using default Client if builder fails (should never happen with these basic options)
        let client = Client::builder()
            .timeout(Duration::from_millis(1))
            .build()
            .unwrap_or_default();

        Self {
            config: HttpClientConfig::default(),
            client,
        }
    }
}

impl HttpClientProvider for NullHttpClientPool {
    fn client(&self) -> &Client {
        &self.client
    }

    fn config(&self) -> &HttpClientConfig {
        &self.config
    }

    fn client_with_timeout(
        &self,
        _timeout: Duration,
    ) -> Result<Client, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.client.clone())
    }

    fn is_enabled(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    async fn test_http_client_pool_creation() -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    {
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
    async fn test_http_client_pool_via_trait(
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
}
