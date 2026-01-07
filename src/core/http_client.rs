//! HTTP Client Pool
//!
//! Shared HTTP client pool for all providers to optimize connection reuse
//! and reduce latency through connection pooling.

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

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

/// Global HTTP client pool instance
static HTTP_CLIENT_POOL: std::sync::OnceLock<Arc<HttpClientPool>> = std::sync::OnceLock::new();

/// Initialize the global HTTP client pool
pub fn init_global_http_client(
    config: HttpClientConfig,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let pool = Arc::new(HttpClientPool::with_config(config)?);
    HTTP_CLIENT_POOL
        .set(pool)
        .map_err(|_| "HTTP client pool already initialized".into())
}

/// Get the global HTTP client pool
pub fn get_global_http_client() -> Option<Arc<HttpClientPool>> {
    HTTP_CLIENT_POOL.get().cloned()
}

/// Get the global HTTP client or create a default one
pub fn get_or_create_global_http_client()
-> Result<Arc<HttpClientPool>, Box<dyn std::error::Error + Send + Sync>> {
    // First check if we already have a pool
    if let Some(pool) = get_global_http_client() {
        return Ok(pool);
    }

    // Try to create and set a new pool
    let pool = Arc::new(HttpClientPool::new()?);

    // Try to set it, but if it fails (already set), get the existing one
    match HTTP_CLIENT_POOL.set(Arc::clone(&pool)) {
        Ok(()) => Ok(pool),
        Err(_) => {
            // Already set by another thread/test, get the existing one
            get_global_http_client().ok_or_else(|| "HTTP client pool not available".into())
        }
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
    async fn test_http_client_pool_creation() {
        let config = HttpClientConfig {
            max_idle_per_host: 5,
            idle_timeout: Duration::from_secs(60),
            keepalive: Duration::from_secs(30),
            timeout: Duration::from_secs(10),
            user_agent: "Test-Agent/1.0".to_string(),
        };

        let pool = HttpClientPool::with_config(config.clone()).unwrap();
        assert_eq!(pool.config().max_idle_per_host, 5);
        assert_eq!(pool.config().user_agent, "Test-Agent/1.0");
    }

    #[tokio::test]
    async fn test_global_http_client() {
        // Reset for test
        // Note: OnceLock can't be reset, so we'll test the creation path

        let config = HttpClientConfig::default();
        let pool = HttpClientPool::with_config(config).unwrap();
        let client = pool.client();

        // Test that we can create a request (doesn't need to succeed)
        let request = client.get("http://httpbin.org/status/200").build().unwrap();
        assert_eq!(request.method(), "GET");
    }
}
