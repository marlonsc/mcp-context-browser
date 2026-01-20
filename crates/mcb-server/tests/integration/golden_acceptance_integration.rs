//! Golden Acceptance Tests for v0.1.2
//!
//! This module validates the core functionality of MCP Context Browser using
//! real providers (NullEmbeddingProvider + InMemoryVectorStore) for deterministic testing.
//!
//! ## Key Principle
//!
//! Golden tests validate:
//! 1. Repository indexing completes successfully from real files
//! 2. Queries execute within time limits
//! 3. Search returns results matching expected_files
//! 4. The architecture works end-to-end without external dependencies
//!
//! Uses `extern crate mcb_providers` to force linkme registration.

// Force linkme registration of all providers
extern crate mcb_providers;

use mcb_domain::entities::CodeChunk;
// Note: EmbeddingProvider/VectorStoreProvider traits are used via ctx.embedding_handle().get()
use mcb_infrastructure::config::AppConfig;
use mcb_infrastructure::di::bootstrap::init_app;
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

/// Test query structure matching the JSON fixture format
#[derive(Debug, Clone, serde::Deserialize)]
#[allow(dead_code)]
pub struct TestQuery {
    pub id: String,
    pub query: String,
    pub description: String,
    pub expected_files: Vec<String>,
    pub max_latency_ms: u64,
    pub min_results: usize,
}

/// Golden queries configuration
#[derive(Debug, Clone, serde::Deserialize)]
#[allow(dead_code)]
pub struct GoldenQueriesConfig {
    pub version: String,
    pub description: String,
    pub queries: Vec<TestQuery>,
    pub config: QueryConfig,
}

/// Query configuration
#[derive(Debug, Clone, serde::Deserialize)]
#[allow(dead_code)]
pub struct QueryConfig {
    pub collection_name: String,
    pub timeout_ms: u64,
    pub relevance_threshold: f64,
    pub top_k: usize,
}

/// Load golden queries from fixture file
fn load_golden_queries() -> GoldenQueriesConfig {
    let fixture_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/golden_queries.json");

    let content = std::fs::read_to_string(&fixture_path)
        .unwrap_or_else(|_| panic!("Failed to read golden queries from {:?}", fixture_path));

    serde_json::from_str(&content).expect("Failed to parse golden queries JSON")
}

/// Read all source files from sample_codebase and create CodeChunks
fn read_sample_codebase_files() -> Vec<CodeChunk> {
    let sample_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample_codebase/src");

    let mut chunks = Vec::new();

    if let Ok(entries) = fs::read_dir(&sample_path) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "rs")
                && let Ok(content) = fs::read_to_string(&path)
            {
                let file_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                let line_count = content.lines().count();

                chunks.push(CodeChunk {
                    id: format!("golden_{}", file_name.replace('.', "_")),
                    file_path: file_name,
                    content,
                    start_line: 1,
                    end_line: line_count as u32,
                    language: "rust".to_string(),
                    metadata: json!({"source": "sample_codebase"}),
                });
            }
        }
    }

    chunks
}

// ============================================================================
// Fixture Validation Tests (always run)
// ============================================================================

#[test]
fn test_golden_queries_fixture_valid() {
    let config = load_golden_queries();

    assert_eq!(config.version, "0.1.2");
    assert!(!config.queries.is_empty(), "Should have test queries");

    for query in &config.queries {
        assert!(!query.id.is_empty(), "Query ID should not be empty");
        assert!(!query.query.is_empty(), "Query string should not be empty");
        assert!(
            !query.expected_files.is_empty(),
            "Expected files should not be empty for query: {}",
            query.id
        );
        assert!(
            query.max_latency_ms > 0,
            "Max latency should be positive for query: {}",
            query.id
        );
    }
}

#[test]
fn test_query_ids_unique() {
    let config = load_golden_queries();
    let mut seen = std::collections::HashSet::new();

    for query in &config.queries {
        assert!(
            seen.insert(&query.id),
            "Duplicate query ID found: {}",
            query.id
        );
    }
}

#[test]
fn test_config_values_reasonable() {
    let config = load_golden_queries();

    assert!(
        config.config.timeout_ms >= 1000,
        "Timeout should be at least 1000ms"
    );
    assert!(
        config.config.top_k >= 1 && config.config.top_k <= 100,
        "top_k should be between 1 and 100"
    );
    assert!(
        config.config.relevance_threshold >= 0.0 && config.config.relevance_threshold <= 1.0,
        "Relevance threshold should be between 0 and 1"
    );
}

#[test]
fn test_sample_codebase_files_exist() {
    let chunks = read_sample_codebase_files();

    assert!(
        !chunks.is_empty(),
        "Should read at least one file from sample_codebase"
    );

    // Verify expected files exist
    let file_names: Vec<&str> = chunks.iter().map(|c| c.file_path.as_str()).collect();

    let expected = [
        "embedding.rs",
        "vector_store.rs",
        "handlers.rs",
        "cache.rs",
        "di.rs",
        "error.rs",
        "chunking.rs",
    ];
    for exp in expected {
        assert!(
            file_names.contains(&exp),
            "Missing expected file {} in sample_codebase. Found: {:?}",
            exp,
            file_names
        );
    }
}

// ============================================================================
// Real Provider Tests (using NullEmbedding + InMemoryVectorStore)
// ============================================================================

#[tokio::test]
async fn test_golden_index_real_files() {
    let config = AppConfig::default();
    let ctx = init_app(config).await.expect("init_app should succeed");

    let embedding = ctx.embedding_handle().get();
    let vector_store = ctx.vector_store_handle().get();

    let golden_config = load_golden_queries();
    let collection = &golden_config.config.collection_name;
    let chunks = read_sample_codebase_files();

    assert!(!chunks.is_empty(), "Should have files to index");

    // Step 1: Create collection
    let create_result = vector_store
        .create_collection(collection, embedding.dimensions())
        .await;
    assert!(
        create_result.is_ok(),
        "Collection creation should succeed: {}",
        create_result
            .err()
            .map(|e| e.to_string())
            .unwrap_or_default()
    );

    // Step 2: Generate embeddings for real file contents
    let start = Instant::now();
    let texts: Vec<String> = chunks.iter().map(|c| c.content.clone()).collect();
    let embeddings = embedding
        .embed_batch(&texts)
        .await
        .expect("Embedding should work");
    let embed_time = start.elapsed();

    assert!(
        embed_time < Duration::from_millis(500),
        "Embedding should be fast with NullProvider: {:?}",
        embed_time
    );

    // Step 3: Build metadata from real chunks
    let metadata: Vec<HashMap<String, serde_json::Value>> = chunks
        .iter()
        .map(|chunk| {
            let mut meta = HashMap::new();
            meta.insert("id".to_string(), json!(chunk.id));
            meta.insert("file_path".to_string(), json!(chunk.file_path));
            meta.insert("content".to_string(), json!(chunk.content));
            meta.insert("start_line".to_string(), json!(chunk.start_line));
            meta.insert("end_line".to_string(), json!(chunk.end_line));
            meta.insert("language".to_string(), json!(chunk.language));
            meta
        })
        .collect();

    // Step 4: Insert into vector store
    let ids = vector_store
        .insert_vectors(collection, &embeddings, metadata)
        .await
        .expect("Insert should succeed");

    assert_eq!(
        ids.len(),
        chunks.len(),
        "Should insert all chunks from sample_codebase"
    );
}

#[tokio::test]
async fn test_golden_search_validates_expected_files() {
    let config = AppConfig::default();
    let ctx = init_app(config).await.expect("init_app should succeed");

    let embedding = ctx.embedding_handle().get();
    let vector_store = ctx.vector_store_handle().get();

    let golden_config = load_golden_queries();
    let collection = "golden_expected_files_test";
    let chunks = read_sample_codebase_files();

    // Setup: Create collection and index real files
    vector_store
        .create_collection(collection, embedding.dimensions())
        .await
        .expect("Create collection");

    let texts: Vec<String> = chunks.iter().map(|c| c.content.clone()).collect();
    let embeddings = embedding.embed_batch(&texts).await.expect("Embed");

    let metadata: Vec<HashMap<String, serde_json::Value>> = chunks
        .iter()
        .map(|c| {
            let mut m = HashMap::new();
            m.insert("file_path".to_string(), json!(c.file_path));
            m.insert("content".to_string(), json!(c.content));
            m
        })
        .collect();

    vector_store
        .insert_vectors(collection, &embeddings, metadata)
        .await
        .expect("Insert");

    // Test: Search with golden query and validate expected_files
    let query = &golden_config.queries[0]; // embedding_provider query

    let start = Instant::now();
    let query_embedding = embedding
        .embed_batch(std::slice::from_ref(&query.query))
        .await
        .expect("Query embed");

    let results = vector_store
        .search_similar(
            collection,
            &query_embedding[0].vector,
            golden_config.config.top_k,
            None,
        )
        .await
        .expect("Search");
    let search_time = start.elapsed();

    // Validate: Results exist
    assert!(
        !results.is_empty(),
        "Golden search should return results for query '{}'",
        query.id
    );

    // Validate: Latency within limits
    assert!(
        search_time < Duration::from_millis(query.max_latency_ms),
        "Query '{}' exceeded latency: {:?} > {}ms",
        query.id,
        search_time,
        query.max_latency_ms
    );

    // Validate: expected_files are found in results
    // SearchResult has file_path directly, not in metadata
    let result_files: Vec<&str> = results.iter().map(|r| r.file_path.as_str()).collect();

    for expected in &query.expected_files {
        assert!(
            result_files.iter().any(|f| f.contains(expected)),
            "Query '{}' should find expected file '{}'. Found files: {:?}",
            query.id,
            expected,
            result_files
        );
    }
}

#[tokio::test]
async fn test_golden_all_queries_find_expected_files() {
    let config = AppConfig::default();
    let ctx = init_app(config).await.expect("init_app should succeed");

    let embedding = ctx.embedding_handle().get();
    let vector_store = ctx.vector_store_handle().get();

    let golden_config = load_golden_queries();
    let collection = "golden_all_queries_test";
    let chunks = read_sample_codebase_files();

    // Setup collection with real files
    vector_store
        .create_collection(collection, embedding.dimensions())
        .await
        .expect("Create collection");

    let texts: Vec<String> = chunks.iter().map(|c| c.content.clone()).collect();
    let embeddings = embedding.embed_batch(&texts).await.expect("Embed");

    let metadata: Vec<HashMap<String, serde_json::Value>> = chunks
        .iter()
        .map(|c| {
            let mut m = HashMap::new();
            m.insert("file_path".to_string(), json!(c.file_path));
            m.insert("content".to_string(), json!(c.content));
            m
        })
        .collect();

    vector_store
        .insert_vectors(collection, &embeddings, metadata)
        .await
        .expect("Insert");

    // Test ALL golden queries
    let mut passed_queries = 0;
    let mut failed_queries = Vec::new();

    for query in &golden_config.queries {
        let start = Instant::now();

        let query_embedding = embedding
            .embed_batch(std::slice::from_ref(&query.query))
            .await
            .expect("Query embed");

        let results = vector_store
            .search_similar(
                collection,
                &query_embedding[0].vector,
                golden_config.config.top_k,
                None,
            )
            .await
            .expect("Search");

        let elapsed = start.elapsed();
        let max_latency = Duration::from_millis(query.max_latency_ms);

        // Check latency
        if elapsed >= max_latency {
            failed_queries.push(format!(
                "Query '{}' exceeded latency: {:?} > {:?}",
                query.id, elapsed, max_latency
            ));
            continue;
        }

        // Check results exist
        if results.is_empty() {
            failed_queries.push(format!("Query '{}' returned no results", query.id));
            continue;
        }

        // Check expected_files found
        // SearchResult has file_path directly, not in metadata
        let result_files: Vec<&str> = results.iter().map(|r| r.file_path.as_str()).collect();

        let mut all_expected_found = true;
        for expected in &query.expected_files {
            if !result_files.iter().any(|f| f.contains(expected)) {
                failed_queries.push(format!(
                    "Query '{}' missing expected file '{}'. Found: {:?}",
                    query.id, expected, result_files
                ));
                all_expected_found = false;
            }
        }

        if all_expected_found {
            passed_queries += 1;
        }
    }

    // Report results
    let total = golden_config.queries.len();
    assert!(
        failed_queries.is_empty(),
        "Golden queries failed: {}/{} passed. Failures:\n{}",
        passed_queries,
        total,
        failed_queries.join("\n")
    );
}

#[tokio::test]
async fn test_golden_full_workflow_end_to_end() {
    // This test validates the complete golden test workflow:
    // 1. Load config
    // 2. Read real files from sample_codebase
    // 3. Create collection
    // 4. Index real file contents
    // 5. Search with all golden queries
    // 6. Validate expected_files found

    let app_config = AppConfig::default();
    let ctx = init_app(app_config).await.expect("init_app should succeed");

    let embedding = ctx.embedding_handle().get();
    let vector_store = ctx.vector_store_handle().get();
    let golden_config = load_golden_queries();

    let collection = &golden_config.config.collection_name;
    let chunks = read_sample_codebase_files();

    // Create collection
    vector_store
        .create_collection(collection, embedding.dimensions())
        .await
        .expect("Create collection");

    // Index real file chunks
    let texts: Vec<String> = chunks.iter().map(|c| c.content.clone()).collect();
    let embeddings = embedding.embed_batch(&texts).await.expect("Embed");

    let metadata: Vec<HashMap<String, serde_json::Value>> = chunks
        .iter()
        .map(|c| {
            let mut m = HashMap::new();
            m.insert("file_path".to_string(), json!(c.file_path));
            m.insert("content".to_string(), json!(c.content));
            m
        })
        .collect();

    let ids = vector_store
        .insert_vectors(collection, &embeddings, metadata)
        .await
        .expect("Insert");

    assert_eq!(ids.len(), chunks.len(), "All chunks indexed");

    // Search with each golden query and validate
    let mut successful_queries = 0;

    for query in &golden_config.queries {
        let query_embedding = embedding
            .embed_batch(std::slice::from_ref(&query.query))
            .await
            .expect("Query embed");

        let results = vector_store
            .search_similar(
                collection,
                &query_embedding[0].vector,
                golden_config.config.top_k,
                None,
            )
            .await
            .expect("Search");

        if !results.is_empty() {
            // Check if any expected file is in results
            // SearchResult has file_path directly, not in metadata
            let result_files: Vec<&str> = results.iter().map(|r| r.file_path.as_str()).collect();

            let found_expected = query
                .expected_files
                .iter()
                .any(|exp| result_files.iter().any(|f| f.contains(exp)));

            if found_expected {
                successful_queries += 1;
            }
        }
    }

    // At least most queries should find their expected files
    let total = golden_config.queries.len();
    let success_rate = (successful_queries as f64) / (total as f64);
    assert!(
        success_rate >= 0.5,
        "At least 50% of golden queries should find expected files. Got: {}/{}",
        successful_queries,
        total
    );
}
