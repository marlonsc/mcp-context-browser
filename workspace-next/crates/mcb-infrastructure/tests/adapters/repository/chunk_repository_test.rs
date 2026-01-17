//! Chunk Repository Tests
//!
//! Tests for the VectorStoreChunkRepository implementation using mock providers.

use async_trait::async_trait;
use mcb_domain::entities::CodeChunk;
use mcb_domain::error::Result;
use mcb_domain::ports::providers::embedding::EmbeddingProvider;
use mcb_domain::ports::providers::vector_store::{VectorStoreAdmin, VectorStoreProvider};
use mcb_domain::repositories::chunk_repository::RepositoryStats;
use mcb_domain::repositories::ChunkRepository;
use mcb_domain::value_objects::{Embedding, SearchResult};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Mock embedding provider for testing
struct MockEmbeddingProvider {
    dimensions: usize,
    embed_count: AtomicUsize,
}

impl MockEmbeddingProvider {
    fn new(dimensions: usize) -> Self {
        Self {
            dimensions,
            embed_count: AtomicUsize::new(0),
        }
    }

    fn embed_count(&self) -> usize {
        self.embed_count.load(Ordering::SeqCst)
    }
}

#[async_trait]
impl EmbeddingProvider for MockEmbeddingProvider {
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>> {
        let mut results = Vec::with_capacity(texts.len());
        for _ in texts {
            self.embed_count.fetch_add(1, Ordering::SeqCst);
            results.push(Embedding {
                vector: vec![0.1; self.dimensions],
                model: "mock".to_string(),
                dimensions: self.dimensions,
            });
        }
        Ok(results)
    }

    fn dimensions(&self) -> usize {
        self.dimensions
    }

    fn provider_name(&self) -> &str {
        "mock-embedding"
    }
}

/// Mock vector store for testing
struct MockVectorStore {
    collections: Arc<Mutex<HashMap<String, Vec<StoredVector>>>>,
    id_counter: AtomicUsize,
}

struct StoredVector {
    id: String,
    #[allow(dead_code)]
    embedding: Vec<f32>,
    metadata: HashMap<String, serde_json::Value>,
}

impl MockVectorStore {
    fn new() -> Self {
        Self {
            collections: Arc::new(Mutex::new(HashMap::new())),
            id_counter: AtomicUsize::new(0),
        }
    }
}

#[async_trait]
impl VectorStoreAdmin for MockVectorStore {
    async fn collection_exists(&self, name: &str) -> Result<bool> {
        let collections = self.collections.lock().await;
        Ok(collections.contains_key(name))
    }

    async fn get_stats(&self, collection: &str) -> Result<HashMap<String, serde_json::Value>> {
        let collections = self.collections.lock().await;
        let count = collections.get(collection).map(|c| c.len()).unwrap_or(0);

        let mut stats = HashMap::new();
        stats.insert(
            "total_vectors".to_string(),
            serde_json::json!(count as u64),
        );
        stats.insert("storage_size_bytes".to_string(), serde_json::json!(0));
        Ok(stats)
    }

    async fn flush(&self, _collection: &str) -> Result<()> {
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "mock-vector-store"
    }
}

#[async_trait]
impl VectorStoreProvider for MockVectorStore {
    async fn create_collection(&self, name: &str, _dimensions: usize) -> Result<()> {
        let mut collections = self.collections.lock().await;
        collections.insert(name.to_string(), Vec::new());
        Ok(())
    }

    async fn delete_collection(&self, name: &str) -> Result<()> {
        let mut collections = self.collections.lock().await;
        collections.remove(name);
        Ok(())
    }

    async fn insert_vectors(
        &self,
        collection: &str,
        embeddings: &[Embedding],
        metadata: Vec<HashMap<String, serde_json::Value>>,
    ) -> Result<Vec<String>> {
        let mut collections = self.collections.lock().await;
        let coll = collections.entry(collection.to_string()).or_default();

        let mut ids = Vec::new();
        for (emb, meta) in embeddings.iter().zip(metadata.into_iter()) {
            let id = format!("vec_{}", self.id_counter.fetch_add(1, Ordering::SeqCst));
            coll.push(StoredVector {
                id: id.clone(),
                embedding: emb.vector.clone(),
                metadata: meta,
            });
            ids.push(id);
        }
        Ok(ids)
    }

    async fn search_similar(
        &self,
        collection: &str,
        _query: &[f32],
        limit: usize,
        _filter: Option<&str>,
    ) -> Result<Vec<SearchResult>> {
        let collections = self.collections.lock().await;
        let coll = collections
            .get(collection)
            .map(|c| c.as_slice())
            .unwrap_or(&[]);

        let results: Vec<SearchResult> = coll
            .iter()
            .take(limit)
            .map(|v| SearchResult {
                id: v.id.clone(),
                file_path: v
                    .metadata
                    .get("file_path")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                start_line: v
                    .metadata
                    .get("start_line")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32,
                content: v
                    .metadata
                    .get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                score: 0.9,
                language: v
                    .metadata
                    .get("language")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
            })
            .collect();
        Ok(results)
    }

    async fn get_vectors_by_ids(
        &self,
        collection: &str,
        ids: &[String],
    ) -> Result<Vec<SearchResult>> {
        let collections = self.collections.lock().await;
        let coll = collections
            .get(collection)
            .map(|c| c.as_slice())
            .unwrap_or(&[]);

        let results: Vec<SearchResult> = coll
            .iter()
            .filter(|v| ids.contains(&v.id))
            .map(|v| SearchResult {
                id: v.id.clone(),
                file_path: v
                    .metadata
                    .get("file_path")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                start_line: v
                    .metadata
                    .get("start_line")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32,
                content: v
                    .metadata
                    .get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                score: 1.0,
                language: v
                    .metadata
                    .get("language")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
            })
            .collect();
        Ok(results)
    }

    async fn delete_vectors(&self, collection: &str, ids: &[String]) -> Result<()> {
        let mut collections = self.collections.lock().await;
        if let Some(coll) = collections.get_mut(collection) {
            coll.retain(|v| !ids.contains(&v.id));
        }
        Ok(())
    }

    async fn list_vectors(&self, collection: &str, limit: usize) -> Result<Vec<SearchResult>> {
        self.search_similar(collection, &[], limit, None).await
    }
}

/// Test chunk repository implementation
struct TestChunkRepository {
    embedding_provider: Arc<dyn EmbeddingProvider>,
    vector_store_provider: Arc<dyn VectorStoreProvider>,
}

impl TestChunkRepository {
    fn new(
        embedding_provider: Arc<dyn EmbeddingProvider>,
        vector_store_provider: Arc<dyn VectorStoreProvider>,
    ) -> Self {
        Self {
            embedding_provider,
            vector_store_provider,
        }
    }

    fn collection_name(&self, collection: &str) -> String {
        format!("mcp_chunks_{}", collection)
    }
}

#[async_trait]
impl ChunkRepository for TestChunkRepository {
    async fn save(&self, collection: &str, chunk: &CodeChunk) -> Result<String> {
        let chunks = vec![chunk.clone()];
        let ids = self.save_batch(collection, &chunks).await?;
        ids.into_iter()
            .next()
            .ok_or_else(|| mcb_domain::error::Error::internal("No ID returned".to_string()))
    }

    async fn save_batch(&self, collection: &str, chunks: &[CodeChunk]) -> Result<Vec<String>> {
        if chunks.is_empty() {
            return Ok(vec![]);
        }

        let collection_name = self.collection_name(collection);
        let texts: Vec<String> = chunks.iter().map(|c| c.content.clone()).collect();
        let embeddings = self.embedding_provider.embed_batch(&texts).await?;

        let metadata: Vec<HashMap<String, serde_json::Value>> = chunks
            .iter()
            .map(|chunk| {
                let mut meta = HashMap::new();
                meta.insert("content".to_string(), serde_json::json!(chunk.content));
                meta.insert("file_path".to_string(), serde_json::json!(chunk.file_path));
                meta.insert(
                    "start_line".to_string(),
                    serde_json::json!(chunk.start_line),
                );
                meta.insert("end_line".to_string(), serde_json::json!(chunk.end_line));
                meta.insert("language".to_string(), serde_json::json!(&chunk.language));
                meta
            })
            .collect();

        if !self
            .vector_store_provider
            .collection_exists(&collection_name)
            .await?
        {
            self.vector_store_provider
                .create_collection(&collection_name, self.embedding_provider.dimensions())
                .await?;
        }

        self.vector_store_provider
            .insert_vectors(&collection_name, &embeddings, metadata)
            .await
    }

    async fn find_by_id(&self, collection: &str, id: &str) -> Result<Option<CodeChunk>> {
        let collection_name = self.collection_name(collection);
        let results = self
            .vector_store_provider
            .get_vectors_by_ids(&collection_name, &[id.to_string()])
            .await?;

        Ok(results.into_iter().next().map(|r| CodeChunk {
            id: r.id,
            content: r.content,
            file_path: r.file_path,
            start_line: r.start_line,
            end_line: r.start_line,
            language: r.language,
            metadata: serde_json::json!({}),
        }))
    }

    async fn find_by_collection(&self, collection: &str, limit: usize) -> Result<Vec<CodeChunk>> {
        let collection_name = self.collection_name(collection);
        let results = self
            .vector_store_provider
            .list_vectors(&collection_name, limit)
            .await?;

        Ok(results
            .into_iter()
            .map(|r| CodeChunk {
                id: r.id,
                content: r.content,
                file_path: r.file_path,
                start_line: r.start_line,
                end_line: r.start_line,
                language: r.language,
                metadata: serde_json::json!({}),
            })
            .collect())
    }

    async fn delete(&self, collection: &str, id: &str) -> Result<()> {
        let collection_name = self.collection_name(collection);
        self.vector_store_provider
            .delete_vectors(&collection_name, &[id.to_string()])
            .await
    }

    async fn delete_collection(&self, collection: &str) -> Result<()> {
        let collection_name = self.collection_name(collection);
        self.vector_store_provider
            .delete_collection(&collection_name)
            .await
    }

    async fn stats(&self) -> Result<RepositoryStats> {
        Ok(RepositoryStats {
            total_chunks: 0,
            total_collections: 1,
            storage_size_bytes: 0,
            avg_chunk_size_bytes: 0.0,
        })
    }
}

fn create_test_chunk(id: &str, content: &str) -> CodeChunk {
    CodeChunk {
        id: id.to_string(),
        content: content.to_string(),
        file_path: "test.rs".to_string(),
        start_line: 1,
        end_line: 10,
        language: "rust".to_string(),
        metadata: serde_json::json!({}),
    }
}

// ============================================================================
// Tests
// ============================================================================

#[tokio::test]
async fn test_save_and_find_chunk() {
    let embedding_provider = Arc::new(MockEmbeddingProvider::new(384));
    let vector_store = Arc::new(MockVectorStore::new());
    let repo = TestChunkRepository::new(embedding_provider.clone(), vector_store);

    let chunk = create_test_chunk("test-1", "fn main() { println!(\"hello\"); }");
    let id = repo.save("test-collection", &chunk).await.unwrap();

    assert!(!id.is_empty(), "Should return a valid ID");
    assert_eq!(embedding_provider.embed_count(), 1, "Should embed once");
}

#[tokio::test]
async fn test_save_batch() {
    let embedding_provider = Arc::new(MockEmbeddingProvider::new(384));
    let vector_store = Arc::new(MockVectorStore::new());
    let repo = TestChunkRepository::new(embedding_provider.clone(), vector_store);

    let chunks = vec![
        create_test_chunk("chunk-1", "fn foo() {}"),
        create_test_chunk("chunk-2", "fn bar() {}"),
        create_test_chunk("chunk-3", "fn baz() {}"),
    ];

    let ids = repo.save_batch("batch-test", &chunks).await.unwrap();

    assert_eq!(ids.len(), 3, "Should return 3 IDs");
    assert_eq!(embedding_provider.embed_count(), 3, "Should embed 3 times");
}

#[tokio::test]
async fn test_find_by_id() {
    let embedding_provider = Arc::new(MockEmbeddingProvider::new(384));
    let vector_store = Arc::new(MockVectorStore::new());
    let repo = TestChunkRepository::new(embedding_provider, vector_store);

    let chunk = create_test_chunk("find-test", "let x = 42;");
    let id = repo.save("find-collection", &chunk).await.unwrap();

    let found = repo.find_by_id("find-collection", &id).await.unwrap();

    assert!(found.is_some(), "Should find the chunk");
    let found_chunk = found.unwrap();
    assert_eq!(found_chunk.content, "let x = 42;");
}

#[tokio::test]
async fn test_find_by_collection() {
    let embedding_provider = Arc::new(MockEmbeddingProvider::new(384));
    let vector_store = Arc::new(MockVectorStore::new());
    let repo = TestChunkRepository::new(embedding_provider, vector_store);

    let chunks = vec![
        create_test_chunk("c1", "code 1"),
        create_test_chunk("c2", "code 2"),
    ];
    repo.save_batch("list-collection", &chunks).await.unwrap();

    let found = repo
        .find_by_collection("list-collection", 10)
        .await
        .unwrap();

    assert_eq!(found.len(), 2, "Should find 2 chunks");
}

#[tokio::test]
async fn test_delete_chunk() {
    let embedding_provider = Arc::new(MockEmbeddingProvider::new(384));
    let vector_store = Arc::new(MockVectorStore::new());
    let repo = TestChunkRepository::new(embedding_provider, vector_store);

    let chunk = create_test_chunk("delete-test", "to be deleted");
    let id = repo.save("delete-collection", &chunk).await.unwrap();

    // Verify it exists
    let found = repo.find_by_id("delete-collection", &id).await.unwrap();
    assert!(found.is_some(), "Should find chunk before delete");

    // Delete it
    repo.delete("delete-collection", &id).await.unwrap();

    // Verify it's gone
    let found_after = repo.find_by_id("delete-collection", &id).await.unwrap();
    assert!(found_after.is_none(), "Should not find chunk after delete");
}

#[tokio::test]
async fn test_delete_collection() {
    let embedding_provider = Arc::new(MockEmbeddingProvider::new(384));
    let vector_store = Arc::new(MockVectorStore::new());
    let repo = TestChunkRepository::new(embedding_provider, vector_store);

    let chunk = create_test_chunk("del-coll-test", "collection to delete");
    repo.save("to-delete", &chunk).await.unwrap();

    // Delete collection
    repo.delete_collection("to-delete").await.unwrap();

    // Verify collection is empty
    let found = repo.find_by_collection("to-delete", 10).await.unwrap();
    assert!(found.is_empty(), "Collection should be empty after delete");
}

#[tokio::test]
async fn test_empty_batch_save() {
    let embedding_provider = Arc::new(MockEmbeddingProvider::new(384));
    let vector_store = Arc::new(MockVectorStore::new());
    let repo = TestChunkRepository::new(embedding_provider.clone(), vector_store);

    let ids = repo.save_batch("empty-test", &[]).await.unwrap();

    assert!(ids.is_empty(), "Empty batch should return empty IDs");
    assert_eq!(
        embedding_provider.embed_count(),
        0,
        "Should not embed for empty batch"
    );
}
