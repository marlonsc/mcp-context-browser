//! Gemini Embedding Provider
//!
//! Implements the EmbeddingProvider port using Google's Gemini embedding API.

use std::time::Duration;

use async_trait::async_trait;
use reqwest::Client;

use mcb_application::ports::EmbeddingProvider;
use mcb_domain::error::{Error, Result};
use mcb_domain::value_objects::Embedding;

use crate::constants::{CONTENT_TYPE_JSON, EMBEDDING_DIMENSION_GEMINI};

/// Error message for request timeouts
use crate::embedding::helpers::constructor;
use crate::utils::HttpResponseUtils;

/// Gemini embedding provider
///
/// Implements the `EmbeddingProvider` domain port using Google's Gemini embedding API.
/// Receives HTTP client via constructor injection.
///
/// ## Example
///
/// ```rust,no_run
/// use mcb_providers::embedding::GeminiEmbeddingProvider;
/// use reqwest::Client;
/// use std::time::Duration;
///
/// fn example() -> Result<(), Box<dyn std::error::Error>> {
///     let client = Client::builder()
///         .timeout(Duration::from_secs(30))
///         .build()?;
///     let provider = GeminiEmbeddingProvider::new(
///         "AIza-your-api-key".to_string(),
///         None,
///         "text-embedding-004".to_string(),
///         Duration::from_secs(30),
///         client,
///     );
///     Ok(())
/// }
/// ```
pub struct GeminiEmbeddingProvider {
    api_key: String,
    base_url: Option<String>,
    model: String,
    timeout: Duration,
    http_client: Client,
}

impl GeminiEmbeddingProvider {
    /// Create a new Gemini embedding provider
    ///
    /// # Arguments
    /// * `api_key` - Google AI API key
    /// * `base_url` - Optional custom base URL (defaults to Google AI API)
    /// * `model` - Model name (e.g., "text-embedding-004")
    /// * `timeout` - Request timeout duration
    /// * `http_client` - Reqwest HTTP client for making API requests
    pub fn new(
        api_key: String,
        base_url: Option<String>,
        model: String,
        timeout: Duration,
        http_client: Client,
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

    /// Fetch embedding for a single text
    async fn fetch_single_embedding(&self, text: &str) -> Result<serde_json::Value> {
        let payload = serde_json::json!({
            "content": { "parts": [{ "text": text }] }
        });

        let url = format!(
            "{}/v1beta/models/{}:embedContent",
            self.effective_base_url(),
            self.api_model_name()
        );

        let response = self
            .http_client
            .post(&url)
            .header("Content-Type", CONTENT_TYPE_JSON)
            .header("x-goog-api-key", &self.api_key)
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

        HttpResponseUtils::check_and_parse(response, "Gemini").await
    }

    /// Parse embedding from response data
    fn parse_embedding(&self, response_data: &serde_json::Value) -> Result<Embedding> {
        let embedding_vec = response_data["embedding"]["values"]
            .as_array()
            .ok_or_else(|| {
                Error::embedding("Invalid response format: missing embedding values".to_string())
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
impl EmbeddingProvider for GeminiEmbeddingProvider {
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        // Gemini API doesn't support batch embedding - process sequentially
        let mut results = Vec::with_capacity(texts.len());
        for text in texts {
            let response_data = self.fetch_single_embedding(text).await?;
            results.push(self.parse_embedding(&response_data)?);
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

// ============================================================================
// Auto-registration via linkme
// ============================================================================

use mcb_application::ports::registry::{EmbeddingProviderConfig, EmbeddingProviderEntry, EMBEDDING_PROVIDERS};

#[linkme::distributed_slice(EMBEDDING_PROVIDERS)]
static GEMINI_PROVIDER: EmbeddingProviderEntry = EmbeddingProviderEntry {
    name: "gemini",
    description: "Google Gemini embedding provider (gemini-embedding-001, text-embedding-004)",
    factory: |config: &EmbeddingProviderConfig| {
        let api_key = config.api_key.clone()
            .ok_or_else(|| "Gemini requires api_key".to_string())?;
        let base_url = config.base_url.clone();
        let model = config.model.clone()
            .unwrap_or_else(|| "text-embedding-004".to_string());
        let timeout = std::time::Duration::from_secs(30);
        let http_client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        Ok(std::sync::Arc::new(GeminiEmbeddingProvider::new(
            api_key, base_url, model, timeout, http_client
        )))
    },
};
