//! EdgeVec Vector Store Provider
//!
//! High-performance embedded vector database implementation using EdgeVec.
//! EdgeVec provides sub-millisecond vector similarity search with HNSW algorithm.
//! This implementation uses the Actor pattern to eliminate locks and ensure non-blocking operation.

use async_trait::async_trait;
use dashmap::DashMap;
use std::collections::HashMap;
use tokio::sync::{mpsc, oneshot};

use crate::constants::{
    EDGEVEC_DEFAULT_DIMENSIONS, EDGEVEC_HNSW_EF_CONSTRUCTION, EDGEVEC_HNSW_EF_SEARCH,
    EDGEVEC_HNSW_M, EDGEVEC_HNSW_M0,
};
use crate::utils::JsonExt;
use edgevec::hnsw::VectorId;
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::providers::{VectorStoreAdmin, VectorStoreBrowser, VectorStoreProvider};
use mcb_domain::value_objects::{CollectionInfo, Embedding, FileInfo, SearchResult};

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
    EDGEVEC_DEFAULT_DIMENSIONS
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
    EDGEVEC_HNSW_M
}
fn default_m0() -> u32 {
    EDGEVEC_HNSW_M0
}
fn default_ef_construction() -> u32 {
    EDGEVEC_HNSW_EF_CONSTRUCTION
}
fn default_ef_search() -> u32 {
    EDGEVEC_HNSW_EF_SEARCH
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
    pub quantization_type: String,
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

/// Messages for the EdgeVec actor
enum EdgeVecMessage {
    CreateCollection {
        name: String,
        tx: oneshot::Sender<Result<()>>,
    },
    DeleteCollection {
        name: String,
        tx: oneshot::Sender<Result<()>>,
    },
    InsertVectors {
        collection: String,
        vectors: Vec<Embedding>,
        metadata: Vec<HashMap<String, serde_json::Value>>,
        tx: oneshot::Sender<Result<Vec<String>>>,
    },
    SearchSimilar {
        collection: String,
        query_vector: Vec<f32>,
        limit: usize,
        tx: oneshot::Sender<Result<Vec<SearchResult>>>,
    },
    DeleteVectors {
        collection: String,
        ids: Vec<String>,
        tx: oneshot::Sender<Result<()>>,
    },
    GetStats {
        collection: String,
        tx: oneshot::Sender<Result<HashMap<String, serde_json::Value>>>,
    },
    ListVectors {
        collection: String,
        limit: usize,
        tx: oneshot::Sender<Result<Vec<SearchResult>>>,
    },
    GetVectorsByIds {
        collection: String,
        ids: Vec<String>,
        tx: oneshot::Sender<Result<Vec<SearchResult>>>,
    },
    CollectionExists {
        name: String,
        tx: oneshot::Sender<Result<bool>>,
    },
    // Browse operations
    ListCollections {
        tx: oneshot::Sender<Result<Vec<CollectionInfo>>>,
    },
    ListFilePaths {
        collection: String,
        limit: usize,
        tx: oneshot::Sender<Result<Vec<FileInfo>>>,
    },
    GetChunksByFile {
        collection: String,
        file_path: String,
        tx: oneshot::Sender<Result<Vec<SearchResult>>>,
    },
}

/// EdgeVec vector store provider implementation using Actor pattern
pub struct EdgeVecVectorStoreProvider {
    sender: mpsc::Sender<EdgeVecMessage>,
    #[allow(dead_code)]
    collection: String,
}

impl EdgeVecVectorStoreProvider {
    /// Create a new EdgeVec vector store provider
    pub fn new(config: EdgeVecConfig) -> Result<Self> {
        let (tx, rx) = mpsc::channel(100);
        let config_clone = config.clone();

        let actor = EdgeVecActor::new(rx, config_clone)?;
        tokio::spawn(async move {
            actor.run().await;
        });

        Ok(Self {
            sender: tx,
            collection: "default".to_string(),
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
impl VectorStoreAdmin for EdgeVecVectorStoreProvider {
    async fn collection_exists(&self, name: &str) -> Result<bool> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .sender
            .send(EdgeVecMessage::CollectionExists {
                name: name.to_string(),
                tx,
            })
            .await;
        rx.await
            .unwrap_or_else(|_| Err(Error::internal("Actor closed")))
    }

    async fn get_stats(&self, collection: &str) -> Result<HashMap<String, serde_json::Value>> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .sender
            .send(EdgeVecMessage::GetStats {
                collection: collection.to_string(),
                tx,
            })
            .await;
        rx.await
            .unwrap_or_else(|_| Err(Error::internal("Actor closed")))
    }

    async fn flush(&self, _collection: &str) -> Result<()> {
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "edgevec"
    }
}

#[async_trait]
impl VectorStoreProvider for EdgeVecVectorStoreProvider {
    async fn create_collection(&self, name: &str, _dimensions: usize) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .sender
            .send(EdgeVecMessage::CreateCollection {
                name: name.to_string(),
                tx,
            })
            .await;
        rx.await
            .unwrap_or_else(|_| Err(Error::internal("Actor closed")))
    }

    async fn delete_collection(&self, name: &str) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .sender
            .send(EdgeVecMessage::DeleteCollection {
                name: name.to_string(),
                tx,
            })
            .await;
        rx.await
            .unwrap_or_else(|_| Err(Error::internal("Actor closed")))
    }

    async fn insert_vectors(
        &self,
        collection: &str,
        vectors: &[Embedding],
        metadata: Vec<HashMap<String, serde_json::Value>>,
    ) -> Result<Vec<String>> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .sender
            .send(EdgeVecMessage::InsertVectors {
                collection: collection.to_string(),
                vectors: vectors.to_vec(),
                metadata,
                tx,
            })
            .await;
        rx.await
            .unwrap_or_else(|_| Err(Error::internal("Actor closed")))
    }

    async fn search_similar(
        &self,
        collection: &str,
        query_vector: &[f32],
        limit: usize,
        _filter: Option<&str>,
    ) -> Result<Vec<SearchResult>> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .sender
            .send(EdgeVecMessage::SearchSimilar {
                collection: collection.to_string(),
                query_vector: query_vector.to_vec(),
                limit,
                tx,
            })
            .await;
        rx.await
            .unwrap_or_else(|_| Err(Error::internal("Actor closed")))
    }

    async fn delete_vectors(&self, collection: &str, ids: &[String]) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .sender
            .send(EdgeVecMessage::DeleteVectors {
                collection: collection.to_string(),
                ids: ids.to_vec(),
                tx,
            })
            .await;
        rx.await
            .unwrap_or_else(|_| Err(Error::internal("Actor closed")))
    }

    async fn get_vectors_by_ids(
        &self,
        collection: &str,
        ids: &[String],
    ) -> Result<Vec<SearchResult>> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .sender
            .send(EdgeVecMessage::GetVectorsByIds {
                collection: collection.to_string(),
                ids: ids.to_vec(),
                tx,
            })
            .await;
        rx.await
            .unwrap_or_else(|_| Err(Error::internal("Actor closed")))
    }

    async fn list_vectors(&self, collection: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .sender
            .send(EdgeVecMessage::ListVectors {
                collection: collection.to_string(),
                limit,
                tx,
            })
            .await;
        rx.await
            .unwrap_or_else(|_| Err(Error::internal("Actor closed")))
    }
}

#[async_trait]
impl VectorStoreBrowser for EdgeVecVectorStoreProvider {
    async fn list_collections(&self) -> Result<Vec<CollectionInfo>> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .sender
            .send(EdgeVecMessage::ListCollections { tx })
            .await;
        rx.await
            .unwrap_or_else(|_| Err(Error::internal("Actor closed")))
    }

    async fn list_file_paths(&self, collection: &str, limit: usize) -> Result<Vec<FileInfo>> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .sender
            .send(EdgeVecMessage::ListFilePaths {
                collection: collection.to_string(),
                limit,
                tx,
            })
            .await;
        rx.await
            .unwrap_or_else(|_| Err(Error::internal("Actor closed")))
    }

    async fn get_chunks_by_file(
        &self,
        collection: &str,
        file_path: &str,
    ) -> Result<Vec<SearchResult>> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .sender
            .send(EdgeVecMessage::GetChunksByFile {
                collection: collection.to_string(),
                file_path: file_path.to_string(),
                tx,
            })
            .await;
        rx.await
            .unwrap_or_else(|_| Err(Error::internal("Actor closed")))
    }
}

struct EdgeVecActor {
    receiver: mpsc::Receiver<EdgeVecMessage>,
    index: edgevec::HnswIndex,
    storage: edgevec::VectorStorage,
    metadata_store: DashMap<String, HashMap<String, serde_json::Value>>,
    id_map: DashMap<String, VectorId>,
    config: EdgeVecConfig,
}

impl EdgeVecActor {
    fn new(receiver: mpsc::Receiver<EdgeVecMessage>, config: EdgeVecConfig) -> Result<Self> {
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

        let storage = edgevec::VectorStorage::new(&hnsw_config, None);
        let index = edgevec::HnswIndex::new(hnsw_config, &storage)
            .map_err(|e| Error::internal(format!("Failed to create EdgeVec HNSW index: {}", e)))?;

        Ok(Self {
            receiver,
            index,
            storage,
            metadata_store: DashMap::new(),
            id_map: DashMap::new(),
            config,
        })
    }

    async fn run(mut self) {
        while let Some(msg) = self.receiver.recv().await {
            match msg {
                EdgeVecMessage::CreateCollection { name, tx } => {
                    self.metadata_store.insert(name, HashMap::new());
                    let _ = tx.send(Ok(()));
                }
                EdgeVecMessage::DeleteCollection { name, tx } => {
                    if let Some((_, collection_metadata)) = self.metadata_store.remove(&name) {
                        for external_id in collection_metadata.keys() {
                            if let Some(vector_id) = self.id_map.remove(external_id) {
                                let _ = self.index.soft_delete(vector_id.1);
                            }
                        }
                    }
                    let _ = tx.send(Ok(()));
                }
                EdgeVecMessage::InsertVectors {
                    collection,
                    vectors,
                    metadata,
                    tx,
                } => {
                    let mut ids = Vec::with_capacity(vectors.len());
                    let mut collection_metadata =
                        self.metadata_store.entry(collection.clone()).or_default();
                    let mut result = Ok(Vec::new());

                    for (vector, meta) in vectors.into_iter().zip(metadata.into_iter()) {
                        let external_id = format!("{}_{}", collection, uuid::Uuid::new_v4());

                        match self.index.insert(&vector.vector, &mut self.storage) {
                            Ok(vector_id) => {
                                self.id_map.insert(external_id.clone(), vector_id);
                                let mut enriched_metadata = meta.clone();
                                enriched_metadata
                                    .insert("id".to_string(), serde_json::json!(external_id));
                                collection_metadata.insert(
                                    external_id.clone(),
                                    serde_json::json!(enriched_metadata),
                                );
                                ids.push(external_id);
                            }
                            Err(e) => {
                                result =
                                    Err(Error::internal(format!("Failed to insert vector: {}", e)));
                                break;
                            }
                        }
                    }
                    let response = if result.is_ok() { Ok(ids) } else { result };
                    let _ = tx.send(response);
                }
                EdgeVecMessage::SearchSimilar {
                    collection,
                    query_vector,
                    limit,
                    tx,
                } => {
                    let result = match self.index.search(&query_vector, limit, &self.storage) {
                        Ok(results) => {
                            let mut final_results = Vec::with_capacity(results.len());
                            if let Some(collection_metadata) = self.metadata_store.get(&collection)
                            {
                                for res in results {
                                    let external_id = self.id_map.iter().find_map(|entry| {
                                        if *entry.value() == res.vector_id {
                                            Some(entry.key().clone())
                                        } else {
                                            None
                                        }
                                    });

                                    if let Some(ext_id) = external_id {
                                        if let Some(meta_val) = collection_metadata.get(&ext_id) {
                                            let meta =
                                                meta_val.as_object().cloned().unwrap_or_default();
                                            let start_line = meta
                                                .opt_u64("start_line")
                                                .or_else(|| meta.opt_u64("line_number"))
                                                .unwrap_or(0)
                                                as u32;
                                            final_results.push(SearchResult {
                                                id: ext_id,
                                                file_path: meta.string_or("file_path", "unknown"),
                                                start_line,
                                                content: meta.string_or("content", ""),
                                                score: res.distance as f64,
                                                language: meta.string_or("language", "unknown"),
                                            });
                                        }
                                    }
                                }
                            }
                            Ok(final_results)
                        }
                        Err(e) => Err(Error::internal(format!("Search failed: {}", e))),
                    };
                    let _ = tx.send(result);
                }
                EdgeVecMessage::DeleteVectors {
                    collection,
                    ids,
                    tx,
                } => {
                    if let Some(mut collection_metadata) = self.metadata_store.get_mut(&collection)
                    {
                        for id in ids {
                            if let Some((_, vector_id)) = self.id_map.remove(&id) {
                                let _ = self.index.soft_delete(vector_id);
                            }
                            collection_metadata.remove(&id);
                        }
                    }
                    let _ = tx.send(Ok(()));
                }
                EdgeVecMessage::GetStats { collection, tx } => {
                    let vector_count = self
                        .metadata_store
                        .get(&collection)
                        .map(|m| m.len())
                        .unwrap_or(0);
                    let mut stats = HashMap::new();
                    stats.insert("collection".to_string(), serde_json::json!(collection));
                    stats.insert("vector_count".to_string(), serde_json::json!(vector_count));
                    stats.insert(
                        "total_indexed_vectors".to_string(),
                        serde_json::json!(self.index.len()),
                    );
                    stats.insert(
                        "dimensions".to_string(),
                        serde_json::json!(self.config.dimensions),
                    );
                    let _ = tx.send(Ok(stats));
                }
                EdgeVecMessage::ListVectors {
                    collection,
                    limit,
                    tx,
                } => {
                    let mut final_results = Vec::new();
                    if let Some(collection_metadata) = self.metadata_store.get(&collection) {
                        for (ext_id, meta_val) in collection_metadata.iter().take(limit) {
                            let meta = meta_val.as_object().cloned().unwrap_or_default();

                            final_results.push(SearchResult {
                                id: ext_id.clone(),
                                file_path: meta.string_or("file_path", "unknown"),
                                start_line: meta
                                    .opt_u64("start_line")
                                    .or_else(|| meta.opt_u64("line_number"))
                                    .unwrap_or(0)
                                    as u32,
                                content: meta.string_or("content", ""),
                                score: 1.0,
                                language: meta.string_or("language", "unknown"),
                            });
                        }
                    }
                    let _ = tx.send(Ok(final_results));
                }
                EdgeVecMessage::GetVectorsByIds {
                    collection,
                    ids,
                    tx,
                } => {
                    let mut final_results = Vec::new();
                    if let Some(collection_metadata) = self.metadata_store.get(&collection) {
                        for id in ids {
                            if let Some(meta_val) = collection_metadata.get(&id) {
                                let meta = meta_val.as_object().cloned().unwrap_or_default();
                                final_results.push(SearchResult {
                                    id: id.clone(),
                                    file_path: meta.string_or("file_path", "unknown"),
                                    start_line: meta
                                        .opt_u64("start_line")
                                        .or_else(|| meta.opt_u64("line_number"))
                                        .unwrap_or(0)
                                        as u32,
                                    content: meta.string_or("content", ""),
                                    score: 1.0,
                                    language: meta.string_or("language", "unknown"),
                                });
                            }
                        }
                    }
                    let _ = tx.send(Ok(final_results));
                }
                EdgeVecMessage::CollectionExists { name, tx } => {
                    let exists = self.metadata_store.contains_key(&name);
                    let _ = tx.send(Ok(exists));
                }
                EdgeVecMessage::ListCollections { tx } => {
                    let collections: Vec<CollectionInfo> = self
                        .metadata_store
                        .iter()
                        .map(|entry| {
                            let name = entry.key().clone();
                            let vector_count = entry.value().len() as u64;

                            // Count unique file paths
                            let file_paths: std::collections::HashSet<&str> = entry
                                .value()
                                .values()
                                .filter_map(|v| {
                                    v.as_object()
                                        .and_then(|o| o.get("file_path"))
                                        .and_then(|v| v.as_str())
                                })
                                .collect();
                            let file_count = file_paths.len() as u64;

                            CollectionInfo::new(name, vector_count, file_count, None, "edgevec")
                        })
                        .collect();
                    let _ = tx.send(Ok(collections));
                }
                EdgeVecMessage::ListFilePaths {
                    collection,
                    limit,
                    tx,
                } => {
                    let mut files = Vec::new();
                    if let Some(collection_metadata) = self.metadata_store.get(&collection) {
                        let mut file_map: HashMap<String, (u32, String)> = HashMap::new();

                        for meta_val in collection_metadata.values() {
                            if let Some(meta) = meta_val.as_object() {
                                if let Some(file_path) =
                                    meta.get("file_path").and_then(|v| v.as_str())
                                {
                                    let language = meta
                                        .get("language")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("unknown")
                                        .to_string();

                                    let entry = file_map
                                        .entry(file_path.to_string())
                                        .or_insert((0, language));
                                    entry.0 += 1;
                                }
                            }
                        }

                        files = file_map
                            .into_iter()
                            .take(limit)
                            .map(|(path, (chunk_count, language))| {
                                FileInfo::new(path, chunk_count, language, None)
                            })
                            .collect();
                    }
                    let _ = tx.send(Ok(files));
                }
                EdgeVecMessage::GetChunksByFile {
                    collection,
                    file_path,
                    tx,
                } => {
                    let mut results = Vec::new();
                    if let Some(collection_metadata) = self.metadata_store.get(&collection) {
                        for (ext_id, meta_val) in collection_metadata.iter() {
                            if let Some(meta) = meta_val.as_object() {
                                if meta
                                    .get("file_path")
                                    .and_then(|v| v.as_str())
                                    .is_some_and(|p| p == file_path)
                                {
                                    let start_line = meta
                                        .opt_u64("start_line")
                                        .or_else(|| meta.opt_u64("line_number"))
                                        .unwrap_or(0)
                                        as u32;

                                    results.push(SearchResult {
                                        id: ext_id.clone(),
                                        file_path: file_path.to_string(),
                                        start_line,
                                        content: meta.string_or("content", ""),
                                        score: 1.0,
                                        language: meta.string_or("language", "unknown"),
                                    });
                                }
                            }
                        }
                    }
                    // Sort by start_line
                    results.sort_by_key(|r| r.start_line);
                    let _ = tx.send(Ok(results));
                }
            }
        }
    }
}

// ============================================================================
// Auto-registration via linkme distributed slice
// ============================================================================

use std::sync::Arc;

use mcb_application::ports::registry::{
    VECTOR_STORE_PROVIDERS, VectorStoreProviderConfig, VectorStoreProviderEntry,
};

/// Factory function for creating EdgeVec vector store provider instances.
fn edgevec_factory(
    config: &VectorStoreProviderConfig,
) -> std::result::Result<Arc<dyn VectorStoreProvider>, String> {
    let dimensions = config.dimensions.unwrap_or(384);
    let edgevec_config = EdgeVecConfig {
        dimensions,
        ..Default::default()
    };
    let provider = EdgeVecVectorStoreProvider::new(edgevec_config)
        .map_err(|e| format!("Failed to create EdgeVec provider: {e}"))?;
    Ok(Arc::new(provider))
}

#[linkme::distributed_slice(VECTOR_STORE_PROVIDERS)]
static EDGEVEC_PROVIDER: VectorStoreProviderEntry = VectorStoreProviderEntry {
    name: "edgevec",
    description: "EdgeVec in-memory HNSW vector store (high-performance)",
    factory: edgevec_factory,
};
