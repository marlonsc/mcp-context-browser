//! Unit tests for ChunkRepository interface

#[cfg(test)]
mod tests {
    use mcb_domain::{ChunkRepository, CodeChunk, RepositoryStats};
    use std::collections::HashMap;

    // Mock implementation for testing
    struct MockChunkRepository {
        storage: HashMap<String, Vec<CodeChunk>>,
        stats: RepositoryStats,
    }

    impl MockChunkRepository {
        fn new() -> Self {
            Self {
                storage: HashMap::new(),
                stats: RepositoryStats {
                    total_chunks: 0,
                    total_collections: 0,
                    storage_size_bytes: 0,
                    avg_chunk_size_bytes: 0.0,
                },
            }
        }
    }

    #[async_trait::async_trait]
    impl ChunkRepository for MockChunkRepository {
        async fn save(&self, _collection: &str, _chunk: &CodeChunk) -> mcb_domain::Result<String> {
            Ok("generated-id".to_string())
        }

        async fn save_batch(&self, _collection: &str, chunks: &[CodeChunk]) -> mcb_domain::Result<Vec<String>> {
            Ok((0..chunks.len()).map(|i| format!("id-{}", i)).collect())
        }

        async fn find_by_id(&self, _collection: &str, _id: &str) -> mcb_domain::Result<Option<CodeChunk>> {
            Ok(Some(CodeChunk {
                id: "test-chunk".to_string(),
                content: "test content".to_string(),
                file_path: "test.rs".to_string(),
                start_line: 1,
                end_line: 5,
                language: "rust".to_string(),
                metadata: serde_json::json!({"type": "function"}),
            }))
        }

        async fn find_by_collection(&self, _collection: &str, limit: usize) -> mcb_domain::Result<Vec<CodeChunk>> {
            let chunks = (0..limit.min(3)).map(|i| CodeChunk {
                id: format!("chunk-{}", i),
                content: format!("content {}", i),
                file_path: format!("file{}.rs", i),
                start_line: (i * 10 + 1) as u32,
                end_line: ((i + 1) * 10) as u32,
                language: "rust".to_string(),
                metadata: serde_json::json!({"index": i}),
            }).collect();
            Ok(chunks)
        }

        async fn delete(&self, _collection: &str, _id: &str) -> mcb_domain::Result<()> {
            Ok(())
        }

        async fn delete_collection(&self, _collection: &str) -> mcb_domain::Result<()> {
            Ok(())
        }

        async fn stats(&self) -> mcb_domain::Result<RepositoryStats> {
            Ok(self.stats.clone())
        }
    }

    #[tokio::test]
    async fn test_save_chunk() {
        let repo = MockChunkRepository::new();

        let chunk = CodeChunk {
            id: "test-chunk".to_string(),
            content: "fn test() {}".to_string(),
            file_path: "test.rs".to_string(),
            start_line: 1,
            end_line: 1,
            language: "rust".to_string(),
            metadata: serde_json::json!({}),
        };

        let result = repo.save("test-collection", &chunk).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "generated-id");
    }

    #[tokio::test]
    async fn test_save_batch() {
        let repo = MockChunkRepository::new();

        let chunks = vec![
            CodeChunk {
                id: "chunk-1".to_string(),
                content: "fn func1() {}".to_string(),
                file_path: "file1.rs".to_string(),
                start_line: 1,
                end_line: 1,
                language: "rust".to_string(),
                metadata: serde_json::json!({}),
            },
            CodeChunk {
                id: "chunk-2".to_string(),
                content: "fn func2() {}".to_string(),
                file_path: "file2.rs".to_string(),
                start_line: 1,
                end_line: 1,
                language: "rust".to_string(),
                metadata: serde_json::json!({}),
            },
        ];

        let result = repo.save_batch("test-collection", &chunks).await;
        assert!(result.is_ok());

        let ids = result.unwrap();
        assert_eq!(ids.len(), 2);
        assert_eq!(ids[0], "id-0");
        assert_eq!(ids[1], "id-1");
    }

    #[tokio::test]
    async fn test_find_by_id() {
        let repo = MockChunkRepository::new();

        let result = repo.find_by_id("test-collection", "test-id").await;
        assert!(result.is_ok());

        let chunk = result.unwrap();
        assert!(chunk.is_some());

        let chunk = chunk.unwrap();
        assert_eq!(chunk.id, "test-chunk");
        assert_eq!(chunk.content, "test content");
        assert_eq!(chunk.language, "rust");
    }

    #[tokio::test]
    async fn test_find_by_collection() {
        let repo = MockChunkRepository::new();

        let result = repo.find_by_collection("test-collection", 5).await;
        assert!(result.is_ok());

        let chunks = result.unwrap();
        assert_eq!(chunks.len(), 3); // Mock returns min(limit, 3)

        for (i, chunk) in chunks.iter().enumerate() {
            assert_eq!(chunk.id, format!("chunk-{}", i));
            assert_eq!(chunk.language, "rust");
        }
    }

    #[tokio::test]
    async fn test_find_by_collection_limit() {
        let repo = MockChunkRepository::new();

        let result = repo.find_by_collection("test-collection", 1).await;
        assert!(result.is_ok());

        let chunks = result.unwrap();
        assert_eq!(chunks.len(), 1);
    }

    #[tokio::test]
    async fn test_delete_operations() {
        let repo = MockChunkRepository::new();

        let delete_result = repo.delete("test-collection", "test-id").await;
        assert!(delete_result.is_ok());

        let delete_collection_result = repo.delete_collection("test-collection").await;
        assert!(delete_collection_result.is_ok());
    }

    #[tokio::test]
    async fn test_stats() {
        let repo = MockChunkRepository::new();

        let result = repo.stats().await;
        assert!(result.is_ok());

        let stats = result.unwrap();
        assert_eq!(stats.total_chunks, 0);
        assert_eq!(stats.total_collections, 0);
        assert_eq!(stats.storage_size_bytes, 0);
        assert_eq!(stats.avg_chunk_size_bytes, 0.0);
    }

    #[test]
    fn test_chunk_repository_trait_object() {
        // Test that we can use ChunkRepository as a trait object
        let repo: Box<dyn ChunkRepository> = Box::new(MockChunkRepository::new());
        // Just test that the trait object can be created
        assert!(true);
    }
}