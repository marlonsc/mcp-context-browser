//! Ollama embedding provider implementation

use crate::core::error::{Error, Result};
use crate::core::http_client::{HttpClientPool, get_or_create_global_http_client};
use crate::core::types::Embedding;
use crate::providers::EmbeddingProvider;
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;

/// Ollama embedding provider
pub struct OllamaEmbeddingProvider {
    base_url: String,
    model: String,
    timeout: Duration,
    http_client: Arc<HttpClientPool>,
}

impl OllamaEmbeddingProvider {
    /// Create a new Ollama embedding provider
    pub fn new(base_url: String, model: String) -> Result<Self> {
        Self::with_timeout(base_url, model, Duration::from_secs(30))
    }

    /// Create a new Ollama embedding provider with custom timeout
    pub fn with_timeout(base_url: String, model: String, timeout: Duration) -> Result<Self> {
        let http_client = get_or_create_global_http_client()?;
        Ok(Self {
            base_url,
            model,
            timeout,
            http_client,
        })
    }

    /// Create a new Ollama embedding provider with custom HTTP client
    pub fn with_http_client(
        base_url: String,
        model: String,
        timeout: Duration,
        http_client: Arc<HttpClientPool>,
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

            if !response.status().is_success() {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_default();
                return Err(Error::embedding(format!(
                    "Ollama API error {}: {}",
                    status, error_text
                )));
            }

            let response_data: serde_json::Value = response
                .json()
                .await
                .map_err(|e| Error::embedding(format!("Failed to parse response: {}", e)))?;

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
            "nomic-embed-text" => 768,
            "all-minilm" => 384,
            "mxbai-embed-large" => 1024,
            "snowflake-arctic-embed" => 768,
            _ => 768, // Default for most Ollama embedding models
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
