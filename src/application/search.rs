//! Search service for querying indexed code

use crate::domain::error::Result;
use crate::domain::ports::{ContextServiceInterface, SearchServiceInterface};
use crate::domain::types::SearchResult;
use async_trait::async_trait;
use std::sync::Arc;

/// Simple search service for MVP
pub struct SearchService {
    context_service: Arc<dyn ContextServiceInterface>,
}

impl SearchService {
    /// Create a new search service
    pub fn new(context_service: Arc<dyn ContextServiceInterface>) -> Self {
        Self { context_service }
    }

    /// Search for code similar to the query
    pub async fn search(
        &self,
        collection: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        self.context_service
            .search_similar(collection, query, limit)
            .await
    }
}

#[async_trait]
impl SearchServiceInterface for SearchService {
    async fn search(
        &self,
        collection: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        self.search(collection, query, limit).await
    }
}
