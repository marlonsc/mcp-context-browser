//! C++ language processor for AST-based code chunking.

use crate::language::common::{
    BaseProcessor, LanguageConfig, LanguageProcessor, NodeExtractionRule, CHUNK_SIZE_CPP,
    AST_NODE_STRUCT_SPECIFIER, TS_NODE_FUNCTION_DEFINITION,
};
use mcb_domain::entities::CodeChunk;
use mcb_domain::value_objects::Language;

/// C++ language processor.
pub struct CppProcessor {
    processor: BaseProcessor,
}

impl Default for CppProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl CppProcessor {
    /// Create a new C++ language processor
    pub fn new() -> Self {
        let config = LanguageConfig::new(tree_sitter_cpp::LANGUAGE.into())
            .with_rules(vec![NodeExtractionRule {
                node_types: vec![
                    TS_NODE_FUNCTION_DEFINITION.to_string(),
                    "class_specifier".to_string(),
                    AST_NODE_STRUCT_SPECIFIER.to_string(),
                ],
                min_length: 30,
                min_lines: 2,
                max_depth: 3,
                priority: 5,
                include_context: true,
            }])
            .with_fallback_patterns(vec![
                r"^class ".to_string(),
                r"^struct ".to_string(),
                r"^[a-zA-Z_].*\(.*\)\s*\{".to_string(),
            ])
            .with_chunk_size(CHUNK_SIZE_CPP);

        Self {
            processor: BaseProcessor::new(config),
        }
    }
}

impl LanguageProcessor for CppProcessor {
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
