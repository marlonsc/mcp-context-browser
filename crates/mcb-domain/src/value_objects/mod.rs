//! Domain Value Objects
//!
//! Immutable value objects that represent concepts in the domain
//! without identity. Value objects are defined by their attributes
//! and can be compared for equality.
//!
//! ## Value Objects
//!
//! | Value Object | Description |
//! |--------------|-------------|
//! | [`Embedding`] | Vector representation of text for semantic search |
//! | [`SearchResult`] | Ranked result from semantic search operation |
//! | [`Language`] | Programming language identifier |
//! | [`OperationType`] | Operation type for metrics and rate limiting |
//! | [`CollectionInfo`] | Metadata about an indexed collection |
//! | [`FileInfo`] | Metadata about an indexed file |

/// Browse-related value objects for code navigation
pub mod browse;
/// Configuration value objects
pub mod config;
/// Semantic embedding value objects
pub mod embedding;
/// Search-related value objects
pub mod search;
/// Type definitions for dynamic domain concepts
pub mod types;

// Re-export commonly used value objects
pub use browse::{CollectionInfo, FileInfo};
pub use config::{CacheConfig, EmbeddingConfig, VectorStoreConfig};
pub use embedding::Embedding;
pub use search::SearchResult;
pub use types::{
    CacheProviderKind, EmbeddingProviderKind, Language, OperationType, VectorStoreProviderKind,
};
