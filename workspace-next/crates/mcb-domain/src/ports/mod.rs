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
//! - **admin.rs** - Administrative interfaces for system management

/// Administrative interfaces for system management and monitoring
pub mod admin;
/// Infrastructure service ports
pub mod infrastructure;
/// External service provider ports
pub mod providers;

// Re-export commonly used port traits for convenience
pub use admin::{
    IndexingOperation, IndexingOperationsInterface, PerformanceMetricsData,
    PerformanceMetricsInterface,
};
pub use infrastructure::{
    AuthServiceInterface, EventBusProvider, LockGuard, LockProvider, SnapshotProvider,
    StateStoreProvider, SyncCoordinator, SystemMetrics, SystemMetricsCollectorInterface,
};
pub use infrastructure::snapshot::SyncProvider;
pub use providers::{EmbeddingProvider, HybridSearchProvider, VectorStoreProvider};
