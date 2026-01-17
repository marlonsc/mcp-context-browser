//! Null provider implementations for testing and Shaku DI defaults
//!
//! These implementations provide no-op behavior and are used as:
//! - Shaku DI defaults for testing
//! - Runtime injection fallbacks
//! - Development without external dependencies

use async_trait::async_trait;
use mcb_domain::error::Result;
use mcb_domain::ports::providers::{EmbeddingProvider, VectorStoreAdmin, VectorStoreProvider};
use mcb_domain::value_objects::{Embedding, SearchResult};
use serde_json::Value;
use std::collections::HashMap;

/// Null embedding provider for testing and Shaku DI default
#[derive(shaku::Component)]
#[shaku(interface = EmbeddingProvider)]
pub struct NullEmbeddingProvider;

impl NullEmbeddingProvider {
    pub fn new() -> Self { Self }
}

impl Default for NullEmbeddingProvider {
    fn default() -> Self { Self::new() }
}

#[async_trait]
impl EmbeddingProvider for NullEmbeddingProvider {
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>> {
        Ok(texts.iter().map(|_| Embedding {
            vector: vec![0.0; 384],
            model: "null".to_string(),
            dimensions: 384,
        }).collect())
    }

    fn dimensions(&self) -> usize { 384 }

    fn provider_name(&self) -> &str { "null" }
}

/// Null vector store provider for testing and Shaku DI default
#[derive(shaku::Component)]
#[shaku(interface = VectorStoreProvider)]
pub struct NullVectorStoreProvider;

impl NullVectorStoreProvider {
    pub fn new() -> Self { Self }
}

impl Default for NullVectorStoreProvider {
    fn default() -> Self { Self::new() }
}

#[async_trait]
impl VectorStoreAdmin for NullVectorStoreProvider {
    async fn collection_exists(&self, _name: &str) -> Result<bool> { Ok(true) }

    async fn get_stats(&self, _collection: &str) -> Result<HashMap<String, Value>> {
        Ok(HashMap::new())
    }

    async fn flush(&self, _collection: &str) -> Result<()> { Ok(()) }

    fn provider_name(&self) -> &str { "null" }
}

#[async_trait]
impl VectorStoreProvider for NullVectorStoreProvider {
    async fn create_collection(&self, _name: &str, _dimensions: usize) -> Result<()> { Ok(()) }

    async fn delete_collection(&self, _name: &str) -> Result<()> { Ok(()) }

    async fn insert_vectors(
        &self,
        _collection: &str,
        embeddings: &[Embedding],
        _metadata: Vec<HashMap<String, Value>>,
    ) -> Result<Vec<String>> {
        Ok((0..embeddings.len()).map(|i| format!("null-{}", i)).collect())
    }

    async fn search_similar(
        &self,
        _collection: &str,
        _query_vector: &[f32],
        _limit: usize,
        _filter: Option<&str>,
    ) -> Result<Vec<SearchResult>> { Ok(vec![]) }

    async fn delete_vectors(&self, _collection: &str, _ids: &[String]) -> Result<()> { Ok(()) }

    async fn get_vectors_by_ids(
        &self,
        _collection: &str,
        _ids: &[String],
    ) -> Result<Vec<SearchResult>> { Ok(vec![]) }

    async fn list_vectors(&self, _collection: &str, _limit: usize) -> Result<Vec<SearchResult>> {
        Ok(vec![])
    }
}