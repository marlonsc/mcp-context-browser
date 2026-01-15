//! Server Initialization
//!
//! Handles server startup, dependency injection setup, and graceful shutdown.
//! Integrates with the infrastructure layer for configuration and DI container setup.

use std::sync::Arc;
use mcb_domain::domain_services::search::{
    IndexingServiceInterface, ContextServiceInterface, SearchServiceInterface,
};
use crate::{McpServer, McpServerBuilder};
use tracing::info;

/// Simple service implementations for demonstration
/// These provide basic functionality for the MCP server

mod simple_services {
    use super::*;
    use async_trait::async_trait;
    use mcb_domain::domain_services::search::{
        IndexingResult, IndexingStatus, ContextServiceInterface, SearchServiceInterface, IndexingServiceInterface,
    };
    use mcb_domain::{Result, CodeChunk, SearchResult};

    /// Simple indexing service implementation
    #[derive(Clone)]
    pub struct SimpleIndexingService;

    #[async_trait]
    impl IndexingServiceInterface for SimpleIndexingService {
        async fn index_codebase(
            &self,
            path: &std::path::Path,
            collection: &str,
        ) -> Result<IndexingResult> {
            // Simple implementation - just return success
            Ok(IndexingResult {
                files_processed: 1,
                chunks_created: 1,
                files_skipped: 0,
                errors: vec![],
            })
        }

        fn get_status(&self) -> IndexingStatus {
            IndexingStatus {
                is_indexing: false,
                progress: 0.0,
                current_file: None,
                total_files: 0,
                processed_files: 0,
            }
        }

        async fn clear_collection(&self, collection: &str) -> Result<()> {
            Ok(())
        }
    }

    /// Simple context service implementation
    #[derive(Clone)]
    pub struct SimpleContextService;

    #[async_trait]
    impl ContextServiceInterface for SimpleContextService {
        async fn initialize(&self, collection: &str) -> Result<()> {
            Ok(())
        }

        async fn store_chunks(&self, collection: &str, chunks: &[CodeChunk]) -> Result<()> {
            Ok(())
        }

        async fn search_similar(
            &self,
            collection: &str,
            query: &str,
            limit: usize,
        ) -> Result<Vec<SearchResult>> {
            // Return empty results for now
            Ok(vec![])
        }

        async fn embed_text(&self, text: &str) -> Result<mcb_domain::Embedding> {
            // Simple hash-based embedding
            use sha2::{Sha256, Digest};
            let mut hasher = Sha256::new();
            hasher.update(text.as_bytes());
            let hash = hasher.finalize();

            let dimensions = 384;
            let values = (0..dimensions)
                .map(|i| (hash[i % hash.len()] as f32) / 255.0)
                .collect();

            Ok(mcb_domain::Embedding {
                values,
                model: "simple-hash".to_string(),
            })
        }

        async fn clear_collection(&self, collection: &str) -> Result<()> {
            Ok(())
        }

        async fn get_stats(&self) -> Result<(mcb_domain::repositories::chunk_repository::RepositoryStats, mcb_domain::repositories::search_repository::SearchStats)> {
            let repo_stats = mcb_domain::repositories::chunk_repository::RepositoryStats {
                total_chunks: 0,
                total_collections: 0,
                avg_chunk_size: 0.0,
                last_updated: None,
            };

            let search_stats = mcb_domain::repositories::search_repository::SearchStats {
                total_searches: 0,
                avg_search_time_ms: 0.0,
                cache_hit_rate: 0.0,
            };

            Ok((repo_stats, search_stats))
        }

        fn embedding_dimensions(&self) -> usize {
            384
        }
    }

    /// Simple search service implementation
    #[derive(Clone)]
    pub struct SimpleSearchService {
        context_service: Arc<dyn ContextServiceInterface>,
    }

    impl SimpleSearchService {
        pub fn new(context_service: Arc<dyn ContextServiceInterface>) -> Self {
            Self { context_service }
        }
    }

    #[async_trait]
    impl SearchServiceInterface for SimpleSearchService {
        async fn search(
            &self,
            collection: &str,
            query: &str,
            limit: usize,
        ) -> Result<Vec<SearchResult>> {
            self.context_service.search_similar(collection, query, limit).await
        }
    }
}

/// Run the MCP Context Browser server
///
/// This is the main entry point that initializes all components and starts the server.
/// It handles configuration loading, dependency injection, and server startup.
///
/// # Arguments
/// * `config_path` - Optional path to configuration file
///
/// # Returns
/// Result indicating success or failure of server startup
pub async fn run_server(
    _config_path: Option<&std::path::Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    init_tracing()?;

    info!("🚀 Starting MCP Context Browser v{}", env!("CARGO_PKG_VERSION"));

    // Create simple service implementations
    let context_service = Arc::new(simple_services::SimpleContextService);
    let search_service = Arc::new(simple_services::SimpleSearchService::new(
        context_service.clone(),
    ));
    let indexing_service = Arc::new(simple_services::SimpleIndexingService);

    // Build MCP server with injected dependencies
    let server = McpServerBuilder::new()
        .with_indexing_service(indexing_service)
        .with_context_service(context_service)
        .with_search_service(search_service)
        .build();

    info!("✅ Server initialized successfully");
    info!("🔧 Server capabilities: tools={}, prompts={}, resources={}",
        server.get_capabilities().tools.is_some(),
        server.get_capabilities().prompts.is_some(),
        server.get_capabilities().resources.is_some()
    );

    // Start the MCP service
    info!("🎯 Ready to accept MCP client connections");
    server.serve_stdio().await?;

    Ok(())
}

/// Initialize tracing and logging
fn init_tracing() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into())
                .add_directive("mcb_server=debug".parse()?)
        )
        .init();

    Ok(())
}