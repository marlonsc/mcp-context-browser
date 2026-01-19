//! Ollama Embedding Provider
//!
//! Implements the EmbeddingProvider port using Ollama's local embedding API.
//! Supports various local embedding models like nomic-embed-text, all-minilm, etc.

use std::time::Duration;

use async_trait::async_trait;
use reqwest::Client;

use mcb_domain::error::{Error, Result};
use mcb_domain::ports::providers::EmbeddingProvider;
use mcb_domain::value_objects::Embedding;

use crate::constants::{
    CONTENT_TYPE_JSON, EMBEDDING_DIMENSION_OLLAMA_ARCTIC, EMBEDDING_DIMENSION_OLLAMA_DEFAULT,
    EMBEDDING_DIMENSION_OLLAMA_MINILM, EMBEDDING_DIMENSION_OLLAMA_MXBAI,
    EMBEDDING_DIMENSION_OLLAMA_NOMIC,
};

/// Error message for request timeouts
use crate::utils::HttpResponseUtils;

/// Ollama embedding provider
///
/// Implements the `EmbeddingProvider` domain port using Ollama's local embedding API.
/// Receives HTTP client via constructor injection.
///
/// ## Example
///
/// ```rust,no_run
/// use mcb_providers::embedding::OllamaEmbeddingProvider;
/// use reqwest::Client;
/// use std::time::Duration;
///
/// fn example() -> Result<(), Box<dyn std::error::Error>> {
///     let client = Client::builder()
///         .timeout(Duration::from_secs(30))
///         .build()?;
///     let provider = OllamaEmbeddingProvider::new(
///         "http://localhost:11434".to_string(),
///         "nomic-embed-text".to_string(),
///         Duration::from_secs(30),
///         client,
///     );
///     Ok(())
/// }
/// ```
pub struct OllamaEmbeddingProvider {
    base_url: String,
    model: String,
    timeout: Duration,
    http_client: Client,
}

impl OllamaEmbeddingProvider {
    /// Create a new Ollama embedding provider
    ///
    /// # Arguments
    /// * `base_url` - Ollama server URL (e.g., "http://localhost:11434")
    /// * `model` - Model name (e.g., "nomic-embed-text")
    /// * `timeout` - Request timeout duration
    /// * `http_client` - Reqwest HTTP client for making API requests
    pub fn new(base_url: String, model: String, timeout: Duration, http_client: Client) -> Self {
        Self {
            base_url,
            model,
            timeout,
            http_client,
        }
    }

    /// Get the model name for this provider
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Get the maximum tokens supported by this provider
    pub fn max_tokens(&self) -> usize {
        match self.model.as_str() {
            "nomic-embed-text" => 8192,
            "all-minilm" => 512,
            "mxbai-embed-large" => 512,
            "snowflake-arctic-embed" => 512,
            _ => 8192,
        }
    }

    /// Fetch embedding for a single text
    async fn fetch_single_embedding(&self, text: &str) -> Result<serde_json::Value> {
        let payload = serde_json::json!({
            "model": self.model,
            "prompt": text,
            "stream": false
        });

        let response = self
            .http_client
            .post(format!(
                "{}/api/embeddings",
                self.base_url.trim_end_matches('/')
            ))
            .header("Content-Type", CONTENT_TYPE_JSON)
            .timeout(self.timeout)
            .json(&payload)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    Error::embedding(format!(
                        "{} {:?}",
                        crate::constants::ERROR_MSG_REQUEST_TIMEOUT,
                        self.timeout
                    ))
                } else {
                    Error::embedding(format!("HTTP request failed: {}", e))
                }
            })?;

        HttpResponseUtils::check_and_parse(response, "Ollama").await
    }

    /// Parse embedding from response data
    fn parse_embedding(&self, response_data: &serde_json::Value) -> Result<Embedding> {
        let embedding_vec = response_data["embedding"]
            .as_array()
            .ok_or_else(|| {
                Error::embedding("Invalid response format: missing embedding array".to_string())
            })?
            .iter()
            .map(|v| v.as_f64().unwrap_or(0.0) as f32)
            .collect::<Vec<f32>>();

        let dimensions = embedding_vec.len();
        Ok(Embedding {
            vector: embedding_vec,
            model: self.model.clone(),
            dimensions,
        })
    }
}

#[async_trait]
impl EmbeddingProvider for OllamaEmbeddingProvider {
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        // Ollama API doesn't support batch embedding - process sequentially
        let mut results = Vec::with_capacity(texts.len());
        for text in texts {
            let response_data = self.fetch_single_embedding(text).await?;
            results.push(self.parse_embedding(&response_data)?);
        }

        Ok(results)
    }

    fn dimensions(&self) -> usize {
        match self.model.as_str() {
            "nomic-embed-text" => EMBEDDING_DIMENSION_OLLAMA_NOMIC,
            "all-minilm" => EMBEDDING_DIMENSION_OLLAMA_MINILM,
            "mxbai-embed-large" => EMBEDDING_DIMENSION_OLLAMA_MXBAI,
            "snowflake-arctic-embed" => EMBEDDING_DIMENSION_OLLAMA_ARCTIC,
            _ => EMBEDDING_DIMENSION_OLLAMA_DEFAULT,
        }
    }

    fn provider_name(&self) -> &str {
        "ollama"
    }
}

// ============================================================================
// Auto-registration via linkme distributed slice
// ============================================================================

use std::sync::Arc;

use mcb_application::ports::registry::{
    EMBEDDING_PROVIDERS, EmbeddingProviderConfig, EmbeddingProviderEntry,
};
use mcb_domain::ports::providers::EmbeddingProvider as EmbeddingProviderPort;

/// Factory function for creating Ollama embedding provider instances.
fn ollama_factory(
    config: &EmbeddingProviderConfig,
) -> std::result::Result<Arc<dyn EmbeddingProviderPort>, String> {
    let base_url = config
        .base_url
        .clone()
        .unwrap_or_else(|| "http://localhost:11434".to_string());
    let model = config
        .model
        .clone()
        .unwrap_or_else(|| "nomic-embed-text".to_string());
    let timeout = Duration::from_secs(30);
    let http_client = Client::builder()
        .timeout(timeout)
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {e}"))?;

    Ok(Arc::new(OllamaEmbeddingProvider::new(
        base_url,
        model,
        timeout,
        http_client,
    )))
}

#[linkme::distributed_slice(EMBEDDING_PROVIDERS)]
static OLLAMA_PROVIDER: EmbeddingProviderEntry = EmbeddingProviderEntry {
    name: "ollama",
    description: "Ollama local embedding provider (nomic-embed-text, all-minilm, etc.)",
    factory: ollama_factory,
};
