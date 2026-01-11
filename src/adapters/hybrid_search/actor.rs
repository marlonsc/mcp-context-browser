//! Hybrid search actor pattern implementation
//!
//! This module implements the actor pattern for managing hybrid search state
//! in a thread-safe manner using message passing.

use crate::domain::error::Result;
use crate::domain::types::CodeChunk;
use std::collections::HashMap;
use tokio::sync::{mpsc, oneshot};

use super::engine::{HybridSearchEngine, HybridSearchResult};

/// Hybrid search message for the actor
pub enum HybridSearchMessage {
    /// Index documents for a collection
    Index {
        collection: String,
        chunks: Vec<CodeChunk>,
    },
    /// Perform hybrid search
    Search {
        query: String,
        semantic_results: Vec<crate::domain::types::SearchResult>,
        limit: usize,
        respond_to: oneshot::Sender<Result<Vec<HybridSearchResult>>>,
    },
    /// Clear a collection
    Clear { collection: String },
    /// Get statistics
    GetStats {
        respond_to: oneshot::Sender<HashMap<String, serde_json::Value>>,
    },
}

/// Hybrid search actor that manages the engine state
pub struct HybridSearchActor {
    engine: HybridSearchEngine,
    receiver: mpsc::Receiver<HybridSearchMessage>,
    indexed_docs: HashMap<String, Vec<CodeChunk>>,
}

impl HybridSearchActor {
    /// Create a new hybrid search actor
    pub fn new(
        receiver: mpsc::Receiver<HybridSearchMessage>,
        bm25_weight: f32,
        semantic_weight: f32,
    ) -> Self {
        Self {
            engine: HybridSearchEngine::new(bm25_weight, semantic_weight),
            receiver,
            indexed_docs: HashMap::new(),
        }
    }

    /// Run the actor loop
    pub async fn run(mut self) {
        while let Some(msg) = self.receiver.recv().await {
            match msg {
                HybridSearchMessage::Index { collection, chunks } => {
                    let docs = self.indexed_docs.entry(collection).or_default();
                    docs.extend(chunks);

                    // Rebuild BM25 index with all documents
                    let all_docs: Vec<CodeChunk> =
                        self.indexed_docs.values().flatten().cloned().collect();
                    self.engine.index_documents(all_docs);
                }
                HybridSearchMessage::Search {
                    query,
                    semantic_results,
                    limit,
                    respond_to,
                } => {
                    let result = self.engine.hybrid_search(&query, semantic_results, limit);
                    let _ = respond_to.send(result);
                }
                HybridSearchMessage::Clear { collection } => {
                    self.indexed_docs.remove(&collection);
                    let all_docs: Vec<CodeChunk> =
                        self.indexed_docs.values().flatten().cloned().collect();
                    self.engine.index_documents(all_docs);
                }
                HybridSearchMessage::GetStats { respond_to } => {
                    let mut stats = HashMap::new();
                    stats.insert("hybrid_search_enabled".to_string(), serde_json::json!(true));
                    stats.insert(
                        "bm25_index_available".to_string(),
                        serde_json::json!(self.engine.has_bm25_index()),
                    );

                    if let Some(bm25_stats) = self.engine.get_bm25_stats() {
                        stats.extend(bm25_stats);
                    }

                    stats.insert(
                        "total_collections".to_string(),
                        serde_json::json!(self.indexed_docs.len()),
                    );
                    stats.insert(
                        "total_indexed_documents".to_string(),
                        serde_json::json!(self
                            .indexed_docs
                            .values()
                            .map(|v| v.len())
                            .sum::<usize>()),
                    );

                    let _ = respond_to.send(stats);
                }
            }
        }
    }
}
