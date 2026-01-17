//! Context Service Use Case
//!
//! Application service for code intelligence and semantic operations.
//! Orchestrates embeddings, vector storage, and caching for semantic code understanding.

use crate::domain_services::search::ContextServiceInterface;
use mcb_domain::entities::CodeChunk;
use mcb_domain::error::Result;
use crate::ports::providers::cache::CacheEntryConfig;
use crate::ports::providers::{EmbeddingProvider, VectorStoreProvider};
// Note: Stats moved to domain_services
use mcb_domain::value_objects::{Embedding, SearchResult};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

/// Context service implementation - manages embeddings and vector storage
#[derive(shaku::Component)]
#[shaku(interface = crate::domain_services::search::ContextServiceInterface)]
pub struct ContextServiceImpl {
    #[shaku(inject)]
    cache: Arc<dyn crate::ports::providers::cache::CacheProvider>,

    #[shaku(inject)]
    embedding_provider: Arc<dyn EmbeddingProvider>,

    #[shaku(inject)]
    vector_store_provider: Arc<dyn VectorStoreProvider>,
}

impl ContextServiceImpl {
    /// Create new context service with injected dependencies
    pub fn new(
        cache: Arc<dyn crate::ports::providers::cache::CacheProvider>,
        embedding_provider: Arc<dyn EmbeddingProvider>,
        vector_store_provider: Arc<dyn VectorStoreProvider>,
    ) -> Self {
        Self {
            cache,
            embedding_provider,
            vector_store_provider,
        }
    }
}

#[async_trait::async_trait]
impl ContextServiceInterface for ContextServiceImpl {
    async fn initialize(&self, collection: &str) -> Result<()> {
        // Check if collection exists in vector store, create if not
        if !self
            .vector_store_provider
            .collection_exists(collection)
            .await?
        {
            let dimensions = self.embedding_provider.dimensions();
            self.vector_store_provider
                .create_collection(collection, dimensions)
                .await?;
        }

        // Also track in cache for metadata
        let collection_key = format!("collection:{}", collection);
        self.cache
            .set_json(
                &collection_key,
                "\"initialized\"",
                CacheEntryConfig::default(),
            )
            .await?;
        Ok(())
    }

    async fn store_chunks(&self, collection: &str, chunks: &[CodeChunk]) -> Result<()> {
        // Generate embeddings for each chunk
        let texts: Vec<String> = chunks.iter().map(|c| c.content.clone()).collect();
        let embeddings = self.embedding_provider.embed_batch(&texts).await?;

        // Build metadata for each chunk
        let metadata: Vec<HashMap<String, serde_json::Value>> = chunks
            .iter()
            .map(|chunk| {
                let mut meta = HashMap::new();
                meta.insert("id".to_string(), json!(chunk.id));
                meta.insert("file_path".to_string(), json!(chunk.file_path));
                meta.insert("content".to_string(), json!(chunk.content));
                meta.insert("start_line".to_string(), json!(chunk.start_line));
                meta.insert("end_line".to_string(), json!(chunk.end_line));
                meta.insert("language".to_string(), json!(chunk.language));
                meta
            })
            .collect();

        // Insert into vector store
        self.vector_store_provider
            .insert_vectors(collection, &embeddings, metadata)
            .await?;

        // Update collection metadata in cache
        let meta_key = format!("collection:{}:meta", collection);
        let chunk_count = chunks.len();
        self.cache
            .set_json(
                &meta_key,
                &chunk_count.to_string(),
                CacheEntryConfig::default(),
            )
            .await?;

        Ok(())
    }

    async fn search_similar(
        &self,
        collection: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        // Generate embedding for the query
        let query_embedding = self.embedding_provider.embed(query).await?;

        // Search in vector store
        let results = self
            .vector_store_provider
            .search_similar(collection, &query_embedding.vector, limit, None)
            .await?;

        Ok(results)
    }

    async fn embed_text(&self, text: &str) -> Result<Embedding> {
        // Use the configured embedding provider
        self.embedding_provider.embed(text).await
    }

    async fn clear_collection(&self, collection: &str) -> Result<()> {
        // Delete the collection from vector store
        if self
            .vector_store_provider
            .collection_exists(collection)
            .await?
        {
            self.vector_store_provider
                .delete_collection(collection)
                .await?;
        }

        // Also clear cache metadata
        let collection_key = format!("collection:{}", collection);
        self.cache.delete(&collection_key).await?;

        let meta_key = format!("collection:{}:meta", collection);
        self.cache.delete(&meta_key).await?;

        Ok(())
    }

    async fn get_stats(&self) -> Result<(i64, i64)> {
        // Return placeholder stats - simplified
        Ok((0, 0))
    }

    fn embedding_dimensions(&self) -> usize {
        self.embedding_provider.dimensions()
    }
}