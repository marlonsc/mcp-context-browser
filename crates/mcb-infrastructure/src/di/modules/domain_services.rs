//! Domain Services DI Module
//!
//! Provides domain service implementations that can be injected into the server.
//! These services implement domain interfaces using infrastructure components.
//!
//! ## Runtime Factory Pattern
//!
//! Services are created via `DomainServicesFactory::create_services()` at runtime
//! rather than through Shaku DI, because they require runtime configuration
//! (embedding provider, vector store, cache).

use crate::cache::provider::SharedCacheProvider;
use crate::config::AppConfig;
use crate::crypto::CryptoService;
use mcb_application::domain_services::search::{
    ContextServiceInterface, IndexingServiceInterface, SearchServiceInterface,
};
use mcb_application::ports::providers::{
    EmbeddingProvider, LanguageChunkingProvider, VectorStoreProvider,
};
use mcb_application::use_cases::{ContextServiceImpl, IndexingServiceImpl, SearchServiceImpl};
use mcb_domain::error::Result;
use std::sync::Arc;

use super::super::bootstrap::AppContext;

/// Domain services container
#[derive(Clone)]
pub struct DomainServicesContainer {
    pub context_service: Arc<dyn ContextServiceInterface>,
    pub search_service: Arc<dyn SearchServiceInterface>,
    pub indexing_service: Arc<dyn IndexingServiceInterface>,
}

/// Dependencies for creating domain services
///
/// Groups all required dependencies into a single struct to reduce
/// function parameter count (KISS principle).
pub struct ServiceDependencies {
    /// Cache provider for caching operations
    pub cache: SharedCacheProvider,
    /// Crypto service (reserved for future use)
    pub crypto: CryptoService,
    /// Application configuration (reserved for future use)
    pub config: AppConfig,
    /// Embedding provider for vector embeddings
    pub embedding_provider: Arc<dyn EmbeddingProvider>,
    /// Vector store provider for similarity search
    pub vector_store_provider: Arc<dyn VectorStoreProvider>,
    /// Language chunker for code processing
    pub language_chunker: Arc<dyn LanguageChunkingProvider>,
}

/// Domain services factory - creates services with runtime dependencies
pub struct DomainServicesFactory;

impl DomainServicesFactory {
    /// Create domain services using infrastructure components
    pub async fn create_services(deps: ServiceDependencies) -> Result<DomainServicesContainer> {
        // Create context service with dependencies
        let context_service: Arc<dyn ContextServiceInterface> =
            Arc::new(mcb_application::use_cases::ContextServiceImpl::new(
                deps.cache.into(),
                deps.embedding_provider,
                deps.vector_store_provider,
            ));

        // Create search service with context service dependency
        let search_service: Arc<dyn SearchServiceInterface> = Arc::new(
            mcb_application::use_cases::SearchServiceImpl::new(Arc::clone(&context_service)),
        );

        // Create indexing service with context service and language chunker dependency
        let indexing_service: Arc<dyn IndexingServiceInterface> =
            Arc::new(mcb_application::use_cases::IndexingServiceImpl::new(
                Arc::clone(&context_service),
                deps.language_chunker,
            ));

        Ok(DomainServicesContainer {
            context_service,
            search_service,
            indexing_service,
        })
    }

    /// Create indexing service from app context
    pub async fn create_indexing_service(
        app_context: &AppContext,
    ) -> Result<Arc<dyn IndexingServiceInterface>> {
        // Get providers from resolved providers
        let language_chunker = Arc::clone(&app_context.providers.language);

        // Create context service first (dependency)
        let context_service = Self::create_context_service(app_context).await?;

        Ok(Arc::new(IndexingServiceImpl::new(
            context_service,
            language_chunker,
        )))
    }

    /// Create context service from app context
    pub async fn create_context_service(
        app_context: &AppContext,
    ) -> Result<Arc<dyn ContextServiceInterface>> {
        // Get providers from resolved providers
        let cache_provider = Arc::clone(&app_context.providers.cache);
        let embedding_provider = Arc::clone(&app_context.providers.embedding);
        let vector_store_provider = Arc::clone(&app_context.providers.vector_store);

        Ok(Arc::new(ContextServiceImpl::new(
            cache_provider,
            embedding_provider,
            vector_store_provider,
        )))
    }

    /// Create search service from app context
    pub async fn create_search_service(
        app_context: &AppContext,
    ) -> Result<Arc<dyn SearchServiceInterface>> {
        // Create context service first (dependency)
        let context_service = Self::create_context_service(app_context).await?;

        Ok(Arc::new(SearchServiceImpl::new(context_service)))
    }
}

// Note: Service implementations have been moved to mcb-application crate
// This module now provides a factory for creating services at runtime
