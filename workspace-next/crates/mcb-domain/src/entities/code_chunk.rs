//! Code Chunk Entity
//!
//! The core domain entity representing a semantically meaningful segment
//! of source code. Code chunks are the fundamental units of semantic indexing
//! and search in the system.

use crate::value_objects::Language;
use serde::{Deserialize, Serialize};

/// Core Entity: Semantically Meaningful Code Segment
///
/// A code chunk represents a meaningful unit of code that has been extracted
/// through AST-based parsing. This is the primary entity for semantic indexing
/// and search operations.
///
/// ## Business Rules
///
/// - Each chunk must have a unique identifier
/// - Content must be non-empty and meaningful
/// - Language identification enables proper parsing
/// - Metadata provides additional context for search
///
/// ## Example
///
/// ```rust
/// use mcb_domain::entities::CodeChunk;
/// use mcb_domain::value_objects::Language;
///
/// let chunk = CodeChunk {
///     id: "chunk_001".to_string(),
///     content: "fn authenticate(user: &str) -> bool { true }".to_string(),
///     file_path: "src/auth.rs".to_string(),
///     start_line: 10,
///     end_line: 12,
///     language: "rust".to_string(),
///     metadata: serde_json::json!({"type": "function", "name": "authenticate"}),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CodeChunk {
    /// Unique identifier for this code chunk
    pub id: String,
    /// The actual code content
    pub content: String,
    /// Path to the source file
    pub file_path: String,
    /// Starting line number in the source file
    pub start_line: u32,
    /// Ending line number in the source file
    pub end_line: u32,
    /// Programming language of the code
    pub language: Language,
    /// Additional metadata as JSON (context, AST info, etc.)
    pub metadata: serde_json::Value,
}