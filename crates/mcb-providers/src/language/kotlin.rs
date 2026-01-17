//! Kotlin language processor for AST-based code chunking.

use crate::language::common::{
    BaseProcessor, LanguageConfig, LanguageProcessor, NodeExtractionRule, CHUNK_SIZE_KOTLIN,
    TS_NODE_CLASS_DECLARATION, TS_NODE_FUNCTION_DECLARATION,
};
use mcb_domain::entities::CodeChunk;
use mcb_domain::value_objects::Language;

/// Kotlin language processor.
pub struct KotlinProcessor {
    processor: BaseProcessor,
}

impl Default for KotlinProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl KotlinProcessor {
    /// Create a new Kotlin language processor
    pub fn new() -> Self {
        let config = LanguageConfig::new(tree_sitter_kotlin_ng::LANGUAGE.into())
            .with_rules(vec![NodeExtractionRule {
                node_types: vec![
                    TS_NODE_FUNCTION_DECLARATION.to_string(),
                    TS_NODE_CLASS_DECLARATION.to_string(),
                    "object_declaration".to_string(),
                ],
                min_length: 30,
                min_lines: 2,
                max_depth: 3,
                priority: 5,
                include_context: true,
            }])
            .with_fallback_patterns(vec![
                r"^fun ".to_string(),
                r"^class ".to_string(),
                r"^data class ".to_string(),
                r"^object ".to_string(),
            ])
            .with_chunk_size(CHUNK_SIZE_KOTLIN);

        Self {
            processor: BaseProcessor::new(config),
        }
    }
}

impl LanguageProcessor for KotlinProcessor {
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
