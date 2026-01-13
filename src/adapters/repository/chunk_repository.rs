//! Repository for managing code chunks with vector embeddings
//!
//! This module provides a repository pattern for storing and retrieving
//! code chunks with their associated vector embeddings.

use crate::domain::error::{Error, Result};
use crate::domain::ports::{ChunkRepository, EmbeddingProvider, VectorStoreProvider};
use crate::domain::types::{CodeChunk, Language, RepositoryStats};
use async_trait::async_trait;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

/// Parse Language from metadata
fn parse_language_from_metadata(metadata: &serde_json::Value) -> Language {
    metadata
        .get("language")
        .and_then(|v| v.as_str())
        .and_then(|s| Language::from_str(s).ok())
        .unwrap_or(Language::Unknown)
}

/// Vector store backed chunk repository
#[derive(shaku::Component)]
#[shaku(interface = ChunkRepository)]
pub struct VectorStoreChunkRepository {
    #[shaku(inject)]
    embedding_provider: Arc<dyn EmbeddingProvider>,
    #[shaku(inject)]
    vector_store_provider: Arc<dyn VectorStoreProvider>,
}

impl VectorStoreChunkRepository {
    pub fn new(
        embedding_provider: Arc<dyn EmbeddingProvider>,
        vector_store_provider: Arc<dyn VectorStoreProvider>,
    ) -> Self {
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
impl ChunkRepository for VectorStoreChunkRepository {
    async fn save(&self, collection: &str, chunk: &CodeChunk) -> Result<String> {
        let chunks = vec![chunk.clone()];
        let ids = self.save_batch(collection, &chunks).await?;
        ids.into_iter()
            .next()
            .ok_or_else(|| Error::internal("Failed to save chunk - no ID returned".to_string()))
    }

    async fn save_batch(&self, collection: &str, chunks: &[CodeChunk]) -> Result<Vec<String>> {
        if chunks.is_empty() {
            return Ok(vec![]);
        }

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
                meta.insert(
                    "start_line".to_string(),
                    serde_json::json!(chunk.start_line),
                );
                meta.insert("end_line".to_string(), serde_json::json!(chunk.end_line));
                meta.insert(
                    "language".to_string(),
                    serde_json::json!(chunk.language.as_str()),
                );
                meta.insert("chunk_type".to_string(), serde_json::json!("code_chunk"));
                meta
            })
            .collect();

        // Ensure collection exists
        if !self
            .vector_store_provider
            .collection_exists(&collection_name)
            .await?
        {
            self.vector_store_provider
                .create_collection(&collection_name, self.embedding_provider.dimensions())
                .await?;
        }

        // Store in vector database
        let ids = self
            .vector_store_provider
            .insert_vectors(&collection_name, &embeddings, metadata)
            .await?;

        Ok(ids)
    }

    async fn find_by_id(&self, collection: &str, id: &str) -> Result<Option<CodeChunk>> {
        let collection_name = self.collection_name(collection);

        // Use direct ID lookup from vector store provider
        let results = self
            .vector_store_provider
            .get_vectors_by_ids(&collection_name, &[id.to_string()])
            .await?;

        if let Some(result) = results.into_iter().next() {
            let file_path = result.file_path;
            let start_line = result.start_line;
            let content = result.content;

            let end_line = result
                .metadata
                .get("end_line")
                .and_then(|v| v.as_u64())
                .unwrap_or(start_line as u64) as u32;
            let language = parse_language_from_metadata(&result.metadata);

            return Ok(Some(CodeChunk {
                id: id.to_string(),
                content,
                file_path,
                start_line,
                end_line,
                language,
                metadata: result.metadata,
            }));
        }

        Ok(None)
    }

    async fn find_by_collection(&self, collection: &str, limit: usize) -> Result<Vec<CodeChunk>> {
        let collection_name = self.collection_name(collection);

        // Use direct list from vector store provider
        let results = self
            .vector_store_provider
            .list_vectors(&collection_name, limit)
            .await?;

        // Convert results back to CodeChunks
        let chunks: Vec<CodeChunk> = results
            .into_iter()
            .map(|result| {
                let start_line = result.start_line;
                let end_line = result
                    .metadata
                    .get("end_line")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(start_line as u64) as u32;

                CodeChunk {
                    id: result.id,
                    content: result.content,
                    file_path: result.file_path,
                    start_line,
                    end_line,
                    language: parse_language_from_metadata(&result.metadata),
                    metadata: result.metadata,
                }
            })
            .collect();

        Ok(chunks)
    }

    async fn delete(&self, collection: &str, id: &str) -> Result<()> {
        let collection_name = self.collection_name(collection);

        // Direct delete using the vector ID
        self.vector_store_provider
            .delete_vectors(&collection_name, &[id.to_string()])
            .await
    }

    async fn delete_collection(&self, collection: &str) -> Result<()> {
        let collection_name = self.collection_name(collection);
        self.vector_store_provider
            .delete_collection(&collection_name)
            .await
    }

    async fn stats(&self) -> Result<RepositoryStats> {
        // Get stats for the default collection as a baseline
        let store_stats = self
            .vector_store_provider
            .get_stats("mcp_chunks_default")
            .await
            .unwrap_or_default();

        Ok(RepositoryStats {
            total_chunks: store_stats
                .get("total_vectors")
                .or_else(|| store_stats.get("vectors_count"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            total_collections: 1, // Simplified for now
            storage_size_bytes: store_stats
                .get("storage_size_bytes")
                .or_else(|| store_stats.get("total_size_bytes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            avg_chunk_size_bytes: store_stats
                .get("avg_vector_size_bytes")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
        })
    }
}
