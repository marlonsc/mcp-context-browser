//! Hybrid search combining BM25 text ranking with semantic embeddings
//!
//! This module implements a hybrid search approach that combines:
//! - BM25: Term frequency-based text ranking algorithm
//! - Semantic Embeddings: Vector similarity for semantic understanding
//!
//! The hybrid approach provides better relevance by combining lexical and semantic matching.

mod actor;
mod bm25;
pub mod config;
mod engine;

// Re-export public types
pub use actor::{HybridSearchActor, HybridSearchMessage};
pub use bm25::{BM25Params, BM25Scorer};
pub use config::HybridSearchConfig;
pub use engine::{HybridSearchEngine, HybridSearchResult};

use crate::domain::error::Result;
use crate::domain::ports::HybridSearchProvider;
use crate::domain::types::{CodeChunk, SearchResult};
use async_trait::async_trait;
use std::collections::HashMap;
use tokio::sync::{mpsc, oneshot};

/// Hybrid search adapter that implements the port
pub struct HybridSearchAdapter {
    sender: mpsc::Sender<HybridSearchMessage>,
}

impl HybridSearchAdapter {
    /// Create a new hybrid search adapter
    pub fn new(sender: mpsc::Sender<HybridSearchMessage>) -> Self {
        Self { sender }
    }
}

#[async_trait]
impl HybridSearchProvider for HybridSearchAdapter {
    async fn index_chunks(&self, collection: &str, chunks: &[CodeChunk]) -> Result<()> {
        self.sender
            .send(HybridSearchMessage::Index {
                collection: collection.to_string(),
                chunks: chunks.to_vec(),
            })
            .await
            .map_err(|e| {
                crate::domain::error::Error::internal(format!(
                    "Failed to send to hybrid search actor: {}",
                    e
                ))
            })
    }

    async fn search(
        &self,
        _collection: &str,
        query: &str,
        semantic_results: Vec<SearchResult>,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        let (respond_to, receiver) = oneshot::channel();
        self.sender
            .send(HybridSearchMessage::Search {
                query: query.to_string(),
                semantic_results,
                limit,
                respond_to,
            })
            .await
            .map_err(|e| {
                crate::domain::error::Error::internal(format!(
                    "Failed to send search to hybrid search actor: {}",
                    e
                ))
            })?;

        let hybrid_results = receiver.await.map_err(|e| {
            crate::domain::error::Error::internal(format!(
                "Failed to receive hybrid search results: {}",
                e
            ))
        })??;

        Ok(hybrid_results
            .into_iter()
            .map(|hybrid_result| {
                let mut result = hybrid_result.result;
                result.score = hybrid_result.hybrid_score;

                let mut new_metadata = serde_json::Map::new();
                if let serde_json::Value::Object(existing) = &result.metadata {
                    new_metadata.extend(existing.clone());
                }
                new_metadata.insert(
                    "bm25_score".to_string(),
                    serde_json::json!(hybrid_result.bm25_score),
                );
                new_metadata.insert(
                    "semantic_score".to_string(),
                    serde_json::json!(hybrid_result.semantic_score),
                );
                new_metadata.insert(
                    "hybrid_score".to_string(),
                    serde_json::json!(hybrid_result.hybrid_score),
                );
                result.metadata = serde_json::Value::Object(new_metadata);

                result
            })
            .collect())
    }

    async fn clear_collection(&self, collection: &str) -> Result<()> {
        self.sender
            .send(HybridSearchMessage::Clear {
                collection: collection.to_string(),
            })
            .await
            .map_err(|e| {
                crate::domain::error::Error::internal(format!(
                    "Failed to send clear to hybrid search actor: {}",
                    e
                ))
            })
    }

    async fn get_stats(&self) -> HashMap<String, serde_json::Value> {
        let (respond_to, receiver) = oneshot::channel();
        if self
            .sender
            .send(HybridSearchMessage::GetStats { respond_to })
            .await
            .is_err()
        {
            return HashMap::new();
        }

        receiver.await.unwrap_or_default()
    }
}
