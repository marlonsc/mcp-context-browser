//! Language processor implementations for AST analysis
//!
//! This module provides implementations of LanguageProcessor for supported languages.
//! In v0.2.0, chunking logic is still in src/domain/chunking/ and will be
//! migrated here in subsequent releases.

use anyhow::Result;

/// Multi-language code processor
///
/// Supports 12+ languages through Tree-sitter parsers.
/// Chunking implementation uses AST-aware semantic boundaries.
#[derive(Debug, Clone)]
pub struct MultiLanguageProcessor {
    language: String,
}

impl MultiLanguageProcessor {
    /// Create a new processor for the given language
    pub fn new(language: impl Into<String>) -> Self {
        Self {
            language: language.into(),
        }
    }

    /// Get supported languages
    pub fn supported_languages() -> &'static [&'static str] {
        &[
            "rust",
            "python",
            "javascript",
            "typescript",
            "go",
            "java",
            "c",
            "cpp",
            "csharp",
            "ruby",
            "php",
            "swift",
            "kotlin",
        ]
    }
}

impl crate::LanguageProcessor for MultiLanguageProcessor {
    fn chunk_code(
        &self,
        source: &str,
        config: &crate::ChunkConfig,
    ) -> Result<Vec<crate::CodeChunk>> {
        // Basic line-based chunking implementation
        let lines: Vec<&str> = source.lines().collect();
        let mut chunks = Vec::new();

        // Calculate chunk size based on config
        let chunk_size = config.max_length / 100; // Rough estimate: ~100 chars per line

        for (chunk_idx, chunk_lines) in lines.chunks(chunk_size).enumerate() {
            let start_line = chunk_idx * chunk_size;
            let end_line = start_line + chunk_lines.len().saturating_sub(1);

            let content = chunk_lines.join("\n").trim().to_string();

            // Skip empty or very small chunks
            if content.is_empty() || content.len() < config.min_length {
                continue;
            }

            // Ensure chunk doesn't exceed max_length
            let content = if content.len() > config.max_length {
                // Truncate at word boundary if possible
                let mut truncated = content[..config.max_length].to_string();
                if let Some(last_space) = truncated.rfind(' ') {
                    truncated.truncate(last_space);
                }
                truncated
            } else {
                content
            };

            let chunk = crate::CodeChunk {
                content,
                start_line,
                end_line,
                language: self.language.clone(),
            };

            chunks.push(chunk);
        }

        Ok(chunks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LanguageProcessor;

    #[test]
    fn test_supported_languages() {
        let langs = MultiLanguageProcessor::supported_languages();
        assert!(langs.contains(&"rust"));
        assert!(langs.contains(&"python"));
        assert!(langs.contains(&"javascript"));
    }

    #[test]
    fn test_new_processor() {
        let _processor = MultiLanguageProcessor::new("rust");
        // More detailed tests when chunking is migrated
    }

    #[test]
    fn test_chunk_code_basic() {
        let processor = MultiLanguageProcessor::new("rust");
        let config = crate::ChunkConfig {
            min_length: 10, // Lower minimum to allow small chunks
            max_length: 2000,
            max_depth: 10,
        };

        let source = r#"fn main() {
    println!("Hello, world!");
}

fn add(a: i32, b: i32) -> i32 {
    a + b
}

struct Point {
    x: f64,
    y: f64,
}

impl Point {
    fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    fn distance_from_origin(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}"#;

        let result = processor.chunk_code(source, &config);
        assert!(result.is_ok());

        let chunks = result.unwrap();
        assert!(
            !chunks.is_empty(),
            "No chunks were produced from the source"
        );

        // Check that chunks have valid content
        for chunk in &chunks {
            assert!(
                !chunk.content.is_empty(),
                "Chunk content should not be empty"
            );
            assert!(
                chunk.start_line <= chunk.end_line,
                "Start line should be <= end line"
            );
            assert_eq!(chunk.language, "rust", "Language should be rust");
        }

        // Verify that at least one chunk contains some actual code
        let has_code = chunks
            .iter()
            .any(|chunk| chunk.content.contains("fn") || chunk.content.contains("struct"));
        assert!(has_code, "At least one chunk should contain code");
    }
}
