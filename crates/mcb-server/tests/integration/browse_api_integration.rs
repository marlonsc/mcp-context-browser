//! Integration tests for browse API endpoints
//!
//! Tests the REST API for browsing indexed collections, files, and chunks.

use async_trait::async_trait;
use mcb_application::ports::admin::{
    IndexingOperation, IndexingOperationsInterface, PerformanceMetricsData,
    PerformanceMetricsInterface,
};
use mcb_application::ports::infrastructure::events::{DomainEventStream, EventBusProvider};
use mcb_domain::error::Result;
use mcb_domain::events::DomainEvent;
use mcb_domain::ports::providers::VectorStoreBrowser;
use mcb_domain::value_objects::{CollectionInfo, FileInfo, SearchResult};
use mcb_server::admin::auth::AdminAuthConfig;
use mcb_server::admin::browse_handlers::BrowseState;
use mcb_server::admin::handlers::AdminState;
use mcb_server::admin::routes::admin_rocket;
use rocket::http::{Header, Status};
use rocket::local::asynchronous::Client;
use std::collections::HashMap;
use std::sync::Arc;

/// Mock VectorStoreBrowser for testing
pub struct MockVectorStoreBrowser {
    collections: Vec<CollectionInfo>,
    files: Vec<FileInfo>,
    chunks: Vec<SearchResult>,
}

impl MockVectorStoreBrowser {
    pub fn new() -> Self {
        Self {
            collections: Vec::new(),
            files: Vec::new(),
            chunks: Vec::new(),
        }
    }

    pub fn with_collections(mut self, collections: Vec<CollectionInfo>) -> Self {
        self.collections = collections;
        self
    }

    pub fn with_files(mut self, files: Vec<FileInfo>) -> Self {
        self.files = files;
        self
    }

    pub fn with_chunks(mut self, chunks: Vec<SearchResult>) -> Self {
        self.chunks = chunks;
        self
    }
}

#[async_trait]
impl VectorStoreBrowser for MockVectorStoreBrowser {
    async fn list_collections(&self) -> Result<Vec<CollectionInfo>> {
        Ok(self.collections.clone())
    }

    async fn list_file_paths(&self, _collection: &str, limit: usize) -> Result<Vec<FileInfo>> {
        Ok(self.files.iter().take(limit).cloned().collect())
    }

    async fn get_chunks_by_file(
        &self,
        _collection: &str,
        _file_path: &str,
    ) -> Result<Vec<SearchResult>> {
        Ok(self.chunks.clone())
    }
}

// ============================================================================
// Mock Implementations
// ============================================================================

/// Mock performance metrics
struct MockMetrics;

impl PerformanceMetricsInterface for MockMetrics {
    fn uptime_secs(&self) -> u64 {
        0
    }

    fn record_query(&self, _response_time_ms: u64, _success: bool, _cache_hit: bool) {}

    fn update_active_connections(&self, _delta: i64) {}

    fn get_performance_metrics(&self) -> PerformanceMetricsData {
        PerformanceMetricsData {
            total_queries: 0,
            successful_queries: 0,
            failed_queries: 0,
            average_response_time_ms: 0.0,
            cache_hit_rate: 0.0,
            active_connections: 0,
            uptime_seconds: 0,
        }
    }
}

/// Mock indexing operations
struct MockIndexing;

impl IndexingOperationsInterface for MockIndexing {
    fn get_operations(&self) -> HashMap<String, IndexingOperation> {
        HashMap::new()
    }
}

/// Mock event bus
struct MockEventBus;

#[async_trait]
impl EventBusProvider for MockEventBus {
    async fn publish_event(&self, _event: DomainEvent) -> Result<()> {
        Ok(())
    }

    async fn subscribe_events(&self) -> Result<DomainEventStream> {
        // Return an empty stream
        Ok(Box::pin(futures::stream::empty()))
    }

    fn has_subscribers(&self) -> bool {
        false
    }

    async fn publish(&self, _topic: &str, _payload: &[u8]) -> Result<()> {
        Ok(())
    }

    async fn subscribe(&self, _topic: &str) -> Result<String> {
        Ok("mock-subscription".to_string())
    }
}

/// Create test admin state with minimal dependencies
fn create_test_admin_state() -> AdminState {
    AdminState {
        metrics: Arc::new(MockMetrics),
        indexing: Arc::new(MockIndexing),
        config_watcher: None,
        config_path: None,
        shutdown_coordinator: None,
        shutdown_timeout_secs: 30,
        event_bus: Arc::new(MockEventBus),
        service_manager: None,
        cache: None,
    }
}

/// Create a test Rocket client with browse state
async fn create_test_client(browse_state: BrowseState) -> Client {
    let admin_state = create_test_admin_state();
    let auth_config = Arc::new(AdminAuthConfig {
        enabled: true,
        header_name: "X-Admin-Key".to_string(),
        api_key: Some("test-key".to_string()),
    });

    let rocket = admin_rocket(admin_state, auth_config, Some(browse_state));
    Client::tracked(rocket)
        .await
        .expect("valid rocket instance")
}

#[tokio::test]
async fn test_list_collections_empty() {
    let browser = MockVectorStoreBrowser::new();
    let browse_state = BrowseState {
        browser: Arc::new(browser),
    };

    let client = create_test_client(browse_state).await;

    let response = client
        .get("/collections")
        .header(Header::new("X-Admin-Key", "test-key"))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().await.expect("response body");
    assert!(body.contains("\"collections\":[]"));
    assert!(body.contains("\"total\":0"));
}

#[tokio::test]
async fn test_list_collections_with_data() {
    let collections = vec![
        CollectionInfo::new("test_collection".to_string(), 100, 10, None, "memory"),
        CollectionInfo::new("another_collection".to_string(), 50, 5, None, "memory"),
    ];

    let browser = MockVectorStoreBrowser::new().with_collections(collections);
    let browse_state = BrowseState {
        browser: Arc::new(browser),
    };

    let client = create_test_client(browse_state).await;

    let response = client
        .get("/collections")
        .header(Header::new("X-Admin-Key", "test-key"))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().await.expect("response body");
    assert!(body.contains("test_collection"));
    assert!(body.contains("another_collection"));
    assert!(body.contains("\"total\":2"));
}

#[tokio::test]
async fn test_list_files_in_collection() {
    let files = vec![
        FileInfo::new("src/main.rs".to_string(), 5, "rust", None),
        FileInfo::new("src/lib.rs".to_string(), 3, "rust", None),
    ];

    let browser = MockVectorStoreBrowser::new().with_files(files);
    let browse_state = BrowseState {
        browser: Arc::new(browser),
    };

    let client = create_test_client(browse_state).await;

    let response = client
        .get("/collections/test_collection/files")
        .header(Header::new("X-Admin-Key", "test-key"))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().await.expect("response body");
    assert!(body.contains("src/main.rs"));
    assert!(body.contains("src/lib.rs"));
    assert!(body.contains("\"total\":2"));
}

#[tokio::test]
async fn test_get_file_chunks() {
    let chunks = vec![
        SearchResult {
            id: "chunk_1".to_string(),
            file_path: "src/main.rs".to_string(),
            content: "fn main() { }".to_string(),
            start_line: 1,
            score: 1.0,
            language: "rust".to_string(),
        },
        SearchResult {
            id: "chunk_2".to_string(),
            file_path: "src/main.rs".to_string(),
            content: "fn helper() { }".to_string(),
            start_line: 5,
            score: 1.0,
            language: "rust".to_string(),
        },
    ];

    let browser = MockVectorStoreBrowser::new().with_chunks(chunks);
    let browse_state = BrowseState {
        browser: Arc::new(browser),
    };

    let client = create_test_client(browse_state).await;

    let response = client
        .get("/collections/test_collection/chunks/src/main.rs")
        .header(Header::new("X-Admin-Key", "test-key"))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().await.expect("response body");
    assert!(body.contains("fn main()"));
    assert!(body.contains("fn helper()"));
    assert!(body.contains("\"total\":2"));
}

#[tokio::test]
async fn test_browse_requires_auth() {
    let browser = MockVectorStoreBrowser::new();
    let browse_state = BrowseState {
        browser: Arc::new(browser),
    };

    let client = create_test_client(browse_state).await;

    // Request without auth header
    let response = client.get("/collections").dispatch().await;

    // Should return unauthorized (401) or forbidden (403)
    assert!(
        response.status() == Status::Unauthorized || response.status() == Status::Forbidden,
        "Expected 401 or 403, got {:?}",
        response.status()
    );
}

#[tokio::test]
async fn test_browse_invalid_auth() {
    let browser = MockVectorStoreBrowser::new();
    let browse_state = BrowseState {
        browser: Arc::new(browser),
    };

    let client = create_test_client(browse_state).await;

    // Request with invalid auth key
    let response = client
        .get("/collections")
        .header(Header::new("X-Admin-Key", "invalid-key"))
        .dispatch()
        .await;

    // Should return unauthorized (401) or forbidden (403)
    assert!(
        response.status() == Status::Unauthorized || response.status() == Status::Forbidden,
        "Expected 401 or 403, got {:?}",
        response.status()
    );
}

// ============================================================================
// Real End-to-End Tests with InMemoryVectorStore
// ============================================================================

use mcb_domain::value_objects::Embedding;
use mcb_providers::vector_store::in_memory::InMemoryVectorStoreProvider;

/// Helper to create metadata for a code chunk
fn create_chunk_metadata(
    file_path: &str,
    content: &str,
    start_line: u32,
    language: &str,
) -> HashMap<String, serde_json::Value> {
    let mut metadata = HashMap::new();
    metadata.insert("file_path".to_string(), serde_json::json!(file_path));
    metadata.insert("content".to_string(), serde_json::json!(content));
    metadata.insert("start_line".to_string(), serde_json::json!(start_line));
    metadata.insert("language".to_string(), serde_json::json!(language));
    metadata.insert(
        "id".to_string(),
        serde_json::json!(format!("chunk_{}_{}", file_path, start_line)),
    );
    metadata
}

/// Helper to create a dummy embedding vector
fn create_dummy_embedding(dimensions: usize) -> Embedding {
    Embedding {
        vector: vec![0.1; dimensions],
        model: "test-model".to_string(),
        dimensions,
    }
}

/// Populate an in-memory vector store with test data simulating real indexed code
async fn populate_test_store(store: &InMemoryVectorStoreProvider, collection: &str) {
    use mcb_domain::ports::providers::VectorStoreProvider;

    // Create collection
    store
        .create_collection(collection, 384)
        .await
        .expect("Failed to create collection");

    // Simulate indexed Rust files from a real project
    let chunks = [
        // lib.rs - 3 chunks
        (
            "src/lib.rs",
            "//! Main library module\n\npub mod config;\npub mod handlers;",
            1,
            "rust",
        ),
        (
            "src/lib.rs",
            "pub fn initialize() -> Result<(), Error> {\n    // Setup code\n    Ok(())\n}",
            6,
            "rust",
        ),
        (
            "src/lib.rs",
            "#[cfg(test)]\nmod tests {\n    use super::*;\n}",
            12,
            "rust",
        ),
        // config.rs - 2 chunks
        (
            "src/config.rs",
            "#[derive(Debug, Clone)]\npub struct Config {\n    pub host: String,\n    pub port: u16,\n}",
            1,
            "rust",
        ),
        (
            "src/config.rs",
            "impl Config {\n    pub fn new() -> Self {\n        Self { host: \"localhost\".into(), port: 8080 }\n    }\n}",
            7,
            "rust",
        ),
        // handlers.rs - 2 chunks
        (
            "src/handlers.rs",
            "use crate::config::Config;\n\npub async fn handle_request(config: &Config) -> Response {",
            1,
            "rust",
        ),
        (
            "src/handlers.rs",
            "    let result = process_data().await?;\n    Ok(Response::new(result))\n}",
            5,
            "rust",
        ),
        // main.rs - 1 chunk
        (
            "src/main.rs",
            "#[tokio::main]\nasync fn main() {\n    let config = Config::new();\n    println!(\"Server starting on {}:{}\", config.host, config.port);\n}",
            1,
            "rust",
        ),
    ];

    let embeddings: Vec<Embedding> = chunks.iter().map(|_| create_dummy_embedding(384)).collect();

    let metadata: Vec<HashMap<String, serde_json::Value>> = chunks
        .iter()
        .map(|(path, content, line, lang)| create_chunk_metadata(path, content, *line, lang))
        .collect();

    store
        .insert_vectors(collection, &embeddings, metadata)
        .await
        .expect("Failed to insert vectors");
}

#[tokio::test]
async fn test_e2e_real_store_list_collections() {
    // Create real in-memory store
    let store = InMemoryVectorStoreProvider::new();

    // Populate with test data
    populate_test_store(&store, "test_project").await;

    // Create browse state with real store
    let browse_state = BrowseState {
        browser: Arc::new(store),
    };

    let client = create_test_client(browse_state).await;

    // List collections
    let response = client
        .get("/collections")
        .header(Header::new("X-Admin-Key", "test-key"))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().await.expect("response body");

    // Parse and validate JSON response
    let json: serde_json::Value = serde_json::from_str(&body).expect("valid JSON");

    // Validate collection exists
    assert!(
        body.contains("test_project"),
        "Should contain collection name"
    );

    // Validate counts
    let collections = json["collections"].as_array().expect("collections array");
    assert_eq!(collections.len(), 1, "Should have 1 collection");

    let collection = &collections[0];
    assert_eq!(collection["name"], "test_project");
    assert_eq!(collection["vector_count"], 8, "Should have 8 chunks total");
    assert_eq!(collection["file_count"], 4, "Should have 4 unique files");
    assert_eq!(collection["provider"], "in_memory");
}

#[tokio::test]
async fn test_e2e_real_store_list_files() {
    let store = InMemoryVectorStoreProvider::new();
    populate_test_store(&store, "test_project").await;

    let browse_state = BrowseState {
        browser: Arc::new(store),
    };

    let client = create_test_client(browse_state).await;

    // List files in collection
    let response = client
        .get("/collections/test_project/files")
        .header(Header::new("X-Admin-Key", "test-key"))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().await.expect("response body");

    let json: serde_json::Value = serde_json::from_str(&body).expect("valid JSON");

    // Validate files list
    let files = json["files"].as_array().expect("files array");
    assert_eq!(files.len(), 4, "Should have 4 files");

    // Check that all expected files are present
    let file_paths: Vec<&str> = files.iter().filter_map(|f| f["path"].as_str()).collect();

    assert!(file_paths.contains(&"src/lib.rs"), "Should contain lib.rs");
    assert!(
        file_paths.contains(&"src/config.rs"),
        "Should contain config.rs"
    );
    assert!(
        file_paths.contains(&"src/handlers.rs"),
        "Should contain handlers.rs"
    );
    assert!(
        file_paths.contains(&"src/main.rs"),
        "Should contain main.rs"
    );

    // Validate chunk counts per file
    for file in files {
        let path = file["path"].as_str().unwrap();
        let chunk_count = file["chunk_count"].as_u64().unwrap();
        match path {
            "src/lib.rs" => assert_eq!(chunk_count, 3, "lib.rs should have 3 chunks"),
            "src/config.rs" => assert_eq!(chunk_count, 2, "config.rs should have 2 chunks"),
            "src/handlers.rs" => assert_eq!(chunk_count, 2, "handlers.rs should have 2 chunks"),
            "src/main.rs" => assert_eq!(chunk_count, 1, "main.rs should have 1 chunk"),
            _ => panic!("Unexpected file: {}", path),
        }
    }
}

#[tokio::test]
async fn test_e2e_real_store_get_file_chunks() {
    let store = InMemoryVectorStoreProvider::new();
    populate_test_store(&store, "test_project").await;

    let browse_state = BrowseState {
        browser: Arc::new(store),
    };

    let client = create_test_client(browse_state).await;

    // Get chunks for lib.rs
    let response = client
        .get("/collections/test_project/chunks/src/lib.rs")
        .header(Header::new("X-Admin-Key", "test-key"))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().await.expect("response body");

    let json: serde_json::Value = serde_json::from_str(&body).expect("valid JSON");

    // Validate chunks
    let chunks = json["chunks"].as_array().expect("chunks array");
    assert_eq!(chunks.len(), 3, "lib.rs should have 3 chunks");

    // Chunks should be sorted by start_line
    let start_lines: Vec<u64> = chunks
        .iter()
        .filter_map(|c| c["start_line"].as_u64())
        .collect();

    assert_eq!(
        start_lines,
        vec![1, 6, 12],
        "Chunks should be sorted by line"
    );

    // Validate chunk content
    let first_chunk = &chunks[0];
    assert!(
        first_chunk["content"]
            .as_str()
            .unwrap()
            .contains("Main library module"),
        "First chunk should contain module doc"
    );

    let second_chunk = &chunks[1];
    assert!(
        second_chunk["content"]
            .as_str()
            .unwrap()
            .contains("pub fn initialize"),
        "Second chunk should contain initialize function"
    );

    let third_chunk = &chunks[2];
    assert!(
        third_chunk["content"]
            .as_str()
            .unwrap()
            .contains("#[cfg(test)]"),
        "Third chunk should contain test module"
    );
}

#[tokio::test]
async fn test_e2e_real_store_navigate_full_flow() {
    // This test simulates the full user flow:
    // 1. List collections
    // 2. Select a collection and list files
    // 3. Select a file and view chunks
    // 4. Validate data at each step

    let store = InMemoryVectorStoreProvider::new();
    populate_test_store(&store, "my_rust_project").await;

    let browse_state = BrowseState {
        browser: Arc::new(store),
    };

    let client = create_test_client(browse_state).await;

    // Step 1: List collections
    let response = client
        .get("/collections")
        .header(Header::new("X-Admin-Key", "test-key"))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().await.expect("response body");
    let json: serde_json::Value = serde_json::from_str(&body).expect("valid JSON");

    let collections = json["collections"].as_array().expect("collections array");
    assert!(
        !collections.is_empty(),
        "Should have at least one collection"
    );

    let collection_name = collections[0]["name"].as_str().expect("collection name");
    assert_eq!(collection_name, "my_rust_project");

    // Step 2: List files in the collection
    let files_url = format!("/collections/{}/files", collection_name);
    let response = client
        .get(&files_url)
        .header(Header::new("X-Admin-Key", "test-key"))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().await.expect("response body");
    let json: serde_json::Value = serde_json::from_str(&body).expect("valid JSON");

    let files = json["files"].as_array().expect("files array");
    assert!(!files.is_empty(), "Should have files");

    // Find config.rs
    let config_file = files
        .iter()
        .find(|f| f["path"].as_str() == Some("src/config.rs"))
        .expect("Should find config.rs");

    let chunk_count = config_file["chunk_count"].as_u64().expect("chunk count");
    assert_eq!(chunk_count, 2, "config.rs should have 2 chunks");

    // Step 3: Get chunks for config.rs
    let chunks_url = format!("/collections/{}/chunks/src/config.rs", collection_name);
    let response = client
        .get(&chunks_url)
        .header(Header::new("X-Admin-Key", "test-key"))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().await.expect("response body");
    let json: serde_json::Value = serde_json::from_str(&body).expect("valid JSON");

    let chunks = json["chunks"].as_array().expect("chunks array");
    assert_eq!(chunks.len(), 2, "config.rs should have 2 chunks");

    // Validate chunk contents
    let contents: Vec<&str> = chunks
        .iter()
        .filter_map(|c| c["content"].as_str())
        .collect();

    assert!(
        contents.iter().any(|c| c.contains("pub struct Config")),
        "Should have Config struct definition"
    );
    assert!(
        contents.iter().any(|c| c.contains("impl Config")),
        "Should have Config impl block"
    );
}

#[tokio::test]
async fn test_e2e_real_store_collection_not_found() {
    let store = InMemoryVectorStoreProvider::new();
    populate_test_store(&store, "existing_collection").await;

    let browse_state = BrowseState {
        browser: Arc::new(store),
    };

    let client = create_test_client(browse_state).await;

    // Try to list files in non-existent collection
    let response = client
        .get("/collections/nonexistent/files")
        .header(Header::new("X-Admin-Key", "test-key"))
        .dispatch()
        .await;

    // Should return error status
    assert!(
        response.status() == Status::NotFound || response.status() == Status::InternalServerError,
        "Expected 404 or 500 for non-existent collection, got {:?}",
        response.status()
    );
}

#[tokio::test]
async fn test_e2e_real_store_multiple_collections() {
    let store = InMemoryVectorStoreProvider::new();

    // Create multiple collections
    populate_test_store(&store, "project_alpha").await;
    populate_test_store(&store, "project_beta").await;

    let browse_state = BrowseState {
        browser: Arc::new(store),
    };

    let client = create_test_client(browse_state).await;

    // List all collections
    let response = client
        .get("/collections")
        .header(Header::new("X-Admin-Key", "test-key"))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().await.expect("response body");
    let json: serde_json::Value = serde_json::from_str(&body).expect("valid JSON");

    let collections = json["collections"].as_array().expect("collections array");
    assert_eq!(collections.len(), 2, "Should have 2 collections");

    let names: Vec<&str> = collections
        .iter()
        .filter_map(|c| c["name"].as_str())
        .collect();

    assert!(
        names.contains(&"project_alpha"),
        "Should have project_alpha"
    );
    assert!(names.contains(&"project_beta"), "Should have project_beta");

    // Validate total count
    assert_eq!(json["total"], 2);
}
