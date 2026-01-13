//! Kotlin language processor for AST-based code chunking.

use crate::chunking::config::{LanguageConfig, NodeExtractionRule};
use crate::chunking::processor::BaseProcessor;
use crate::chunking::LanguageProcessor;
use crate::domain::types::{CodeChunk, Language};
use crate::infrastructure::constants::CHUNK_SIZE_KOTLIN;

/// Kotlin language processor with function, class, and object extraction.
pub struct KotlinProcessor {
    processor: BaseProcessor,
}

impl Default for KotlinProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl KotlinProcessor {
    pub fn new() -> Self {
        let config = LanguageConfig::new(tree_sitter_kotlin_ng::LANGUAGE.into())
            .with_rules(vec![
                NodeExtractionRule {
                    node_types: vec![
                        "function_declaration".to_string(),
                        "class_declaration".to_string(),
                        "object_declaration".to_string(),
                        "interface_declaration".to_string(),
                    ],
                    min_length: 35,
                    min_lines: 2,
                    max_depth: 3,
                    priority: 9,
                    include_context: true,
                },
                NodeExtractionRule {
                    node_types: vec![
                        "property_declaration".to_string(),
                        "companion_object".to_string(),
                        "anonymous_function".to_string(),
                        "lambda_literal".to_string(),
                    ],
                    min_length: 25,
                    min_lines: 2,
                    max_depth: 2,
                    priority: 6,
                    include_context: false,
                },
            ])
            .with_fallback_patterns(vec![
                r"^fun ".to_string(),
                r"^class ".to_string(),
                r"^object ".to_string(),
                r"^interface ".to_string(),
                r"^data class ".to_string(),
                r"^sealed class ".to_string(),
            ])
            .with_chunk_size(CHUNK_SIZE_KOTLIN);

        Self {
            processor: BaseProcessor::new(config),
        }
    }
}

crate::chunking::processor::impl_language_processor!(KotlinProcessor);
