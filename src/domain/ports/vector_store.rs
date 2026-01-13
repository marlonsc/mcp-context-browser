use crate::domain::error::Result;
use crate::domain::types::{Embedding, SearchResult};
use async_trait::async_trait;
use serde_json::Value;
use shaku::Interface;
use std::collections::HashMap;

/// Enterprise Vector Storage Interface
///
/// Defines the business contract for vector storage systems that persist and
/// retrieve semantic embeddings at enterprise scale. This abstraction supports
/// multiple storage backends from in-memory development stores to production
/// Milvus clusters, ensuring optimal performance for different business needs.
#[async_trait]
pub trait VectorStoreProvider: Interface + Send + Sync {
    async fn create_collection(&self, name: &str, dimensions: usize) -> Result<()>;
    async fn delete_collection(&self, name: &str) -> Result<()>;
    async fn collection_exists(&self, name: &str) -> Result<bool>;
    async fn insert_vectors(
        &self,
        collection: &str,
        vectors: &[Embedding],
        metadata: Vec<HashMap<String, Value>>,
    ) -> Result<Vec<String>>;
    async fn search_similar(
        &self,
        collection: &str,
        query_vector: &[f32],
        limit: usize,
        filter: Option<&str>,
    ) -> Result<Vec<SearchResult>>;
    async fn delete_vectors(&self, collection: &str, ids: &[String]) -> Result<()>;
    async fn get_vectors_by_ids(
        &self,
        collection: &str,
        ids: &[String],
    ) -> Result<Vec<SearchResult>>;
    async fn list_vectors(&self, collection: &str, limit: usize) -> Result<Vec<SearchResult>>;
    async fn get_stats(&self, collection: &str) -> Result<HashMap<String, Value>>;
    async fn flush(&self, collection: &str) -> Result<()>;
    fn provider_name(&self) -> &str;

    /// Health check for the provider (default implementation)
    async fn health_check(&self) -> Result<()> {
        // Default implementation - try a simple operation
        self.collection_exists("__health_check__").await?;
        Ok(())
    }
}
