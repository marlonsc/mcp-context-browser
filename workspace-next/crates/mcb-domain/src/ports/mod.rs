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

/// External service provider ports
pub mod providers;
/// Infrastructure service ports
pub mod infrastructure;
/// Administrative interfaces for system management and monitoring
pub mod admin;

// Re-export commonly used port traits for convenience
pub use admin::{
    IndexingOperation, IndexingOperationsInterface, PerformanceMetricsData,
    PerformanceMetricsInterface,
};
pub use infrastructure::{SnapshotProvider, SyncProvider};
pub use providers::{
    EmbeddingProvider, HybridSearchProvider, VectorStoreProvider,
};
