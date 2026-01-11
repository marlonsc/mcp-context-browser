//! EdgeVec Vector Store Provider
//!
//! High-performance embedded vector database implementation using EdgeVec.
//! EdgeVec provides sub-millisecond vector similarity search with HNSW algorithm.
//! This implementation uses the Actor pattern to eliminate locks and ensure non-blocking operation.

use async_trait::async_trait;
use dashmap::DashMap;
use std::collections::HashMap;
use tokio::sync::{mpsc, oneshot};

use crate::domain::error::{Error, Result};
use crate::domain::ports::VectorStoreProvider;
use crate::domain::types::{Embedding, SearchResult};
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
    CollectionExists {
        name: String,
        tx: oneshot::Sender<Result<bool>>,
    },
}

/// EdgeVec vector store provider implementation using Actor pattern
pub struct EdgeVecVectorStoreProvider {
    sender: mpsc::Sender<EdgeVecMessage>,
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
                                    // Slow search for external ID, but this is a simplified actor
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
                                            final_results.push(SearchResult {
                                                id: ext_id,
                                                file_path: meta
                                                    .get("file_path")
                                                    .and_then(|v| v.as_str())
                                                    .unwrap_or("unknown")
                                                    .to_string(),
                                                line_number: meta
                                                    .get("line_number")
                                                    .and_then(|v| v.as_u64())
                                                    .unwrap_or(0)
                                                    as u32,
                                                content: meta
                                                    .get("content")
                                                    .and_then(|v| v.as_str())
                                                    .unwrap_or("")
                                                    .to_string(),
                                                score: res.distance,
                                                metadata: serde_json::json!(meta),
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
                EdgeVecMessage::CollectionExists { name, tx } => {
                    let exists = self.metadata_store.contains_key(&name);
                    let _ = tx.send(Ok(exists));
                }
            }
        }
    }
}
