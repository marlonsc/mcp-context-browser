//! Code Chunker Domain Port
//!
//! Defines the business contract for code chunking operations. This abstraction
//! enables services to chunk code files without coupling to specific AST parsing
//! or language-specific implementations.

use async_trait::async_trait;
use mcb_domain::entities::CodeChunk;
use mcb_domain::error::Result;
use mcb_domain::value_objects::Language;
use shaku::Interface;
use std::path::Path;
use std::sync::Arc;

/// Options for chunking operations
#[derive(Debug, Clone, Copy)]
pub struct ChunkingOptions {
    /// Maximum size of a single chunk in characters
    pub max_chunk_size: usize,
    /// Whether to include surrounding context (imports, class declarations, etc.)
    pub include_context: bool,
    /// Maximum number of chunks per file
    pub max_chunks_per_file: usize,
}

impl Default for ChunkingOptions {
    fn default() -> Self {
        Self {
            max_chunk_size: 512,
            include_context: true,
            max_chunks_per_file: 50,
        }
    }
}

/// Result of chunking a single file
#[derive(Debug, Clone)]
pub struct ChunkingResult {
    /// File path that was chunked
    pub file_path: String,
    /// Language detected for the file
    pub language: Language,
    /// Extracted chunks
    pub chunks: Vec<CodeChunk>,
    /// Whether AST parsing was successful (vs fallback)
    pub used_ast: bool,
}

impl ChunkingResult {
    /// Create a new chunking result
    fn new(file_path: String, language: Language, chunks: Vec<CodeChunk>, used_ast: bool) -> Self {
        Self {
            file_path,
            language,
            chunks,
            used_ast,
        }
    }

    /// Create a successful AST-based result
    pub fn from_ast(file_path: String, language: Language, chunks: Vec<CodeChunk>) -> Self {
        Self::new(file_path, language, chunks, true)
    }

    /// Create a fallback result (no AST parsing)
    pub fn from_fallback(file_path: String, language: Language, chunks: Vec<CodeChunk>) -> Self {
        Self::new(file_path, language, chunks, false)
    }
}

/// Domain Port for Code Chunking Operations
///
/// This trait defines the contract for extracting semantic chunks from code files.
/// Implementations typically use AST parsing (tree-sitter) for supported languages
/// and fall back to pattern-based extraction for others.
///
/// # Example
///
/// ```ignore
/// use mcb_application::domain_services::chunking::{CodeChunker, ChunkingOptions};
/// use std::path::Path;
///
/// async fn process_file(
///     chunker: &dyn CodeChunker,
///     path: &Path,
/// ) -> mcb_domain::Result<Vec<mcb_domain::CodeChunk>> {
///     let result = chunker.chunk_file(path, ChunkingOptions::default()).await?;
///     Ok(result.chunks)
/// }
/// ```
#[async_trait]
pub trait CodeChunker: Interface + Send + Sync {
    /// Chunk a single file
    ///
    /// Reads the file, detects its language, and extracts semantic chunks
    /// (functions, classes, methods, etc.) using AST parsing when available.
    ///
    /// # Arguments
    ///
    /// * `file_path` - Path to the file to chunk
    /// * `options` - Chunking configuration options
    ///
    /// # Returns
    ///
    /// Chunking result containing the extracted chunks and metadata
    async fn chunk_file(
        &self,
        file_path: &Path,
        options: ChunkingOptions,
    ) -> Result<ChunkingResult>;

    /// Chunk code content directly (without reading from file)
    ///
    /// Useful when file content is already in memory or for testing.
    ///
    /// # Arguments
    ///
    /// * `content` - Source code content
    /// * `file_name` - Name of the file (for language detection and metadata)
    /// * `language` - Programming language of the content
    /// * `options` - Chunking configuration options
    ///
    /// # Returns
    ///
    /// Chunking result containing the extracted chunks
    async fn chunk_content(
        &self,
        content: &str,
        file_name: &str,
        language: Language,
        options: ChunkingOptions,
    ) -> Result<ChunkingResult>;

    /// Chunk multiple files in batch
    ///
    /// Processes multiple files efficiently, potentially in parallel.
    ///
    /// # Arguments
    ///
    /// * `file_paths` - List of files to chunk
    /// * `options` - Chunking configuration options
    ///
    /// # Returns
    ///
    /// List of chunking results (one per file)
    async fn chunk_batch(
        &self,
        file_paths: &[&Path],
        options: ChunkingOptions,
    ) -> Result<Vec<ChunkingResult>>;

    /// Get supported languages
    ///
    /// Returns the list of languages that have AST-based chunking support.
    fn supported_languages(&self) -> Vec<Language>;

    /// Check if a language is supported for AST parsing
    fn is_language_supported(&self, language: &Language) -> bool {
        self.supported_languages().contains(language)
    }
}

/// Shared code chunker for dependency injection
pub type SharedCodeChunker = Arc<dyn CodeChunker>;
