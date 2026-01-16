//! Tests for Domain Service Interfaces
//!
//! Validates the contracts of the domain service interfaces used by the application layer.

use async_trait::async_trait;
use mcb_domain::domain_services::search::{
    ChunkingOrchestratorInterface, ContextServiceInterface, IndexingResult, IndexingServiceInterface,
    IndexingStatus, SearchServiceInterface,
};
use mcb_domain::entities::CodeChunk;
use mcb_domain::error::Result;
use mcb_domain::repositories::{chunk_repository::RepositoryStats, search_repository::SearchStats};
use mcb_domain::value_objects::{Embedding, SearchResult};
use std::path::Path;
use std::sync::Arc;

// ============================================================================
// Mock Search Service
// ============================================================================

struct MockSearchService {
    should_fail: bool,
}

impl MockSearchService {
    fn new() -> Self {
        Self { should_fail: false }
    }

    fn failing() -> Self {
        Self { should_fail: true }
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
        if self.should_fail {
            return Err(mcb_domain::error::Error::internal("Simulated failure"));
        }

        let results: Vec<SearchResult> = (0..limit.min(5))
            .map(|i| SearchResult {
                id: format!("result_{}", i),
                file_path: format!("src/module_{}.rs", i),
                start_line: (i * 10 + 1) as u32,
                content: format!("fn function_{}() {{ }}", i),
                score: 0.9 - (i as f64 * 0.1),
                language: "rust".to_string(),
            })
            .collect();

        Ok(results)
    }
}

// ============================================================================
// Mock Indexing Service
// ============================================================================

struct MockIndexingService {
    status: IndexingStatus,
    should_fail: bool,
}

impl MockIndexingService {
    fn new() -> Self {
        Self {
            status: IndexingStatus::default(),
            should_fail: false,
        }
    }

    fn in_progress() -> Self {
        Self {
            status: IndexingStatus {
                is_indexing: true,
                progress: 0.5,
                current_file: Some("src/lib.rs".to_string()),
                total_files: 100,
                processed_files: 50,
            },
            should_fail: false,
        }
    }

    fn failing() -> Self {
        Self {
            status: IndexingStatus::default(),
            should_fail: true,
        }
    }
}

#[async_trait]
impl IndexingServiceInterface for MockIndexingService {
    async fn index_codebase(&self, _path: &Path, _collection: &str) -> Result<IndexingResult> {
        if self.should_fail {
            return Err(mcb_domain::error::Error::internal("Simulated failure"));
        }

        Ok(IndexingResult {
            files_processed: 50,
            chunks_created: 250,
            files_skipped: 2,
            errors: Vec::new(),
        })
    }

    fn get_status(&self) -> IndexingStatus {
        self.status.clone()
    }

    async fn clear_collection(&self, _collection: &str) -> Result<()> {
        if self.should_fail {
            return Err(mcb_domain::error::Error::internal("Simulated failure"));
        }
        Ok(())
    }
}

// ============================================================================
// Mock Context Service
// ============================================================================

struct MockContextService {
    dimensions: usize,
    should_fail: bool,
}

impl MockContextService {
    fn new() -> Self {
        Self {
            dimensions: 384,
            should_fail: false,
        }
    }

    fn with_dimensions(dimensions: usize) -> Self {
        Self {
            dimensions,
            should_fail: false,
        }
    }

    fn failing() -> Self {
        Self {
            dimensions: 384,
            should_fail: true,
        }
    }
}

#[async_trait]
impl ContextServiceInterface for MockContextService {
    async fn initialize(&self, _collection: &str) -> Result<()> {
        if self.should_fail {
            return Err(mcb_domain::error::Error::internal("Simulated failure"));
        }
        Ok(())
    }

    async fn store_chunks(&self, _collection: &str, _chunks: &[CodeChunk]) -> Result<()> {
        if self.should_fail {
            return Err(mcb_domain::error::Error::internal("Simulated failure"));
        }
        Ok(())
    }

    async fn search_similar(
        &self,
        _collection: &str,
        _query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        if self.should_fail {
            return Err(mcb_domain::error::Error::internal("Simulated failure"));
        }

        let results: Vec<SearchResult> = (0..limit.min(5))
            .map(|i| SearchResult {
                id: format!("chunk_{}", i),
                file_path: format!("src/file_{}.rs", i),
                start_line: (i * 10 + 1) as u32,
                content: format!("Content {}", i),
                score: 0.95 - (i as f64 * 0.05),
                language: "rust".to_string(),
            })
            .collect();

        Ok(results)
    }

    async fn embed_text(&self, _text: &str) -> Result<Embedding> {
        if self.should_fail {
            return Err(mcb_domain::error::Error::internal("Simulated failure"));
        }

        Ok(Embedding {
            vector: vec![0.1; self.dimensions],
            model: "mock".to_string(),
            dimensions: self.dimensions,
        })
    }

    async fn clear_collection(&self, _collection: &str) -> Result<()> {
        if self.should_fail {
            return Err(mcb_domain::error::Error::internal("Simulated failure"));
        }
        Ok(())
    }

    async fn get_stats(&self) -> Result<(RepositoryStats, SearchStats)> {
        if self.should_fail {
            return Err(mcb_domain::error::Error::internal("Simulated failure"));
        }

        Ok((
            RepositoryStats {
                total_chunks: 100,
                total_collections: 5,
                storage_size_bytes: 1024 * 1024,
                avg_chunk_size_bytes: 512.0,
            },
            SearchStats {
                total_queries: 50,
                avg_response_time_ms: 45.0,
                cache_hit_rate: 0.8,
                indexed_documents: 1000,
            },
        ))
    }

    fn embedding_dimensions(&self) -> usize {
        self.dimensions
    }
}

// ============================================================================
// Mock Chunking Orchestrator
// ============================================================================

struct MockChunkingOrchestrator {
    should_fail: bool,
}

impl MockChunkingOrchestrator {
    fn new() -> Self {
        Self { should_fail: false }
    }

    fn failing() -> Self {
        Self { should_fail: true }
    }
}

#[async_trait]
impl ChunkingOrchestratorInterface for MockChunkingOrchestrator {
    async fn process_files(&self, files: Vec<(String, String)>) -> Result<Vec<CodeChunk>> {
        if self.should_fail {
            return Err(mcb_domain::error::Error::internal("Simulated failure"));
        }

        let chunks: Vec<CodeChunk> = files
            .iter()
            .enumerate()
            .map(|(i, (path, content))| CodeChunk {
                id: format!("chunk_{}", i),
                file_path: path.clone(),
                start_line: 1,
                end_line: content.lines().count() as u32,
                content: content.clone(),
                language: "rust".to_string(),
                metadata: serde_json::json!({"ast_type": "function"}),
            })
            .collect();

        Ok(chunks)
    }

    async fn process_file(&self, path: &Path, content: &str) -> Result<Vec<CodeChunk>> {
        if self.should_fail {
            return Err(mcb_domain::error::Error::internal("Simulated failure"));
        }

        Ok(vec![CodeChunk {
            id: "chunk_0".to_string(),
            file_path: path.to_string_lossy().to_string(),
            start_line: 1,
            end_line: content.lines().count() as u32,
            content: content.to_string(),
            language: "rust".to_string(),
            metadata: serde_json::json!({"ast_type": "function"}),
        }])
    }

    async fn chunk_file(&self, path: &Path) -> Result<Vec<CodeChunk>> {
        if self.should_fail {
            return Err(mcb_domain::error::Error::internal("Simulated failure"));
        }

        Ok(vec![CodeChunk {
            id: "chunk_from_file".to_string(),
            file_path: path.to_string_lossy().to_string(),
            start_line: 1,
            end_line: 10,
            content: "// File content".to_string(),
            language: "rust".to_string(),
            metadata: serde_json::json!({}),
        }])
    }
}

// ============================================================================
// Search Service Tests
// ============================================================================

#[test]
fn test_search_service_interface_trait_object() {
    let _service: Arc<dyn SearchServiceInterface> = Arc::new(MockSearchService::new());
}

#[tokio::test]
async fn test_search_service_interface_contract() {
    let service: Arc<dyn SearchServiceInterface> = Arc::new(MockSearchService::new());

    let results = service.search("test_collection", "find functions", 5).await;

    assert!(results.is_ok());
    let results = results.expect("Expected results");
    assert!(!results.is_empty());
    assert!(results.len() <= 5);
}

#[tokio::test]
async fn test_search_service_interface_failure() {
    let service: Arc<dyn SearchServiceInterface> = Arc::new(MockSearchService::failing());

    let result = service.search("test", "query", 5).await;

    assert!(result.is_err());
}

// ============================================================================
// Indexing Service Tests
// ============================================================================

#[test]
fn test_indexing_service_interface_trait_object() {
    let _service: Arc<dyn IndexingServiceInterface> = Arc::new(MockIndexingService::new());
}

#[tokio::test]
async fn test_indexing_service_interface_contract() {
    let service: Arc<dyn IndexingServiceInterface> = Arc::new(MockIndexingService::new());

    let result = service
        .index_codebase(Path::new("/test/path"), "test_collection")
        .await;

    assert!(result.is_ok());
    let result = result.expect("Expected result");
    assert!(result.files_processed > 0);
    assert!(result.chunks_created > 0);
}

#[test]
fn test_indexing_service_status_idle() {
    let service = MockIndexingService::new();

    let status = service.get_status();

    assert!(!status.is_indexing);
    assert_eq!(status.progress, 0.0);
    assert!(status.current_file.is_none());
}

#[test]
fn test_indexing_service_status_in_progress() {
    let service = MockIndexingService::in_progress();

    let status = service.get_status();

    assert!(status.is_indexing);
    assert_eq!(status.progress, 0.5);
    assert!(status.current_file.is_some());
    assert_eq!(status.total_files, 100);
    assert_eq!(status.processed_files, 50);
}

#[tokio::test]
async fn test_indexing_service_clear_collection() {
    let service: Arc<dyn IndexingServiceInterface> = Arc::new(MockIndexingService::new());

    let result = service.clear_collection("test_collection").await;

    assert!(result.is_ok());
}

// ============================================================================
// Context Service Tests
// ============================================================================

#[test]
fn test_context_service_interface_trait_object() {
    let _service: Arc<dyn ContextServiceInterface> = Arc::new(MockContextService::new());
}

#[tokio::test]
async fn test_context_service_interface_contract() {
    let service: Arc<dyn ContextServiceInterface> = Arc::new(MockContextService::new());

    // Test initialize
    let result = service.initialize("test_collection").await;
    assert!(result.is_ok());

    // Test store_chunks
    let chunks: Vec<CodeChunk> = vec![];
    let result = service.store_chunks("test_collection", &chunks).await;
    assert!(result.is_ok());

    // Test search_similar
    let results = service
        .search_similar("test_collection", "query", 5)
        .await;
    assert!(results.is_ok());

    // Test embed_text
    let embedding = service.embed_text("test text").await;
    assert!(embedding.is_ok());

    // Test get_stats
    let stats = service.get_stats().await;
    assert!(stats.is_ok());

    // Test embedding_dimensions
    let dims = service.embedding_dimensions();
    assert_eq!(dims, 384);
}

#[test]
fn test_context_service_embedding_dimensions() {
    let service_384 = MockContextService::with_dimensions(384);
    assert_eq!(service_384.embedding_dimensions(), 384);

    let service_1536 = MockContextService::with_dimensions(1536);
    assert_eq!(service_1536.embedding_dimensions(), 1536);
}

#[tokio::test]
async fn test_context_service_failure_handling() {
    let service: Arc<dyn ContextServiceInterface> = Arc::new(MockContextService::failing());

    assert!(service.initialize("test").await.is_err());
    assert!(service.store_chunks("test", &[]).await.is_err());
    assert!(service.search_similar("test", "q", 5).await.is_err());
    assert!(service.embed_text("text").await.is_err());
    assert!(service.get_stats().await.is_err());
}

// ============================================================================
// Chunking Orchestrator Tests
// ============================================================================

#[test]
fn test_chunking_orchestrator_interface_trait_object() {
    let _orchestrator: Arc<dyn ChunkingOrchestratorInterface> =
        Arc::new(MockChunkingOrchestrator::new());
}

#[tokio::test]
async fn test_chunking_orchestrator_process_files() {
    let orchestrator: Arc<dyn ChunkingOrchestratorInterface> =
        Arc::new(MockChunkingOrchestrator::new());

    let files = vec![
        ("src/lib.rs".to_string(), "fn main() {}".to_string()),
        ("src/utils.rs".to_string(), "fn helper() {}".to_string()),
    ];

    let chunks = orchestrator.process_files(files).await;

    assert!(chunks.is_ok());
    let chunks = chunks.expect("Expected chunks");
    assert_eq!(chunks.len(), 2);
}

#[tokio::test]
async fn test_chunking_orchestrator_process_file() {
    let orchestrator: Arc<dyn ChunkingOrchestratorInterface> =
        Arc::new(MockChunkingOrchestrator::new());

    let chunks = orchestrator
        .process_file(Path::new("src/lib.rs"), "fn main() {}")
        .await;

    assert!(chunks.is_ok());
    let chunks = chunks.expect("Expected chunks");
    assert!(!chunks.is_empty());
}

#[tokio::test]
async fn test_chunking_orchestrator_chunk_file() {
    let orchestrator: Arc<dyn ChunkingOrchestratorInterface> =
        Arc::new(MockChunkingOrchestrator::new());

    let chunks = orchestrator.chunk_file(Path::new("src/lib.rs")).await;

    assert!(chunks.is_ok());
}

#[tokio::test]
async fn test_chunking_orchestrator_failure() {
    let orchestrator: Arc<dyn ChunkingOrchestratorInterface> =
        Arc::new(MockChunkingOrchestrator::failing());

    assert!(orchestrator.process_files(vec![]).await.is_err());
    assert!(orchestrator
        .process_file(Path::new("test"), "content")
        .await
        .is_err());
    assert!(orchestrator.chunk_file(Path::new("test")).await.is_err());
}

// ============================================================================
// IndexingResult Tests
// ============================================================================

#[test]
fn test_indexing_result_creation() {
    let result = IndexingResult {
        files_processed: 100,
        chunks_created: 500,
        files_skipped: 5,
        errors: vec!["Error 1".to_string(), "Error 2".to_string()],
    };

    assert_eq!(result.files_processed, 100);
    assert_eq!(result.chunks_created, 500);
    assert_eq!(result.files_skipped, 5);
    assert_eq!(result.errors.len(), 2);
}

#[test]
fn test_indexing_result_empty() {
    let result = IndexingResult {
        files_processed: 0,
        chunks_created: 0,
        files_skipped: 0,
        errors: Vec::new(),
    };

    assert_eq!(result.files_processed, 0);
    assert!(result.errors.is_empty());
}

// ============================================================================
// IndexingStatus Tests
// ============================================================================

#[test]
fn test_indexing_status_default() {
    let status = IndexingStatus::default();

    assert!(!status.is_indexing);
    assert_eq!(status.progress, 0.0);
    assert!(status.current_file.is_none());
    assert_eq!(status.total_files, 0);
    assert_eq!(status.processed_files, 0);
}

#[test]
fn test_indexing_status_in_progress() {
    let status = IndexingStatus {
        is_indexing: true,
        progress: 0.75,
        current_file: Some("src/main.rs".to_string()),
        total_files: 100,
        processed_files: 75,
    };

    assert!(status.is_indexing);
    assert_eq!(status.progress, 0.75);
    assert_eq!(status.current_file, Some("src/main.rs".to_string()));
    assert_eq!(status.processed_files, 75);
}

#[test]
fn test_indexing_status_completed() {
    let status = IndexingStatus {
        is_indexing: false,
        progress: 1.0,
        current_file: None,
        total_files: 100,
        processed_files: 100,
    };

    assert!(!status.is_indexing);
    assert_eq!(status.progress, 1.0);
    assert_eq!(status.processed_files, status.total_files);
}
