//! HTTP Client Provider Trait
//!
//! Defines the interface for HTTP client operations used by API-based providers.
//! This trait enables dependency injection for HTTP-based adapters.
//!
//! ## Design Rationale
//!
//! This trait is defined in mcb-providers (not mcb-infrastructure) following
//! Clean Architecture's dependency inversion principle. Provider implementations
//! depend on this abstraction, while mcb-infrastructure provides the concrete
//! implementation that gets injected at runtime.

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// HTTP client configuration
///
/// Controls connection pooling, timeouts, and other HTTP client behavior.
/// Used by `HttpClientProvider` to configure HTTP requests.
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
            user_agent: format!("mcb/{}", env!("CARGO_PKG_VERSION")),
        }
    }
}

impl HttpClientConfig {
    /// Create configuration with custom timeout only
    pub fn with_timeout(timeout: Duration) -> Self {
        Self {
            timeout,
            ..Default::default()
        }
    }
}

/// HTTP client provider trait
///
/// Defines the interface for HTTP client operations used by API-based providers.
/// Implementations are provided by mcb-infrastructure and injected via DI.
///
/// ## Thread Safety
///
/// All implementations must be `Send + Sync` for thread-safe sharing
/// across async contexts.
///
/// # Example
///
/// ```no_run
/// use mcb_providers::http::HttpClientProvider;
///
/// async fn fetch_embeddings(provider: &dyn HttpClientProvider, text: &str) {
///     // let response = provider.client()
///     //     .post("https://api.example.com/embeddings")
///     //     .json(&serde_json::json!({ "input": text }))
///     //     .send()
///     //     .await;
/// }
/// ```
pub trait HttpClientProvider: Send + Sync {
    /// Get a reference to the underlying reqwest Client
    fn client(&self) -> &Client;

    /// Get the configuration
    fn config(&self) -> &HttpClientConfig;

    /// Create a new client with custom timeout for specific operations
    ///
    /// Some API calls (like large batch embeddings) may need longer
    /// timeouts than the default pool configuration.
    fn client_with_timeout(
        &self,
        timeout: Duration,
    ) -> Result<Client, Box<dyn std::error::Error + Send + Sync>>;

    /// Check if the client pool is enabled
    ///
    /// Returns `false` for null/test implementations.
    fn is_enabled(&self) -> bool;
}
