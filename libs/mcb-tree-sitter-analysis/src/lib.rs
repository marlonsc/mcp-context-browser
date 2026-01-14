//! Tree-sitter AST Analysis Library
//!
//! Unified AST analysis library for semantic code analysis using Tree-sitter.
//!
//! **v0.2.0**: Chunking only (existing MCB code moved here)
//! **v0.3.0+**: Analysis capabilities will be added
//!
//! This library provides a unified interface for:
//! - Code chunking (AST-aware semantic chunks)
//! - Function extraction
//! - Code structure analysis
//! - Future: Complexity metrics, technical debt detection, SATD detection

#![warn(missing_docs)]
#![warn(unsafe_code)]
#![warn(missing_debug_implementations)]

pub mod processor;

// Future analysis modules (empty in v0.2.0)
#[cfg(feature = "analysis")]
pub mod complexity;

#[cfg(feature = "analysis")]
pub mod structure;

// Re-exports
// Types are defined in this module, implementations in processor module

use anyhow::Result;

/// Configuration for code chunking
#[derive(Debug, Clone)]
pub struct ChunkConfig {
    /// Minimum chunk size in characters
    pub min_length: usize,
    /// Maximum chunk size in characters
    pub max_length: usize,
    /// Maximum nesting depth for semantic boundaries
    pub max_depth: usize,
}

impl Default for ChunkConfig {
    fn default() -> Self {
        Self {
            min_length: 100,
            max_length: 2000,
            max_depth: 10,
        }
    }
}

/// Unified language processor trait
///
/// Provides both existing (v0.2.0) and future (v0.3.0+) capabilities
/// for AST-aware code analysis.
pub trait LanguageProcessor: Send + Sync {
    /// Chunk code into semantic pieces (implemented in v0.2.0)
    fn chunk_code(&self, source: &str, config: &ChunkConfig) -> Result<Vec<CodeChunk>>;

    /// Analyze code complexity (future - v0.3.0+)
    #[cfg(feature = "analysis")]
    fn analyze_complexity(&self, source: &str) -> Result<ComplexityMetrics> {
        let _ = source;
        Err(anyhow::anyhow!("Not implemented in v0.2.0"))
    }

    /// Extract function definitions (future - v0.3.0+)
    #[cfg(feature = "analysis")]
    fn extract_functions(&self, source: &str) -> Result<Vec<FunctionInfo>> {
        let _ = source;
        Err(anyhow::anyhow!("Not implemented in v0.2.0"))
    }
}

/// Code chunk result
#[derive(Debug, Clone)]
pub struct CodeChunk {
    /// Content of the chunk
    pub content: String,
    /// Start line number
    pub start_line: usize,
    /// End line number
    pub end_line: usize,
    /// Language identifier
    pub language: String,
}

/// Placeholder for v0.3.0 complexity metrics
#[cfg(feature = "analysis")]
#[derive(Debug, Clone)]
pub struct ComplexityMetrics {
    /// Cyclomatic complexity
    pub cyclomatic: f64,
    /// Cognitive complexity
    pub cognitive: f64,
}

/// Placeholder for v0.3.0 function info
#[cfg(feature = "analysis")]
#[derive(Debug, Clone)]
pub struct FunctionInfo {
    /// Function name
    pub name: String,
    /// Start line
    pub start_line: usize,
    /// End line
    pub end_line: usize,
    /// Parameter count
    pub param_count: usize,
}
