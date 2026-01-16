//! OpenAI Embedding Provider
//!
//! Implements the EmbeddingProvider port using OpenAI's embedding API.
//! Supports text-embedding-3-small, text-embedding-3-large, and ada-002.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;

use mcb_domain::error::{Error, Result};
use mcb_domain::ports::EmbeddingProvider;
use mcb_domain::value_objects::Embedding;

use crate::adapters::http_client::HttpClientProvider;
use crate::adapters::providers::embedding::helpers::constructor;
use crate::cache::{CacheEntryConfig, SharedCacheProvider};
use crate::constants::{
    CONTENT_TYPE_JSON, EMBEDDING_DIMENSION_OPENAI_ADA, EMBEDDING_DIMENSION_OPENAI_LARGE,
    EMBEDDING_DIMENSION_OPENAI_SMALL,
};

/// Error message for request timeouts
use crate::utils::HttpResponseUtils;

/// OpenAI embedding provider
///
/// Implements the `EmbeddingProvider` domain port using OpenAI's embedding API.
/// Receives HTTP client via constructor injection for DI compliance.
///
/// ## Example
///
/// ```rust,no_run
/// use mcb_infrastructure::adapters::providers::embedding::OpenAIEmbeddingProvider;
/// use mcb_infrastructure::adapters::http_client::HttpClientPool;
/// use std::sync::Arc;
/// use std::time::Duration;
///
/// let http_client = Arc::new(HttpClientPool::new().unwrap());
/// let provider = OpenAIEmbeddingProvider::new(
///     "sk-your-api-key".to_string(),
///     None,
///     "text-embedding-3-small".to_string(),
///     Duration::from_secs(30),
///     http_client,
/// );
/// ```
pub struct OpenAIEmbeddingProvider {
    api_key: String,
    base_url: Option<String>,
    model: String,
    timeout: Duration,
    http_client: Arc<dyn HttpClientProvider>,
    cache_provider: Option<SharedCacheProvider>,
}

impl OpenAIEmbeddingProvider {
    /// Create a new OpenAI embedding provider
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
        let payload = serde_json::json!({
            "input": texts,
            "model": self.model,
            "encoding_format": "float"
        });

        let client = if self.timeout != self.http_client.config().timeout {
            self.http_client
                .client_with_timeout(self.timeout)
                .map_err(|e| Error::embedding(format!("Failed to create HTTP client: {}", e)))?
        } else {
            self.http_client.client().clone()
        };

        let response = client
            .post(format!("{}/embeddings", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.api_key))
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
            HttpResponseUtils::check_and_parse(response, "OpenAI").await?;

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
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        // Check cache first
        let mut cached_embeddings = Vec::new();
        let mut uncached_texts = Vec::new();
        let mut uncached_indices = Vec::new();

        if let Some(cache_provider) = &self.cache_provider {
            let embeddings_cache = cache_provider.namespaced("embeddings");
            for (i, text) in texts.iter().enumerate() {
                let cache_key = self.generate_cache_key(text);

                if let Ok(Some(embedding_data)) = embeddings_cache.get::<Vec<f32>>(&cache_key).await
                {
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

                uncached_texts.push(text.clone());
                uncached_indices.push(i);
            }
        } else {
            uncached_texts.extend(texts.iter().cloned());
            uncached_indices.extend(0..texts.len());
        }

        // Fetch uncached embeddings from API
        let mut new_embeddings = Vec::new();
        if !uncached_texts.is_empty() {
            new_embeddings = self.fetch_embeddings_from_api(&uncached_texts).await?;

            // Cache the new embeddings
            if let Some(cache_provider) = &self.cache_provider {
                let embeddings_cache = cache_provider.namespaced("embeddings");
                for (i, embedding) in new_embeddings.iter().enumerate() {
                    let text = &uncached_texts[i];
                    let cache_key = self.generate_cache_key(text);
                    let config = CacheEntryConfig::new().with_ttl(Duration::from_secs(7200));
                    let _ = embeddings_cache
                        .set(&cache_key, &embedding.vector, config)
                        .await;
                }
            }
        }

        // Combine cached and new embeddings in correct order
        let mut result: Vec<Option<Embedding>> = vec![None; texts.len()];

        for (original_index, embedding) in cached_embeddings {
            result[original_index] = Some(embedding);
        }

        for (i, embedding) in new_embeddings.into_iter().enumerate() {
            let original_index = uncached_indices[i];
            result[original_index] = Some(embedding);
        }

        // Convert Option<Embedding> to Embedding (all should be Some at this point)
        Ok(result.into_iter().flatten().collect())
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
