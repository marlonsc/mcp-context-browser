//! OpenAI embedding provider implementation

use crate::core::cache::{CacheResult, get_global_cache_manager};
use crate::core::error::{Error, Result};
use crate::core::http_client::{HttpClientPool, get_or_create_global_http_client};
use crate::core::types::Embedding;
use crate::providers::EmbeddingProvider;
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;

/// OpenAI embedding provider
pub struct OpenAIEmbeddingProvider {
    api_key: String,
    base_url: Option<String>,
    model: String,
    timeout: Duration,
    http_client: Arc<HttpClientPool>,
}

impl OpenAIEmbeddingProvider {
    /// Create a new OpenAI embedding provider
    pub fn new(api_key: String, base_url: Option<String>, model: String) -> Result<Self> {
        Self::with_timeout(api_key, base_url, model, Duration::from_secs(30))
    }

    /// Create a new OpenAI embedding provider with custom timeout
    pub fn with_timeout(
        api_key: String,
        base_url: Option<String>,
        model: String,
        timeout: Duration,
    ) -> Result<Self> {
        let api_key = api_key.trim().to_string();
        let base_url = base_url.map(|url| url.trim().to_string());
        let http_client = get_or_create_global_http_client()?;

        Ok(Self {
            api_key,
            base_url,
            model,
            timeout,
            http_client,
        })
    }

    /// Get the effective base URL
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

    /// Fetch embeddings from OpenAI API
    async fn fetch_embeddings_from_api(&self, texts: &[String]) -> Result<Vec<Embedding>> {
        // Prepare request payload
        let payload = serde_json::json!({
            "input": texts,
            "model": self.model,
            "encoding_format": "float"
        });

        // Use pooled HTTP client
        let client = if self.timeout != self.http_client.config().timeout {
            // Create custom client if timeout differs from pool default
            self.http_client.client_with_timeout(self.timeout)?
        } else {
            self.http_client.client().clone()
        };

        let response = client
            .post(format!("{}/embeddings", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.api_key))
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
                "OpenAI API error {}: {}",
                status, error_text
            )));
        }

        let response_data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| Error::embedding(format!("Failed to parse response: {}", e)))?;

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

    /// Generate cache key for text
    fn generate_cache_key(&self, text: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        self.model.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

#[async_trait]
impl EmbeddingProvider for OpenAIEmbeddingProvider {
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

        // Check cache first
        let mut cached_embeddings = Vec::new();
        let mut uncached_texts = Vec::new();
        let mut uncached_indices = Vec::new();

        if let Some(cache_manager) = get_global_cache_manager() {
            for (i, text) in texts.iter().enumerate() {
                let cache_key = self.generate_cache_key(text);
                let cache_result: CacheResult<Vec<f32>> =
                    cache_manager.get("embeddings", &cache_key).await;

                match cache_result {
                    CacheResult::Hit(embedding_data) => {
                        cached_embeddings.push((
                            i,
                            Embedding {
                                vector: embedding_data,
                                model: self.model.clone(),
                                dimensions: self.dimensions(),
                            },
                        ));
                    }
                    _ => {
                        uncached_texts.push(text.clone());
                        uncached_indices.push(i);
                    }
                }
            }
        } else {
            // No cache available, process all texts
            uncached_texts.extend(texts.iter().cloned());
            uncached_indices.extend(0..texts.len());
        }

        // Fetch uncached embeddings from API
        let mut new_embeddings = Vec::new();
        if !uncached_texts.is_empty() {
            new_embeddings = self.fetch_embeddings_from_api(&uncached_texts).await?;

            // Cache the new embeddings
            if let Some(cache_manager) = get_global_cache_manager() {
                for (i, embedding) in new_embeddings.iter().enumerate() {
                    let text = &uncached_texts[i];
                    let cache_key = self.generate_cache_key(text);
                    let _ = cache_manager
                        .set("embeddings", &cache_key, embedding.vector.clone())
                        .await;
                }
            }
        }

        // Combine cached and new embeddings in correct order
        let mut result = vec![Embedding::default(); texts.len()];

        // Place cached embeddings
        for (original_index, embedding) in cached_embeddings {
            result[original_index] = embedding;
        }

        // Place new embeddings
        for (i, embedding) in new_embeddings.into_iter().enumerate() {
            let original_index = uncached_indices[i];
            result[original_index] = embedding;
        }

        Ok(result)
    }

    fn dimensions(&self) -> usize {
        match self.model.as_str() {
            "text-embedding-3-small" => 1536,
            "text-embedding-3-large" => 3072,
            "text-embedding-ada-002" => 1536,
            _ => 1536, // Default fallback
        }
    }

    fn provider_name(&self) -> &str {
        "openai"
    }
}
