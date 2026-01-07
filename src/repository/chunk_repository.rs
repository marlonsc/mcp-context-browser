//! Repository for managing code chunks with vector embeddings
//!
//! This module provides a repository pattern for storing and retrieving
//! code chunks with their associated vector embeddings.

use crate::core::error::{Error, Result};
use crate::core::types::{CodeChunk, Embedding};
use crate::providers::{EmbeddingProvider, VectorStoreProvider};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

/// Statistics about the repository
#[derive(Debug, Clone)]
pub struct RepositoryStats {
    pub total_chunks: u64,
    pub total_collections: u64,
    pub storage_size_bytes: u64,
    pub avg_chunk_size_bytes: f64,
}

/// Repository trait for code chunks
#[async_trait]
pub trait ChunkRepository {
    async fn save(&self, chunk: &CodeChunk) -> Result<String>;
    async fn save_batch(&self, chunks: &[CodeChunk]) -> Result<Vec<String>>;
    async fn find_by_id(&self, id: &str) -> Result<Option<CodeChunk>>;
    async fn find_by_collection(&self, collection: &str, limit: usize) -> Result<Vec<CodeChunk>>;
    async fn delete(&self, id: &str) -> Result<()>;
    async fn delete_collection(&self, collection: &str) -> Result<()>;
    async fn stats(&self) -> Result<RepositoryStats>;
}

/// Vector store backed chunk repository
pub struct VectorStoreChunkRepository<E, V> {
    embedding_provider: Arc<E>,
    vector_store_provider: Arc<V>,
}

impl<E, V> VectorStoreChunkRepository<E, V> {
    pub fn new(embedding_provider: Arc<E>, vector_store_provider: Arc<V>) -> Self {
        Self {
            embedding_provider,
            vector_store_provider,
        }
    }

    fn collection_name(&self, collection: &str) -> String {
        format!("mcp_chunks_{}", collection)
    }
}

#[async_trait]
impl<E, V> ChunkRepository for VectorStoreChunkRepository<E, V>
where
    E: EmbeddingProvider + Send + Sync,
    V: VectorStoreProvider + Send + Sync,
{
    async fn save(&self, chunk: &CodeChunk) -> Result<String> {
        let chunks = vec![chunk.clone()];
        let ids = self.save_batch(&chunks).await?;
        ids.into_iter().next().ok_or_else(|| {
            Error::internal("Failed to save chunk - no ID returned".to_string())
        })
    }

    async fn save_batch(&self, chunks: &[CodeChunk]) -> Result<Vec<String>> {
        if chunks.is_empty() {
            return Ok(vec![]);
        }

        let collection = "default";
        let collection_name = self.collection_name(collection);

        // Generate embeddings for all chunks
        let texts: Vec<String> = chunks.iter().map(|c| c.content.clone()).collect();
        let embeddings = self.embedding_provider.embed_batch(&texts).await?;

        // Prepare metadata for each chunk
        let metadata: Vec<HashMap<String, serde_json::Value>> = chunks
            .iter()
            .map(|chunk| {
                let mut meta = HashMap::new();
                meta.insert("content".to_string(), serde_json::json!(chunk.content));
                meta.insert("file_path".to_string(), serde_json::json!(chunk.file_path));
                meta.insert("start_line".to_string(), serde_json::json!(chunk.start_line));
                meta.insert("end_line".to_string(), serde_json::json!(chunk.end_line));
                meta.insert("language".to_string(), serde_json::json!(format!("{:?}", chunk.language)));
                meta.insert("chunk_type".to_string(), serde_json::json!("code_chunk"));
                meta
            })
            .collect();

        // Ensure collection exists
        if !self.vector_store_provider.collection_exists(&collection_name).await? {
            self.vector_store_provider
                .create_collection(&collection_name, self.embedding_provider.dimensions())
                .await?;
        }

        // Store in vector database
        let ids = self.vector_store_provider
            .insert_vectors(&collection_name, &embeddings, metadata)
            .await?;

        Ok(ids)
    }

    async fn find_by_id(&self, _id: &str) -> Result<Option<CodeChunk>> {
        // Vector stores typically don't support finding by ID directly
        Ok(None)
    }

    async fn find_by_collection(&self, collection: &str, limit: usize) -> Result<Vec<CodeChunk>> {
        let collection_name = self.collection_name(collection);

        // Get vectors from store - need to embed the query first
        // For now, return empty results as embedding is required
        let results = vec![]; // TODO: Implement proper query embedding

        // Convert results back to CodeChunks
        let chunks: Vec<CodeChunk> = results
            .into_iter()
            .filter_map(|result| {
                // Extract metadata to reconstruct CodeChunk
                let content = result.metadata.get("content")?
                    .as_str()?.to_string();
                let file_path = result.metadata.get("file_path")?
                    .as_str()?.to_string();
                let start_line = result.metadata.get("start_line")?
                    .as_u64()? as u32;
                let end_line = result.metadata.get("end_line")?
                    .as_u64()? as u32;

                Some(CodeChunk {
                    id: format!("{}_{}", file_path, start_line), // Generate ID from file and line
                    content,
                    file_path,
                    start_line,
                    end_line,
                    language: crate::core::types::Language::Rust, // Default
                    metadata: result.metadata.clone(),
                })
            })
            .collect();

        Ok(chunks)
    }

    async fn delete(&self, _id: &str) -> Result<()> {
        // Not implemented for vector stores
        Err(Error::generic("Delete by ID not implemented for vector store repository"))
    }

    async fn delete_collection(&self, collection: &str) -> Result<()> {
        let collection_name = self.collection_name(collection);
        self.vector_store_provider.delete_collection(&collection_name).await
    }

    async fn stats(&self) -> Result<RepositoryStats> {
        // Get stats from vector store if available
        let store_stats = self.vector_store_provider
            .get_stats("default")
            .await
            .unwrap_or_default();

        Ok(RepositoryStats {
            total_chunks: store_stats.get("total_vectors").and_then(|v| v.as_u64()).unwrap_or(0),
            total_collections: 1,
            storage_size_bytes: store_stats.get("storage_size_bytes").and_then(|v| v.as_u64()).unwrap_or(0),
            avg_chunk_size_bytes: store_stats.get("avg_vector_size_bytes").and_then(|v| v.as_f64()).unwrap_or(0.0),
        })
    }
}