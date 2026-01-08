//! EdgeVec Vector Store Provider
//!
//! High-performance embedded vector database implementation using EdgeVec.
//! EdgeVec provides sub-millisecond vector similarity search with HNSW algorithm.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::core::error::{Error, Result};
use crate::core::types::{Embedding, SearchResult};
use crate::services::VectorStoreProvider;
use edgevec::hnsw::VectorId;

/// EdgeVec vector store configuration
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, schemars::JsonSchema)]
pub struct EdgeVecConfig {
    /// Vector dimensionality
    #[serde(default = "default_dimensions")]
    pub dimensions: usize,

    /// HNSW parameters for index optimization
    #[serde(default)]
    pub hnsw_config: HnswConfig,

    /// Distance metric to use
    #[serde(default)]
    pub metric: MetricType,

    /// Whether to use quantization for memory optimization
    #[serde(default)]
    pub use_quantization: bool,

    /// Quantization configuration
    #[serde(default)]
    pub quantizer_config: QuantizerConfig,
}

fn default_dimensions() -> usize {
    1536 // Default for OpenAI embeddings
}

/// HNSW configuration for EdgeVec
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, schemars::JsonSchema)]
pub struct HnswConfig {
    /// Maximum connections per node in layers > 0
    #[serde(default = "default_m")]
    pub m: u32,

    /// Maximum connections per node in layer 0
    #[serde(default = "default_m0")]
    pub m0: u32,

    /// Construction-time candidate list size
    #[serde(default = "default_ef_construction")]
    pub ef_construction: u32,

    /// Search-time candidate list size
    #[serde(default = "default_ef_search")]
    pub ef_search: u32,
}

fn default_m() -> u32 {
    16
}
fn default_m0() -> u32 {
    32
}
fn default_ef_construction() -> u32 {
    200
}
fn default_ef_search() -> u32 {
    64
}

impl Default for HnswConfig {
    fn default() -> Self {
        Self {
            m: default_m(),
            m0: default_m0(),
            ef_construction: default_ef_construction(),
            ef_search: default_ef_search(),
        }
    }
}

/// Distance metrics supported by EdgeVec
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, schemars::JsonSchema, Default)]
pub enum MetricType {
    /// L2 Squared (Euclidean) distance
    L2Squared,
    /// Cosine similarity
    #[default]
    Cosine,
    /// Dot product
    DotProduct,
}

/// Quantization configuration for memory optimization
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, schemars::JsonSchema)]
pub struct QuantizerConfig {
    /// Quantization type - only ScalarQuantization is available in v0.6.0
    #[serde(default)]
    pub quantization_type: String, // Will be "scalar" for now
}

impl Default for QuantizerConfig {
    fn default() -> Self {
        Self {
            quantization_type: "scalar".to_string(),
        }
    }
}

impl Default for EdgeVecConfig {
    fn default() -> Self {
        Self {
            dimensions: default_dimensions(),
            hnsw_config: HnswConfig::default(),
            metric: MetricType::default(),
            use_quantization: false,
            quantizer_config: QuantizerConfig::default(),
        }
    }
}

/// EdgeVec vector store provider implementation
pub struct EdgeVecVectorStoreProvider {
    /// HNSW index for vector search
    index: Arc<RwLock<edgevec::HnswIndex>>,
    /// Vector storage backend
    storage: Arc<RwLock<edgevec::VectorStorage>>,
    /// Configuration
    config: EdgeVecConfig,
    /// Collection name
    collection: String,
    /// Metadata storage (EdgeVec doesn't support metadata natively)
    metadata_store: Arc<RwLock<HashMap<String, HashMap<String, serde_json::Value>>>>,
    /// ID mapping for external IDs to internal vector IDs
    id_map: Arc<RwLock<HashMap<String, VectorId>>>,
}

impl EdgeVecVectorStoreProvider {
    /// Create a new EdgeVec vector store provider
    pub fn new(config: EdgeVecConfig) -> Result<Self> {
        // Create HNSW configuration
        let hnsw_config = edgevec::HnswConfig {
            m: config.hnsw_config.m,
            m0: config.hnsw_config.m0,
            ef_construction: config.hnsw_config.ef_construction,
            ef_search: config.hnsw_config.ef_search,
            dimensions: config.dimensions as u32,
            metric: match config.metric {
                MetricType::L2Squared => edgevec::HnswConfig::METRIC_L2_SQUARED,
                MetricType::Cosine => edgevec::HnswConfig::METRIC_COSINE,
                MetricType::DotProduct => edgevec::HnswConfig::METRIC_DOT_PRODUCT,
            },
            _reserved: [0; 2],
        };

        // Create vector storage
        let storage = edgevec::VectorStorage::new(&hnsw_config, None);

        // Create HNSW index
        let index = edgevec::HnswIndex::new(hnsw_config, &storage)
            .map_err(|e| Error::internal(format!("Failed to create EdgeVec HNSW index: {}", e)))?;

        Ok(Self {
            index: Arc::new(RwLock::new(index)),
            storage: Arc::new(RwLock::new(storage)),
            config,
            collection: "default".to_string(),
            metadata_store: Arc::new(RwLock::new(HashMap::new())),
            id_map: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Create a new EdgeVec provider with custom collection
    pub fn with_collection(config: EdgeVecConfig, collection: String) -> Result<Self> {
        let mut provider = Self::new(config)?;
        provider.collection = collection;
        Ok(provider)
    }
}

#[async_trait]
impl VectorStoreProvider for EdgeVecVectorStoreProvider {
    async fn create_collection(&self, name: &str, _dimensions: usize) -> Result<()> {
        let mut metadata_store = self.metadata_store.write().await;
        metadata_store
            .entry(name.to_string())
            .or_insert_with(HashMap::new);
        Ok(())
    }

    async fn delete_collection(&self, name: &str) -> Result<()> {
        let mut index = self.index.write().await;
        let mut metadata_store = self.metadata_store.write().await;
        let mut id_map = self.id_map.write().await;

        // Remove all vectors for this collection
        if let Some(collection_metadata) = metadata_store.get(name) {
            for external_id in collection_metadata.keys() {
                if let Some(&vector_id) = id_map.get(external_id)
                    && let Err(e) = index.soft_delete(vector_id)
                {
                    tracing::warn!("Failed to remove vector {}: {}", external_id, e);
                }
                id_map.remove(external_id);
            }
        }

        // Remove collection metadata
        metadata_store.remove(name);
        Ok(())
    }

    async fn collection_exists(&self, name: &str) -> Result<bool> {
        let metadata_store = self.metadata_store.read().await;
        Ok(metadata_store.contains_key(name))
    }

    async fn insert_vectors(
        &self,
        collection: &str,
        vectors: &[Embedding],
        metadata: Vec<HashMap<String, serde_json::Value>>,
    ) -> Result<Vec<String>> {
        if vectors.len() != metadata.len() {
            return Err(Error::invalid_argument(
                "Number of vectors must match number of metadata entries",
            ));
        }

        let mut ids = Vec::with_capacity(vectors.len());
        let mut index = self.index.write().await;
        let mut storage = self.storage.write().await;
        let mut metadata_store = self.metadata_store.write().await;
        let mut id_map = self.id_map.write().await;

        // Ensure collection exists in metadata store
        let collection_metadata = metadata_store
            .entry(collection.to_string())
            .or_insert_with(HashMap::new);

        for (vector, meta) in vectors.iter().zip(metadata.iter()) {
            // Generate unique external ID
            let external_id = format!("{}_{}", collection, uuid::Uuid::new_v4());

            // Insert vector into EdgeVec storage and index
            let vector_id = index
                .insert(&vector.vector, &mut storage)
                .map_err(|e| Error::internal(format!("Failed to insert vector: {}", e)))?;

            // Map external ID to internal vector ID
            id_map.insert(external_id.clone(), vector_id);

            // Store metadata - for vector stores, we need to map to SearchResult fields
            let mut enriched_metadata = meta.clone();
            // Ensure we have the required fields for SearchResult
            enriched_metadata
                .entry("id".to_string())
                .or_insert(serde_json::json!(external_id));
            collection_metadata.insert(
                external_id.clone(),
                serde_json::Value::Object(serde_json::Map::from_iter(
                    enriched_metadata.into_iter(),
                )),
            );
            ids.push(external_id);
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
        let index = self.index.read().await;
        let storage = self.storage.read().await;
        let metadata_store = self.metadata_store.read().await;
        let id_map = self.id_map.read().await;

        // Get collection metadata
        let collection_metadata = metadata_store
            .get(collection)
            .ok_or_else(|| Error::not_found(format!("Collection '{}' not found", collection)))?;

        // Perform similarity search
        let results = index
            .search(query_vector, limit, &storage)
            .map_err(|e| Error::internal(format!("Search failed: {}", e)))?;

        let mut final_results = Vec::with_capacity(results.len());

        for result in results {
            // Find external ID for this vector ID
            let external_id = id_map
                .iter()
                .find_map(|(ext_id, &vec_id)| {
                    if vec_id == result.vector_id {
                        Some(ext_id)
                    } else {
                        None
                    }
                })
                .ok_or_else(|| {
                    Error::internal(format!(
                        "Vector ID {:?} not found in mapping",
                        result.vector_id
                    ))
                })?;

            // Get metadata for this vector
            let (file_path, line_number, content, metadata) = if let Some(
                serde_json::Value::Object(obj),
            ) =
                collection_metadata.get(external_id)
            {
                let map = obj.clone().into_iter().collect::<HashMap<_, _>>();
                (
                    map.get("file_path")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string(),
                    map.get("line_number").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                    map.get("content")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    serde_json::to_value(&map).unwrap_or_default(),
                )
            } else {
                (
                    "unknown".to_string(),
                    0,
                    "".to_string(),
                    serde_json::Value::Null,
                )
            };

            final_results.push(SearchResult {
                file_path,
                line_number,
                content,
                score: result.distance,
                metadata,
            });
        }

        Ok(final_results)
    }

    async fn delete_vectors(&self, collection: &str, ids: &[String]) -> Result<()> {
        let mut index = self.index.write().await;
        let mut metadata_store = self.metadata_store.write().await;
        let mut id_map = self.id_map.write().await;

        if let Some(collection_metadata) = metadata_store.get_mut(collection) {
            for id in ids {
                // Get internal vector ID
                if let Some(&vector_id) = id_map.get(id) {
                    // Remove from EdgeVec (this will mark as deleted in the index)
                    if let Err(e) = index.soft_delete(vector_id) {
                        tracing::warn!("Failed to remove vector {}: {}", id, e);
                    }
                }

                // Remove from our mappings
                id_map.remove(id);
                collection_metadata.remove(id);
            }
        }

        Ok(())
    }

    async fn get_stats(&self, collection: &str) -> Result<HashMap<String, serde_json::Value>> {
        let metadata_store = self.metadata_store.read().await;
        let index = self.index.read().await;

        let vector_count = if let Some(collection_metadata) = metadata_store.get(collection) {
            collection_metadata.len()
        } else {
            0
        };

        let mut stats = HashMap::new();
        stats.insert("collection".to_string(), serde_json::json!(collection));
        stats.insert("vector_count".to_string(), serde_json::json!(vector_count));
        stats.insert(
            "total_indexed_vectors".to_string(),
            serde_json::json!(index.len()),
        );
        stats.insert(
            "dimensions".to_string(),
            serde_json::json!(self.config.dimensions),
        );
        stats.insert("metric".to_string(), serde_json::json!(self.config.metric));
        stats.insert(
            "use_quantization".to_string(),
            serde_json::json!(self.config.use_quantization),
        );

        Ok(stats)
    }

    async fn flush(&self, _collection: &str) -> Result<()> {
        // EdgeVec handles persistence automatically via WAL
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "edgevec"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_edgevec_provider_creation() {
        let config = EdgeVecConfig::default();
        let provider = EdgeVecVectorStoreProvider::new(config);
        assert!(provider.is_ok());
    }

    #[tokio::test]
    async fn test_edgevec_collection_management() {
        let config = EdgeVecConfig::default();
        let provider = EdgeVecVectorStoreProvider::new(config).unwrap();

        // Test collection creation
        assert!(
            provider
                .create_collection("test_collection", 1536)
                .await
                .is_ok()
        );

        // Test collection exists
        assert!(provider.collection_exists("test_collection").await.unwrap());

        // Test collection stats
        let stats = provider.get_stats("test_collection").await.unwrap();
        assert_eq!(stats.get("collection").unwrap(), "test_collection");
        assert_eq!(stats.get("vector_count").unwrap(), &serde_json::json!(0));
    }

    #[tokio::test]
    async fn test_edgevec_vector_operations() {
        let config = EdgeVecConfig {
            dimensions: 3,
            hnsw_config: HnswConfig::default(),
            metric: MetricType::Cosine,
            use_quantization: false,
            quantizer_config: Default::default(),
        };

        let provider = EdgeVecVectorStoreProvider::new(config).unwrap();

        // Create collection
        provider.create_collection("test", 3).await.unwrap();

        // Test vector storage
        let vectors = vec![
            Embedding {
                vector: vec![1.0, 0.0, 0.0],
                dimensions: 3,
                model: "test".to_string(),
            },
            Embedding {
                vector: vec![0.0, 1.0, 0.0],
                dimensions: 3,
                model: "test".to_string(),
            },
            Embedding {
                vector: vec![0.0, 0.0, 1.0],
                dimensions: 3,
                model: "test".to_string(),
            },
        ];

        let metadata = vec![
            HashMap::from([
                ("file_path".to_string(), serde_json::json!("test.rs")),
                ("line_number".to_string(), serde_json::json!(1)),
                ("content".to_string(), serde_json::json!("vec1")),
                ("type".to_string(), serde_json::json!("vec1")),
            ]),
            HashMap::from([
                ("file_path".to_string(), serde_json::json!("test.rs")),
                ("line_number".to_string(), serde_json::json!(2)),
                ("content".to_string(), serde_json::json!("vec2")),
                ("type".to_string(), serde_json::json!("vec2")),
            ]),
            HashMap::from([
                ("file_path".to_string(), serde_json::json!("test.rs")),
                ("line_number".to_string(), serde_json::json!(3)),
                ("content".to_string(), serde_json::json!("vec3")),
                ("type".to_string(), serde_json::json!("vec3")),
            ]),
        ];

        let ids = provider
            .insert_vectors("test", &vectors, metadata)
            .await
            .unwrap();
        assert_eq!(ids.len(), 3);

        // Test similarity search
        let query = vec![1.0, 0.1, 0.1];
        let results = provider
            .search_similar("test", &query, 2, None)
            .await
            .unwrap();
        assert_eq!(results.len(), 2);

        // Test vector deletion
        provider.delete_vectors("test", &ids[..1]).await.unwrap();

        // Verify deletion
        let stats = provider.get_stats("test").await.unwrap();
        assert_eq!(stats.get("vector_count").unwrap(), &serde_json::json!(2));
    }
}
