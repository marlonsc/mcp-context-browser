//! Java language processor for AST-based code chunking.

use crate::chunking::config::{LanguageConfig, NodeExtractionRule};
use crate::chunking::processor::BaseProcessor;
use crate::chunking::LanguageProcessor;
use crate::domain::types::{CodeChunk, Language};
use crate::infrastructure::constants::CHUNK_SIZE_JAVA;

/// Java language processor with method and class extraction.
pub struct JavaProcessor {
    processor: BaseProcessor,
}

impl Default for JavaProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl JavaProcessor {
    pub fn new() -> Self {
        let config = LanguageConfig::new(tree_sitter_java::LANGUAGE.into())
            .with_rules(vec![NodeExtractionRule {
                node_types: vec![
                    "method_declaration".to_string(),
                    "class_declaration".to_string(),
                    "interface_declaration".to_string(),
                    "constructor_declaration".to_string(),
                ],
                min_length: 40,
                min_lines: 3,
                max_depth: 3,
                priority: 9,
                include_context: true,
            }])
            .with_fallback_patterns(vec![
                r"^public .*\(.*\)".to_string(),
                r"^private .*\(.*\)".to_string(),
                r"^protected .*\(.*\)".to_string(),
                r"^class ".to_string(),
                r"^interface ".to_string(),
            ])
            .with_chunk_size(CHUNK_SIZE_JAVA);

        Self {
            processor: BaseProcessor::new(config),
        }
    }
}

crate::chunking::processor::impl_language_processor!(JavaProcessor);
