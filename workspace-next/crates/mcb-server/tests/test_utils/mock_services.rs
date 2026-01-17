//! Mock implementations of domain service interfaces for testing

use async_trait::async_trait;
use mcb_application::domain_services::search::{
    ContextServiceInterface, IndexingResult, IndexingServiceInterface, IndexingStatus,
    SearchServiceInterface,
};
use mcb_domain::entities::CodeChunk;
use mcb_domain::error::Result;
use mcb_domain::value_objects::{Embedding, SearchResult};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

// ============================================================================
// Mock Search Service
// ============================================================================

/// Mock implementation of SearchServiceInterface for testing
pub struct MockSearchService {
    /// Pre-configured results to return
    results: Arc<Mutex<Vec<SearchResult>>>,
    /// Whether the next call should fail
    should_fail: Arc<AtomicBool>,
    /// Error message to return on failure
    error_message: Arc<Mutex<String>>,
}

impl MockSearchService {
    /// Create a new mock search service
    pub fn new() -> Self {
        Self {
            results: Arc::new(Mutex::new(Vec::new())),
            should_fail: Arc::new(AtomicBool::new(false)),
            error_message: Arc::new(Mutex::new("Simulated search failure".to_string())),
        }
    }

    /// Configure the mock to return specific results
    pub fn with_results(mut self, results: Vec<SearchResult>) -> Self {
        *self.results.lock().expect("Lock poisoned") = results;
        self
    }

    /// Configure the mock to fail on next call
    pub fn with_failure(self, message: &str) -> Self {
        self.should_fail.store(true, Ordering::SeqCst);
        *self.error_message.lock().expect("Lock poisoned") = message.to_string();
        self
    }
}

impl Default for MockSearchService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SearchServiceInterface for MockSearchService {
    async fn search(
        &self,
        _collection: &str,
        _query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        if self.should_fail.load(Ordering::SeqCst) {
            let msg = self.error_message.lock().expect("Lock poisoned").clone();
            return Err(mcb_domain::error::Error::internal(msg));
        }

        let results = self.results.lock().expect("Lock poisoned");
        Ok(results.iter().take(limit).cloned().collect())
    }
}

// ============================================================================
// Mock Indexing Service
// ============================================================================

/// Mock implementation of IndexingServiceInterface for testing
pub struct MockIndexingService {
    /// Pre-configured indexing result
    indexing_result: Arc<Mutex<Option<IndexingResult>>>,
    /// Current status to return
    status: Arc<Mutex<IndexingStatus>>,
    /// Whether the next indexing call should fail
    should_fail: Arc<AtomicBool>,
    /// Error message to return on failure
    error_message: Arc<Mutex<String>>,
}

impl MockIndexingService {
    /// Create a new mock indexing service
    pub fn new() -> Self {
        Self {
            indexing_result: Arc::new(Mutex::new(Some(IndexingResult {
                files_processed: 0,
                chunks_created: 0,
                files_skipped: 0,
                errors: Vec::new(),
            }))),
            status: Arc::new(Mutex::new(IndexingStatus::default())),
            should_fail: Arc::new(AtomicBool::new(false)),
            error_message: Arc::new(Mutex::new("Simulated indexing failure".to_string())),
        }
    }

    /// Configure the mock to return specific indexing result
    pub fn with_result(self, result: IndexingResult) -> Self {
        *self.indexing_result.lock().expect("Lock poisoned") = Some(result);
        self
    }

    /// Configure the mock to return specific status
    pub fn with_status(self, status: IndexingStatus) -> Self {
        *self.status.lock().expect("Lock poisoned") = status;
        self
    }

    /// Configure the mock to fail on next call
    pub fn with_failure(self, message: &str) -> Self {
        self.should_fail.store(true, Ordering::SeqCst);
        *self.error_message.lock().expect("Lock poisoned") = message.to_string();
        self
    }
}

impl Default for MockIndexingService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl IndexingServiceInterface for MockIndexingService {
    async fn index_codebase(&self, _path: &Path, _collection: &str) -> Result<IndexingResult> {
        if self.should_fail.load(Ordering::SeqCst) {
            let msg = self.error_message.lock().expect("Lock poisoned").clone();
            return Err(mcb_domain::error::Error::internal(msg));
        }

        let result = self.indexing_result.lock().expect("Lock poisoned");
        Ok(result.clone().unwrap_or_else(|| IndexingResult {
            files_processed: 0,
            chunks_created: 0,
            files_skipped: 0,
            errors: Vec::new(),
        }))
    }

    fn get_status(&self) -> IndexingStatus {
        self.status.lock().expect("Lock poisoned").clone()
    }

    async fn clear_collection(&self, _collection: &str) -> Result<()> {
        if self.should_fail.load(Ordering::SeqCst) {
            let msg = self.error_message.lock().expect("Lock poisoned").clone();
            return Err(mcb_domain::error::Error::internal(msg));
        }
        Ok(())
    }
}

// ============================================================================
// Mock Context Service
// ============================================================================

/// Mock implementation of ContextServiceInterface for testing
pub struct MockContextService {
    /// Pre-configured search results
    search_results: Arc<Mutex<Vec<SearchResult>>>,
    /// Embedding dimensions
    dimensions: usize,
    /// Whether the next call should fail
    should_fail: Arc<AtomicBool>,
    /// Error message to return on failure
    error_message: Arc<Mutex<String>>,
}

impl MockContextService {
    /// Create a new mock context service
    pub fn new() -> Self {
        Self {
            search_results: Arc::new(Mutex::new(Vec::new())),
            dimensions: 384,
            should_fail: Arc::new(AtomicBool::new(false)),
            error_message: Arc::new(Mutex::new("Simulated context failure".to_string())),
        }
    }

    /// Configure the mock to return specific search results
    pub fn with_search_results(mut self, results: Vec<SearchResult>) -> Self {
        *self.search_results.lock().expect("Lock poisoned") = results;
        self
    }

    /// Configure the mock to use specific dimensions
    pub fn with_dimensions(mut self, dims: usize) -> Self {
        self.dimensions = dims;
        self
    }

    /// Configure the mock to fail on next call
    pub fn with_failure(self, message: &str) -> Self {
        self.should_fail.store(true, Ordering::SeqCst);
        *self.error_message.lock().expect("Lock poisoned") = message.to_string();
        self
    }
}

impl Default for MockContextService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ContextServiceInterface for MockContextService {
    async fn initialize(&self, _collection: &str) -> Result<()> {
        if self.should_fail.load(Ordering::SeqCst) {
            let msg = self.error_message.lock().expect("Lock poisoned").clone();
            return Err(mcb_domain::error::Error::internal(msg));
        }
        Ok(())
    }

    async fn store_chunks(&self, _collection: &str, _chunks: &[CodeChunk]) -> Result<()> {
        if self.should_fail.load(Ordering::SeqCst) {
            let msg = self.error_message.lock().expect("Lock poisoned").clone();
            return Err(mcb_domain::error::Error::internal(msg));
        }
        Ok(())
    }

    async fn search_similar(
        &self,
        _collection: &str,
        _query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        if self.should_fail.load(Ordering::SeqCst) {
            let msg = self.error_message.lock().expect("Lock poisoned").clone();
            return Err(mcb_domain::error::Error::internal(msg));
        }

        let results = self.search_results.lock().expect("Lock poisoned");
        Ok(results.iter().take(limit).cloned().collect())
    }

    async fn embed_text(&self, _text: &str) -> Result<Embedding> {
        if self.should_fail.load(Ordering::SeqCst) {
            let msg = self.error_message.lock().expect("Lock poisoned").clone();
            return Err(mcb_domain::error::Error::internal(msg));
        }

        Ok(Embedding {
            vector: vec![0.1; self.dimensions],
            model: "mock".to_string(),
            dimensions: self.dimensions,
        })
    }

    async fn clear_collection(&self, _collection: &str) -> Result<()> {
        if self.should_fail.load(Ordering::SeqCst) {
            let msg = self.error_message.lock().expect("Lock poisoned").clone();
            return Err(mcb_domain::error::Error::internal(msg));
        }
        Ok(())
    }

    async fn get_stats(&self) -> Result<(i64, i64)> {
        if self.should_fail.load(Ordering::SeqCst) {
            let msg = self.error_message.lock().expect("Lock poisoned").clone();
            return Err(mcb_domain::error::Error::internal(msg));
        }

        // Return (total_chunks, total_queries)
        Ok((100, 10))
    }

    fn embedding_dimensions(&self) -> usize {
        self.dimensions
    }
}
