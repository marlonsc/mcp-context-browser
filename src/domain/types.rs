//! Enterprise Code Intelligence Domain Model
//!
//! Defines the fundamental business entities that power the semantic code search
//! platform. These types represent the core business concepts of code intelligence,
//! from semantic embeddings that capture code meaning to search results that deliver
//! business value to development teams.

use serde::{Deserialize, Serialize};
use validator::Validate;

/// AI Semantic Understanding Representation
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
    Shell,
    SQL,
    HTML,
    XML,
    JSON,
    YAML,
    TOML,
    Markdown,
    PlainText,
    Unknown,
}

/// System operation types for metrics and rate limiting
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum OperationType {
    Indexing,
    Search,
    Embedding,
    Maintenance,
    Other(String),
}

// =============================================================================
// Provider Kind Enums (Type-Safe Provider Selection)
// =============================================================================

/// Type-safe embedding provider selection
///
/// Replaces string-based provider selection with compile-time type safety.
/// Invalid provider names are caught at config deserialization time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum EmbeddingProviderKind {
    /// OpenAI embedding API
    OpenAI,
    /// Ollama local embeddings
    Ollama,
    /// VoyageAI embedding API
    VoyageAI,
    /// Google Gemini embeddings
    Gemini,
    /// FastEmbed local embeddings (default)
    #[default]
    FastEmbed,
}

impl std::fmt::Display for EmbeddingProviderKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OpenAI => write!(f, "openai"),
            Self::Ollama => write!(f, "ollama"),
            Self::VoyageAI => write!(f, "voyageai"),
            Self::Gemini => write!(f, "gemini"),
            Self::FastEmbed => write!(f, "fastembed"),
        }
    }
}

impl EmbeddingProviderKind {
    /// Parse a provider string into the enum variant.
    pub fn from_string(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "openai" => Some(Self::OpenAI),
            "ollama" => Some(Self::Ollama),
            "voyageai" => Some(Self::VoyageAI),
            "gemini" => Some(Self::Gemini),
            "fastembed" => Some(Self::FastEmbed),
            _ => None,
        }
    }

    /// Get all supported provider names
    pub fn supported_providers() -> &'static [&'static str] {
        &["openai", "ollama", "voyageai", "gemini", "fastembed"]
    }
}

/// Type-safe vector store provider selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum VectorStoreProviderKind {
    /// In-memory vector store (for testing/development)
    #[serde(rename = "in-memory")]
    InMemory,
    /// Filesystem-based vector store
    #[default]
    Filesystem,
    /// Milvus vector database
    #[cfg(feature = "milvus")]
    Milvus,
    /// EdgeVec in-memory store
    EdgeVec,
}

impl std::fmt::Display for VectorStoreProviderKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InMemory => write!(f, "in-memory"),
            Self::Filesystem => write!(f, "filesystem"),
            #[cfg(feature = "milvus")]
            Self::Milvus => write!(f, "milvus"),
            Self::EdgeVec => write!(f, "edgevec"),
        }
    }
}

impl VectorStoreProviderKind {
    /// Parse a provider string into the enum variant.
    pub fn from_string(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "in-memory" | "inmemory" => Some(Self::InMemory),
            "filesystem" => Some(Self::Filesystem),
            #[cfg(feature = "milvus")]
            "milvus" => Some(Self::Milvus),
            "edgevec" => Some(Self::EdgeVec),
            _ => None,
        }
    }

    /// Get all supported provider names
    pub fn supported_providers() -> Vec<&'static str> {
        let mut providers = vec!["in-memory", "filesystem", "edgevec"];
        #[cfg(feature = "milvus")]
        providers.push("milvus");
        providers
    }
}

/// Query performance metrics tracking
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct QueryPerformanceMetrics {
    pub total_queries: u64,
    pub average_latency: f64,
    pub p99_latency: f64,
    pub success_rate: f64,
}

/// Cache performance metrics tracking
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct CacheMetrics {
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
    pub size: u64,
}

impl std::fmt::Display for OperationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperationType::Indexing => write!(f, "indexing"),
            OperationType::Search => write!(f, "search"),
            OperationType::Embedding => write!(f, "embedding"),
            OperationType::Maintenance => write!(f, "maintenance"),
            OperationType::Other(s) => write!(f, "{}", s),
        }
    }
}

impl From<&str> for OperationType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "indexing" => OperationType::Indexing,
            "search" => OperationType::Search,
            "embedding" => OperationType::Embedding,
            "maintenance" => OperationType::Maintenance,
            _ => OperationType::Other(s.to_string()),
        }
    }
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
            "sh" | "bash" | "zsh" | "fish" => Language::Shell,
            "sql" => Language::SQL,
            "html" => Language::HTML,
            "xml" => Language::XML,
            "json" => Language::JSON,
            "yaml" | "yml" => Language::YAML,
            "toml" => Language::TOML,
            "md" | "markdown" => Language::Markdown,
            "txt" | "text" => Language::PlainText,
            _ => Language::Unknown,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Language::Rust => "Rust",
            Language::Python => "Python",
            Language::JavaScript => "JavaScript",
            Language::TypeScript => "TypeScript",
            Language::Go => "Go",
            Language::Java => "Java",
            Language::C => "C",
            Language::Cpp => "Cpp",
            Language::CSharp => "CSharp",
            Language::Php => "Php",
            Language::Ruby => "Ruby",
            Language::Swift => "Swift",
            Language::Kotlin => "Kotlin",
            Language::Scala => "Scala",
            Language::Haskell => "Haskell",
            Language::Shell => "Shell",
            Language::SQL => "SQL",
            Language::HTML => "HTML",
            Language::XML => "XML",
            Language::JSON => "JSON",
            Language::YAML => "YAML",
            Language::TOML => "TOML",
            Language::Markdown => "Markdown",
            Language::PlainText => "PlainText",
            Language::Unknown => "Unknown",
        }
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for Language {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "Rust" => Ok(Language::Rust),
            "Python" => Ok(Language::Python),
            "JavaScript" => Ok(Language::JavaScript),
            "TypeScript" => Ok(Language::TypeScript),
            "Go" => Ok(Language::Go),
            "Java" => Ok(Language::Java),
            "C" => Ok(Language::C),
            "Cpp" => Ok(Language::Cpp),
            "CSharp" => Ok(Language::CSharp),
            "Php" => Ok(Language::Php),
            "Ruby" => Ok(Language::Ruby),
            "Swift" => Ok(Language::Swift),
            "Kotlin" => Ok(Language::Kotlin),
            "Scala" => Ok(Language::Scala),
            "Haskell" => Ok(Language::Haskell),
            "Shell" => Ok(Language::Shell),
            "SQL" => Ok(Language::SQL),
            "HTML" => Ok(Language::HTML),
            "XML" => Ok(Language::XML),
            "JSON" => Ok(Language::JSON),
            "YAML" => Ok(Language::YAML),
            "TOML" => Ok(Language::TOML),
            "Markdown" => Ok(Language::Markdown),
            "PlainText" => Ok(Language::PlainText),
            _ => Ok(Language::Unknown),
        }
    }
}

/// Semantic search result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchResult {
    pub id: String,
    pub file_path: String,
    pub start_line: u32,
    pub content: String,
    pub score: f32,
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

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            provider: "fastembed".to_string(),
            model: "BAAI/bge-small-en-v1.5".to_string(),
            api_key: None,
            base_url: None,
            dimensions: Some(384),
            max_tokens: None,
        }
    }
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
    pub timeout_secs: Option<u64>,
}

impl Default for VectorStoreConfig {
    fn default() -> Self {
        Self {
            provider: "filesystem".to_string(),
            address: None,
            token: None,
            collection: None,
            dimensions: Some(384),
            timeout_secs: Some(30),
        }
    }
}

/// Sync batch for queue processing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SyncBatch {
    pub id: String,
    pub path: String,
    pub created_at: u64,
}

/// Statistics for repository operations
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct RepositoryStats {
    pub total_chunks: u64,
    pub total_collections: u64,
    pub storage_size_bytes: u64,
    pub avg_chunk_size_bytes: f64,
}

/// Statistics for search operations
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct SearchStats {
    pub total_queries: u64,
    pub avg_response_time_ms: f64,
    pub cache_hit_rate: f64,
    pub indexed_documents: u64,
}

/// File snapshot with metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileSnapshot {
    pub path: String,
    pub size: u64,
    pub modified: u64,
    pub hash: String,
    pub extension: String,
}

/// Codebase snapshot with all files
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CodebaseSnapshot {
    pub root_path: String,
    pub created_at: u64,
    pub files: std::collections::HashMap<String, FileSnapshot>,
    pub file_count: usize,
    pub total_size: u64,
}

/// Changes between snapshots
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SnapshotChanges {
    pub added: Vec<String>,
    pub modified: Vec<String>,
    pub removed: Vec<String>,
    pub unchanged: Vec<String>,
}

impl SnapshotChanges {
    pub fn has_changes(&self) -> bool {
        !self.added.is_empty() || !self.modified.is_empty() || !self.removed.is_empty()
    }

    pub fn total_changes(&self) -> usize {
        self.added.len() + self.modified.len() + self.removed.len()
    }
}
