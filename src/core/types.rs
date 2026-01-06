//! Core types for the MCP Context Browser

use serde::{Deserialize, Serialize};

/// Embedding vector representation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Embedding {
    pub vector: Vec<f32>,
    pub model: String,
    pub dimensions: usize,
}

/// Code chunk with metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CodeChunk {
    pub id: String,
    pub content: String,
    pub file_path: String,
    pub start_line: u32,
    pub end_line: u32,
    pub language: Language,
    pub metadata: serde_json::Value,
}

/// Programming language detection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

/// Search result from semantic code search
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchResult {
    pub file_path: String,
    pub line_number: u32,
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    pub provider: String,
    pub model: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub dimensions: Option<usize>,
    pub max_tokens: Option<usize>,
}

/// Configuration for vector store providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStoreConfig {
    pub provider: String,
    pub address: Option<String>,
    pub token: Option<String>,
    pub collection: Option<String>,
    pub dimensions: Option<usize>,
}
