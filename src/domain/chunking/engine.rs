//! Intelligent chunking engine
//!
//! This module provides the main IntelligentChunker that orchestrates
//! language-specific chunking using tree-sitter and fallback methods.

use crate::domain::error::Result;
use crate::domain::ports::chunking::{ChunkingOptions, ChunkingResult, CodeChunker};
use crate::domain::types::{CodeChunk, Language};
use crate::infrastructure::constants::CHUNK_SIZE_GENERIC;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;

/// Intelligent chunking engine using tree-sitter
#[derive(Default)]
pub struct IntelligentChunker;

impl IntelligentChunker {
    /// Create a new intelligent chunker
    pub fn new() -> Self {
        Self
    }

    /// Check if a language is supported for intelligent chunking
    pub fn is_language_supported(language: &Language) -> bool {
        crate::domain::chunking::LANGUAGE_CONFIGS.contains_key(language)
    }

    /// Chunk code based on language-specific structural analysis
    pub fn chunk_code(&self, content: &str, file_name: &str, language: Language) -> Vec<CodeChunk> {
        if let Some(processor) = crate::domain::chunking::LANGUAGE_CONFIGS.get(&language) {
            // Try tree-sitter parsing first
            match self.parse_with_tree_sitter(content, processor.get_language()) {
                Ok(tree) => {
                    let chunks = processor
                        .extract_chunks_with_tree_sitter(&tree, content, file_name, &language);
                    if !chunks.is_empty() {
                        return chunks;
                    }
                }
                Err(_) => {
                    // Fall back to pattern-based chunking
                    let chunks = processor.extract_chunks_fallback(content, file_name, &language);
                    if !chunks.is_empty() {
                        return chunks;
                    }
                }
            }
        }

        // Ultimate fallback to generic chunking
        self.chunk_generic(content, file_name, language)
    }

    /// Chunk code asynchronously (offloads to blocking thread)
    pub async fn chunk_code_async(
        &self,
        content: String,
        file_name: String,
        language: Language,
    ) -> Vec<CodeChunk> {
        tokio::task::spawn_blocking(move || {
            let chunker = Self::new();
            chunker.chunk_code(&content, &file_name, language)
        })
        .await
        .unwrap_or_default()
    }

    /// Generic chunking for unsupported languages
    fn chunk_generic(&self, content: &str, file_name: &str, language: Language) -> Vec<CodeChunk> {
        let lines: Vec<&str> = content.lines().collect();
        let mut chunks = Vec::new();
        let chunk_size = CHUNK_SIZE_GENERIC;

        for (chunk_idx, chunk_lines) in lines.chunks(chunk_size).enumerate() {
            let start_line = chunk_idx * chunk_size;
            let end_line = start_line + chunk_lines.len() - 1;

            let content = chunk_lines.join("\n").trim().to_string();
            if content.is_empty() || content.len() < 20 {
                continue;
            }

            let chunk = CodeChunk {
                id: format!("{}_{}", file_name, chunk_idx),
                content,
                file_path: file_name.to_string(),
                start_line: start_line as u32,
                end_line: end_line as u32,
                language: language.clone(),
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("file".to_string(), serde_json::json!(file_name));
                    meta.insert("chunk_index".to_string(), serde_json::json!(chunk_idx));
                    meta.insert("chunk_type".to_string(), serde_json::json!("generic"));
                    serde_json::to_value(meta).unwrap_or(serde_json::json!({}))
                },
            };
            chunks.push(chunk);
        }

        chunks
    }

    /// Parse code with tree-sitter
    fn parse_with_tree_sitter(
        &self,
        content: &str,
        language: tree_sitter::Language,
    ) -> Result<tree_sitter::Tree> {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&language).map_err(|e| {
            crate::domain::error::Error::internal(format!(
                "Failed to set tree-sitter language: {:?}",
                e
            ))
        })?;

        let tree = parser.parse(content, None).ok_or_else(|| {
            crate::domain::error::Error::internal("Tree-sitter parsing failed".to_string())
        })?;

        Ok(tree)
    }
}

#[async_trait]
impl CodeChunker for IntelligentChunker {
    async fn chunk_file(
        &self,
        file_path: &Path,
        _options: ChunkingOptions,
    ) -> Result<ChunkingResult> {
        let content = tokio::fs::read_to_string(file_path)
            .await
            .map_err(|e| crate::domain::error::Error::io(e.to_string()))?;

        let file_name = file_path.to_string_lossy().to_string();
        let ext = file_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        let language = Language::from_extension(ext);

        self.chunk_content(&content, &file_name, language, _options)
            .await
    }

    async fn chunk_content(
        &self,
        content: &str,
        file_name: &str,
        language: Language,
        _options: ChunkingOptions,
    ) -> Result<ChunkingResult> {
        let chunks = self.chunk_code(content, file_name, language.clone());
        let used_ast = Self::is_language_supported(&language);

        Ok(ChunkingResult {
            file_path: file_name.to_string(),
            language,
            chunks,
            used_ast,
        })
    }

    async fn chunk_batch(
        &self,
        file_paths: &[&Path],
        options: ChunkingOptions,
    ) -> Result<Vec<ChunkingResult>> {
        let mut results = Vec::with_capacity(file_paths.len());
        for path in file_paths {
            results.push(self.chunk_file(path, options.clone()).await?);
        }
        Ok(results)
    }

    fn supported_languages(&self) -> Vec<Language> {
        crate::domain::chunking::LANGUAGE_CONFIGS
            .keys()
            .cloned()
            .collect()
    }
}
