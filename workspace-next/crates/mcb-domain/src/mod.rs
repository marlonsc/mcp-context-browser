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

/// Core business entities with identity
pub mod entities;
/// Immutable value objects
pub mod value_objects;
/// Domain service interfaces
pub mod domain_services;
/// Repository interfaces for data persistence
pub mod repositories;
/// Domain event interfaces
pub mod events;
/// External system port interfaces
pub mod ports;
/// Domain-level constants
pub mod constants;
/// Domain error types
pub mod error;

// Re-export commonly used types for convenience
pub use entities::*;
pub use value_objects::{Embedding, SearchResult, EmbeddingConfig, VectorStoreConfig, Language, OperationType, EmbeddingProviderKind, VectorStoreProviderKind};
pub use domain_services::{CodeChunker, ContextServiceInterface, SearchServiceInterface, IndexingServiceInterface};
pub use repositories::{ChunkRepository, SearchRepository};
pub use events::{DomainEvent, EventPublisher};
pub use error::{Error, Result};