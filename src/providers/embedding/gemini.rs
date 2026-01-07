//! Gemini (Google AI) embedding provider implementation

use crate::core::error::{Error, Result};
use crate::core::http_client::{HttpClientPool, get_or_create_global_http_client};
use crate::core::types::Embedding;
use crate::providers::EmbeddingProvider;
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;

/// Gemini embedding provider
pub struct GeminiEmbeddingProvider {
    api_key: String,
    base_url: Option<String>,
    model: String,
    timeout: Duration,
    http_client: Arc<HttpClientPool>,
}

impl GeminiEmbeddingProvider {
    /// Create a new Gemini embedding provider
    pub fn new(api_key: String, base_url: Option<String>, model: String) -> Result<Self> {
        Self::with_timeout(api_key, base_url, model, Duration::from_secs(30))
    }

    /// Create a new Gemini embedding provider with custom timeout
    pub fn with_timeout(
        api_key: String,
        base_url: Option<String>,
        model: String,
        timeout: Duration,
    ) -> Result<Self> {
        let http_client = get_or_create_global_http_client()?;
        Ok(Self {
            api_key,
            base_url,
            model,
            timeout,
            http_client,
        })
    }

    /// Create a new Gemini embedding provider with custom HTTP client
    pub fn with_http_client(
        api_key: String,
        base_url: Option<String>,
        model: String,
        timeout: Duration,
        http_client: Arc<HttpClientPool>,
    ) -> Self {
        Self {
            api_key,
            base_url,
            model,
            timeout,
            http_client,
        }
    }

    /// Get the effective base URL
    fn effective_base_url(&self) -> &str {
        self.base_url
            .as_deref()
            .unwrap_or("https://generativelanguage.googleapis.com")
    }

    /// Get the model name for API calls (remove prefix if present)
    pub fn api_model_name(&self) -> &str {
        self.model.strip_prefix("models/").unwrap_or(&self.model)
    }
}

#[async_trait]
impl EmbeddingProvider for GeminiEmbeddingProvider {
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

        let mut results = Vec::new();

        // Gemini API currently doesn't support batch embedding in a single request
        // So we need to make individual requests for each text
        for text in texts {
            // Prepare request payload
            let payload = serde_json::json!({
                "content": {
                    "parts": [
                        {
                            "text": text
                        }
                    ]
                }
            });

            // Use pooled HTTP client
            let client = if self.timeout != self.http_client.config().timeout {
                // Create custom client if timeout differs from pool default
                self.http_client.client_with_timeout(self.timeout)?
            } else {
                self.http_client.client().clone()
            };

            let url = format!(
                "{}/v1beta/models/{}:embedContent?key={}",
                self.effective_base_url(),
                self.api_model_name(),
                self.api_key
            );

            let response = client
                .post(&url)
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
                    "Gemini API error {}: {}",
                    status, error_text
                )));
            }

            let response_data: serde_json::Value = response
                .json()
                .await
                .map_err(|e| Error::embedding(format!("Failed to parse response: {}", e)))?;

            // Parse embedding from response
            let embedding_vec = response_data["embedding"]["values"]
                .as_array()
                .ok_or_else(|| {
                    Error::embedding(
                        "Invalid response format: missing embedding values".to_string(),
                    )
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
        match self.api_model_name() {
            "gemini-embedding-001" => 768,
            "text-embedding-004" => 768,
            _ => 768, // Default for Gemini embedding models
        }
    }

    fn provider_name(&self) -> &str {
        "gemini"
    }
}

impl GeminiEmbeddingProvider {
    /// Get the model name for this provider
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Get the maximum tokens supported by this provider
    pub fn max_tokens(&self) -> usize {
        match self.api_model_name() {
            "gemini-embedding-001" => 2048,
            "text-embedding-004" => 2048,
            _ => 2048, // Default max tokens for Gemini
        }
    }

    /// Get the API key for this provider
    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    /// Get the base URL for this provider
    pub fn base_url(&self) -> &str {
        self.base_url
            .as_deref()
            .unwrap_or("https://generativelanguage.googleapis.com")
    }
}
