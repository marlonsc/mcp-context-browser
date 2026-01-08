//! Code Intelligence Business Service
//!
//! The Context Service transforms raw code into semantic understanding through
//! AI embeddings and intelligent storage. This business service powers the core
//! intelligence behind semantic code search, enabling development teams to find
//! code by meaning rather than keywords.
//!
//! ## Business Impact
//!
//! - **Semantic Understanding**: Converts code syntax into business meaning and context
//! - **Intelligent Storage**: Organizes code chunks for optimal retrieval performance
//! - **Multi-Modal Search**: Combines keyword and semantic search for comprehensive results
//! - **Enterprise Scale**: Handles millions of code chunks with sub-second response times

use crate::core::error::{Error, Result};
use crate::core::hybrid_search::{HybridSearchConfig, HybridSearchEngine};
use crate::core::types::{CodeChunk, Embedding, SearchResult};
use crate::providers::{EmbeddingProvider, VectorStoreProvider};
use crate::core::locks::{lock_rwlock_read, lock_rwlock_write};
use std::collections::HashMap;
use std::sync::Arc;

/// Enterprise Code Intelligence Coordinator
///
/// The ContextService orchestrates the complete business workflow of transforming
/// code into searchable intelligence. It coordinates AI providers for semantic
/// understanding and vector stores for enterprise-scale persistence, ensuring
/// development teams can instantly access codebase knowledge through natural language.
pub struct ContextService {
    embedding_provider: Arc<dyn EmbeddingProvider>,
    vector_store_provider: Arc<dyn VectorStoreProvider>,
    hybrid_search_engine: Arc<std::sync::RwLock<HybridSearchEngine>>,
    indexed_documents: Arc<std::sync::RwLock<HashMap<String, Vec<CodeChunk>>>>,
}

/// Repository-based context service using Repository pattern
pub struct RepositoryContextService<C, S>
where
    C: crate::repository::ChunkRepository + Send + Sync,
    S: crate::repository::SearchRepository + Send + Sync,
{
    chunk_repository: Arc<C>,
    search_repository: Arc<S>,
}

impl<C, S> RepositoryContextService<C, S>
where
    C: crate::repository::ChunkRepository + Send + Sync,
    S: crate::repository::SearchRepository + Send + Sync,
{
    /// Create a new repository-based context service
    pub fn new(chunk_repository: Arc<C>, search_repository: Arc<S>) -> Self {
        Self {
            chunk_repository,
            search_repository,
        }
    }

    /// Generate embeddings for text using repository-based approach
    pub async fn embed_text(&self, _text: &str) -> Result<Embedding> {
        // Note: This would need access to an embedding provider
        // For now, this is a placeholder - in practice, the repository
        // might handle embedding internally or we'd need to inject an embedder
        Err(Error::generic("Repository-based embedding not implemented"))
    }

    /// Store code chunks using the chunk repository
    pub async fn store_chunks(&self, _collection: &str, chunks: &[CodeChunk]) -> Result<()> {
        // Save chunks via repository
        self.chunk_repository.save_batch(chunks).await?;

        // Index for hybrid search
        self.search_repository
            .index_for_hybrid_search(chunks)
            .await?;

        Ok(())
    }

    /// Search for similar code chunks using repository-based search
    pub async fn search_similar(
        &self,
        collection: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        // For repository-based search, we need the query vector
        // This would typically be computed by an embedding service
        // For now, we'll use a zero vector as placeholder
        let query_vector = vec![0.0f32; 384]; // Mock dimension

        // Perform hybrid search using the search repository
        self.search_repository
            .hybrid_search(collection, query, &query_vector, limit)
            .await
    }

    /// Clear a collection using repositories
    pub async fn clear_collection(&self, collection: &str) -> Result<()> {
        // Clear chunks
        self.chunk_repository.delete_collection(collection).await?;

        // Clear search index
        self.search_repository.clear_index(collection).await?;

        Ok(())
    }

    /// Get repository statistics
    pub async fn get_repository_stats(
        &self,
    ) -> Result<(
        crate::repository::RepositoryStats,
        crate::repository::SearchStats,
    )> {
        let chunk_stats = self.chunk_repository.stats().await?;
        let search_stats = self.search_repository.search_stats().await?;

        Ok((chunk_stats, search_stats))
    }
}

impl ContextService {
    /// Create a new context service with specified providers
    pub fn new(
        embedding_provider: Arc<dyn EmbeddingProvider>,
        vector_store_provider: Arc<dyn VectorStoreProvider>,
    ) -> Self {
        let config = HybridSearchConfig::from_env();
        let hybrid_search_engine = if config.enabled {
            HybridSearchEngine::new(config.bm25_weight, config.semantic_weight)
        } else {
            HybridSearchEngine::new(0.0, 1.0) // Semantic-only fallback
        };

        Self {
            embedding_provider,
            vector_store_provider,
            hybrid_search_engine: Arc::new(std::sync::RwLock::new(hybrid_search_engine)),
            indexed_documents: Arc::new(std::sync::RwLock::new(HashMap::new())),
        }
    }

    /// Generate embeddings for text
    pub async fn embed_text(&self, text: &str) -> Result<Embedding> {
        self.embedding_provider.embed(text).await
    }

    /// Generate embeddings for multiple texts
    pub async fn embed_texts(&self, texts: &[String]) -> Result<Vec<Embedding>> {
        self.embedding_provider.embed_batch(texts).await
    }

    /// Store code chunks in vector database
    pub async fn store_chunks(&self, collection: &str, chunks: &[CodeChunk]) -> Result<()> {
        let texts: Vec<String> = chunks.iter().map(|c| c.content.clone()).collect();
        let embeddings = self.embed_texts(&texts).await?;

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
                meta
            })
            .collect();

        // Ensure collection exists
        if !self
            .vector_store_provider
            .collection_exists(collection)
            .await?
        {
            self.vector_store_provider
                .create_collection(collection, self.embedding_dimensions())
                .await?;
        }

        self.vector_store_provider
            .insert_vectors(collection, &embeddings, metadata)
            .await?;

        // Index documents for hybrid search (BM25)
        self.index_chunks_for_hybrid_search(collection, chunks)
            .await?;

        Ok(())
    }

    /// Search for similar code chunks using hybrid search (BM25 + semantic embeddings)
    pub async fn search_similar(
        &self,
        collection: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        let query_embedding = self.embed_text(query).await?;

        // Get semantic search results (expanded limit for better hybrid ranking)
        let expanded_limit = (limit * 2).clamp(20, 100); // Get more results for hybrid ranking
        let semantic_results = self
            .vector_store_provider
            .search_similar(collection, &query_embedding.vector, expanded_limit, None)
            .await?;

        let semantic_search_results: Vec<SearchResult> = semantic_results
            .into_iter()
            .map(|result| SearchResult {
                file_path: result
                    .metadata
                    .get("file_path")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                line_number: result
                    .metadata
                    .get("start_line")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32,
                content: result
                    .metadata
                    .get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                score: result.score,
                metadata: result.metadata,
            })
            .collect();

        // Apply hybrid search if available
        let hybrid_engine = lock_rwlock_read(&self.hybrid_search_engine, "ContextService::search_similar")?;
        if hybrid_engine.has_bm25_index() {
            let hybrid_results =
                hybrid_engine.hybrid_search(query, semantic_search_results, limit)?;

            Ok(hybrid_results
                .into_iter()
                .map(|hybrid_result| {
                    let mut result = hybrid_result.result;
                    // Update score to hybrid score and add BM25 metadata
                    result.score = hybrid_result.hybrid_score;

                    // Create new metadata object with hybrid scores
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
        } else {
            // Fallback to semantic search only
            Ok(semantic_search_results.into_iter().take(limit).collect())
        }
    }

    /// Clear a collection
    pub async fn clear_collection(&self, collection: &str) -> Result<()> {
        self.vector_store_provider
            .delete_collection(collection)
            .await?;
        self.clear_indexed_documents(collection).await?;
        Ok(())
    }

    /// Get embedding dimensions
    pub fn embedding_dimensions(&self) -> usize {
        self.embedding_provider.dimensions()
    }

    /// Index chunks for hybrid search (BM25)
    async fn index_chunks_for_hybrid_search(
        &self,
        collection: &str,
        chunks: &[CodeChunk],
    ) -> Result<()> {
        let mut indexed_docs = lock_rwlock_write(&self.indexed_documents, "ContextService::index_chunks_for_hybrid_search")?;
        let collection_docs = indexed_docs.entry(collection.to_string()).or_default();

        // Add new chunks to the collection
        collection_docs.extend(chunks.iter().cloned());

        // Update BM25 index
        let mut engine = lock_rwlock_write(&self.hybrid_search_engine, "ContextService::index_chunks_for_hybrid_search")?;
        engine.index_documents(collection_docs.clone());

        Ok(())
    }

    /// Clear indexed documents for a collection
    pub async fn clear_indexed_documents(&self, collection: &str) -> Result<()> {
        let mut indexed_docs = lock_rwlock_write(&self.indexed_documents, "ContextService::clear_indexed_documents")?;
        indexed_docs.remove(collection);

        // Rebuild BM25 index without this collection
        let all_docs: Vec<CodeChunk> = indexed_docs.values().flatten().cloned().collect();
        let mut engine = lock_rwlock_write(&self.hybrid_search_engine, "ContextService::clear_indexed_documents")?;
        engine.index_documents(all_docs);

        Ok(())
    }

    /// Get hybrid search statistics
    pub async fn get_hybrid_search_stats(&self) -> HashMap<String, serde_json::Value> {
        let engine = match lock_rwlock_read(&self.hybrid_search_engine, "ContextService::get_hybrid_search_stats") {
            Ok(e) => e,
            Err(_) => return HashMap::new(),
        };
        let mut stats = HashMap::new();

        stats.insert("hybrid_search_enabled".to_string(), serde_json::json!(true));
        stats.insert(
            "bm25_index_available".to_string(),
            serde_json::json!(engine.has_bm25_index()),
        );

        if let Some(bm25_stats) = engine.get_bm25_stats() {
            stats.extend(bm25_stats);
        }

        let indexed_docs = match lock_rwlock_read(&self.indexed_documents, "ContextService::get_hybrid_search_stats") {
            Ok(d) => d,
            Err(_) => return stats,
        };
        stats.insert(
            "total_collections".to_string(),
            serde_json::json!(indexed_docs.len()),
        );
        stats.insert(
            "total_indexed_documents".to_string(),
            serde_json::json!(indexed_docs.values().map(|v| v.len()).sum::<usize>()),
        );

        stats
    }
}

/// Generic context service using Strategy pattern with trait bounds
///
/// This service uses compile-time generic parameters instead of dynamic dispatch,
/// implementing the Strategy pattern for better performance and type safety.
pub struct GenericContextService<E, V>
where
    E: EmbeddingProvider + Send + Sync,
    V: VectorStoreProvider + Send + Sync,
{
    embedding_provider: Arc<E>,
    vector_store_provider: Arc<V>,
    hybrid_search_engine: Arc<std::sync::RwLock<HybridSearchEngine>>,
    indexed_documents: Arc<std::sync::RwLock<HashMap<String, Vec<CodeChunk>>>>,
}

impl<E, V> GenericContextService<E, V>
where
    E: EmbeddingProvider + Send + Sync,
    V: VectorStoreProvider + Send + Sync,
{
    /// Create a new generic context service with specified provider strategies
    pub fn new(embedding_provider: Arc<E>, vector_store_provider: Arc<V>) -> Self {
        let config = HybridSearchConfig::from_env();
        let hybrid_search_engine = if config.enabled {
            HybridSearchEngine::new(config.bm25_weight, config.semantic_weight)
        } else {
            HybridSearchEngine::new(0.0, 1.0) // Semantic-only fallback
        };

        Self {
            embedding_provider,
            vector_store_provider,
            hybrid_search_engine: Arc::new(std::sync::RwLock::new(hybrid_search_engine)),
            indexed_documents: Arc::new(std::sync::RwLock::new(HashMap::new())),
        }
    }

    /// Generate embeddings for text using the embedding strategy
    pub async fn embed_text(&self, text: &str) -> Result<Embedding> {
        self.embedding_provider.embed(text).await
    }

    /// Generate embeddings for multiple texts using the embedding strategy
    pub async fn embed_texts(&self, texts: &[String]) -> Result<Vec<Embedding>> {
        self.embedding_provider.embed_batch(texts).await
    }

    /// Search for similar code chunks using hybrid search with both strategies
    pub async fn search_similar(
        &self,
        collection: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        let query_embedding = self.embedding_provider.embed(query).await?;

        // Get semantic search results using the vector store strategy
        let expanded_limit = (limit * 2).clamp(20, 100);
        let semantic_results = self
            .vector_store_provider
            .search_similar(collection, &query_embedding.vector, expanded_limit, None)
            .await?;

        let semantic_search_results: Vec<SearchResult> = semantic_results
            .into_iter()
            .map(|result| SearchResult {
                file_path: result
                    .metadata
                    .get("file_path")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                line_number: result
                    .metadata
                    .get("start_line")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32,
                content: result
                    .metadata
                    .get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                score: result.score,
                metadata: result.metadata,
            })
            .collect();

        // Apply hybrid search if available
        let hybrid_engine = lock_rwlock_read(&self.hybrid_search_engine, "GenericContextService::search_similar")?;
        if hybrid_engine.has_bm25_index() {
            let hybrid_results =
                hybrid_engine.hybrid_search(query, semantic_search_results, limit)?;

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
        } else {
            Ok(semantic_search_results.into_iter().take(limit).collect())
        }
    }

    /// Get embedding dimensions from the embedding strategy
    pub fn embedding_dimensions(&self) -> usize {
        self.embedding_provider.dimensions()
    }

    /// Get hybrid search statistics
    pub fn get_hybrid_search_stats(&self) -> HashMap<String, serde_json::Value> {
        let engine = match lock_rwlock_read(&self.hybrid_search_engine, "GenericContextService::get_hybrid_search_stats") {
            Ok(e) => e,
            Err(_) => return HashMap::new(),
        };
        let mut stats = HashMap::new();

        stats.insert("hybrid_search_enabled".to_string(), serde_json::json!(true));
        stats.insert(
            "bm25_index_available".to_string(),
            serde_json::json!(engine.has_bm25_index()),
        );

        if let Some(bm25_stats) = engine.get_bm25_stats() {
            stats.extend(bm25_stats);
        }

        let indexed_docs = match lock_rwlock_read(&self.indexed_documents, "GenericContextService::get_hybrid_search_stats") {
            Ok(d) => d,
            Err(_) => return stats,
        };
        stats.insert(
            "total_collections".to_string(),
            serde_json::json!(indexed_docs.len()),
        );
        stats.insert(
            "total_indexed_documents".to_string(),
            serde_json::json!(indexed_docs.values().map(|v| v.len()).sum::<usize>()),
        );

        stats
    }
}

impl Default for ContextService {
    fn default() -> Self {
        // Use mock providers for default
        let embedding_provider = Arc::new(crate::providers::MockEmbeddingProvider::new());
        let vector_store_provider = Arc::new(crate::providers::InMemoryVectorStoreProvider::new());
        Self::new(embedding_provider, vector_store_provider)
    }
}
