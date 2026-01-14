//! Adapters DI Module Implementation
//!
//! Contains HTTP client, external provider adapters, and repositories.
//!
//! ## Provider Strategy
//!
//! The module registers null providers (NullEmbeddingProvider, NullVectorStoreProvider)
//! as defaults. Production code uses `with_component_override` to inject
//! config-based providers at runtime.
//!
//! ## Repository Integration
//!
//! Repositories inject providers from this same module:
//! - VectorStoreChunkRepository injects EmbeddingProvider + VectorStoreProvider
//! - VectorStoreSearchRepository injects VectorStoreProvider

use shaku::module;

use super::traits::AdaptersModule;
use crate::adapters::http_client::HttpClientPool;
use crate::adapters::providers::embedding::NullEmbeddingProvider;
use crate::adapters::providers::vector_store::NullVectorStoreProvider;
use crate::adapters::repository::{VectorStoreChunkRepository, VectorStoreSearchRepository};

// Implementation of the AdaptersModule trait providing external service integrations.
// This module provides HTTP clients, embedding providers, vector stores, and repository implementations.
//
// Generated components:
// - `HttpClientPool`: Connection pool for HTTP requests to external APIs
// - `NullEmbeddingProvider`: Fallback embedding provider for testing/development
// - `NullVectorStoreProvider`: Fallback vector store provider for testing/development
// - `VectorStoreChunkRepository`: Repository for storing and retrieving code chunks
// - `VectorStoreSearchRepository`: Repository for semantic search operations
module! {
    pub AdaptersModuleImpl: AdaptersModule {
        components = [
            HttpClientPool,
            NullEmbeddingProvider,
            NullVectorStoreProvider,
            VectorStoreChunkRepository,
            VectorStoreSearchRepository
        ],
        providers = []
    }
}
