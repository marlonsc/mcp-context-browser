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

use crate::infrastructure::constants::{
    HTTP_CLIENT_IDLE_TIMEOUT, HTTP_KEEPALIVE_DURATION, HTTP_MAX_IDLE_PER_HOST, HTTP_REQUEST_TIMEOUT,
};

/// Trait for HTTP client pool operations (enables DI and testing)
pub trait HttpClientProvider: shaku::Interface + Send + Sync {
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
            max_idle_per_host: HTTP_MAX_IDLE_PER_HOST as usize,
            idle_timeout: HTTP_CLIENT_IDLE_TIMEOUT,
            keepalive: HTTP_KEEPALIVE_DURATION,
            timeout: HTTP_REQUEST_TIMEOUT,
            user_agent: format!("MCP-Context-Browser/{}", env!("CARGO_PKG_VERSION")),
        }
    }
}

/// Thread-safe HTTP client pool
#[derive(Clone, shaku::Component)]
#[shaku(interface = HttpClientProvider)]
pub struct HttpClientPool {
    #[shaku(default = Client::new())]
    // This is just for shaku, will be overwritten in new() or used defaults
    client: Client,
    #[shaku(default = HttpClientConfig::default())]
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
        let client = Self::build_client(&config, config.timeout)?;
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
        Self::build_client(&self.config, timeout)
    }

    /// Build a reqwest Client from config (DRY helper)
    fn build_client(
        config: &HttpClientConfig,
        timeout: Duration,
    ) -> Result<Client, Box<dyn std::error::Error + Send + Sync>> {
        Client::builder()
            .pool_max_idle_per_host(config.max_idle_per_host)
            .pool_idle_timeout(config.idle_timeout)
            .tcp_keepalive(config.keepalive)
            .timeout(timeout)
            .user_agent(&config.user_agent)
            .build()
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
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

// Note: NullHttpClientPool in test_utils module (Phase 5 DI audit)
// Public for external test access but NOT re-exported at parent level
// Tests import via: mcp_context_browser::adapters::http_client::test_utils::NullHttpClientPool
pub mod test_utils {
    use super::*;

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
}

// NullHttpClientPool NOT re-exported at module level
// Tests import via: mcp_context_browser::adapters::http_client::test_utils::NullHttpClientPool
