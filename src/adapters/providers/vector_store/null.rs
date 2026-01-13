//! Null vector store provider for testing

use crate::domain::error::{Error, Result};
use crate::domain::ports::VectorStoreProvider;
use crate::domain::types::{Embedding, SearchResult};
use async_trait::async_trait;
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;

/// Null storage entry type
type CollectionEntry = (Embedding, HashMap<String, serde_json::Value>);

/// Null vector store provider for testing
#[derive(shaku::Component)]
#[shaku(interface = VectorStoreProvider)]
pub struct NullVectorStoreProvider {
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

    async fn collection_exists(&self, name: &str) -> Result<bool> {
        Ok(self.collections.contains_key(name))
    }

    async fn insert_vectors(
        &self,
        _collection: &str,
        vectors: &[Embedding],
        _metadata: Vec<std::collections::HashMap<String, serde_json::Value>>,
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

    async fn get_stats(
        &self,
        collection: &str,
    ) -> Result<std::collections::HashMap<String, serde_json::Value>> {
        let mut stats = std::collections::HashMap::new();
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
