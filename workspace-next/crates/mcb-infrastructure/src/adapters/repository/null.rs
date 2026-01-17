//! Null repository implementations for testing and Shaku DI defaults
//!
//! These implementations provide no-op behavior for repositories.
//! They implement the interfaces defined in mcb-domain/repositories.

use async_trait::async_trait;
use mcb_domain::entities::CodeChunk;
use mcb_domain::repositories::{ChunkRepository, RepositoryStats, SearchRepository};
use mcb_domain::repositories::search_repository::SearchStats;
use mcb_domain::value_objects::search::SearchResult;

/// Null implementation for chunk repository - testing and Shaku DI default
#[derive(shaku::Component)]
#[shaku(interface = ChunkRepository)]
pub struct NullChunkRepository;

impl NullChunkRepository {
    pub fn new() -> Self { Self }
}

impl Default for NullChunkRepository {
    fn default() -> Self { Self::new() }
}

#[async_trait]
impl ChunkRepository for NullChunkRepository {
    async fn save(&self, _collection: &str, _chunk: &CodeChunk) -> mcb_domain::error::Result<String> {
        Ok("null-id".to_string())
    }

    async fn save_batch(&self, _collection: &str, chunks: &[CodeChunk]) -> mcb_domain::error::Result<Vec<String>> {
        Ok((0..chunks.len()).map(|i| format!("null-{}", i)).collect())
    }

    async fn find_by_id(&self, _collection: &str, _id: &str) -> mcb_domain::error::Result<Option<CodeChunk>> {
        Ok(None)
    }

    async fn find_by_collection(&self, _collection: &str, _limit: usize) -> mcb_domain::error::Result<Vec<CodeChunk>> {
        Ok(vec![])
    }

    async fn delete(&self, _collection: &str, _id: &str) -> mcb_domain::error::Result<()> {
        Ok(())
    }

    async fn delete_collection(&self, _collection: &str) -> mcb_domain::error::Result<()> {
        Ok(())
    }

    async fn stats(&self) -> mcb_domain::error::Result<RepositoryStats> {
        Ok(RepositoryStats {
            total_chunks: 0,
            total_collections: 0,
            storage_size_bytes: 0,
            avg_chunk_size_bytes: 0.0,
        })
    }
}

/// Null implementation for search repository - testing and Shaku DI default
#[derive(shaku::Component)]
#[shaku(interface = SearchRepository)]
pub struct NullSearchRepository;

impl NullSearchRepository {
    pub fn new() -> Self { Self }
}

impl Default for NullSearchRepository {
    fn default() -> Self { Self::new() }
}

#[async_trait]
impl SearchRepository for NullSearchRepository {
    async fn semantic_search(
        &self,
        _collection: &str,
        _query_vector: &[f32],
        _limit: usize,
        _filter: Option<&str>,
    ) -> mcb_domain::error::Result<Vec<SearchResult>> {
        Ok(vec![])
    }

    async fn index_for_hybrid_search(&self, _chunks: &[CodeChunk]) -> mcb_domain::error::Result<()> {
        Ok(())
    }

    async fn hybrid_search(
        &self,
        _collection: &str,
        _query: &str,
        _query_vector: &[f32],
        _limit: usize,
    ) -> mcb_domain::error::Result<Vec<SearchResult>> {
        Ok(vec![])
    }

    async fn clear_index(&self, _collection: &str) -> mcb_domain::error::Result<()> {
        Ok(())
    }

    async fn stats(&self) -> mcb_domain::error::Result<SearchStats> {
        Ok(SearchStats {
            total_queries: 0,
            avg_response_time_ms: 0.0,
            cache_hit_rate: 0.0,
            indexed_documents: 0,
        })
    }
}