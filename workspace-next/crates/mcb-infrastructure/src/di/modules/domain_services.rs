//! Domain Services DI Module
//!
//! Provides domain service implementations that can be injected into the server.
//! These services implement domain interfaces using infrastructure components.

use crate::cache::provider::SharedCacheProvider;
use crate::config::AppConfig;
use crate::crypto::CryptoService;
use crate::health::HealthRegistry;
use mcb_domain::domain_services::chunking::CodeChunker;
use mcb_domain::domain_services::search::IndexingServiceInterface;
use mcb_domain::domain_services::search::{ContextServiceInterface, SearchServiceInterface};
use mcb_domain::entities::CodeChunk;
use mcb_domain::error::Result;
use mcb_domain::repositories::{chunk_repository::RepositoryStats, search_repository::SearchStats};
use mcb_domain::value_objects::{Embedding, SearchResult, config::SyncBatch, types::OperationType};
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
        health: HealthRegistry,
        config: AppConfig,
    ) -> Result<DomainServicesContainer> {
        // Create context service implementation
        let context_service = ContextServiceImpl::new(cache.clone(), crypto.clone(), config.clone());

        // Create search service implementation
        let search_service = SearchServiceImpl::new(cache.clone(), config.clone());

        // Create indexing service implementation
        let indexing_service = IndexingServiceImpl::new(cache.clone(), crypto.clone(), config.clone());

        Ok(DomainServicesContainer {
            context_service: Arc::new(context_service),
            search_service: Arc::new(search_service),
            indexing_service: Arc::new(indexing_service),
        })
    }
}

/// Context service implementation
pub struct ContextServiceImpl {
    cache: SharedCacheProvider,
    crypto: CryptoService,
    config: AppConfig,
}

impl ContextServiceImpl {
    pub fn new(cache: SharedCacheProvider, crypto: CryptoService, config: AppConfig) -> Self {
        Self { cache, crypto, config }
    }
}

#[async_trait::async_trait]
impl ContextServiceInterface for ContextServiceImpl {
    async fn initialize(&self, collection: &str) -> Result<()> {
        // Initialize collection-specific resources
        // For now, just ensure the collection key exists in cache
        let collection_key = format!("collection:{}", collection);
        self.cache.set(&collection_key, "initialized", crate::cache::config::CacheEntryConfig::default()).await?;
        Ok(())
    }

    async fn store_chunks(&self, collection: &str, chunks: &[CodeChunk]) -> Result<()> {
        // Store chunks in cache with collection prefix
        for chunk in chunks {
            let key = format!("chunk:{}:{}", collection, chunk.id);
            self.cache.set(&key, chunk, crate::cache::config::CacheEntryConfig::default()).await?;
        }

        // Update collection metadata
        let meta_key = format!("collection:{}:meta", collection);
        let chunk_count = chunks.len();
        self.cache.set(&meta_key, chunk_count, crate::cache::config::CacheEntryConfig::default()).await?;

        Ok(())
    }

    async fn search_similar(
        &self,
        collection: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        // Simple implementation: return cached chunks that contain the query
        // In a real implementation, this would use vector similarity search
        let pattern = format!("chunk:{}:*", collection);
        let mut results = Vec::new();

        // For now, return empty results as we don't have a full implementation
        // This would need integration with vector stores and embeddings
        Ok(results.into_iter().take(limit).collect())
    }

    async fn embed_text(&self, text: &str) -> Result<Embedding> {
        // Generate a simple hash-based embedding for demonstration
        // In a real implementation, this would use configured embedding providers
        let hash = self.crypto.sha256(text.as_bytes());
        let dimensions = 384; // Standard embedding dimension

        // Convert hash bytes to f32 values for embedding
        let mut values = Vec::with_capacity(dimensions);
        for i in 0..dimensions {
            let byte_idx = i % hash.len();
            values.push((hash[byte_idx] as f32) / 255.0);
        }

        Ok(Embedding {
            values,
            model: "simple-hash".to_string(),
        })
    }

    async fn clear_collection(&self, collection: &str) -> Result<()> {
        // In a real implementation, this would clear all collection data
        // For now, just remove the collection metadata
        let meta_key = format!("collection:{}:meta", collection);
        self.cache.delete(&meta_key).await?;
        Ok(())
    }

    async fn get_stats(&self) -> Result<(RepositoryStats, SearchStats)> {
        // Return placeholder stats
        let repo_stats = RepositoryStats {
            total_chunks: 0,
            total_collections: 0,
            avg_chunk_size: 0.0,
            last_updated: None,
        };

        let search_stats = SearchStats {
            total_searches: 0,
            avg_search_time_ms: 0.0,
            cache_hit_rate: 0.0,
        };

        Ok((repo_stats, search_stats))
    }

    fn embedding_dimensions(&self) -> usize {
        384 // Standard dimension for demonstration
    }
}

/// Search service implementation
pub struct SearchServiceImpl {
    cache: SharedCacheProvider,
    config: AppConfig,
}

impl SearchServiceImpl {
    pub fn new(cache: SharedCacheProvider, config: AppConfig) -> Self {
        Self { cache, config }
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
        // Simple text-based search implementation
        // In a real implementation, this would use semantic search with embeddings
        let mut results = Vec::new();

        // For demonstration, return empty results
        // A full implementation would:
        // 1. Generate embedding for query
        // 2. Search vector store for similar chunks
        // 3. Apply BM25 scoring
        // 4. Combine and rank results

        Ok(results.into_iter().take(limit).collect())
    }
}

/// Indexing service implementation
pub struct IndexingServiceImpl {
    cache: SharedCacheProvider,
    crypto: CryptoService,
    config: AppConfig,
}

impl IndexingServiceImpl {
    pub fn new(cache: SharedCacheProvider, crypto: CryptoService, config: AppConfig) -> Self {
        Self { cache, crypto, config }
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
        use futures::future::join_all;

        // Initialize collection
        self.context_service.initialize(collection).await?;

        let start_time = std::time::Instant::now();
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
                    errors.push(format!("Failed to read directory {}: {}", dir_path.display(), e));
                    continue;
                }
            };

            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();

                if path.is_dir() {
                    // Skip common directories
                    if !path.ends_with(".git") && !path.ends_with("node_modules") &&
                       !path.ends_with("target") && !path.ends_with("__pycache__") {
                        dirs_to_visit.push(path);
                    }
                } else if let Some(ext) = path.extension() {
                    // Process supported file types
                    let ext_str = ext.to_string_lossy().to_lowercase();
                    if matches!(ext_str.as_str(), "rs" | "py" | "js" | "ts" | "java" | "cpp" | "c" | "go") {
                        files_to_process.push(path);
                    }
                }
            }
        }

        // Process files in batches
        const BATCH_SIZE: usize = 10;
        for batch in files_to_process.chunks(BATCH_SIZE) {
            let batch_futures = batch.iter().map(|file_path| {
                async {
                    match fs::read_to_string(file_path).await {
                        Ok(content) => {
                            // Create simple chunks
                            let chunks = self.chunk_file_content(&content, file_path).await;
                            Ok((file_path.clone(), chunks))
                        }
                        Err(e) => Err((file_path.clone(), e.to_string())),
                    }
                }
            });

            let batch_results = join_all(batch_futures).await;

            for result in batch_results {
                match result {
                    Ok((file_path, chunks)) => {
                        if let Err(e) = self.context_service.store_chunks(collection, &chunks).await {
                            errors.push(format!("Failed to store chunks for {}: {}", file_path.display(), e));
                        } else {
                            files_processed += 1;
                            chunks_created += chunks.len();
                        }
                    }
                    Err((file_path, error)) => {
                        errors.push(format!("Failed to read {}: {}", file_path.display(), error));
                        files_skipped += 1;
                    }
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
    async fn chunk_file_content(&self, content: &str, path: &Path) -> Vec<CodeChunk> {
        // Simple chunking strategy - split by lines and create chunks
        let lines: Vec<&str> = content.lines().collect();
        let chunk_size = 50; // lines per chunk

        lines.chunks(chunk_size)
            .enumerate()
            .map(|(i, chunk_lines)| {
                let start_line = i * chunk_size + 1;
                let content = chunk_lines.join("\n");

                CodeChunk {
                    id: format!("{}_{}", path.display(), i),
                    file_path: path.to_string_lossy().to_string(),
                    content,
                    start_line,
                    end_line: start_line + chunk_lines.len() - 1,
                    language: path.extension()
                        .and_then(|ext| ext.to_str())
                        .unwrap_or("unknown")
                        .to_string(),
                    metadata: serde_json::json!({
                        "chunk_index": i,
                        "total_lines": lines.len()
                    }),
                }
            })
            .collect()
    }
}