//! Application Service Interfaces
//!
//! Defines the port interfaces for application layer services.
//! These traits are implemented by the application services and
//! used by the presentation layer (handlers).

use crate::domain::error::Result;
use crate::domain::types::{CodeChunk, Embedding, RepositoryStats, SearchResult, SearchStats};
use async_trait::async_trait;
use shaku::Interface;
use std::path::Path;

// ============================================================================
// Context Service Interface
// ============================================================================

/// Code Intelligence Service Interface
///
/// Defines the contract for semantic code understanding operations.
#[async_trait]
pub trait ContextServiceInterface: Interface + Send + Sync {
    /// Initialize the service for a collection
    async fn initialize(&self, collection: &str) -> Result<()>;

    /// Store code chunks in the repository
    async fn store_chunks(&self, collection: &str, chunks: &[CodeChunk]) -> Result<()>;

    /// Search for code similar to the query
    async fn search_similar(
        &self,
        collection: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>>;

    /// Get embedding for text
    async fn embed_text(&self, text: &str) -> Result<Embedding>;

    /// Clear/delete a collection
    async fn clear_collection(&self, collection: &str) -> Result<()>;

    /// Get combined stats for the service
    async fn get_stats(&self) -> Result<(RepositoryStats, SearchStats)>;

    /// Get embedding dimensions
    fn embedding_dimensions(&self) -> usize;
}

// ============================================================================
// Search Service Interface
// ============================================================================

/// Search Service Interface
///
/// Simplified search interface for code queries.
#[async_trait]
pub trait SearchServiceInterface: Interface + Send + Sync {
    /// Search for code similar to the query
    async fn search(
        &self,
        collection: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>>;
}

// ============================================================================
// Indexing Service Interface
// ============================================================================

/// Indexing Service Interface
///
/// Defines the contract for codebase indexing operations.
#[async_trait]
pub trait IndexingServiceInterface: Interface + Send + Sync {
    /// Index a codebase at the given path
    async fn index_codebase(&self, path: &Path, collection: &str) -> Result<IndexingResult>;

    /// Get the current indexing status
    fn get_status(&self) -> IndexingStatus;

    /// Clear all indexed data from a collection
    async fn clear_collection(&self, collection: &str) -> Result<()>;
}

/// Result of an indexing operation
#[derive(Debug, Clone)]
pub struct IndexingResult {
    /// Number of files processed
    pub files_processed: usize,
    /// Number of chunks created
    pub chunks_created: usize,
    /// Number of files skipped
    pub files_skipped: usize,
    /// Any errors encountered (non-fatal)
    pub errors: Vec<String>,
}

/// Current indexing status
#[derive(Debug, Clone, Default)]
pub struct IndexingStatus {
    /// Whether indexing is currently in progress
    pub is_indexing: bool,
    /// Current progress (0.0 to 1.0)
    pub progress: f64,
    /// Current file being processed
    pub current_file: Option<String>,
    /// Total files to process
    pub total_files: usize,
    /// Files processed so far
    pub processed_files: usize,
}

// ============================================================================
// Chunking Orchestrator Interface
// ============================================================================

/// Chunking Orchestrator Interface
///
/// Coordinates batch code chunking operations.
#[async_trait]
pub trait ChunkingOrchestratorInterface: Interface + Send + Sync {
    /// Process multiple files and return chunks
    async fn process_files(&self, files: Vec<(String, String)>) -> Result<Vec<CodeChunk>>;

    /// Process a single file
    async fn process_file(&self, path: &Path, content: &str) -> Result<Vec<CodeChunk>>;
}
