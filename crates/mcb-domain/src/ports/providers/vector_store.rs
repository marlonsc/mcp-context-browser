use crate::error::Result;
use crate::value_objects::{CollectionInfo, Embedding, FileInfo, SearchResult};
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

/// Vector Store Administrative Operations
///
/// Defines administrative and monitoring operations for vector stores.
/// This trait is separated to keep trait sizes manageable per SOLID principles.
///
/// # Example
///
/// ```ignore
/// use mcb_domain::ports::providers::VectorStoreAdmin;
///
/// // Check if a collection exists
/// if provider.collection_exists("code_embeddings").await? {
///     let stats = provider.get_stats("code_embeddings").await?;
///     println!("Collection has {} vectors", stats["total_count"]);
///
///     // Flush pending writes
///     provider.flush("code_embeddings").await?;
/// }
/// ```
#[async_trait]
pub trait VectorStoreAdmin: Send + Sync {
    /// Check if a collection exists
    ///
    /// # Arguments
    /// * `name` - Name of the collection to check
    ///
    /// # Returns
    /// Ok(true) if collection exists, Ok(false) if it doesn't exist, Error if check failed
    async fn collection_exists(&self, name: &str) -> Result<bool>;

    /// Get statistics about a collection
    ///
    /// # Arguments
    /// * `collection` - Name of the collection to get stats for
    ///
    /// # Returns
    /// Ok(hashmap) containing various statistics about the collection
    async fn get_stats(&self, collection: &str) -> Result<HashMap<String, Value>>;

    /// Flush pending operations for a collection
    ///
    /// # Arguments
    /// * `collection` - Name of the collection to flush
    ///
    /// # Returns
    /// Ok(()) if flush completed successfully, Error if flush failed
    async fn flush(&self, collection: &str) -> Result<()>;

    /// Get the name/identifier of this vector store provider
    ///
    /// # Returns
    /// A string identifier for the provider (e.g., "milvus", "edgevec", "null")
    fn provider_name(&self) -> &str;

    /// Health check for the provider (default implementation)
    async fn health_check(&self) -> Result<()> {
        // Default implementation - try a simple operation
        self.collection_exists("__health_check__").await?;
        Ok(())
    }
}

/// Enterprise Vector Storage Interface
///
/// Defines the business contract for vector storage systems that persist and
/// retrieve semantic embeddings at enterprise scale. This abstraction supports
/// multiple storage backends from in-memory development stores to production
/// Milvus clusters, ensuring optimal performance for different business needs.
///
/// # Example
///
/// ```ignore
/// use mcb_domain::ports::providers::VectorStoreProvider;
///
/// // Create a collection for code embeddings
/// provider.create_collection("rust_code", 1536).await?;
///
/// // Insert vectors with metadata
/// let metadata = vec![hashmap!{ "file_path" => "src/main.rs".into() }];
/// let ids = provider.insert_vectors("rust_code", &embeddings, metadata).await?;
///
/// // Search for similar code
/// let results = provider.search_similar("rust_code", &query_vec, 10, None).await?;
/// for result in results {
///     println!("Found: {} (score: {})", result.file_path, result.score);
/// }
/// ```
#[async_trait]
pub trait VectorStoreProvider: VectorStoreAdmin + Send + Sync {
    /// Create a new vector collection with specified dimensions
    ///
    /// # Arguments
    /// * `name` - Name of the collection to create
    /// * `dimensions` - Number of dimensions for vectors in this collection
    ///
    /// # Returns
    /// Ok(()) if collection was created successfully, Error if creation failed
    async fn create_collection(&self, name: &str, dimensions: usize) -> Result<()>;

    /// Delete an existing vector collection
    ///
    /// # Arguments
    /// * `name` - Name of the collection to delete
    ///
    /// # Returns
    /// Ok(()) if collection was deleted successfully, Error if deletion failed
    async fn delete_collection(&self, name: &str) -> Result<()>;

    /// Insert vectors into a collection with associated metadata
    ///
    /// # Arguments
    /// * `collection` - Name of the collection to insert into
    /// * `vectors` - Slice of embedding vectors to insert
    /// * `metadata` - Vector of metadata maps, one per vector
    ///
    /// # Returns
    /// Ok(vector_of_ids) containing the IDs assigned to each inserted vector
    async fn insert_vectors(
        &self,
        collection: &str,
        vectors: &[Embedding],
        metadata: Vec<HashMap<String, Value>>,
    ) -> Result<Vec<String>>;

    /// Search for vectors similar to a query vector
    ///
    /// # Arguments
    /// * `collection` - Name of the collection to search in
    /// * `query_vector` - The query vector to find similar vectors for
    /// * `limit` - Maximum number of results to return
    /// * `filter` - Optional filter expression to restrict search scope
    ///
    /// # Returns
    /// Ok(vector_of_results) containing the search results ordered by similarity
    async fn search_similar(
        &self,
        collection: &str,
        query_vector: &[f32],
        limit: usize,
        filter: Option<&str>,
    ) -> Result<Vec<SearchResult>>;

    /// Delete vectors by their IDs
    ///
    /// # Arguments
    /// * `collection` - Name of the collection to delete from
    /// * `ids` - Slice of vector IDs to delete
    ///
    /// # Returns
    /// Ok(()) if all vectors were deleted successfully, Error if deletion failed
    async fn delete_vectors(&self, collection: &str, ids: &[String]) -> Result<()>;

    /// Retrieve vectors by their IDs
    ///
    /// # Arguments
    /// * `collection` - Name of the collection to retrieve from
    /// * `ids` - Slice of vector IDs to retrieve
    ///
    /// # Returns
    /// Ok(vector_of_results) containing the requested vectors with their metadata
    async fn get_vectors_by_ids(
        &self,
        collection: &str,
        ids: &[String],
    ) -> Result<Vec<SearchResult>>;

    /// List vectors in a collection with pagination
    ///
    /// # Arguments
    /// * `collection` - Name of the collection to list vectors from
    /// * `limit` - Maximum number of vectors to return
    ///
    /// # Returns
    /// Ok(vector_of_results) containing the vectors in the collection
    async fn list_vectors(&self, collection: &str, limit: usize) -> Result<Vec<SearchResult>>;
}

/// Vector Store Browse Operations for Admin UI
///
/// Provides collection and file browsing capabilities for the Admin UI.
/// This trait extends the base vector store functionality with navigation
/// operations useful for exploring indexed codebases.
///
/// # Example
///
/// ```ignore
/// use mcb_domain::ports::providers::VectorStoreBrowser;
///
/// // List all indexed collections
/// let collections = provider.list_collections().await?;
/// for coll in collections {
///     println!("Collection: {} ({} vectors)", coll.name, coll.vector_count);
/// }
///
/// // List files in a collection
/// let files = provider.list_file_paths("my-project", 100).await?;
/// for file in files {
///     println!("File: {}", file.path);
/// }
///
/// // Get chunks for a specific file
/// let chunks = provider.get_chunks_by_file("my-project", "src/main.rs").await?;
/// for chunk in chunks {
///     println!("Chunk {}: lines {}-{}", chunk.id, chunk.start_line, chunk.end_line);
/// }
/// ```
#[async_trait]
pub trait VectorStoreBrowser: Send + Sync {
    /// List all collections with their statistics
    ///
    /// Returns metadata about all indexed collections, including
    /// vector counts, file counts, and provider information.
    ///
    /// # Returns
    /// Ok(vector_of_collection_info) containing info about all collections
    async fn list_collections(&self) -> Result<Vec<CollectionInfo>>;

    /// List unique file paths in a collection
    ///
    /// Returns information about all files indexed in a collection,
    /// useful for building file browser UIs.
    ///
    /// # Arguments
    /// * `collection` - Name of the collection to list files from
    /// * `limit` - Maximum number of files to return
    ///
    /// # Returns
    /// Ok(vector_of_file_info) containing info about indexed files
    async fn list_file_paths(&self, collection: &str, limit: usize) -> Result<Vec<FileInfo>>;

    /// Get all chunks for a specific file path
    ///
    /// Retrieves all code chunks that were extracted from a specific
    /// file, ordered by line number.
    ///
    /// # Arguments
    /// * `collection` - Name of the collection to search in
    /// * `file_path` - Path of the file to get chunks for
    ///
    /// # Returns
    /// Ok(vector_of_results) containing chunks from the specified file
    async fn get_chunks_by_file(
        &self,
        collection: &str,
        file_path: &str,
    ) -> Result<Vec<SearchResult>>;
}
