//! Null vector store provider for testing

use crate::core::error::{Error, Result};
use crate::core::types::Embedding;
use crate::providers::VectorStoreProvider;
use crate::core::locks::lock_mutex;
use async_trait::async_trait;
use std::collections::HashMap;

/// Null vector store provider for testing
#[allow(clippy::type_complexity)]
pub struct NullVectorStoreProvider {
    collections:
        std::sync::Mutex<HashMap<String, Vec<(Embedding, HashMap<String, serde_json::Value>)>>>,
}

#[allow(dead_code, clippy::needless_borrows_for_generic_args)]
impl NullVectorStoreProvider {
    /// Create a new null vector store provider
    pub fn new() -> Self {
        Self {
            collections: std::sync::Mutex::new(HashMap::new()),
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
        let mut collections = lock_mutex(&self.collections, "NullVectorStoreProvider::create_collection")?;
        if collections.contains_key(name) {
            return Err(Error::vector_db(format!(
                "Collection '{}' already exists",
                name
            )));
        }
        collections.insert(name.to_string(), Vec::new());
        Ok(())
    }

    async fn delete_collection(&self, name: &str) -> Result<()> {
        let mut collections = lock_mutex(&self.collections, "NullVectorStoreProvider::delete_collection")?;
        collections.remove(name);
        Ok(())
    }

    async fn collection_exists(&self, name: &str) -> Result<bool> {
        let collections = lock_mutex(&self.collections, "NullVectorStoreProvider::collection_exists")?;
        Ok(collections.contains_key(name))
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
    ) -> Result<Vec<crate::core::types::SearchResult>> {
        // Null provider always returns empty results
        Ok(Vec::new())
    }

    async fn delete_vectors(&self, _collection: &str, _ids: &[String]) -> Result<()> {
        Ok(())
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
