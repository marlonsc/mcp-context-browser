//! Swift language processor for AST-based code chunking.

use crate::language::common::{
    BaseProcessor, LanguageConfig, LanguageProcessor, NodeExtractionRule, CHUNK_SIZE_SWIFT,
    TS_NODE_CLASS_DECLARATION, TS_NODE_FUNCTION_DECLARATION,
};
use mcb_domain::entities::CodeChunk;
use mcb_domain::value_objects::Language;

/// Swift language processor.
pub struct SwiftProcessor {
    processor: BaseProcessor,
}

impl Default for SwiftProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl SwiftProcessor {
    /// Create a new Swift language processor
    pub fn new() -> Self {
        let config = LanguageConfig::new(tree_sitter_swift::LANGUAGE.into())
            .with_rules(vec![NodeExtractionRule {
                node_types: vec![
                    TS_NODE_FUNCTION_DECLARATION.to_string(),
                    TS_NODE_CLASS_DECLARATION.to_string(),
                    "struct_declaration".to_string(),
                    "protocol_declaration".to_string(),
                ],
                min_length: 30,
                min_lines: 2,
                max_depth: 3,
                priority: 5,
                include_context: true,
            }])
            .with_fallback_patterns(vec![
                r"^func ".to_string(),
                r"^class ".to_string(),
                r"^struct ".to_string(),
                r"^protocol ".to_string(),
            ])
            .with_chunk_size(CHUNK_SIZE_SWIFT);

        Self {
            processor: BaseProcessor::new(config),
        }
    }
}

impl LanguageProcessor for SwiftProcessor {
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
