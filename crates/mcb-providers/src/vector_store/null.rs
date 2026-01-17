//! Null vector store provider for testing
//!
//! Provides a stub implementation of VectorStoreProvider that performs
//! no actual storage operations. Useful for unit testing and dependency injection defaults.

use async_trait::async_trait;
use dashmap::DashMap;
use mcb_application::ports::providers::{VectorStoreAdmin, VectorStoreProvider};
use mcb_domain::error::{Error, Result};
use mcb_domain::value_objects::{Embedding, SearchResult};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// Null storage entry type
type CollectionEntry = (Embedding, HashMap<String, Value>);

/// Null vector store provider for testing
///
/// This provider implements the VectorStoreProvider trait with no-op operations.
/// Collections are tracked in memory but vectors are not actually stored.
/// Useful for:
/// - Unit testing where vector storage is not needed
/// - Default DI binding when no real provider is configured
/// - Performance testing of non-vector-store code paths
#[derive(shaku::Component)]
#[shaku(interface = VectorStoreProvider)]
pub struct NullVectorStoreProvider {
    /// In-memory collection storage for testing purposes
    #[shaku(default = Arc::new(DashMap::new()))]
    collections: Arc<DashMap<String, Vec<CollectionEntry>>>,
}

impl NullVectorStoreProvider {
    /// Create a new null vector store provider
    pub fn new() -> Self {
        Self {
            collections: Arc::new(DashMap::new()),
        }
    }
}

impl Default for NullVectorStoreProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl VectorStoreAdmin for NullVectorStoreProvider {
    async fn collection_exists(&self, name: &str) -> Result<bool> {
        Ok(self.collections.contains_key(name))
    }

    async fn get_stats(&self, collection: &str) -> Result<HashMap<String, Value>> {
        let mut stats = HashMap::new();
        stats.insert("collection".to_string(), serde_json::json!(collection));
        stats.insert("status".to_string(), serde_json::json!("active"));
        stats.insert("vectors_count".to_string(), serde_json::json!(0));
        stats.insert(
            "provider".to_string(),
            serde_json::json!(self.provider_name()),
        );
        Ok(stats)
    }

    async fn flush(&self, _collection: &str) -> Result<()> {
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "null"
    }
}

#[async_trait]
impl VectorStoreProvider for NullVectorStoreProvider {
    async fn create_collection(&self, name: &str, _dimensions: usize) -> Result<()> {
        if self.collections.contains_key(name) {
            return Err(Error::vector_db(format!(
                "Collection '{}' already exists",
                name
            )));
        }
        self.collections.insert(name.to_string(), Vec::new());
        Ok(())
    }

    async fn delete_collection(&self, name: &str) -> Result<()> {
        self.collections.remove(name);
        Ok(())
    }

    async fn insert_vectors(
        &self,
        _collection: &str,
        vectors: &[Embedding],
        _metadata: Vec<HashMap<String, Value>>,
    ) -> Result<Vec<String>> {
        // Null provider always succeeds but returns empty IDs
        Ok(vec!["".to_string(); vectors.len()])
    }

    async fn search_similar(
        &self,
        _collection: &str,
        _query_vector: &[f32],
        _limit: usize,
        _filter: Option<&str>,
    ) -> Result<Vec<SearchResult>> {
        // Null provider always returns empty results
        Ok(Vec::new())
    }

    async fn delete_vectors(&self, _collection: &str, _ids: &[String]) -> Result<()> {
        Ok(())
    }

    async fn get_vectors_by_ids(
        &self,
        _collection: &str,
        _ids: &[String],
    ) -> Result<Vec<SearchResult>> {
        Ok(Vec::new())
    }

    async fn list_vectors(&self, _collection: &str, _limit: usize) -> Result<Vec<SearchResult>> {
        Ok(Vec::new())
    }
}
