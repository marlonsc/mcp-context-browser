//! Ollama Embedding Provider
//!
//! Implements the EmbeddingProvider port using Ollama's local embedding API.
//! Supports various local embedding models like nomic-embed-text, all-minilm, etc.

use std::time::Duration;

use async_trait::async_trait;
use reqwest::Client;

use mcb_domain::error::{Error, Result};
use mcb_domain::ports::EmbeddingProvider;
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
/// fn example() {
///     let client = Client::builder()
///         .timeout(Duration::from_secs(30))
///         .build()
///         .unwrap();
///     let provider = OllamaEmbeddingProvider::new(
///         "http://localhost:11434".to_string(),
///         "nomic-embed-text".to_string(),
///         Duration::from_secs(30),
///         client,
///     );
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
    pub fn new(
        base_url: String,
        model: String,
        timeout: Duration,
        http_client: Client,
    ) -> Self {
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
}

#[async_trait]
impl EmbeddingProvider for OllamaEmbeddingProvider {
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        // Ollama embeddings API expects individual requests
        let mut results = Vec::new();

        for text in texts {
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
                        Error::embedding(format!("{} {:?}", crate::constants::ERROR_MSG_REQUEST_TIMEOUT, self.timeout))
                    } else {
                        Error::embedding(format!("HTTP request failed: {}", e))
                    }
                })?;

            let response_data: serde_json::Value =
                HttpResponseUtils::check_and_parse(response, "Ollama").await?;

            let embedding_vec = response_data["embedding"]
                .as_array()
                .ok_or_else(|| {
                    Error::embedding("Invalid response format: missing embedding array".to_string())
                })?
                .iter()
                .map(|v| v.as_f64().unwrap_or(0.0) as f32)
                .collect::<Vec<f32>>();

            let dimensions = embedding_vec.len();
            results.push(Embedding {
                vector: embedding_vec,
                model: self.model.clone(),
                dimensions,
            });
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
