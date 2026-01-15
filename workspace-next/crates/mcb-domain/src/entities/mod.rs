//! Domain Entities
//!
//! Core business entities that represent the fundamental concepts
//! of the semantic code search domain. Entities have identity and
//! encapsulate business rules.
//!
//! ## Entities
//!
//! | Entity | Description |
//! |--------|-------------|
//! | [`CodeChunk`] | Core entity representing a semantically meaningful code segment |
//! | [`CodebaseSnapshot`] | Entity representing a complete codebase state at a point in time |
//! | [`FileSnapshot`] | Entity representing a file's state for change tracking |

/// Core entity representing a semantically meaningful code segment
pub mod code_chunk;
/// Entities for codebase state management and change tracking
pub mod codebase;

// Re-export commonly used entities
pub use code_chunk::CodeChunk;
pub use codebase::{CodebaseSnapshot, FileSnapshot};