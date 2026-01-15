//! Indexing Domain Service Interface
//!
//! Interface for the indexing service that handles code ingestion,
//! processing, and indexing operations.

use crate::entities::CodeChunk;
use crate::error::Result;
use crate::value_objects::{config::SyncBatch, types::OperationType};
use async_trait::async_trait;
use shaku::Interface;
use std::path::Path;

/// Domain Service: Code Indexing Operations
///
/// Defines the interface for indexing services that handle the ingestion,
/// processing, and indexing of code into the semantic search system.
#[async_trait]
pub trait IndexingServiceInterface: Interface + Send + Sync {
    /// Index a batch of code chunks
    ///
    /// Processes and stores code chunks in the vector database,
    /// making them available for semantic search.
    async fn index_chunks(&self, chunks: &[CodeChunk]) -> Result<()>;

    /// Index files from a directory
    ///
    /// Recursively processes all files in a directory, extracting
    /// code chunks and indexing them.
    async fn index_directory(&self, path: &Path) -> Result<()>;

    /// Process a synchronization batch
    ///
    /// Handles incremental updates from file system changes.
    async fn process_sync_batch(&self, batch: &SyncBatch) -> Result<()>;

    /// Rebuild index for a collection
    ///
    /// Completely rebuilds the index for a given collection,
    /// useful for recovery or schema changes.
    async fn rebuild_index(&self, collection: &str) -> Result<()>;

    /// Get indexing statistics
    ///
    /// Returns current indexing metrics and status.
    async fn get_indexing_stats(&self) -> Result<IndexingStats>;
}

/// Value Object: Indexing Operation Result
#[derive(Debug, Clone)]
pub struct IndexingResult {
    /// Number of chunks successfully indexed
    pub chunks_indexed: usize,
    /// Number of chunks that failed to index
    pub chunks_failed: usize,
    /// Total processing time in milliseconds
    pub processing_time_ms: u64,
    /// Any errors that occurred during indexing
    pub errors: Vec<String>,
}

/// Value Object: Indexing Status
#[derive(Debug, Clone)]
pub struct IndexingStatus {
    /// Whether indexing is currently running
    pub is_indexing: bool,
    /// Current operation being performed
    pub current_operation: Option<OperationType>,
    /// Progress percentage (0.0 to 1.0)
    pub progress: f64,
    /// Estimated time remaining in seconds
    pub estimated_time_remaining: Option<u64>,
}

/// Value Object: Indexing Statistics
#[derive(Debug, Clone)]
pub struct IndexingStats {
    /// Total number of chunks indexed
    pub total_chunks: u64,
    /// Total number of collections
    pub total_collections: u64,
    /// Last indexing operation timestamp
    pub last_indexed_at: Option<i64>,
    /// Average indexing throughput (chunks per second)
    pub avg_throughput: f64,
}