use crate::error::Result;
use crate::value_objects::{Embedding, SearchResult};
use async_trait::async_trait;
use serde_json::Value;
use shaku::Interface;
use std::collections::HashMap;

/// Enterprise Vector Storage Interface
///
/// Defines the business contract for vector storage systems that persist and
/// retrieve semantic embeddings at enterprise scale. This abstraction supports
/// multiple storage backends from in-memory development stores to production
/// Milvus clusters, ensuring optimal performance for different business needs.
#[async_trait]
pub trait VectorStoreProvider: Interface + Send + Sync {
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

    /// Check if a collection exists
    ///
    /// # Arguments
    /// * `name` - Name of the collection to check
    ///
    /// # Returns
    /// Ok(true) if collection exists, Ok(false) if it doesn't exist, Error if check failed
    async fn collection_exists(&self, name: &str) -> Result<bool>;

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
