//! Gemini Embedding Provider
//!
//! Implements the EmbeddingProvider port using Google's Gemini embedding API.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;

use mcb_domain::error::{Error, Result};
use mcb_domain::ports::EmbeddingProvider;
use mcb_domain::value_objects::Embedding;

use crate::adapters::http_client::HttpClientProvider;
use crate::adapters::providers::embedding::helpers::constructor;
use crate::constants::{CONTENT_TYPE_JSON, EMBEDDING_DIMENSION_GEMINI};

/// Error message for request timeouts
use crate::utils::HttpResponseUtils;

/// Gemini embedding provider
///
/// Implements the `EmbeddingProvider` domain port using Google's Gemini embedding API.
/// Receives HTTP client via constructor injection for DI compliance.
///
/// ## Example
///
/// ```rust,no_run
/// use mcb_infrastructure::adapters::providers::embedding::GeminiEmbeddingProvider;
/// use mcb_infrastructure::adapters::http_client::HttpClientPool;
/// use std::sync::Arc;
/// use std::time::Duration;
///
/// let http_client = Arc::new(HttpClientPool::new().unwrap());
/// let provider = GeminiEmbeddingProvider::new(
///     "AIza-your-api-key".to_string(),
///     None,
///     "text-embedding-004".to_string(),
///     Duration::from_secs(30),
///     http_client,
/// );
/// ```
pub struct GeminiEmbeddingProvider {
    api_key: String,
    base_url: Option<String>,
    model: String,
    timeout: Duration,
    http_client: Arc<dyn HttpClientProvider>,
}

impl GeminiEmbeddingProvider {
    /// Create a new Gemini embedding provider
    ///
    /// # Arguments
    /// * `api_key` - Google AI API key
    /// * `base_url` - Optional custom base URL (defaults to Google AI API)
    /// * `model` - Model name (e.g., "text-embedding-004")
    /// * `timeout` - Request timeout duration
    /// * `http_client` - Injected HTTP client (required for DI compliance)
    pub fn new(
        api_key: String,
        base_url: Option<String>,
        model: String,
        timeout: Duration,
        http_client: Arc<dyn HttpClientProvider>,
    ) -> Self {
        let api_key = constructor::validate_api_key(&api_key);
        let base_url = constructor::validate_url(base_url);
        Self {
            api_key,
            base_url,
            model,
            timeout,
            http_client,
        }
    }

    /// Get the effective base URL
    fn effective_base_url(&self) -> String {
        constructor::get_effective_url(
            self.base_url.as_deref(),
            "https://generativelanguage.googleapis.com",
        )
    }

    /// Get the model name for API calls (remove prefix if present)
    pub fn api_model_name(&self) -> &str {
        self.model.strip_prefix("models/").unwrap_or(&self.model)
    }

    /// Get the model name for this provider
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Get the maximum tokens supported by this provider
    pub fn max_tokens(&self) -> usize {
        match self.api_model_name() {
            "gemini-embedding-001" => 2048,
            "text-embedding-004" => 2048,
            _ => 2048,
        }
    }

    /// Get the API key for this provider
    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    /// Get the base URL for this provider
    pub fn base_url(&self) -> String {
        self.effective_base_url()
    }
}

#[async_trait]
impl EmbeddingProvider for GeminiEmbeddingProvider {
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let mut results = Vec::new();

        // Gemini API currently doesn't support batch embedding in a single request
        for text in texts {
            let payload = serde_json::json!({
                "content": {
                    "parts": [
                        {
                            "text": text
                        }
                    ]
                }
            });

            let client = if self.timeout != self.http_client.config().timeout {
                self.http_client
                    .client_with_timeout(self.timeout)
                    .map_err(|e| Error::embedding(format!("Failed to create HTTP client: {}", e)))?
            } else {
                self.http_client.client().clone()
            };

            let base_url = self.effective_base_url();
            let url = format!(
                "{}/v1beta/models/{}:embedContent?key={}",
                base_url,
                self.api_model_name(),
                self.api_key
            );

            let response = client
                .post(&url)
                .header("Content-Type", CONTENT_TYPE_JSON)
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
                HttpResponseUtils::check_and_parse(response, "Gemini").await?;

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
            "gemini-embedding-001" => EMBEDDING_DIMENSION_GEMINI,
            "text-embedding-004" => EMBEDDING_DIMENSION_GEMINI,
            _ => EMBEDDING_DIMENSION_GEMINI,
        }
    }

    fn provider_name(&self) -> &str {
        "gemini"
    }
}
