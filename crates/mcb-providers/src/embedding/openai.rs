//! OpenAI Embedding Provider
//!
//! Implements the EmbeddingProvider port using OpenAI's embedding API.
//! Supports text-embedding-3-small, text-embedding-3-large, and ada-002.

use std::time::Duration;

use async_trait::async_trait;
use reqwest::Client;
use shaku::Component;

use mcb_application::ports::EmbeddingProvider;
use mcb_domain::error::{Error, Result};
use mcb_domain::value_objects::Embedding;

use crate::constants::{
    CONTENT_TYPE_JSON, EMBEDDING_DIMENSION_OPENAI_ADA, EMBEDDING_DIMENSION_OPENAI_LARGE,
    EMBEDDING_DIMENSION_OPENAI_SMALL,
};

/// Error message for request timeouts
use crate::embedding::helpers::constructor;
use crate::utils::HttpResponseUtils;

/// OpenAI embedding provider
///
/// Implements the `EmbeddingProvider` domain port using OpenAI's embedding API.
/// Receives HTTP client via constructor injection.
///
/// ## Example
///
/// ```rust,no_run
/// use mcb_providers::embedding::OpenAIEmbeddingProvider;
/// use reqwest::Client;
/// use std::time::Duration;
///
/// fn example() -> Result<(), Box<dyn std::error::Error>> {
///     let client = Client::builder()
///         .timeout(Duration::from_secs(30))
///         .build()?;
///     let provider = OpenAIEmbeddingProvider::new(
///         "sk-your-api-key".to_string(),
///         None,
///         "text-embedding-3-small".to_string(),
///         Duration::from_secs(30),
///         client,
///     );
///     Ok(())
/// }
/// ```
#[derive(Component)]
#[shaku(interface = EmbeddingProvider)]
pub struct OpenAIEmbeddingProvider {
    api_key: String,
    base_url: Option<String>,
    model: String,
    timeout: Duration,
    http_client: Client,
}

impl OpenAIEmbeddingProvider {
    /// Create a new OpenAI embedding provider
    ///
    /// # Arguments
    /// * `api_key` - OpenAI API key
    /// * `base_url` - Optional custom base URL (defaults to OpenAI API)
    /// * `model` - Model name (e.g., "text-embedding-3-small")
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

    /// Get the base URL for this provider
    pub fn base_url(&self) -> &str {
        self.base_url
            .as_deref()
            .unwrap_or("https://api.openai.com/v1")
    }

    /// Get the model name
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Get the maximum tokens for this model
    pub fn max_tokens(&self) -> usize {
        match self.model.as_str() {
            "text-embedding-3-small" => 8192,
            "text-embedding-3-large" => 8192,
            "text-embedding-ada-002" => 8192,
            _ => 8192, // Default fallback
        }
    }

    /// Send embedding request and get response data
    async fn fetch_embeddings(&self, texts: &[String]) -> Result<serde_json::Value> {
        let payload = serde_json::json!({
            "input": texts,
            "model": self.model,
            "encoding_format": "float"
        });

        let response = self
            .http_client
            .post(format!("{}/embeddings", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.api_key))
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

        HttpResponseUtils::check_and_parse(response, "OpenAI").await
    }

    /// Parse embedding vector from response data
    fn parse_embedding(&self, index: usize, item: &serde_json::Value) -> Result<Embedding> {
        let embedding_vec = item["embedding"]
            .as_array()
            .ok_or_else(|| {
                Error::embedding(format!("Invalid embedding format for text {}", index))
            })?
            .iter()
            .map(|v| v.as_f64().unwrap_or(0.0) as f32)
            .collect::<Vec<f32>>();

        Ok(Embedding {
            vector: embedding_vec,
            model: self.model.clone(),
            dimensions: self.dimensions(),
        })
    }
}

#[async_trait]
impl EmbeddingProvider for OpenAIEmbeddingProvider {
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let response_data = self.fetch_embeddings(texts).await?;

        let data = response_data["data"].as_array().ok_or_else(|| {
            Error::embedding("Invalid response format: missing data array".to_string())
        })?;

        if data.len() != texts.len() {
            return Err(Error::embedding(format!(
                "Response data count mismatch: expected {}, got {}",
                texts.len(),
                data.len()
            )));
        }

        data.iter()
            .enumerate()
            .map(|(i, item)| self.parse_embedding(i, item))
            .collect()
    }

    fn dimensions(&self) -> usize {
        match self.model.as_str() {
            "text-embedding-3-small" => EMBEDDING_DIMENSION_OPENAI_SMALL,
            "text-embedding-3-large" => EMBEDDING_DIMENSION_OPENAI_LARGE,
            "text-embedding-ada-002" => EMBEDDING_DIMENSION_OPENAI_ADA,
            _ => EMBEDDING_DIMENSION_OPENAI_SMALL,
        }
    }

    fn provider_name(&self) -> &str {
        "openai"
    }
}

// ============================================================================
// Auto-registration via inventory
// ============================================================================

use mcb_application::ports::registry::{EmbeddingProviderConfig, EmbeddingProviderEntry, EMBEDDING_PROVIDERS};

#[linkme::distributed_slice(mcb_application::ports::registry::EMBEDDING_PROVIDERS)]
static OPENAI_PROVIDER: mcb_application::ports::registry::EmbeddingProviderEntry = mcb_application::ports::registry::EmbeddingProviderEntry {
    name: "openai",
    description: "OpenAI embedding provider (text-embedding-3-small/large, ada-002)",
    factory: |config: &mcb_application::ports::EmbeddingProviderConfig| {
        let api_key = config.api_key.clone()
            .ok_or_else(|| "OpenAI requires api_key".to_string())?;
        let base_url = config.base_url.clone();
        let model = config.model.clone()
            .unwrap_or_else(|| "text-embedding-3-small".to_string());
        let timeout = std::time::Duration::from_secs(30);
        let http_client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        Ok(std::sync::Arc::new(OpenAIEmbeddingProvider::new(
            api_key, base_url, model, timeout, http_client
        )))
    },
};
