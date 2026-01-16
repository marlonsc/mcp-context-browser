//! Fallback chunking using regex patterns
//!
//! This module provides regex-based chunking as a fallback when tree-sitter
//! parsing is not available or fails.

use super::config::LanguageConfig;
use mcb_domain::entities::CodeChunk;
use mcb_domain::value_objects::Language;
use regex::Regex;
use std::collections::HashMap;

/// Parameters for creating a code chunk
#[derive(Debug)]
pub struct ChunkCreationParams<'a> {
    /// Lines of code to include in the chunk
    pub lines: &'a [String],
    /// Starting line number
    pub start_line: usize,
    /// Ending line number
    pub end_line: usize,
    /// Source file name
    pub file_name: &'a str,
    /// Programming language
    pub language: &'a Language,
}

/// Generic fallback chunker using regex patterns
pub struct GenericFallbackChunker<'a> {
    #[allow(dead_code)]
    config: &'a LanguageConfig,
    /// Precompiled regex patterns for block detection
    compiled_patterns: Vec<Regex>,
}

impl<'a> GenericFallbackChunker<'a> {
    /// Create a new generic fallback chunker with language configuration
    pub fn new(config: &'a LanguageConfig) -> Self {
        let compiled_patterns = config
            .fallback_patterns
            .iter()
            .filter_map(|pattern| Regex::new(pattern).ok())
            .collect();

        Self {
            config,
            compiled_patterns,
        }
    }

    /// Chunk content using regex patterns as a fallback
    pub fn chunk_with_patterns(
        &self,
        content: &str,
        file_name: &str,
        language: &Language,
    ) -> Vec<CodeChunk> {
        let mut chunks = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut current_block = Vec::new();
        let mut block_start = 0;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            let is_block_start = self
                .compiled_patterns
                .iter()
                .any(|regex| regex.is_match(trimmed));

            if is_block_start {
                if !current_block.is_empty() {
                    self.create_chunk(
                        &ChunkCreationParams {
                            lines: &current_block,
                            start_line: block_start,
                            end_line: i - 1,
                            file_name,
                            language,
                        },
                        &mut chunks,
                    );
                    current_block.clear();
                }
                current_block.push(line.to_string());
                block_start = i;
            } else if !current_block.is_empty() {
                current_block.push(line.to_string());

                if self.is_block_complete(&current_block) {
                    self.create_chunk(
                        &ChunkCreationParams {
                            lines: &current_block,
                            start_line: block_start,
                            end_line: i,
                            file_name,
                            language,
                        },
                        &mut chunks,
                    );
                    current_block.clear();
                    block_start = i + 1;
                }
            }
        }

        if !current_block.is_empty() {
            self.create_chunk(
                &ChunkCreationParams {
                    lines: &current_block,
                    start_line: block_start,
                    end_line: lines.len() - 1,
                    file_name,
                    language,
                },
                &mut chunks,
            );
        }

        chunks
    }

    fn is_block_complete(&self, block: &[String]) -> bool {
        let open_count: usize = block
            .iter()
            .map(|line| line.chars().filter(|&c| c == '{').count())
            .sum();
        let close_count: usize = block
            .iter()
            .map(|line| line.chars().filter(|&c| c == '}').count())
            .sum();

        open_count > 0 && open_count == close_count && block.len() > 2
    }

    fn create_chunk(
        &self,
        params: &ChunkCreationParams,
        chunks: &mut Vec<CodeChunk>,
    ) {
        let content = params.lines.join("\n").trim().to_string();
        if content.is_empty() || content.len() < 20 {
            return;
        }

        let chunk = CodeChunk {
            id: format!("{}_{}_{}", params.file_name, params.start_line, params.end_line),
            content,
            file_path: params.file_name.to_string(),
            start_line: params.start_line as u32,
            end_line: params.end_line as u32,
            language: params.language.clone(),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("file".to_string(), serde_json::json!(params.file_name));
                meta.insert("chunk_type".to_string(), serde_json::json!("fallback"));
                serde_json::to_value(meta).unwrap_or(serde_json::json!({}))
            },
        };
        chunks.push(chunk);
    }
}
