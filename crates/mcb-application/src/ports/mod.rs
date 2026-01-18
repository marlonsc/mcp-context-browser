//! Domain Port Interfaces
//!
//! Defines all boundary contracts between domain and external layers.
//! Ports are organized by their purpose and enable dependency injection
//! with clear separation of concerns.
//!
//! ## Organization
//!
//! - **providers/** - External service providers (embeddings, vector stores, search)
//! - **infrastructure/** - Infrastructure services (sync, snapshots)
//! - **registry/** - Auto-registration system for plugin providers
//! - **services.rs** - Application service interfaces (context, search, indexing)
//! - **admin.rs** - Administrative interfaces for system management

/// Administrative interfaces for system management and monitoring
pub mod admin;
/// Infrastructure service ports
pub mod infrastructure;
/// External service provider ports
pub mod providers;
/// Provider registry for dynamic provider discovery
pub mod registry;
/// Application service interfaces
pub mod services;

// Re-export commonly used port traits for convenience
pub use admin::{
    IndexingOperation, IndexingOperationsInterface, PerformanceMetricsData,
    PerformanceMetricsInterface,
};
pub use infrastructure::snapshot::SyncProvider;
pub use infrastructure::{
    AuthServiceInterface, EventBusProvider, LockGuard, LockProvider, SnapshotProvider,
    StateStoreProvider, SyncCoordinator, SystemMetrics, SystemMetricsCollectorInterface,
};
pub use providers::{EmbeddingProvider, HybridSearchProvider, VectorStoreProvider};
pub use registry::{
    EmbeddingProviderEntry, EmbeddingProviderConfig,
    VectorStoreProviderEntry, VectorStoreProviderConfig,
    CacheProviderEntry, CacheProviderConfig,
    LanguageProviderEntry, LanguageProviderConfig,
    resolve_embedding_provider, list_embedding_providers,
    resolve_vector_store_provider, list_vector_store_providers,
    resolve_cache_provider, list_cache_providers,
    resolve_language_provider, list_language_providers,
};
pub use services::{
    BatchIndexingServiceInterface, ChunkingOrchestratorInterface, ContextServiceInterface,
    IndexingResult, IndexingServiceInterface, IndexingStats, IndexingStatus,
    SearchServiceInterface,
};
