//! Enterprise Code Intelligence Domain Model
//!
//! Defines the fundamental business entities that power the semantic code search
//! platform. These types represent the core business concepts of code intelligence,
//! from semantic embeddings that capture code meaning to search results that deliver
//! business value to development teams.
//!
//! ## Business Domain Overview
//!
//! - **Embeddings**: AI-generated semantic representations of code meaning
//! - **Code Chunks**: Intelligently processed code segments with context
//! - **Search Results**: Ranked, relevant code discoveries for developers
//! - **Languages**: Supported programming languages with specialized processing
//!
//! ## Enterprise Value
//!
//! These domain types enable the transformation of raw code repositories into
//! searchable business intelligence, making complex codebases accessible through
//! natural language queries and accelerating development team productivity.

use serde::{Deserialize, Serialize};
use validator::Validate;

/// AI Semantic Understanding Representation
///
/// An Embedding captures the semantic meaning of code or text through AI analysis.
/// This business entity transforms human-readable code into mathematical representations
/// that enable semantic similarity comparisons, powering the "find code by meaning"
/// capability that accelerates development teams from hours of searching to seconds
/// of discovery.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default, Validate)]
pub struct Embedding {
    /// The embedding vector values
    #[validate(length(min = 1, message = "Embedding vector cannot be empty"))]
    pub vector: Vec<f32>,
    /// Name of the model that generated this embedding
    #[validate(length(min = 1, message = "Model name cannot be empty"))]
    pub model: String,
    /// Dimensionality of the embedding vector
    #[validate(range(min = 1, message = "Dimensions must be positive"))]
    pub dimensions: usize,
}

/// Intelligent Code Segment with Business Context
///
/// A CodeChunk represents a semantically meaningful segment of code that has been
/// processed for enterprise search. This business entity combines the actual code
/// content with rich metadata that enables precise, contextually relevant search
/// results for development teams seeking specific functionality or patterns.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Validate)]
pub struct CodeChunk {
    /// Unique identifier for this code chunk
    #[validate(length(min = 1, message = "ID cannot be empty"))]
    pub id: String,
    /// The actual code content
    #[validate(length(
        min = 1,
        max = 10000,
        message = "Content must be between 1 and 10000 characters"
    ))]
    pub content: String,
    /// Path to the source file
    #[validate(length(min = 1, message = "File path cannot be empty"))]
    pub file_path: String,
    /// Starting line number in the source file
    #[validate(range(min = 1, message = "Start line must be positive"))]
    pub start_line: u32,
    /// Ending line number in the source file
    #[validate(range(min = 1, message = "End line must be positive"))]
    pub end_line: u32,
    /// Programming language of the code
    pub language: Language,
    /// Additional metadata as JSON (context, AST info, etc.)
    pub metadata: serde_json::Value,
}

/// Supported programming languages
///
/// Defines the programming languages that the system can process.
/// Language detection is based on file extensions and syntax analysis.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Language {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Go,
    Java,
    C,
    Cpp,
    CSharp,
    Php,
    Ruby,
    Swift,
    Kotlin,
    Scala,
    Haskell,
    Unknown,
}

impl Language {
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "rs" => Language::Rust,
            "py" => Language::Python,
            "js" => Language::JavaScript,
            "ts" => Language::TypeScript,
            "go" => Language::Go,
            "java" => Language::Java,
            "c" => Language::C,
            "cpp" | "cc" | "cxx" => Language::Cpp,
            "cs" => Language::CSharp,
            "php" => Language::Php,
            "rb" => Language::Ruby,
            "swift" => Language::Swift,
            "kt" => Language::Kotlin,
            "scala" => Language::Scala,
            "hs" => Language::Haskell,
            _ => Language::Unknown,
        }
    }
}

/// Semantic search result
///
/// Represents a single result from a semantic code search operation.
/// Contains the matched code snippet with relevance scoring and metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchResult {
    /// Unique identifier from the vector store
    pub id: String,
    /// Path to the file containing the match
    pub file_path: String,
    /// Line number where the match starts
    pub line_number: u32,
    /// The actual code content that matched
    pub content: String,
    /// Relevance score (higher = more relevant)
    pub score: f32,
    /// Additional metadata about the match
    pub metadata: serde_json::Value,
}

/// Indexing statistics
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IndexingStats {
    pub total_files: u32,
    pub indexed_files: u32,
    pub total_chunks: u32,
    pub duration_ms: u64,
}

/// Configuration for embedding providers
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct EmbeddingConfig {
    #[validate(length(min = 1))]
    pub provider: String,
    #[validate(length(min = 1))]
    pub model: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub dimensions: Option<usize>,
    pub max_tokens: Option<usize>,
}

/// Configuration for vector store providers
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct VectorStoreConfig {
    #[validate(length(min = 1))]
    pub provider: String,
    pub address: Option<String>,
    pub token: Option<String>,
    pub collection: Option<String>,
    pub dimensions: Option<usize>,
}

/// Sync batch for queue processing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SyncBatch {
    /// Unique identifier for this batch
    pub id: String,
    /// Path to the codebase being synced
    pub path: String,
    /// Timestamp when the batch was created
    pub created_at: u64,
}
