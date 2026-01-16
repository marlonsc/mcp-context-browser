//! PHP language processor for AST-based code chunking.

use crate::adapters::chunking::config::{LanguageConfig, NodeExtractionRule};
use crate::adapters::chunking::constants::CHUNK_SIZE_PHP;
use crate::adapters::chunking::processor::{BaseProcessor, LanguageProcessor};
use mcb_domain::entities::CodeChunk;
use mcb_domain::value_objects::Language;

/// PHP language processor.
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
            .with_rules(vec![NodeExtractionRule {
                node_types: vec![
                    "function_definition".to_string(),
                    "method_declaration".to_string(),
                    "class_declaration".to_string(),
                ],
                min_length: 30,
                min_lines: 2,
                max_depth: 2,
                priority: 5,
                include_context: true,
            }])
            .with_fallback_patterns(vec![
                r"^function ".to_string(),
                r"^\s*public function ".to_string(),
                r"^class ".to_string(),
            ])
            .with_chunk_size(CHUNK_SIZE_PHP);

        Self {
            processor: BaseProcessor::new(config),
        }
    }
}

impl LanguageProcessor for PhpProcessor {
    fn config(&self) -> &LanguageConfig {
        self.processor.config()
    }

    fn extract_chunks_with_tree_sitter(
        &self,
        tree: &tree_sitter::Tree,
        content: &str,
        file_name: &str,
        language: &Language,
    ) -> Vec<CodeChunk> {
        self.processor
            .extract_chunks_with_tree_sitter(tree, content, file_name, language)
    }

    fn extract_chunks_fallback(
        &self,
        content: &str,
        file_name: &str,
        language: &Language,
    ) -> Vec<CodeChunk> {
        self.processor
            .extract_chunks_fallback(content, file_name, language)
    }
}
