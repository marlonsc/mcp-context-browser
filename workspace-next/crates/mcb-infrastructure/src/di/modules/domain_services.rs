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
use mcb_application::use_cases::{ContextServiceImpl, IndexingServiceImpl, SearchServiceImpl};
use mcb_domain::domain_services::search::{
    ContextServiceInterface, IndexingServiceInterface, SearchServiceInterface,
};
use mcb_domain::error::Result;
use mcb_domain::ports::providers::{EmbeddingProvider, VectorStoreProvider};
use std::sync::Arc;

/// Domain services container
#[derive(Clone)]
pub struct DomainServicesContainer {
    pub context_service: Arc<dyn ContextServiceInterface>,
    pub search_service: Arc<dyn SearchServiceInterface>,
    pub indexing_service: Arc<dyn IndexingServiceInterface>,
}

/// Domain services factory - creates services with runtime dependencies
pub struct DomainServicesFactory;

impl DomainServicesFactory {
    /// Create domain services using infrastructure components
    pub async fn create_services(
        cache: SharedCacheProvider,
        _crypto: CryptoService,
        _config: AppConfig,
        embedding_provider: Arc<dyn EmbeddingProvider>,
        vector_store_provider: Arc<dyn VectorStoreProvider>,
        language_chunker: Arc<dyn mcb_domain::ports::providers::LanguageChunkingProvider>,
    ) -> Result<DomainServicesContainer> {
        // Create context service with dependencies
        let context_service: Arc<dyn ContextServiceInterface> = Arc::new(
            mcb_application::use_cases::ContextServiceImpl::new(
                cache.into(),
                embedding_provider,
                vector_store_provider,
            )
        );

        // Create search service with context service dependency
        let search_service: Arc<dyn SearchServiceInterface> = Arc::new(
            mcb_application::use_cases::SearchServiceImpl::new(Arc::clone(&context_service))
        );

        // Create indexing service with context service and language chunker dependency
        let indexing_service: Arc<dyn IndexingServiceInterface> = Arc::new(
            mcb_application::use_cases::IndexingServiceImpl::new(
                Arc::clone(&context_service),
                language_chunker,
            )
        );

        Ok(DomainServicesContainer {
            context_service,
            search_service,
            indexing_service,
        })
    }
}

// Note: Service implementations have been moved to mcb-application crate
// This module now provides a factory for creating services at runtime
