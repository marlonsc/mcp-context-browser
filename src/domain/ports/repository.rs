//! Repository Ports
//!
//! Abstractions for persistent storage of code chunks and search indices.
//!
//! The repository pattern separates business logic from data access, allowing
//! different storage backends (database, filesystem, cloud storage) without
//! changing the application code.

use crate::domain::error::Result;
use crate::domain::types::{CodeChunk, RepositoryStats, SearchResult, SearchStats};
use async_trait::async_trait;
use shaku::Interface;

/// Repository for managing code chunks
///
/// Persists extracted code chunks and provides retrieval by collection or ID.
/// Chunks are the granular unit of semantic indexing - each chunk can be
/// independently searched, updated, or deleted.
///
/// # Example
///
/// ```rust,no_run
/// use mcp_context_browser::domain::ports::ChunkRepository;
/// use mcp_context_browser::domain::types::{CodeChunk, Language};
///
/// # async fn example(repo: &dyn ChunkRepository) -> anyhow::Result<()> {
/// // Create a code chunk
/// let chunk = CodeChunk {
///     id: "chunk_001".to_string(),
///     content: "fn hello() { println!(\"Hello!\"); }".to_string(),
///     file_path: "src/lib.rs".to_string(),
///     start_line: 1,
///     end_line: 3,
///     language: Language::Rust,
///     metadata: serde_json::json!({"type": "function"}),
/// };
///
/// // Save to collection
/// let collection = "my-codebase";
/// let id = repo.save(collection, &chunk).await?;
///
/// // Retrieve chunk
/// if let Some(retrieved) = repo.find_by_id(collection, &id).await? {
///     assert_eq!(retrieved.content, chunk.content);
/// }
///
/// // List all chunks in collection
/// let chunks = repo.find_by_collection(collection, 100).await?;
/// assert!(!chunks.is_empty());
///
/// // Get statistics
/// let stats = repo.stats().await?;
/// assert!(stats.total_chunks > 0);
/// # Ok(())
/// # }
/// ```
#[async_trait]
pub trait ChunkRepository: Interface + Send + Sync {
    /// Save a single code chunk to the repository
    ///
    /// # Arguments
    /// - `collection`: Collection/namespace identifier
    /// - `chunk`: Code chunk to persist
    ///
    /// # Returns
    /// ID assigned to the chunk for later retrieval/deletion
    async fn save(&self, collection: &str, chunk: &CodeChunk) -> Result<String>;

    /// Save multiple code chunks efficiently
    ///
    /// # Arguments
    /// - `collection`: Collection/namespace identifier
    /// - `chunks`: Batch of chunks to persist
    ///
    /// # Returns
    /// IDs assigned to each chunk in the same order
    async fn save_batch(&self, collection: &str, chunks: &[CodeChunk]) -> Result<Vec<String>>;

    /// Retrieve a specific chunk by ID
    ///
    /// # Returns
    /// `Some(chunk)` if found, `None` if not found
    async fn find_by_id(&self, collection: &str, id: &str) -> Result<Option<CodeChunk>>;

    /// List chunks in a collection (paginated)
    ///
    /// # Arguments
    /// - `collection`: Collection/namespace identifier
    /// - `limit`: Maximum chunks to return (implementation may return fewer)
    ///
    /// # Returns
    /// Vector of chunks in the collection, up to the limit
    async fn find_by_collection(&self, collection: &str, limit: usize) -> Result<Vec<CodeChunk>>;

    /// Delete a specific chunk by ID
    ///
    /// # Arguments
    /// - `collection`: Collection/namespace identifier
    /// - `id`: Unique chunk identifier to delete
    async fn delete(&self, collection: &str, id: &str) -> Result<()>;

    /// Delete all chunks in a collection
    ///
    /// # Arguments
    /// - `collection`: Collection/namespace identifier to clear completely
    async fn delete_collection(&self, collection: &str) -> Result<()>;

    /// Get repository storage statistics
    ///
    /// # Returns
    /// Stats including chunk counts, collections, and storage size
    async fn stats(&self) -> Result<RepositoryStats>;
}

/// Repository for managing search operations and results
///
/// Implements semantic search (vector similarity) and hybrid search
/// (combining semantic + keyword/BM25 ranking). Encapsulates all
/// search query execution.
///
/// # Example
///
/// ```rust,no_run
/// use mcp_context_browser::domain::ports::SearchRepository;
///
/// # async fn example(repo: &dyn SearchRepository) -> anyhow::Result<()> {
/// // Semantic search with embedding vector
/// let query_vector = vec![0.1, 0.2, 0.3, 0.4]; // Example embedding
/// let results = repo.semantic_search("my-codebase", &query_vector, 10, None).await?;
/// for result in results {
///     println!("Found: {} (score: {})", result.file_path, result.score);
/// }
///
/// // Hybrid search: semantic + keyword combined
/// let hybrid_results = repo.hybrid_search(
///     "my-codebase",
///     "authentication logic",  // Natural language query
///     &query_vector,
///     10
/// ).await?;
/// # Ok(())
/// # }
/// ```
#[async_trait]
pub trait SearchRepository: Interface + Send + Sync {
    /// Semantic search using vector similarity
    ///
    /// # Arguments
    /// - `collection`: Collection to search
    /// - `query_vector`: Embedding vector from the query
    /// - `limit`: Maximum results to return
    /// - `filter`: Optional metadata filter predicate
    ///
    /// # Returns
    /// Results ranked by similarity score (highest first)
    async fn semantic_search(
        &self,
        collection: &str,
        query_vector: &[f32],
        limit: usize,
        filter: Option<&str>,
    ) -> Result<Vec<SearchResult>>;

    /// Index chunks for keyword/BM25 search
    ///
    /// Called during indexing to make chunks searchable via keyword matching.
    async fn index_for_hybrid_search(&self, chunks: &[CodeChunk]) -> Result<()>;

    /// Hybrid search combining semantic and keyword relevance
    ///
    /// # Arguments
    /// - `collection`: Collection to search
    /// - `query`: Natural language query text
    /// - `query_vector`: Embedding of the query
    /// - `limit`: Maximum results
    ///
    /// # Returns
    /// Results ranked by combined semantic + BM25 score
    async fn hybrid_search(
        &self,
        collection: &str,
        query: &str,
        query_vector: &[f32],
        limit: usize,
    ) -> Result<Vec<SearchResult>>;

    /// Clear keyword search index for a collection
    ///
    /// # Arguments
    /// - `collection`: Collection/namespace identifier to clear the search index for
    ///
    /// # Note
    /// This only affects the keyword/BM25 search index, not the semantic vector storage
    async fn clear_index(&self, collection: &str) -> Result<()>;

    /// Get search operation statistics
    ///
    /// # Returns
    /// Stats including query counts, response times, and cache hit rates
    async fn search_stats(&self) -> Result<SearchStats>;
}
