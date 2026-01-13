//! Code Chunker Domain Port
//!
//! Defines the business contract for code chunking operations. This abstraction
//! enables services to chunk code files without coupling to specific AST parsing
//! or language-specific implementations.

use crate::domain::error::Result;
use crate::domain::types::{CodeChunk, Language};
use async_trait::async_trait;
use std::path::Path;
use std::sync::Arc;

/// Options for chunking operations
#[derive(Debug, Clone)]
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
    /// Create a successful AST-based result
    pub fn from_ast(file_path: String, language: Language, chunks: Vec<CodeChunk>) -> Self {
        Self {
            file_path,
            language,
            chunks,
            used_ast: true,
        }
    }

    /// Create a fallback result (no AST parsing)
    pub fn from_fallback(file_path: String, language: Language, chunks: Vec<CodeChunk>) -> Self {
        Self {
            file_path,
            language,
            chunks,
            used_ast: false,
        }
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
/// ```rust,ignore
/// use crate::domain::ports::chunking::{CodeChunker, ChunkingOptions};
///
/// async fn process_file(
///     chunker: &dyn CodeChunker,
///     path: &Path,
/// ) -> Result<Vec<CodeChunk>> {
///     let result = chunker.chunk_file(path, ChunkingOptions::default()).await?;
///     Ok(result.chunks)
/// }
/// ```
#[async_trait]
pub trait CodeChunker: Send + Sync {
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
    async fn chunk_file(&self, file_path: &Path, options: ChunkingOptions)
        -> Result<ChunkingResult>;

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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Mock code chunker for testing
    struct MockCodeChunker {
        supported: Vec<Language>,
    }

    impl MockCodeChunker {
        fn new() -> Self {
            Self {
                supported: vec![
                    Language::Rust,
                    Language::Python,
                    Language::JavaScript,
                    Language::TypeScript,
                ],
            }
        }
    }

    #[async_trait]
    impl CodeChunker for MockCodeChunker {
        async fn chunk_file(
            &self,
            file_path: &Path,
            options: ChunkingOptions,
        ) -> Result<ChunkingResult> {
            let file_name = file_path.to_string_lossy().to_string();
            let ext = file_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");
            let language = Language::from_extension(ext);

            self.chunk_content("// mock content", &file_name, language, options)
                .await
        }

        async fn chunk_content(
            &self,
            content: &str,
            file_name: &str,
            language: Language,
            _options: ChunkingOptions,
        ) -> Result<ChunkingResult> {
            let chunk = CodeChunk {
                id: format!("chunk-{}", file_name),
                content: content.to_string(),
                file_path: file_name.to_string(),
                start_line: 1,
                end_line: 10,
                language: language.clone(),
                metadata: json!({"mock": true}),
            };

            Ok(ChunkingResult::from_ast(
                file_name.to_string(),
                language,
                vec![chunk],
            ))
        }

        async fn chunk_batch(
            &self,
            file_paths: &[&Path],
            options: ChunkingOptions,
        ) -> Result<Vec<ChunkingResult>> {
            let mut results = Vec::new();
            for path in file_paths {
                results.push(self.chunk_file(path, options.clone()).await?);
            }
            Ok(results)
        }

        fn supported_languages(&self) -> Vec<Language> {
            self.supported.clone()
        }
    }

    #[tokio::test]
    async fn test_chunk_file() {
        let chunker = MockCodeChunker::new();
        let result = chunker
            .chunk_file(Path::new("test.rs"), ChunkingOptions::default())
            .await;

        assert!(result.is_ok());
        let chunking_result = result.unwrap();
        assert_eq!(chunking_result.language, Language::Rust);
        assert!(chunking_result.used_ast);
        assert!(!chunking_result.chunks.is_empty());
    }

    #[tokio::test]
    async fn test_chunk_content() {
        let chunker = MockCodeChunker::new();
        let result = chunker
            .chunk_content(
                "fn main() {}",
                "main.rs",
                Language::Rust,
                ChunkingOptions::default(),
            )
            .await;

        assert!(result.is_ok());
        let chunking_result = result.unwrap();
        assert_eq!(chunking_result.file_path, "main.rs");
    }

    #[tokio::test]
    async fn test_chunk_batch() {
        let chunker = MockCodeChunker::new();
        let paths = vec![Path::new("a.rs"), Path::new("b.py")];
        let result = chunker
            .chunk_batch(&paths, ChunkingOptions::default())
            .await;

        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_supported_languages() {
        let chunker = MockCodeChunker::new();
        let supported = chunker.supported_languages();

        assert!(supported.contains(&Language::Rust));
        assert!(supported.contains(&Language::Python));
    }

    #[test]
    fn test_is_language_supported() {
        let chunker = MockCodeChunker::new();

        assert!(chunker.is_language_supported(&Language::Rust));
        assert!(!chunker.is_language_supported(&Language::SQL));
    }

    #[test]
    fn test_chunking_options_default() {
        let options = ChunkingOptions::default();

        assert_eq!(options.max_chunk_size, 512);
        assert!(options.include_context);
        assert_eq!(options.max_chunks_per_file, 50);
    }

    #[test]
    fn test_chunking_result_from_ast() {
        let result = ChunkingResult::from_ast("test.rs".to_string(), Language::Rust, vec![]);

        assert!(result.used_ast);
    }

    #[test]
    fn test_chunking_result_from_fallback() {
        let result = ChunkingResult::from_fallback("test.txt".to_string(), Language::Unknown, vec![]);

        assert!(!result.used_ast);
    }
}
