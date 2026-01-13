//! Ollama embedding provider implementation

use crate::domain::error::{Error, Result};
use crate::domain::ports::EmbeddingProvider;
use crate::domain::types::Embedding;
use crate::infrastructure::constants::{
    EMBEDDING_DIMENSION_OLLAMA_ARCTIC, EMBEDDING_DIMENSION_OLLAMA_DEFAULT,
    EMBEDDING_DIMENSION_OLLAMA_MINILM, EMBEDDING_DIMENSION_OLLAMA_MXBAI,
    EMBEDDING_DIMENSION_OLLAMA_NOMIC,
};
use crate::infrastructure::utils::HttpResponseUtils;
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;

/// Ollama embedding provider
pub struct OllamaEmbeddingProvider {
    base_url: String,
    model: String,
    timeout: Duration,
    http_client: Arc<dyn crate::adapters::http_client::HttpClientProvider>,
}

impl OllamaEmbeddingProvider {
    /// Create a new Ollama embedding provider with injected HTTP client
    ///
    /// # Arguments
    /// * `base_url` - Ollama server URL (e.g., "http://localhost:11434")
    /// * `model` - Model name (e.g., "nomic-embed-text")
    /// * `timeout` - Request timeout duration
    /// * `http_client` - Injected HTTP client (required for DI compliance)
    pub fn new(
        base_url: String,
        model: String,
        timeout: Duration,
        http_client: Arc<dyn crate::adapters::http_client::HttpClientProvider>,
    ) -> Self {
        Self {
            base_url,
            model,
            timeout,
            http_client,
        }
    }
}

#[async_trait]
impl EmbeddingProvider for OllamaEmbeddingProvider {
    async fn embed(&self, text: &str) -> Result<Embedding> {
        let embeddings = self.embed_batch(&[text.to_string()]).await?;
        embeddings
            .into_iter()
            .next()
            .ok_or_else(|| Error::embedding("No embedding returned".to_string()))
    }

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

            // Use pooled HTTP client
            let client = if self.timeout != self.http_client.config().timeout {
                // Create custom client if timeout differs from pool default
                self.http_client.client_with_timeout(self.timeout)?
            } else {
                self.http_client.client().clone()
            };

            let response = client
                .post(format!(
                    "{}/api/embeddings",
                    self.base_url.trim_end_matches('/')
                ))
                .header("Content-Type", "application/json")
                .json(&payload)
                .send()
                .await
                .map_err(|e| {
                    if e.is_timeout() {
                        Error::embedding(format!("Request timed out after {:?}", self.timeout))
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
            _ => EMBEDDING_DIMENSION_OLLAMA_DEFAULT, // Default for most Ollama embedding models
        }
    }

    fn provider_name(&self) -> &str {
        "ollama"
    }
}

impl OllamaEmbeddingProvider {
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
            _ => 8192, // Default max tokens
        }
    }
}
