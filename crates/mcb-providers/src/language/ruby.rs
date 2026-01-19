//! Ruby language processor for AST-based code chunking.

use crate::language::common::{
    BaseProcessor, CHUNK_SIZE_RUBY, LanguageConfig, LanguageProcessor, NodeExtractionRule,
};
use mcb_domain::entities::CodeChunk;
use mcb_domain::value_objects::Language;

/// Ruby language processor.
pub struct RubyProcessor {
    processor: BaseProcessor,
}

impl Default for RubyProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl RubyProcessor {
    /// Create a new Ruby language processor
    pub fn new() -> Self {
        let config = LanguageConfig::new(tree_sitter_ruby::LANGUAGE.into())
            .with_rules(vec![NodeExtractionRule {
                node_types: vec![
                    "method".to_string(),
                    "class".to_string(),
                    "module".to_string(),
                ],
                min_length: 30,
                min_lines: 2,
                max_depth: 2,
                priority: 5,
                include_context: true,
            }])
            .with_fallback_patterns(vec![
                r"^def ".to_string(),
                r"^class ".to_string(),
                r"^module ".to_string(),
            ])
            .with_chunk_size(CHUNK_SIZE_RUBY);

        Self {
            processor: BaseProcessor::new(config),
        }
    }
}

impl LanguageProcessor for RubyProcessor {
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
