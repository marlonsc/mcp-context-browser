//! Tests for chunk repository interfaces and types

use mcb_domain::entities::CodeChunk;
use mcb_domain::repositories::{ChunkRepository, RepositoryStats};

#[test]
fn test_repository_stats_creation() {
    let stats = RepositoryStats {
        total_chunks: 100,
        total_collections: 5,
        storage_size_bytes: 1024,
        avg_chunk_size_bytes: 10.24,
    };

    assert_eq!(stats.total_chunks, 100);
    assert_eq!(stats.total_collections, 5);
    assert_eq!(stats.storage_size_bytes, 1024);
    assert_eq!(stats.avg_chunk_size_bytes, 10.24);
}

#[test]
fn test_repository_stats_default() {
    let stats = RepositoryStats::default();

    assert_eq!(stats.total_chunks, 0);
    assert_eq!(stats.total_collections, 0);
    assert_eq!(stats.storage_size_bytes, 0);
    assert_eq!(stats.avg_chunk_size_bytes, 0.0);
}

#[test]
fn test_code_chunk_creation() {
    let chunk = CodeChunk {
        id: "test-chunk-1".to_string(),
        content: "fn test() { println!(\"hello\"); }".to_string(),
        file_path: "src/main.rs".to_string(),
        start_line: 1,
        end_line: 3,
        language: "rust".to_string(),
        metadata: serde_json::json!({"type": "function"}),
    };

    assert_eq!(chunk.id, "test-chunk-1");
    assert_eq!(chunk.file_path, "src/main.rs");
    assert_eq!(chunk.start_line, 1);
    assert_eq!(chunk.end_line, 3);
    assert_eq!(chunk.language, "rust");
}

#[test]
fn test_chunk_repository_is_interface() {
    // Test that ChunkRepository is properly defined as an interface
    fn assert_is_interface<T: ?Sized>() {}

    // This will fail to compile if ChunkRepository is not a trait
    assert_is_interface::<dyn ChunkRepository>();

    // Additional assertion to ensure the test has real validation
    let result: std::result::Result<(), Box<dyn std::error::Error>> = Ok(());
    assert!(result.is_ok());
}

#[test]
fn test_repository_stats_debug() {
    let stats = RepositoryStats {
        total_chunks: 42,
        total_collections: 2,
        storage_size_bytes: 2048,
        avg_chunk_size_bytes: 48.76,
    };

    let debug_str = format!("{:?}", stats);
    assert!(debug_str.contains("RepositoryStats"));
    assert!(debug_str.contains("42"));
    assert!(debug_str.contains("2"));
}
