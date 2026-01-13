//! PHP language processor for AST-based code chunking.

use crate::chunking::config::{LanguageConfig, NodeExtractionRule};
use crate::chunking::processor::BaseProcessor;
use crate::chunking::LanguageProcessor;
use crate::domain::types::{CodeChunk, Language};
use crate::infrastructure::constants::CHUNK_SIZE_PHP;

/// PHP language processor with function, class, and trait extraction.
pub struct PhpProcessor {
    processor: BaseProcessor,
}

impl Default for PhpProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl PhpProcessor {
    pub fn new() -> Self {
        let config = LanguageConfig::new(tree_sitter_php::LANGUAGE_PHP.into())
            .with_rules(vec![
                NodeExtractionRule {
                    node_types: vec![
                        "function_definition".to_string(),
                        "method_declaration".to_string(),
                        "class_declaration".to_string(),
                        "interface_declaration".to_string(),
                        "trait_declaration".to_string(),
                    ],
                    min_length: 35,
                    min_lines: 2,
                    max_depth: 3,
                    priority: 9,
                    include_context: true,
                },
                NodeExtractionRule {
                    node_types: vec![
                        "anonymous_function_creation_expression".to_string(),
                        "arrow_function".to_string(),
                    ],
                    min_length: 20,
                    min_lines: 1,
                    max_depth: 2,
                    priority: 5,
                    include_context: false,
                },
            ])
            .with_fallback_patterns(vec![
                r"^function ".to_string(),
                r"^public function ".to_string(),
                r"^private function ".to_string(),
                r"^protected function ".to_string(),
                r"^class ".to_string(),
                r"^interface ".to_string(),
                r"^trait ".to_string(),
            ])
            .with_chunk_size(CHUNK_SIZE_PHP);

        Self {
            processor: BaseProcessor::new(config),
        }
    }
}

crate::chunking::processor::impl_language_processor!(PhpProcessor);
