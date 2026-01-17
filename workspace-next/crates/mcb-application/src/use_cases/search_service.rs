//! Search Service Use Case
//!
//! Application service for semantic search operations.
//! Orchestrates search functionality using context service for semantic understanding.

use crate::domain_services::search::{ContextServiceInterface, SearchServiceInterface};
use mcb_domain::error::Result;
use mcb_domain::value_objects::SearchResult;
use std::sync::Arc;

/// Search service implementation - delegates to context service
#[derive(shaku::Component)]
#[shaku(interface = crate::domain_services::search::SearchServiceInterface)]
pub struct SearchServiceImpl {
    #[shaku(inject)]
    context_service: Arc<dyn ContextServiceInterface>,
}

impl SearchServiceImpl {
    /// Create new search service with injected dependencies
    pub fn new(context_service: Arc<dyn ContextServiceInterface>) -> Self {
        Self { context_service }
    }
}

#[async_trait::async_trait]
impl SearchServiceInterface for SearchServiceImpl {
    async fn search(
        &self,
        collection: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        // Delegate to context service for semantic search
        // Future: add BM25 scoring and hybrid ranking
        self.context_service
            .search_similar(collection, query, limit)
            .await
    }
}