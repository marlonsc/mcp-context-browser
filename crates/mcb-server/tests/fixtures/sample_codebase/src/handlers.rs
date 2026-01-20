//! MCP protocol handlers for code browser operations
//!
//! This module contains HTTP handlers that implement the MCP protocol
//! for indexing and searching code.

use std::sync::Arc;

/// MCP protocol handler for search operations
pub struct SearchHandler {
    search_service: Arc<dyn SearchService>,
}

impl SearchHandler {
    pub fn new(search_service: Arc<dyn SearchService>) -> Self {
        Self { search_service }
    }

    /// Handle MCP search request
    pub async fn handle_search(&self, request: SearchRequest) -> Result<SearchResponse, Error> {
        let results = self.search_service.search(&request.query, request.limit).await?;
        Ok(SearchResponse { results })
    }
}

/// MCP protocol handler for indexing operations
pub struct IndexHandler {
    indexing_service: Arc<dyn IndexingService>,
}

impl IndexHandler {
    pub fn new(indexing_service: Arc<dyn IndexingService>) -> Self {
        Self { indexing_service }
    }

    /// Handle MCP index codebase request
    pub async fn handle_index(&self, request: IndexRequest) -> Result<IndexResponse, Error> {
        let stats = self.indexing_service.index_codebase(&request.path, &request.collection).await?;
        Ok(IndexResponse { stats })
    }
}

/// Search request from MCP protocol
pub struct SearchRequest {
    pub query: String,
    pub limit: usize,
    pub collection: String,
}

/// Search response for MCP protocol
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
}

/// Index request from MCP protocol
pub struct IndexRequest {
    pub path: String,
    pub collection: String,
}

/// Index response for MCP protocol
pub struct IndexResponse {
    pub stats: IndexingStats,
}

/// Search result item
pub struct SearchResult {
    pub file_path: String,
    pub content: String,
    pub score: f32,
}

/// Indexing statistics
pub struct IndexingStats {
    pub files_indexed: usize,
    pub chunks_created: usize,
}

/// Search service trait
pub trait SearchService: Send + Sync {
    fn search(&self, query: &str, limit: usize) -> impl std::future::Future<Output = Result<Vec<SearchResult>, Error>> + Send;
}

/// Indexing service trait
pub trait IndexingService: Send + Sync {
    fn index_codebase(&self, path: &str, collection: &str) -> impl std::future::Future<Output = Result<IndexingStats, Error>> + Send;
}

/// Error type for handlers
#[derive(Debug)]
pub struct Error {
    pub message: String,
}
