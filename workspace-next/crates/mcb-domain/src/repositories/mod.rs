//! Repository Interfaces
//!
//! Interfaces for data persistence and retrieval operations.
//! Repositories abstract the storage and retrieval of domain entities,
//! providing a consistent interface regardless of the underlying storage technology.
//!
//! ## Repositories
//!
//! | Repository | Description |
//! |------------|-------------|
//! | [`ChunkRepository`] | Persistence operations for code chunks |
//! | [`SearchRepository`] | Query operations for semantic search |

/// Code chunk repository interface
pub mod chunk_repository;
/// Search repository interface
pub mod search_repository;

// Re-export repository interfaces
pub use chunk_repository::{ChunkRepository, RepositoryStats};
pub use search_repository::{SearchRepository, SearchStats};
