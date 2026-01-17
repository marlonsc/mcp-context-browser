//! Language Chunking Provider Port
//!
//! Port for language-specific code chunking providers. Each provider handles
//! a specific programming language and transforms source code into semantic
//! code chunks using AST-based parsing.
//!
//! ## Provider Pattern
//!
//! This port follows the same pattern as [`EmbeddingProvider`] and
//! [`VectorStoreProvider`], enabling consistent provider registration,
//! factory creation, and feature-flag based compilation.

use mcb_domain::entities::CodeChunk;
use mcb_domain::value_objects::Language;
use shaku::Interface;

/// Language-Specific Code Chunking Provider
///
/// Defines the contract for providers that parse and chunk source code
/// in a specific programming language. Each provider uses AST-based
/// parsing to extract semantically meaningful code segments.
///
/// # Implementations
///
/// The system supports multiple language providers:
/// - Rust, Python, JavaScript, TypeScript
/// - Go, Java, C, C++, C#
/// - Ruby, PHP, Swift, Kotlin
///
/// # Example
///
/// ```ignore
/// use mcb_domain::ports::providers::LanguageChunkingProvider;
/// use mcb_domain::value_objects::Language;
///
/// // Get provider from registry
/// let provider: Arc<dyn LanguageChunkingProvider> = registry.get_by_extension("rs")?;
///
/// // Chunk source code
/// let chunks = provider.chunk(source_code, "src/main.rs");
/// println!("Extracted {} chunks from {}", chunks.len(), provider.provider_name());
/// ```
pub trait LanguageChunkingProvider: Interface + Send + Sync {
    /// Get the language identifier this provider handles
    ///
    /// # Returns
    /// The language string (e.g., "rust", "python", "javascript")
    fn language(&self) -> Language;

    /// Get the file extensions this provider handles
    ///
    /// # Returns
    /// Slice of file extensions without the dot (e.g., ["rs"], ["py"], ["js", "jsx"])
    fn extensions(&self) -> &[&'static str];

    /// Extract semantic code chunks from source code
    ///
    /// Uses AST-based parsing to identify and extract meaningful code
    /// segments like functions, classes, methods, and modules.
    ///
    /// # Arguments
    /// * `content` - The source code content to parse
    /// * `file_path` - The path to the source file (for metadata)
    ///
    /// # Returns
    /// Vector of extracted code chunks with metadata
    fn chunk(&self, content: &str, file_path: &str) -> Vec<CodeChunk>;

    /// Get the name/identifier of this provider implementation
    ///
    /// # Returns
    /// A string identifier for the provider (e.g., "tree-sitter-rust")
    fn provider_name(&self) -> &str;

    /// Check if this provider supports a given file extension
    ///
    /// Default implementation checks against the extensions() slice.
    ///
    /// # Arguments
    /// * `ext` - File extension without the dot (e.g., "rs", "py")
    fn supports_extension(&self, ext: &str) -> bool {
        self.extensions()
            .iter()
            .any(|e| e.eq_ignore_ascii_case(ext))
    }

    /// Get the maximum chunk size for this language
    ///
    /// Different languages may have different optimal chunk sizes
    /// based on their syntax structure.
    ///
    /// # Returns
    /// Maximum number of lines for a single chunk (default: 50)
    fn max_chunk_size(&self) -> usize {
        50
    }
}
