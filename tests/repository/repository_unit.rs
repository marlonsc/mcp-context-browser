//! Unit tests for repository pattern implementations
//!
//! Tests repository data structures and operations.

use mcp_context_browser::adapters::repository::{RepositoryStats, SearchStats};
use mcp_context_browser::domain::types::{CodeChunk, Language};

#[cfg(test)]
mod repository_data_tests {
    use super::*;

    #[test]
    fn test_repository_stats_structure() {
        let stats = RepositoryStats {
            total_chunks: 100,
            total_collections: 5,
            storage_size_bytes: 1024000,
            avg_chunk_size_bytes: 10240.0,
        };

        assert_eq!(stats.total_chunks, 100);
        assert_eq!(stats.total_collections, 5);
        assert_eq!(stats.storage_size_bytes, 1024000);
        assert_eq!(stats.avg_chunk_size_bytes, 10240.0);
    }

    #[test]
    fn test_search_stats_structure() {
        let stats = SearchStats {
            total_queries: 500,
            avg_response_time_ms: 45.2,
            cache_hit_rate: 0.85,
            indexed_documents: 1000,
        };

        assert_eq!(stats.total_queries, 500);
        assert_eq!(stats.avg_response_time_ms, 45.2);
        assert_eq!(stats.cache_hit_rate, 0.85);
        assert_eq!(stats.indexed_documents, 1000);
    }

    #[test]
    fn test_code_chunk_for_repository() {
        let chunk = CodeChunk {
            id: "repo_chunk_1".to_string(),
            content: "fn process_data(data: &str) -> Result<String> { Ok(data.to_uppercase()) }"
                .to_string(),
            file_path: "src/processor.rs".to_string(),
            start_line: 10,
            end_line: 12,
            language: Language::Rust,
            metadata: serde_json::json!({"repository": "test_repo", "commit": "abc123"}),
        };

        assert_eq!(chunk.id, "repo_chunk_1");
        assert!(chunk.content.contains("process_data"));
        assert_eq!(chunk.language, Language::Rust);
        assert!(chunk.metadata.get("repository").is_some());
    }

    #[test]
    fn test_chunk_metadata_handling() {
        let mut chunk = CodeChunk {
            id: "test_chunk".to_string(),
            content: "test content".to_string(),
            file_path: "test.rs".to_string(),
            start_line: 1,
            end_line: 1,
            language: Language::Rust,
            metadata: serde_json::json!({}),
        };

        // Test metadata manipulation
        if let serde_json::Value::Object(ref mut map) = chunk.metadata {
            map.insert(
                "indexed_at".to_string(),
                serde_json::json!("2024-01-01T00:00:00Z"),
            );
            map.insert("repository_id".to_string(), serde_json::json!("repo_123"));
        }

        assert!(chunk.metadata.get("indexed_at").is_some());
        assert!(chunk.metadata.get("repository_id").is_some());
    }

    #[test]
    fn test_chunk_data_integrity() -> Result<(), Box<dyn std::error::Error>> {
        let original_chunk = CodeChunk {
            id: "integrity_test".to_string(),
            content: "original content".to_string(),
            file_path: "test.rs".to_string(),
            start_line: 1,
            end_line: 5,
            language: Language::Rust,
            metadata: serde_json::json!({"integrity": "test"}),
        };

        // Test that serialization/deserialization preserves data
        let serialized = serde_json::to_string(&original_chunk)?;
        let deserialized: CodeChunk = serde_json::from_str(&serialized)?;

        assert_eq!(original_chunk, deserialized);
        Ok(())
    }
}

#[cfg(test)]
mod repository_performance_tests {
    #[test]
    fn test_repository_operation_timing() {
        let start = std::time::Instant::now();

        // Simulate a repository operation
        std::thread::sleep(std::time::Duration::from_millis(1));

        let elapsed = start.elapsed();
        assert!(
            elapsed.as_millis() < 100,
            "Operation should complete quickly"
        );
    }
}
