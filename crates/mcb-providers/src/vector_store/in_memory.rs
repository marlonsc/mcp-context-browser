//! In-memory vector store provider implementation
//!
//! Provides an in-memory vector storage backend for development and testing.
//! Data is not persisted and will be lost on restart.

use crate::utils::JsonExt;
use async_trait::async_trait;
use dashmap::DashMap;
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::providers::{VectorStoreAdmin, VectorStoreBrowser, VectorStoreProvider};
use mcb_domain::value_objects::{CollectionInfo, Embedding, FileInfo, SearchResult};
use serde_json::Value;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::sync::Arc;

/// In-memory storage entry type
type CollectionEntry = (Embedding, HashMap<String, Value>);

/// In-memory vector store provider
///
/// Stores vectors and metadata in memory using concurrent hash maps.
/// Useful for development and testing where persistence is not required.
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
impl VectorStoreAdmin for InMemoryVectorStoreProvider {
    async fn collection_exists(&self, name: &str) -> Result<bool> {
        Ok(self.collections.contains_key(name))
    }

    async fn get_stats(&self, collection: &str) -> Result<HashMap<String, Value>> {
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

    async fn insert_vectors(
        &self,
        collection: &str,
        vectors: &[Embedding],
        metadata: Vec<HashMap<String, Value>>,
    ) -> Result<Vec<String>> {
        let mut coll = self
            .collections
            .get_mut(collection)
            .ok_or_else(|| Error::vector_db(format!("Collection '{}' not found", collection)))?;

        let mut ids = Vec::with_capacity(vectors.len());
        for (vector, mut meta) in vectors.iter().zip(metadata) {
            let id = format!("{}_{}", collection, coll.len());
            // Store the generated ID in metadata for deletion
            meta.insert("generated_id".to_string(), serde_json::json!(&id));
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
        // Return empty results for non-existent collections (graceful degradation)
        let coll = match self.collections.get(collection) {
            Some(coll) => coll,
            None => return Ok(Vec::new()),
        };

        // Precompute query norm once (avoids redundant calculation per vector)
        let query_norm = compute_norm(query_vector);

        // Use min-heap for top-k selection: O(n log k) instead of O(n log n)
        let mut heap: BinaryHeap<ScoredItem> = BinaryHeap::with_capacity(limit + 1);

        for (i, (embedding, _metadata)) in coll.iter().enumerate() {
            let similarity =
                cosine_similarity_with_norm(query_vector, &embedding.vector, query_norm);

            if heap.len() < limit {
                heap.push(ScoredItem {
                    score: similarity,
                    index: i,
                });
            } else if let Some(min) = heap.peek() {
                // Only add if better than current minimum
                if similarity > min.score {
                    heap.pop();
                    heap.push(ScoredItem {
                        score: similarity,
                        index: i,
                    });
                }
            }
        }

        // Extract results in descending score order
        let mut items: Vec<_> = heap.into_iter().collect();
        items.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(Ordering::Equal));

        let search_results = items
            .into_iter()
            .map(|item| {
                let (_embedding, metadata) = &coll[item.index];
                metadata_to_search_result(metadata, item.score as f64)
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
            .map(|(_embedding, metadata)| metadata_to_search_result(metadata, 1.0))
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
            .map(|(_embedding, metadata)| metadata_to_search_result(metadata, 1.0))
            .collect();

        Ok(results)
    }
}

#[async_trait]
impl VectorStoreBrowser for InMemoryVectorStoreProvider {
    async fn list_collections(&self) -> Result<Vec<CollectionInfo>> {
        let collections: Vec<CollectionInfo> = self
            .collections
            .iter()
            .map(|entry| {
                let name = entry.key().clone();
                let data = entry.value();
                let vector_count = data.len() as u64;

                // Count unique file paths
                let file_paths: HashSet<&str> = data
                    .iter()
                    .filter_map(|(_, metadata)| metadata.get("file_path").and_then(|v| v.as_str()))
                    .collect();
                let file_count = file_paths.len() as u64;

                CollectionInfo::new(name, vector_count, file_count, None, self.provider_name())
            })
            .collect();
        Ok(collections)
    }

    async fn list_file_paths(&self, collection: &str, limit: usize) -> Result<Vec<FileInfo>> {
        let coll = self
            .collections
            .get(collection)
            .ok_or_else(|| Error::vector_db(format!("Collection '{}' not found", collection)))?;

        // Aggregate file info from chunks
        let mut file_map: HashMap<String, (u32, String)> = HashMap::new();

        for (_embedding, metadata) in coll.iter() {
            if let Some(file_path) = metadata.get("file_path").and_then(|v| v.as_str()) {
                let language = metadata
                    .get("language")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();

                let entry = file_map
                    .entry(file_path.to_string())
                    .or_insert((0, language));
                entry.0 += 1; // Increment chunk count
            }
        }

        let files: Vec<FileInfo> = file_map
            .into_iter()
            .take(limit)
            .map(|(path, (chunk_count, language))| FileInfo::new(path, chunk_count, language, None))
            .collect();

        Ok(files)
    }

    async fn get_chunks_by_file(
        &self,
        collection: &str,
        file_path: &str,
    ) -> Result<Vec<SearchResult>> {
        let coll = self
            .collections
            .get(collection)
            .ok_or_else(|| Error::vector_db(format!("Collection '{}' not found", collection)))?;

        let mut results: Vec<SearchResult> = coll
            .iter()
            .filter(|(_embedding, metadata)| {
                metadata
                    .get("file_path")
                    .and_then(|v| v.as_str())
                    .is_some_and(|p| p == file_path)
            })
            .map(|(_embedding, metadata)| metadata_to_search_result(metadata, 1.0))
            .collect();

        // Sort by start_line for logical ordering
        results.sort_by_key(|r| r.start_line);

        Ok(results)
    }
}

/// Scored item for heap-based top-k selection
///
/// Uses reverse ordering so BinaryHeap acts as a min-heap (smallest scores at top).
#[derive(PartialEq)]
struct ScoredItem {
    score: f32,
    index: usize,
}

impl Eq for ScoredItem {}

impl Ord for ScoredItem {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for min-heap behavior: smallest at top
        other
            .score
            .partial_cmp(&self.score)
            .unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for ScoredItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Compute the L2 norm of a vector
fn compute_norm(v: &[f32]) -> f32 {
    v.iter().map(|x| x * x).sum::<f32>().sqrt()
}

/// Convert metadata to SearchResult
fn metadata_to_search_result(metadata: &HashMap<String, Value>, score: f64) -> SearchResult {
    let id = metadata.string_or("generated_id", "");
    let start_line = metadata
        .opt_u64("start_line")
        .or_else(|| metadata.opt_u64("line_number"))
        .unwrap_or(0) as u32;
    let language = metadata.string_or("language", "unknown");

    SearchResult {
        id,
        file_path: metadata.string_or("file_path", ""),
        start_line,
        content: metadata.string_or("content", ""),
        score,
        language,
    }
}

/// Cosine similarity with precomputed query norm (optimized version)
fn cosine_similarity_with_norm(a: &[f32], b: &[f32], norm_a: f32) -> f32 {
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        // Normalize to [0, 1] range
        (dot_product / (norm_a * norm_b) + 1.0) / 2.0
    }
}

// ============================================================================
// Auto-registration via linkme distributed slice
// ============================================================================

use mcb_application::ports::registry::{
    VECTOR_STORE_PROVIDERS, VectorStoreProviderConfig, VectorStoreProviderEntry,
};

/// Factory function for creating in-memory vector store provider instances.
fn in_memory_vector_store_factory(
    _config: &VectorStoreProviderConfig,
) -> std::result::Result<Arc<dyn VectorStoreProvider>, String> {
    Ok(Arc::new(InMemoryVectorStoreProvider::new()))
}

#[linkme::distributed_slice(VECTOR_STORE_PROVIDERS)]
static MEMORY_PROVIDER: VectorStoreProviderEntry = VectorStoreProviderEntry {
    name: "memory",
    description: "In-memory vector store (fast, non-persistent)",
    factory: in_memory_vector_store_factory,
};
