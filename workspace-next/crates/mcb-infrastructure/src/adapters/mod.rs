//! Infrastructure Adapters
//!
//! Provides adapter interfaces and null implementations for DI integration.
//! Following Clean Architecture: adapters implement domain interfaces.
//!
//! **ARCHITECTURE**:
//! - Provider implementations are in mcb-providers crate
//! - This module provides null implementations for testing
//! - Real implementations are injected at runtime

// Repository adapters
pub mod repository {
    use async_trait::async_trait;
    use mcb_domain::entities::CodeChunk;
    use mcb_domain::error::Result;
    use shaku::Interface;

    /// Chunk storage repository interface
    #[async_trait]
    pub trait ChunkRepository: Interface + Send + Sync {
        async fn store_chunks(&self, chunks: Vec<CodeChunk>) -> Result<()>;
        async fn get_chunks(&self, ids: Vec<String>) -> Result<Vec<CodeChunk>>;
        async fn delete_chunks(&self, ids: Vec<String>) -> Result<()>;
    }

    /// Search result repository interface
    #[async_trait]
    pub trait SearchRepository: Interface + Send + Sync {
        async fn store_search_result(&self, query: &str, results: Vec<CodeChunk>) -> Result<()>;
        async fn get_cached_results(&self, query: &str) -> Result<Option<Vec<CodeChunk>>>;
    }

    /// Null implementation for testing
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
        async fn store_chunks(&self, _chunks: Vec<CodeChunk>) -> Result<()> { Ok(()) }
        async fn get_chunks(&self, _ids: Vec<String>) -> Result<Vec<CodeChunk>> { Ok(vec![]) }
        async fn delete_chunks(&self, _ids: Vec<String>) -> Result<()> { Ok(()) }
    }

    /// Null implementation for testing
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
        async fn store_search_result(&self, _query: &str, _results: Vec<CodeChunk>) -> Result<()> { Ok(()) }
        async fn get_cached_results(&self, _query: &str) -> Result<Option<Vec<CodeChunk>>> { Ok(None) }
    }
}
