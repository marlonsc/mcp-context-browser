//! Repository Ports
//!
//! Abstractions for persistent storage of code chunks and search indices.
//!
//! The repository pattern separates business logic from data access, allowing
//! different storage backends (database, filesystem, cloud storage) without
//! changing the application code.

use crate::error::Result;
use crate::entities::CodeChunk;
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
/// use mcb_domain::ChunkRepository;
/// use mcb_domain::CodeChunk;
///
/// # async fn example(repo: &dyn ChunkRepository) -> mcb_domain::Result<()> {
/// // Create a code chunk
/// let chunk = CodeChunk {
///     id: "chunk_001".to_string(),
///     content: "fn hello() { println!(\"Hello!\"); }".to_string(),
///     file_path: "src/lib.rs".to_string(),
///     start_line: 1,
///     end_line: 3,
///     language: "rust".to_string(),
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
/// Value Object: Repository Statistics
#[derive(Debug, Clone)]
pub struct RepositoryStats {
    /// Total chunks stored across all collections
    pub total_chunks: u64,
    /// Number of collections/namespaces
    pub total_collections: u64,
    /// Total storage used in bytes
    pub storage_size_bytes: u64,
    /// Average chunk size in bytes
    pub avg_chunk_size_bytes: f64,
}
