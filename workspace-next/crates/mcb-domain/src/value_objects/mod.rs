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

/// Semantic embedding value objects
pub mod embedding;
/// Search-related value objects
pub mod search;
/// Configuration value objects
pub mod config;
/// Type definitions for dynamic domain concepts
pub mod types;

// Re-export commonly used value objects
pub use embedding::Embedding;
pub use search::SearchResult;
pub use config::{EmbeddingConfig, VectorStoreConfig};
pub use types::{Language, OperationType, EmbeddingProviderKind, VectorStoreProviderKind};