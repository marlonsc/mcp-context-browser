//! Go language processor for AST-based code chunking.

use crate::language::common::{
    BaseProcessor, CHUNK_SIZE_GO, LanguageConfig, LanguageProcessor, NodeExtractionRule,
    TS_NODE_FUNCTION_DECLARATION, TS_NODE_METHOD_DECLARATION,
};
use mcb_domain::entities::CodeChunk;
use mcb_domain::value_objects::Language;

/// Go language processor.
pub struct GoProcessor {
    processor: BaseProcessor,
}

impl Default for GoProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl GoProcessor {
    /// Create a new Go language processor
    pub fn new() -> Self {
        let config = LanguageConfig::new(tree_sitter_go::LANGUAGE.into())
            .with_rules(vec![NodeExtractionRule {
                node_types: vec![
                    TS_NODE_FUNCTION_DECLARATION.to_string(),
                    TS_NODE_METHOD_DECLARATION.to_string(),
                    "type_declaration".to_string(),
                ],
                min_length: 30,
                min_lines: 2,
                max_depth: 2,
                priority: 5,
                include_context: true,
            }])
            .with_fallback_patterns(vec![
                r"^func ".to_string(),
                r"^type ".to_string(),
                r"^interface ".to_string(),
                r"^struct ".to_string(),
            ])
            .with_chunk_size(CHUNK_SIZE_GO);

        Self {
            processor: BaseProcessor::new(config),
        }
    }
}

impl LanguageProcessor for GoProcessor {
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
