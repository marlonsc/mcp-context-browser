//! OpenAI embedding provider implementation

use crate::adapters::providers::embedding::helpers::constructor;
use crate::domain::error::{Error, Result};
use crate::domain::ports::EmbeddingProvider;
use crate::domain::types::Embedding;
use crate::infrastructure::cache::SharedCacheProvider;
use crate::infrastructure::constants::{
    EMBEDDING_DIMENSION_OPENAI_ADA, EMBEDDING_DIMENSION_OPENAI_LARGE,
    EMBEDDING_DIMENSION_OPENAI_SMALL,
};
use crate::infrastructure::utils::HttpResponseUtils;
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;

/// OpenAI embedding provider
pub struct OpenAIEmbeddingProvider {
    api_key: String,
    base_url: Option<String>,
    model: String,
    timeout: Duration,
    http_client: Arc<dyn crate::adapters::http_client::HttpClientProvider>,
    cache_provider: Option<SharedCacheProvider>,
}

impl OpenAIEmbeddingProvider {
    /// Create a new OpenAI embedding provider with injected HTTP client
    ///
    /// # Arguments
    /// * `api_key` - OpenAI API key
    /// * `base_url` - Optional custom base URL (defaults to OpenAI API)
    /// * `model` - Model name (e.g., "text-embedding-3-small")
    /// * `timeout` - Request timeout duration
    /// * `http_client` - Injected HTTP client (required for DI compliance)
    pub fn new(
        api_key: String,
        base_url: Option<String>,
        model: String,
        timeout: Duration,
        http_client: Arc<dyn crate::adapters::http_client::HttpClientProvider>,
    ) -> Self {
        let api_key = constructor::validate_api_key(&api_key);
        let base_url = constructor::validate_url(base_url);

        Self {
            api_key,
            base_url,
            model,
            timeout,
            http_client,
            cache_provider: None,
        }
    }

    /// Set the cache provider for this provider
    pub fn with_cache(mut self, cache_provider: SharedCacheProvider) -> Self {
        self.cache_provider = Some(cache_provider);
        self
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

        let response_data: serde_json::Value =
            HttpResponseUtils::check_and_parse(response, "OpenAI").await?;

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

        if let Some(cache_provider) = &self.cache_provider {
            for (i, text) in texts.iter().enumerate() {
                let cache_key = self.generate_cache_key(text);

                if let Ok(Some(cached_bytes)) = cache_provider.get("embeddings", &cache_key).await {
                    // Try to deserialize the cached embedding
                    if let Ok(embedding_data) = serde_json::from_slice::<Vec<f32>>(&cached_bytes) {
                        cached_embeddings.push((
                            i,
                            Embedding {
                                vector: embedding_data,
                                model: self.model.clone(),
                                dimensions: self.dimensions(),
                            },
                        ));
                        continue;
                    }
                }

                // Cache miss or deserialization error
                uncached_texts.push(text.clone());
                uncached_indices.push(i);
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
            if let Some(cache_provider) = &self.cache_provider {
                for (i, embedding) in new_embeddings.iter().enumerate() {
                    let text = &uncached_texts[i];
                    let cache_key = self.generate_cache_key(text);
                    if let Ok(serialized) = serde_json::to_vec(&embedding.vector) {
                        let _ = cache_provider
                            .set(
                                "embeddings",
                                &cache_key,
                                serialized,
                                Duration::from_secs(7200),
                            )
                            .await;
                    }
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
            "text-embedding-3-small" => EMBEDDING_DIMENSION_OPENAI_SMALL,
            "text-embedding-3-large" => EMBEDDING_DIMENSION_OPENAI_LARGE,
            "text-embedding-ada-002" => EMBEDDING_DIMENSION_OPENAI_ADA,
            _ => EMBEDDING_DIMENSION_OPENAI_SMALL, // Default fallback
        }
    }

    fn provider_name(&self) -> &str {
        "openai"
    }
}
