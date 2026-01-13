//! Chunking Orchestrator - Coordinates batch code chunking operations
//!
//! Single Responsibility: Orchestrate code chunking across multiple files.

use crate::chunking::IntelligentChunker;
use crate::domain::error::{Error, Result};
use crate::domain::types::{CodeChunk, Language};
use crate::infrastructure::constants::{
    INDEXING_BATCH_SIZE, INDEXING_CHUNKS_MAX_PER_FILE, INDEXING_CHUNK_MIN_LENGTH,
    INDEXING_CHUNK_MIN_LINES,
};
use futures::future::join_all;
use std::path::Path;
use tokio::fs;

/// Configuration for chunking operations
#[derive(Debug, Clone)]
pub struct ChunkingConfig {
    /// Batch size for parallel processing
    pub batch_size: usize,
    /// Minimum chunk length in characters
    pub min_chunk_length: usize,
    /// Minimum chunk lines
    pub min_chunk_lines: usize,
    /// Maximum chunks per file
    pub max_chunks_per_file: usize,
}

impl Default for ChunkingConfig {
    fn default() -> Self {
        Self {
            batch_size: INDEXING_BATCH_SIZE,
            min_chunk_length: INDEXING_CHUNK_MIN_LENGTH,
            min_chunk_lines: INDEXING_CHUNK_MIN_LINES,
            max_chunks_per_file: INDEXING_CHUNKS_MAX_PER_FILE,
        }
    }
}

/// Result of chunking a single file
#[derive(Debug)]
pub struct FileChunkResult {
    /// Source file path
    pub path: String,
    /// Extracted chunks
    pub chunks: Vec<CodeChunk>,
    /// Whether chunking succeeded
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

/// Result of batch chunking operation
#[derive(Debug)]
pub struct BatchChunkResult {
    /// Successfully chunked files
    pub successful: Vec<FileChunkResult>,
    /// Failed files
    pub failed: Vec<FileChunkResult>,
    /// Total chunks extracted
    pub total_chunks: usize,
}

/// Orchestrates code chunking operations
pub struct ChunkingOrchestrator {
    chunker: IntelligentChunker,
    config: ChunkingConfig,
}

impl Default for ChunkingOrchestrator {
    fn default() -> Self {
        Self::new(ChunkingConfig::default())
    }
}

impl ChunkingOrchestrator {
    /// Create a new chunking orchestrator
    pub fn new(config: ChunkingConfig) -> Self {
        Self {
            chunker: IntelligentChunker::new(),
            config,
        }
    }

    /// Chunk a single file
    pub async fn chunk_file(&self, path: &Path) -> Result<Vec<CodeChunk>> {
        let content = fs::read_to_string(path)
            .await
            .map_err(|e| Error::io(format!("Failed to read file {}: {}", path.display(), e)))?;

        if content.trim().is_empty() {
            return Ok(Vec::new());
        }

        let file_name = path.display().to_string();
        let language = self.detect_language(path);

        // Use intelligent tree-sitter based chunking
        let mut chunks = self
            .chunker
            .chunk_code_async(content, file_name, language)
            .await;

        // Apply filters
        self.filter_chunks(&mut chunks);

        Ok(chunks)
    }

    /// Chunk content directly (without file I/O)
    pub async fn chunk_content(
        &self,
        content: String,
        file_name: String,
        language: Language,
    ) -> Vec<CodeChunk> {
        if content.trim().is_empty() {
            return Vec::new();
        }

        let mut chunks = self
            .chunker
            .chunk_code_async(content, file_name, language)
            .await;

        self.filter_chunks(&mut chunks);
        chunks
    }

    /// Process multiple files in parallel batches
    pub async fn chunk_batch(&self, files: &[impl AsRef<Path>]) -> BatchChunkResult {
        let mut result = BatchChunkResult {
            successful: Vec::new(),
            failed: Vec::new(),
            total_chunks: 0,
        };

        // Process in batches
        for batch in files.chunks(self.config.batch_size) {
            let futures: Vec<_> = batch
                .iter()
                .map(|path| {
                    let path = path.as_ref().to_path_buf();
                    async move {
                        let path_str = path.display().to_string();
                        match self.chunk_file(&path).await {
                            Ok(chunks) => FileChunkResult {
                                path: path_str,
                                chunks,
                                success: true,
                                error: None,
                            },
                            Err(e) => FileChunkResult {
                                path: path_str,
                                chunks: Vec::new(),
                                success: false,
                                error: Some(e.to_string()),
                            },
                        }
                    }
                })
                .collect();

            let batch_results = join_all(futures).await;

            for file_result in batch_results {
                if file_result.success {
                    result.total_chunks += file_result.chunks.len();
                    result.successful.push(file_result);
                } else {
                    result.failed.push(file_result);
                }
            }
        }

        result
    }

    /// Filter chunks based on configuration
    fn filter_chunks(&self, chunks: &mut Vec<CodeChunk>) {
        // Filter out chunks that are too small
        chunks.retain(|chunk| {
            chunk.content.len() >= self.config.min_chunk_length
                && chunk.content.lines().count() >= self.config.min_chunk_lines
        });

        // Limit chunks per file
        if chunks.len() > self.config.max_chunks_per_file {
            chunks.truncate(self.config.max_chunks_per_file);
        }
    }

    /// Detect language from file path
    fn detect_language(&self, path: &Path) -> Language {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        Language::from_extension(ext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ChunkingConfig::default();
        assert!(config.batch_size > 0);
        assert!(config.min_chunk_length > 0);
    }

    #[tokio::test]
    async fn test_chunk_empty_content() {
        let orchestrator = ChunkingOrchestrator::default();
        let chunks = orchestrator
            .chunk_content("".to_string(), "test.rs".to_string(), Language::Rust)
            .await;
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_detect_language() {
        let orchestrator = ChunkingOrchestrator::default();
        assert_eq!(
            orchestrator.detect_language(Path::new("test.rs")),
            Language::Rust
        );
    }
}
