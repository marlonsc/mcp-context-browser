//! Repository for managing code chunks with vector embeddings
//!
//! This module provides a repository pattern for storing and retrieving
//! code chunks with their associated vector embeddings.

use crate::adapters::repository::{ChunkRepository, RepositoryStats};
use crate::domain::error::{Error, Result};
use crate::domain::ports::{EmbeddingProvider, VectorStoreProvider};
use crate::domain::types::{CodeChunk, Language};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

/// Parse Language from metadata (stored as "{:?}" format)
fn parse_language_from_metadata(metadata: &serde_json::Value) -> Language {
    let lang_str = metadata
        .get("language")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown");

    match lang_str {
        "Rust" => Language::Rust,
        "Python" => Language::Python,
        "JavaScript" => Language::JavaScript,
        "TypeScript" => Language::TypeScript,
        "Go" => Language::Go,
        "Java" => Language::Java,
        "C" => Language::C,
        "Cpp" => Language::Cpp,
        "CSharp" => Language::CSharp,
        "Php" => Language::Php,
        "Ruby" => Language::Ruby,
        "Swift" => Language::Swift,
        "Kotlin" => Language::Kotlin,
        "Scala" => Language::Scala,
        "Haskell" => Language::Haskell,
        _ => Language::Unknown,
    }
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
        ids.into_iter()
            .next()
            .ok_or_else(|| Error::internal("Failed to save chunk - no ID returned".to_string()))
    }

    async fn save_batch(&self, chunks: &[CodeChunk]) -> Result<Vec<String>> {
        if chunks.is_empty() {
            return Ok(vec![]);
        }

        // Use the first chunk's collection or default to "default"
        let collection = chunks
            .first()
            .and_then(|chunk| chunk.metadata.get("collection"))
            .and_then(|c| c.as_str())
            .unwrap_or("default");
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
                    serde_json::json!(format!("{:?}", chunk.language)),
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

    async fn find_by_id(&self, id: &str) -> Result<Option<CodeChunk>> {
        // Search all collections for a chunk with matching ID
        let collections = ["default"]; // Active collections

        for collection in collections {
            let collection_name = self.collection_name(collection);

            // Check if collection exists
            if !self
                .vector_store_provider
                .collection_exists(&collection_name)
                .await?
            {
                continue;
            }

            // Search for vectors and filter by ID in metadata
            let query_vector = vec![0.0; self.embedding_provider.dimensions()];
            let results = self
                .vector_store_provider
                .search_similar(&collection_name, &query_vector, 1000, None)
                .await?;

            // Find chunk with matching ID
            for result in results {
                // Use file_path from result directly, or from metadata
                let file_path = if !result.file_path.is_empty() {
                    result.file_path.clone()
                } else {
                    let Some(fp) = result.metadata.get("file_path").and_then(|v| v.as_str()) else {
                        continue; // Skip if file_path is missing
                    };
                    fp.to_string()
                };

                let Some(start_line_val) =
                    result.metadata.get("start_line").and_then(|v| v.as_u64())
                else {
                    continue; // Skip if start_line is missing
                };
                let start_line = start_line_val as u32;

                let generated_id = format!("{}_{}", file_path, start_line);

                if generated_id == id {
                    let content = if !result.content.is_empty() {
                        result.content.clone()
                    } else {
                        let Some(c) = result.metadata.get("content").and_then(|v| v.as_str())
                        else {
                            continue; // Skip if content is missing
                        };
                        c.to_string()
                    };

                    let end_line = result
                        .metadata
                        .get("end_line")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(start_line as u64) as u32;
                    let language = parse_language_from_metadata(&result.metadata);

                    return Ok(Some(CodeChunk {
                        id: generated_id,
                        content,
                        file_path,
                        start_line,
                        end_line,
                        language,
                        metadata: result.metadata.clone(),
                    }));
                }
            }
        }

        Ok(None)
    }

    async fn find_by_collection(&self, collection: &str, limit: usize) -> Result<Vec<CodeChunk>> {
        let collection_name = self.collection_name(collection);

        // For collection search without specific query, we need to search all vectors
        // This is a simplified implementation - in practice you'd want to search by a meaningful query
        let query_vector = vec![0.0; self.embedding_provider.dimensions()]; // Zero vector for collection search

        let results = self
            .vector_store_provider
            .search_similar(&collection_name, &query_vector, limit, None)
            .await?;

        // Convert results back to CodeChunks
        let chunks: Vec<CodeChunk> = results
            .into_iter()
            .filter_map(|result: crate::domain::types::SearchResult| {
                // Extract metadata to reconstruct CodeChunk
                // Use result fields if available, fallback to metadata
                let content = if !result.content.is_empty() {
                    result.content.clone()
                } else {
                    result.metadata.get("content")?.as_str()?.to_string()
                };
                let file_path = if !result.file_path.is_empty() {
                    result.file_path.clone()
                } else {
                    result.metadata.get("file_path")?.as_str()?.to_string()
                };

                let start_line =
                    if let Some(sl) = result.metadata.get("start_line").and_then(|v| v.as_u64()) {
                        sl as u32
                    } else {
                        result.line_number
                    };

                let end_line = result
                    .metadata
                    .get("end_line")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(start_line as u64) as u32;

                Some(CodeChunk {
                    id: format!("{}_{}", file_path, start_line), // Generate ID from file and line
                    content,
                    file_path,
                    start_line,
                    end_line,
                    language: parse_language_from_metadata(&result.metadata),
                    metadata: result.metadata.clone(),
                })
            })
            .collect();

        Ok(chunks)
    }

    async fn delete(&self, id: &str) -> Result<()> {
        // Delete individual chunks by searching for matching ID
        // Note: This is a workaround since SearchResult doesn't expose vector store IDs
        // We search for the chunk and use delete_vectors with a generated ID

        let collections = ["default"]; // Active collections

        for collection in collections {
            let collection_name = self.collection_name(collection);

            // Check if collection exists
            if !self
                .vector_store_provider
                .collection_exists(&collection_name)
                .await?
            {
                continue;
            }

            // Search for vectors to find one with matching ID
            let query_vector = vec![0.0; self.embedding_provider.dimensions()];
            let results = self
                .vector_store_provider
                .search_similar(&collection_name, &query_vector, 1000, None)
                .await?;

            // Find vector with matching chunk ID
            for result in results {
                let file_path = if !result.file_path.is_empty() {
                    result.file_path.clone()
                } else {
                    result
                        .metadata
                        .get("file_path")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string()
                };
                let start_line = result
                    .metadata
                    .get("start_line")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(result.line_number as u64) as u32;
                let generated_id = format!("{}_{}", file_path, start_line);

                if generated_id == id {
                    // Found the chunk - attempt delete using the generated ID as vector ID
                    // This relies on the vector store implementation using consistent IDs
                    let _ = self
                        .vector_store_provider
                        .delete_vectors(&collection_name, &[generated_id])
                        .await;
                    return Ok(());
                }
            }
        }

        // Chunk not found is still OK - delete is idempotent
        Ok(())
    }

    async fn delete_collection(&self, collection: &str) -> Result<()> {
        let collection_name = self.collection_name(collection);
        self.vector_store_provider
            .delete_collection(&collection_name)
            .await
    }

    async fn stats(&self) -> Result<RepositoryStats> {
        // Get stats from vector store if available
        let store_stats = self
            .vector_store_provider
            .get_stats("default")
            .await
            .unwrap_or_default();

        Ok(RepositoryStats {
            total_chunks: store_stats
                .get("total_vectors")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            total_collections: 1,
            storage_size_bytes: store_stats
                .get("storage_size_bytes")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            avg_chunk_size_bytes: store_stats
                .get("avg_vector_size_bytes")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
        })
    }
}
