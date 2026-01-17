//! Adapters Module Implementation - External Integrations
//!
//! This module provides adapters for external systems and services.
//! It follows the Shaku strict pattern with no external dependencies.
//!
//! ## Services Provided
//!
//! - Embedding provider (with null fallback for testing)
//! - Vector store provider (with null fallback for testing)
//! - Chunk repository for data persistence
//! - Search repository for semantic search operations

use shaku::module;

// Import null providers from mcb-providers for testing fallbacks
use mcb_providers::embedding::NullEmbeddingProvider;
use mcb_providers::vector_store::NullVectorStoreProvider;

// Import null repositories from adapters bridge
use crate::adapters::repository::{NullChunkRepository, NullSearchRepository};

// Import traits
use super::traits::AdaptersModule;

/// Adapters module implementation following Shaku strict pattern.
///
/// This module provides external service integrations with no dependencies.
/// Uses null providers as defaults for testing, with runtime overrides for production.
///
/// ## Provider Strategy
///
/// - **Null Providers as Defaults**: `NullEmbeddingProvider`, `NullVectorStoreProvider`
/// - **Runtime Overrides**: Production providers injected via `with_component_override()`
/// - **Null Repositories**: For testing, can be overridden for production
///
/// ## Construction
///
/// ```rust,ignore
/// let adapters = AdaptersModuleImpl::builder().build();
/// ```
///
/// ## Production Configuration
///
/// ```rust,ignore
/// let adapters = AdaptersModuleImpl::builder()
///     .with_component_override::<dyn EmbeddingProvider>(openai_provider)
///     .with_component_override::<dyn VectorStoreProvider>(milvus_provider)
///     .build();
/// ```
module! {
    pub AdaptersModuleImpl: AdaptersModule {
        components = [
            // Null providers (testing fallbacks, overridden in production)
            NullEmbeddingProvider,
            NullVectorStoreProvider,

            // Null repositories (testing fallbacks)
            NullChunkRepository,
            NullSearchRepository
        ],
        providers = []
    }
}