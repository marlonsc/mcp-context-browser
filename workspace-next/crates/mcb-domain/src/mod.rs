//! # Domain Layer
//!
//! Core business logic and domain types for semantic code analysis.
//! Organized following Clean Architecture principles with clear separation
//! of entities, value objects, domain services, and external ports.
//!
//! ## Architecture
//!
//! | Component | Description |
//! |-----------|-------------|
//! | [`entities`] | Core business entities with identity |
//! | [`value_objects`] | Immutable value objects |
//! | [`domain_services`] | Domain service interfaces |
//! | [`repositories`] | Data persistence interfaces |
//! | [`events`] | Domain event interfaces |
//! | [`ports`] | External system interfaces |
//! | [`constants`] | Domain constants |
//! | [`error`] | Domain error types |
//!
//! ## Clean Architecture Principles
//!
//! - **Entities** are at the center with business rules
//! - **Value Objects** are immutable and compared by value
//! - **Domain Services** encapsulate complex business logic
//! - **Ports** define external dependencies as interfaces
//! - **No external dependencies** - only standard library and core traits
//!
//! ## Example
//!
//! ```rust
//! use mcb_domain::{
//!     entities::CodeChunk,
//!     value_objects::{Language, Embedding},
//! };
//!
//! // Create a code chunk entity
//! let chunk = CodeChunk {
//!     id: "chunk-1".to_string(),
//!     content: "fn main() {}".to_string(),
//!     file_path: "example.rs".to_string(),
//!     start_line: 1,
//!     end_line: 1,
//!     language: "rust".to_string(),
//!     metadata: serde_json::json!({"type": "function"}),
//! };
//!
//! // Create an embedding value object
//! let embedding = Embedding {
//!     vector: vec![0.1, 0.2, 0.3],
//!     model: "text-embedding-ada-002".to_string(),
//!     dimensions: 1536,
//! };
//! ```

/// Domain-level constants
pub mod constants;
/// Domain service interfaces
pub mod domain_services;
/// Core business entities with identity
pub mod entities;
/// Domain error types
pub mod error;
/// Domain event interfaces
pub mod events;
/// External system port interfaces
pub mod ports;
/// Repository interfaces for data persistence
pub mod repositories;
/// Immutable value objects
pub mod value_objects;

// Re-export commonly used types for convenience
pub use constants::{
    INDEXING_BATCH_SIZE, INDEXING_CHUNKS_MAX_PER_FILE, INDEXING_CHUNK_MIN_LENGTH,
    INDEXING_CHUNK_MIN_LINES,
};
pub use domain_services::{
    ChunkingOptions, ChunkingResult, CodeChunker, ContextServiceInterface,
    IndexingServiceInterface, SearchServiceInterface,
};
pub use entities::*;
pub use error::{Error, Result};
pub use events::{DomainEvent, EventPublisher};
pub use repositories::{ChunkRepository, RepositoryStats, SearchRepository, SearchStats};
pub use value_objects::{
    Embedding, EmbeddingConfig, EmbeddingProviderKind, Language, OperationType, SearchResult,
    VectorStoreConfig, VectorStoreProviderKind,
};
