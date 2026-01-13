//! VoyageAI embedding provider implementation

use crate::adapters::providers::embedding::helpers::constructor;
use crate::domain::error::{Error, Result};
use crate::domain::ports::EmbeddingProvider;
use crate::domain::types::Embedding;
use crate::infrastructure::constants::{
    EMBEDDING_DIMENSION_VOYAGEAI_CODE, EMBEDDING_DIMENSION_VOYAGEAI_DEFAULT,
};
use crate::infrastructure::utils::HttpResponseUtils;
use async_trait::async_trait;
use std::sync::Arc;

/// VoyageAI embedding provider
pub struct VoyageAIEmbeddingProvider {
    api_key: String,
    base_url: Option<String>,
    model: String,
    http_client: Arc<dyn crate::adapters::http_client::HttpClientProvider>,
}

impl VoyageAIEmbeddingProvider {
    /// Create a new VoyageAI embedding provider with injected HTTP client
    ///
    /// # Arguments
    /// * `api_key` - VoyageAI API key
    /// * `base_url` - Optional custom base URL (defaults to VoyageAI API)
    /// * `model` - Model name (e.g., "voyage-code-3")
    /// * `http_client` - Injected HTTP client (required for DI compliance)
    pub fn new(
        api_key: String,
        base_url: Option<String>,
        model: String,
        http_client: Arc<dyn crate::adapters::http_client::HttpClientProvider>,
    ) -> Self {
        let api_key = constructor::validate_api_key(&api_key);
        let base_url = constructor::validate_url(base_url);
        Self {
            api_key,
            base_url,
            model,
            http_client,
        }
    }

    /// Get the effective base URL
    fn effective_base_url(&self) -> String {
        constructor::get_effective_url(self.base_url.as_deref(), "https://api.voyageai.com/v1")
    }
}

#[async_trait]
impl EmbeddingProvider for VoyageAIEmbeddingProvider {
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

        // Prepare request payload
        let payload = serde_json::json!({
            "input": texts,
            "model": self.model
        });

        // Use pooled HTTP client for connection pooling and performance
        let client = self.http_client.client();
        let base_url = self.effective_base_url();

        let response = client
            .post(format!("{}/embeddings", base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| Error::embedding(format!("HTTP request failed: {}", e)))?;

        let response_data: serde_json::Value =
            HttpResponseUtils::check_and_parse(response, "VoyageAI").await?;

        // Parse embeddings from response
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

        let embeddings = data
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let embedding_vec = item["embedding"]
                    .as_array()
                    .ok_or_else(|| {
                        Error::embedding(format!("Invalid embedding format for text {}", i))
                    })?
                    .iter()
                    .map(|v| v.as_f64().unwrap_or(0.0) as f32)
                    .collect::<Vec<f32>>();

                Ok(Embedding {
                    vector: embedding_vec,
                    model: self.model.clone(),
                    dimensions: self.dimensions(),
                })
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(embeddings)
    }

    fn dimensions(&self) -> usize {
        match self.model.as_str() {
            "voyage-code-3" => EMBEDDING_DIMENSION_VOYAGEAI_CODE,
            _ => EMBEDDING_DIMENSION_VOYAGEAI_DEFAULT, // Default for VoyageAI models
        }
    }

    fn provider_name(&self) -> &str {
        "voyageai"
    }
}

impl VoyageAIEmbeddingProvider {
    /// Get the model name for this provider
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Get the maximum tokens supported by this provider
    pub fn max_tokens(&self) -> usize {
        match self.model.as_str() {
            "voyage-code-3" => 16000,
            _ => 16000, // Default max tokens
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
