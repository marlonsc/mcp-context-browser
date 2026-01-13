//! In-memory vector store provider implementation

use crate::domain::error::{Error, Result};
use crate::domain::ports::VectorStoreProvider;
use crate::domain::types::{Embedding, SearchResult};
use crate::infrastructure::utils::JsonExt;
use async_trait::async_trait;
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;

/// In-memory storage entry type
type CollectionEntry = (Embedding, HashMap<String, serde_json::Value>);

/// In-memory vector store provider
pub struct InMemoryVectorStoreProvider {
    collections: Arc<DashMap<String, Vec<CollectionEntry>>>,
}

impl InMemoryVectorStoreProvider {
    /// Create a new in-memory vector store provider
    pub fn new() -> Self {
        Self {
            collections: Arc::new(DashMap::new()),
        }
    }
}

impl Default for InMemoryVectorStoreProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl VectorStoreProvider for InMemoryVectorStoreProvider {
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
        collection: &str,
        vectors: &[Embedding],
        metadata: Vec<HashMap<String, serde_json::Value>>,
    ) -> Result<Vec<String>> {
        let mut coll = self
            .collections
            .get_mut(collection)
            .ok_or_else(|| Error::vector_db(format!("Collection '{}' not found", collection)))?;

        let mut ids = Vec::new();
        for (vector, mut meta) in vectors.iter().zip(metadata) {
            let id = format!("{}_{}", collection, coll.len());
            // Store the generated ID in metadata for deletion
            meta.insert("generated_id".to_string(), serde_json::json!(id.clone()));
            coll.push((vector.clone(), meta));
            ids.push(id);
        }

        Ok(ids)
    }

    async fn search_similar(
        &self,
        collection: &str,
        query_vector: &[f32],
        limit: usize,
        _filter: Option<&str>,
    ) -> Result<Vec<SearchResult>> {
        let coll = self
            .collections
            .get(collection)
            .ok_or_else(|| Error::vector_db(format!("Collection '{}' not found", collection)))?;

        // Simple cosine similarity search
        let mut results: Vec<_> = coll
            .iter()
            .enumerate()
            .map(|(i, (embedding, metadata))| {
                let similarity = cosine_similarity(query_vector, &embedding.vector);
                (similarity, i, embedding, metadata)
            })
            .collect();

        results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);

        let search_results = results
            .into_iter()
            .map(|(score, _i, _embedding, metadata)| {
                let id = metadata.string_or("generated_id", "");
                // Fallback for backward compatibility
                let start_line = metadata
                    .opt_u64("start_line")
                    .or_else(|| metadata.opt_u64("line_number"))
                    .unwrap_or(0) as u32;

                SearchResult {
                    id,
                    file_path: metadata.string_or("file_path", ""),
                    start_line,
                    content: metadata.string_or("content", ""),
                    score,
                    metadata: serde_json::to_value(metadata).unwrap_or(serde_json::json!({})),
                }
            })
            .collect();

        Ok(search_results)
    }

    async fn delete_vectors(&self, collection: &str, ids: &[String]) -> Result<()> {
        let mut coll = self
            .collections
            .get_mut(collection)
            .ok_or_else(|| Error::vector_db(format!("Collection '{}' not found", collection)))?;

        // Remove vectors by their generated IDs
        coll.retain(|(_embedding, metadata)| {
            let generated_id = metadata.str_or("generated_id", "");
            !ids.contains(&generated_id.to_string())
        });
        Ok(())
    }

    async fn get_vectors_by_ids(
        &self,
        collection: &str,
        ids: &[String],
    ) -> Result<Vec<SearchResult>> {
        let coll = self
            .collections
            .get(collection)
            .ok_or_else(|| Error::vector_db(format!("Collection '{}' not found", collection)))?;

        let results = coll
            .iter()
            .filter(|(_embedding, metadata)| {
                let generated_id = metadata.str_or("generated_id", "");
                ids.contains(&generated_id.to_string())
            })
            .map(|(_embedding, metadata)| {
                let id = metadata.string_or("generated_id", "");
                // Fallback for backward compatibility
                let start_line = metadata
                    .opt_u64("start_line")
                    .or_else(|| metadata.opt_u64("line_number"))
                    .unwrap_or(0) as u32;

                SearchResult {
                    id,
                    file_path: metadata.string_or("file_path", ""),
                    start_line,
                    content: metadata.string_or("content", ""),
                    score: 1.0, // Exact match
                    metadata: serde_json::to_value(metadata).unwrap_or(serde_json::json!({})),
                }
            })
            .collect();

        Ok(results)
    }

    async fn list_vectors(&self, collection: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let coll = self
            .collections
            .get(collection)
            .ok_or_else(|| Error::vector_db(format!("Collection '{}' not found", collection)))?;

        let results = coll
            .iter()
            .take(limit)
            .map(|(_embedding, metadata)| {
                let id = metadata.string_or("generated_id", "");
                // Fallback for backward compatibility
                let start_line = metadata
                    .opt_u64("start_line")
                    .or_else(|| metadata.opt_u64("line_number"))
                    .unwrap_or(0) as u32;

                SearchResult {
                    id,
                    file_path: metadata.string_or("file_path", ""),
                    start_line,
                    content: metadata.string_or("content", ""),
                    score: 1.0,
                    metadata: serde_json::to_value(metadata).unwrap_or(serde_json::json!({})),
                }
            })
            .collect();

        Ok(results)
    }

    async fn get_stats(&self, collection: &str) -> Result<HashMap<String, serde_json::Value>> {
        let count = self
            .collections
            .get(collection)
            .map(|data| data.len())
            .unwrap_or(0);

        let mut stats = HashMap::new();
        stats.insert("collection".to_string(), serde_json::json!(collection));
        stats.insert("status".to_string(), serde_json::json!("active"));
        stats.insert("vectors_count".to_string(), serde_json::json!(count));
        stats.insert(
            "provider".to_string(),
            serde_json::json!(self.provider_name()),
        );
        Ok(stats)
    }

    async fn flush(&self, _collection: &str) -> Result<()> {
        // No-op for in-memory store
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "in_memory"
    }
}

/// Cosine similarity calculation
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        // Normalize to [0, 1] range
        (dot_product / (norm_a * norm_b) + 1.0) / 2.0
    }
}
