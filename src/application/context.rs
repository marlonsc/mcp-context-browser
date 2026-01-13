//! Code Intelligence Business Service
//!
//! The Context Service transforms raw code into semantic understanding through
//! AI embeddings and intelligent storage. This business service powers the core
//! intelligence behind semantic code search, enabling development teams to find
//! code by meaning rather than keywords.

use crate::domain::error::Result;
use crate::domain::ports::{
    ChunkRepository, ContextServiceInterface, EmbeddingProvider, SearchRepository,
};
use crate::domain::types::{CodeChunk, Embedding, RepositoryStats, SearchResult, SearchStats};
use async_trait::async_trait;
use std::sync::Arc;

/// Enterprise Code Intelligence Coordinator
///
/// This service orchestrates the high-level business logic for code intelligence,
/// delegating data access and specialized search operations to repositories.
pub struct ContextService {
    chunk_repository: Arc<dyn ChunkRepository>,
    search_repository: Arc<dyn SearchRepository>,
    embedding_provider: Arc<dyn EmbeddingProvider>,
}

impl ContextService {
    /// Create a new context service with specified repositories and providers
    pub fn new(
        chunk_repository: Arc<dyn ChunkRepository>,
        search_repository: Arc<dyn SearchRepository>,
        embedding_provider: Arc<dyn EmbeddingProvider>,
    ) -> Self {
        Self {
            chunk_repository,
            search_repository,
            embedding_provider,
        }
    }

    /// Create a new context service with specified providers (test-friendly constructor)
    pub fn new_with_providers(
        embedding_provider: Arc<dyn EmbeddingProvider>,
        vector_store_provider: Arc<dyn crate::domain::ports::VectorStoreProvider>,
    ) -> Self {
        let chunk_repo = Arc::new(
            crate::adapters::repository::VectorStoreChunkRepository::new(
                Arc::clone(&embedding_provider),
                Arc::clone(&vector_store_provider),
            ),
        );
        let search_repo = Arc::new(
            crate::adapters::repository::VectorStoreSearchRepository::new(Arc::clone(
                &vector_store_provider,
            )),
        );
        Self::new(chunk_repo, search_repo, embedding_provider)
    }

    /// Initialize the context service by loading existing data
    pub async fn initialize(&self, collection: &str) -> Result<()> {
        tracing::info!(
            "[CONTEXT] Initializing hybrid search index for collection: {}",
            collection
        );

        // Load chunks from the repository
        let chunks = self
            .chunk_repository
            .find_by_collection(collection, 10000)
            .await?;

        if !chunks.is_empty() {
            tracing::info!(
                "[CONTEXT] Loading {} chunks into hybrid search engine",
                chunks.len()
            );
            self.search_repository
                .index_for_hybrid_search(&chunks)
                .await?;
        }

        Ok(())
    }

    /// Generate embeddings for text
    pub async fn embed_text(&self, text: &str) -> Result<Embedding> {
        self.embedding_provider.embed(text).await
    }

    /// Generate embeddings for multiple texts
    pub async fn embed_texts(&self, texts: &[String]) -> Result<Vec<Embedding>> {
        self.embedding_provider.embed_batch(texts).await
    }

    /// Store code chunks using the repositories
    pub async fn store_chunks(&self, collection: &str, chunks: &[CodeChunk]) -> Result<()> {
        // Save chunks to primary storage (vector store via repository)
        self.chunk_repository.save_batch(collection, chunks).await?;

        // Index chunks for hybrid search (lexical search)
        self.search_repository
            .index_for_hybrid_search(chunks)
            .await?;

        Ok(())
    }

    /// Search for similar code chunks using hybrid search
    pub async fn search_similar(
        &self,
        collection: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        let query_embedding = self.embed_text(query).await?;

        // Delegate hybrid search to the search repository
        self.search_repository
            .hybrid_search(collection, query, &query_embedding.vector, limit)
            .await
    }

    /// Clear all data for a collection
    pub async fn clear_collection(&self, collection: &str) -> Result<()> {
        // Clear both primary storage and search index
        self.chunk_repository.delete_collection(collection).await?;
        self.search_repository.clear_index(collection).await?;
        Ok(())
    }

    /// Get combined repository and search statistics
    pub async fn get_stats(&self) -> Result<(RepositoryStats, SearchStats)> {
        let chunk_stats = self.chunk_repository.stats().await?;
        let search_stats = self.search_repository.search_stats().await?;
        Ok((chunk_stats, search_stats))
    }

    /// Get embedding dimensions
    pub fn embedding_dimensions(&self) -> usize {
        self.embedding_provider.dimensions()
    }
}

// Note: impl Default removed (Phase 5 DI audit)
// ContextService requires explicit dependency injection via new()

#[async_trait]
impl ContextServiceInterface for ContextService {
    async fn initialize(&self, collection: &str) -> Result<()> {
        self.initialize(collection).await
    }

    async fn store_chunks(&self, collection: &str, chunks: &[CodeChunk]) -> Result<()> {
        self.store_chunks(collection, chunks).await
    }

    async fn search_similar(
        &self,
        collection: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        self.search_similar(collection, query, limit).await
    }

    async fn embed_text(&self, text: &str) -> Result<Embedding> {
        self.embed_text(text).await
    }

    async fn clear_collection(&self, collection: &str) -> Result<()> {
        self.clear_collection(collection).await
    }

    async fn get_stats(&self) -> Result<(RepositoryStats, SearchStats)> {
        self.get_stats().await
    }

    fn embedding_dimensions(&self) -> usize {
        self.embedding_dimensions()
    }
}
