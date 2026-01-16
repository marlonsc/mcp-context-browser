//! Domain Services DI Module
//!
//! Provides domain service implementations that can be injected into the server.
//! These services implement domain interfaces using infrastructure components.

use crate::adapters::chunking::{language_from_extension, IntelligentChunker};
use crate::cache::provider::SharedCacheProvider;
use crate::config::AppConfig;
use crate::crypto::CryptoService;
use crate::health::HealthRegistry;
use mcb_domain::domain_services::search::{
    ContextServiceInterface, IndexingServiceInterface, SearchServiceInterface,
};
use mcb_domain::entities::CodeChunk;
use mcb_domain::error::Result;
use mcb_domain::ports::providers::{EmbeddingProvider, VectorStoreProvider};
use mcb_domain::repositories::{chunk_repository::RepositoryStats, search_repository::SearchStats};
use mcb_domain::value_objects::{Embedding, SearchResult};
use serde_json::json;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

/// Domain services container
#[derive(Clone)]
pub struct DomainServicesContainer {
    pub context_service: Arc<dyn ContextServiceInterface>,
    pub search_service: Arc<dyn SearchServiceInterface>,
    pub indexing_service: Arc<dyn IndexingServiceInterface>,
}

/// Domain services factory
pub struct DomainServicesFactory;

impl DomainServicesFactory {
    /// Create domain services using infrastructure components
    pub async fn create_services(
        cache: SharedCacheProvider,
        crypto: CryptoService,
        _health: HealthRegistry,
        config: AppConfig,
        embedding_provider: Arc<dyn EmbeddingProvider>,
        vector_store_provider: Arc<dyn VectorStoreProvider>,
    ) -> Result<DomainServicesContainer> {
        // Create context service implementation
        let context_service: Arc<dyn ContextServiceInterface> = Arc::new(ContextServiceImpl::new(
            cache.clone(),
            crypto.clone(),
            config.clone(),
            embedding_provider,
            vector_store_provider,
        ));

        // Create search service implementation
        let search_service: Arc<dyn SearchServiceInterface> = Arc::new(SearchServiceImpl::new(
            cache.clone(),
            config.clone(),
            context_service.clone(),
        ));

        // Create indexing service implementation (needs context_service)
        let indexing_service: Arc<dyn IndexingServiceInterface> =
            Arc::new(IndexingServiceImpl::new(
                cache.clone(),
                crypto.clone(),
                config.clone(),
                context_service.clone(),
            ));

        Ok(DomainServicesContainer {
            context_service,
            search_service,
            indexing_service,
        })
    }
}

/// Context service implementation
pub struct ContextServiceImpl {
    cache: SharedCacheProvider,
    #[allow(dead_code)]
    crypto: CryptoService,
    #[allow(dead_code)]
    config: AppConfig,
    embedding_provider: Arc<dyn EmbeddingProvider>,
    vector_store_provider: Arc<dyn VectorStoreProvider>,
}

impl ContextServiceImpl {
    pub fn new(
        cache: SharedCacheProvider,
        crypto: CryptoService,
        config: AppConfig,
        embedding_provider: Arc<dyn EmbeddingProvider>,
        vector_store_provider: Arc<dyn VectorStoreProvider>,
    ) -> Self {
        Self {
            cache,
            crypto,
            config,
            embedding_provider,
            vector_store_provider,
        }
    }
}

#[async_trait::async_trait]
impl ContextServiceInterface for ContextServiceImpl {
    async fn initialize(&self, collection: &str) -> Result<()> {
        // Check if collection exists in vector store, create if not
        if !self.vector_store_provider.collection_exists(collection).await? {
            let dimensions = self.embedding_provider.dimensions();
            self.vector_store_provider.create_collection(collection, dimensions).await?;
        }

        // Also track in cache for metadata
        let collection_key = format!("collection:{}", collection);
        self.cache
            .set_json(
                &collection_key,
                "\"initialized\"",
                crate::cache::config::CacheEntryConfig::default(),
            )
            .await?;
        Ok(())
    }

    async fn store_chunks(&self, collection: &str, chunks: &[CodeChunk]) -> Result<()> {
        // Generate embeddings for each chunk
        let texts: Vec<String> = chunks.iter().map(|c| c.content.clone()).collect();
        let embeddings = self.embedding_provider.embed_batch(&texts).await?;

        // Build metadata for each chunk
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

        // Insert into vector store
        self.vector_store_provider
            .insert_vectors(collection, &embeddings, metadata)
            .await?;

        // Update collection metadata in cache
        let meta_key = format!("collection:{}:meta", collection);
        let chunk_count = chunks.len();
        self.cache
            .set(
                &meta_key,
                &chunk_count,
                crate::cache::config::CacheEntryConfig::default(),
            )
            .await?;

        Ok(())
    }

    async fn search_similar(
        &self,
        collection: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        // Generate embedding for the query
        let query_embedding = self.embedding_provider.embed(query).await?;

        // Search in vector store
        let results = self.vector_store_provider
            .search_similar(collection, &query_embedding.vector, limit, None)
            .await?;

        Ok(results)
    }

    async fn embed_text(&self, text: &str) -> Result<Embedding> {
        // Use the configured embedding provider
        self.embedding_provider.embed(text).await
    }

    async fn clear_collection(&self, collection: &str) -> Result<()> {
        // Delete the collection from vector store
        if self.vector_store_provider.collection_exists(collection).await? {
            self.vector_store_provider.delete_collection(collection).await?;
        }

        // Also clear cache metadata
        let collection_key = format!("collection:{}", collection);
        self.cache.delete(&collection_key).await?;

        let meta_key = format!("collection:{}:meta", collection);
        self.cache.delete(&meta_key).await?;

        Ok(())
    }

    async fn get_stats(&self) -> Result<(RepositoryStats, SearchStats)> {
        // Return placeholder stats
        let repo_stats = RepositoryStats {
            total_chunks: 0,
            total_collections: 0,
            storage_size_bytes: 0,
            avg_chunk_size_bytes: 0.0,
        };

        let search_stats = SearchStats {
            total_queries: 0,
            avg_response_time_ms: 0.0,
            cache_hit_rate: 0.0,
            indexed_documents: 0,
        };

        Ok((repo_stats, search_stats))
    }

    fn embedding_dimensions(&self) -> usize {
        self.embedding_provider.dimensions()
    }
}

/// Search service implementation
pub struct SearchServiceImpl {
    #[allow(dead_code)]
    cache: SharedCacheProvider,
    #[allow(dead_code)]
    config: AppConfig,
    context_service: Arc<dyn ContextServiceInterface>,
}

impl SearchServiceImpl {
    pub fn new(
        cache: SharedCacheProvider,
        config: AppConfig,
        context_service: Arc<dyn ContextServiceInterface>,
    ) -> Self {
        Self {
            cache,
            config,
            context_service,
        }
    }
}

#[async_trait::async_trait]
impl SearchServiceInterface for SearchServiceImpl {
    async fn search(
        &self,
        collection: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        // Delegate to context service for semantic search
        // Future: add BM25 scoring and hybrid ranking
        self.context_service.search_similar(collection, query, limit).await
    }
}

/// Indexing service implementation
pub struct IndexingServiceImpl {
    #[allow(dead_code)]
    cache: SharedCacheProvider,
    #[allow(dead_code)]
    crypto: CryptoService,
    #[allow(dead_code)]
    config: AppConfig,
    context_service: Arc<dyn ContextServiceInterface>,
    chunker: IntelligentChunker,
}

impl IndexingServiceImpl {
    pub fn new(
        cache: SharedCacheProvider,
        crypto: CryptoService,
        config: AppConfig,
        context_service: Arc<dyn ContextServiceInterface>,
    ) -> Self {
        Self {
            cache,
            crypto,
            config,
            context_service,
            chunker: IntelligentChunker::new(),
        }
    }
}

#[async_trait::async_trait]
impl IndexingServiceInterface for IndexingServiceImpl {
    async fn index_codebase(
        &self,
        path: &Path,
        collection: &str,
    ) -> Result<mcb_domain::domain_services::search::IndexingResult> {
        use tokio::fs;

        // Initialize collection
        self.context_service.initialize(collection).await?;
        let mut files_processed = 0;
        let mut chunks_created = 0;
        let mut files_skipped = 0;
        let mut errors = Vec::new();

        // Discover files recursively
        let mut files_to_process = Vec::new();
        let mut dirs_to_visit = vec![path.to_path_buf()];

        while let Some(dir_path) = dirs_to_visit.pop() {
            let mut entries = match fs::read_dir(&dir_path).await {
                Ok(entries) => entries,
                Err(e) => {
                    errors.push(format!(
                        "Failed to read directory {}: {}",
                        dir_path.display(),
                        e
                    ));
                    continue;
                }
            };

            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();

                if path.is_dir() {
                    // Skip common directories
                    if !path.ends_with(".git")
                        && !path.ends_with("node_modules")
                        && !path.ends_with("target")
                        && !path.ends_with("__pycache__")
                    {
                        dirs_to_visit.push(path);
                    }
                } else if let Some(ext) = path.extension() {
                    // Process supported file types
                    let ext_str = ext.to_string_lossy().to_lowercase();
                    if matches!(
                        ext_str.as_str(),
                        "rs" | "py" | "js" | "ts" | "java" | "cpp" | "c" | "go"
                    ) {
                        files_to_process.push(path);
                    }
                }
            }
        }

        // Process files sequentially (simpler, avoids async borrowing issues)
        for file_path in files_to_process {
            match fs::read_to_string(&file_path).await {
                Ok(content) => {
                    let chunks = self.chunk_file_content(&content, &file_path);
                    if let Err(e) = self.context_service.store_chunks(collection, &chunks).await {
                        errors.push(format!(
                            "Failed to store chunks for {}: {}",
                            file_path.display(),
                            e
                        ));
                    } else {
                        files_processed += 1;
                        chunks_created += chunks.len();
                    }
                }
                Err(e) => {
                    errors.push(format!("Failed to read {}: {}", file_path.display(), e));
                    files_skipped += 1;
                }
            }
        }

        Ok(mcb_domain::domain_services::search::IndexingResult {
            files_processed,
            chunks_created,
            files_skipped,
            errors,
        })
    }

    fn get_status(&self) -> mcb_domain::domain_services::search::IndexingStatus {
        mcb_domain::domain_services::search::IndexingStatus {
            is_indexing: false, // Simple implementation
            progress: 0.0,
            current_file: None,
            total_files: 0,
            processed_files: 0,
        }
    }

    async fn clear_collection(&self, collection: &str) -> Result<()> {
        self.context_service.clear_collection(collection).await
    }
}

impl IndexingServiceImpl {
    fn chunk_file_content(&self, content: &str, path: &Path) -> Vec<CodeChunk> {
        // Use AST-based intelligent chunking when possible
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let language = language_from_extension(ext);
        let file_name = path.to_string_lossy().to_string();

        self.chunker.chunk_code(content, &file_name, &language)
    }
}
