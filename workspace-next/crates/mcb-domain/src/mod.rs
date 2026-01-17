//! # Domain Layer
//!
//! Core business logic and domain types for semantic code analysis.
//! Contains only pure domain entities, value objects, and business rules.
//!
//! ## Architecture
//!
//! | Component | Description |
//! |-----------|-------------|
//! | [`entities`] | Core business entities with identity |
//! | [`value_objects`] | Immutable value objects |
//! | [`constants`] | Domain constants |
//! | [`error`] | Domain error types |
//!
//! ## Clean Architecture Principles
//!
//! - **Entities** are at the center with business rules
//! - **Value Objects** are immutable and compared by value
//! - **No external dependencies** - only standard library and core traits
//! - **Pure business logic** - no infrastructure or application concerns
//!
//! ## Example
//!
//! ```rust
//! use mcb_domain::{
//!     entities::CodeChunk,
//!     value_objects::{Language, Embedding},
//!     error::Result,
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
//! };
//!
//! // Create an embedding value object
//! let embedding = Embedding::new(vec![0.1, 0.2, 0.3], "example text".to_string())?;
//! # Ok::<(), mcb_domain::error::Error>(())
//! ```

/// Domain-level constants
pub mod constants;
/// Core business entities with identity
pub mod entities;
/// Domain error types
pub mod error;
/// Domain event interfaces
pub mod events;
/// Repository interfaces
pub mod repositories;
/// Immutable value objects
pub mod value_objects;

// Re-export commonly used types for convenience
pub use constants::*;
pub use entities::*;
pub use error::{Error, Result};
pub use events::{DomainEvent, EventPublisher};
pub use value_objects::*;
