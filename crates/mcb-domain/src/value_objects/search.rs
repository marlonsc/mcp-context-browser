//! Search-Related Value Objects
//!
//! Value objects representing search results and related concepts
//! for semantic search operations.

use crate::value_objects::Language;
use serde::{Deserialize, Serialize};

/// Value Object: Ranked Search Result
///
/// Represents a single result from a semantic search operation.
/// Results are ranked by relevance score and contain the matched
/// code content with location information.
///
/// ## Business Rules
///
/// - Score represents semantic similarity (higher is better)
/// - Content includes the actual matched code
/// - File location enables navigation to source
///
/// ## Example
///
/// ```rust
/// use mcb_domain::value_objects::SearchResult;
///
/// let result = SearchResult {
///     id: "chunk_abc123".to_string(),
///     file_path: "src/auth/login.rs".to_string(),
///     start_line: 42,
///     content: "pub fn authenticate(token: &str) -> Result<User> { ... }".to_string(),
///     score: 0.92,
///     language: "rust".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchResult {
    /// Unique identifier of the matched code chunk
    pub id: String,
    /// Path to the source file
    pub file_path: String,
    /// Starting line number in the source file
    pub start_line: u32,
    /// The matched code content
    pub content: String,
    /// Semantic similarity score (0.0 to 1.0, higher is better)
    pub score: f64,
    /// Programming language of the matched code
    pub language: Language,
}
