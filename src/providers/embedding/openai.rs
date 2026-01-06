//! OpenAI embedding provider implementation

use crate::error::{Error, Result};
use crate::providers::embedding::EmbeddingProvider;
use crate::types::Embedding;
use async_trait::async_trait;

/// OpenAI embedding provider
pub struct OpenAIEmbeddingProvider {
    api_key: String,
    base_url: Option<String>,
    model: String,
}

impl OpenAIEmbeddingProvider {
    /// Create a new OpenAI embedding provider
    pub fn new(api_key: String, base_url: Option<String>, model: String) -> Self {
        Self {
            api_key,
            base_url,
            model,
        }
    }

    /// Get the effective base URL
    fn base_url(&self) -> &str {
        self.base_url.as_deref().unwrap_or("https://api.openai.com/v1")
    }
}

#[async_trait]
impl EmbeddingProvider for OpenAIEmbeddingProvider {
    async fn embed(&self, text: &str) -> Result<Embedding> {
        let embeddings = self.embed_batch(&[text.to_string()]).await?;
        embeddings.into_iter().next()
            .ok_or_else(|| Error::embedding("No embedding returned".to_string()))
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        // Prepare request payload
        let payload = serde_json::json!({
            "input": texts,
            "model": self.model,
            "encoding_format": "float"
        });

        // Make HTTP request to OpenAI
        let client = reqwest::Client::new();
        let response = client
            .post(&format!("{}/embeddings", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| Error::embedding(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::embedding(format!("OpenAI API error {}: {}", status, error_text)));
        }

        let response_data: serde_json::Value = response.json().await
            .map_err(|e| Error::embedding(format!("Failed to parse response: {}", e)))?;

        // Parse embeddings from response
        let data = response_data["data"].as_array()
            .ok_or_else(|| Error::embedding("Invalid response format: missing data array".to_string()))?;

        if data.len() != texts.len() {
            return Err(Error::embedding(format!("Response data count mismatch: expected {}, got {}", texts.len(), data.len())));
        }

        let embeddings = data.iter().enumerate().map(|(i, item)| {
            let embedding_vec = item["embedding"].as_array()
                .ok_or_else(|| Error::embedding(format!("Invalid embedding format for text {}", i)))?
                .iter()
                .map(|v| v.as_f64().unwrap_or(0.0) as f32)
                .collect::<Vec<f32>>();

            Ok(Embedding {
                vector: embedding_vec,
                model: self.model.clone(),
                dimensions: self.dimensions(),
            })
        }).collect::<Result<Vec<_>>>()?;

        Ok(embeddings)
    }

    fn model(&self) -> &str {
        &self.model
    }

    fn dimensions(&self) -> usize {
        1536 // OpenAI text-embedding-3-small
    }

    fn max_tokens(&self) -> usize {
        8192
    }

    fn provider_name(&self) -> &str {
        "openai"
    }
}