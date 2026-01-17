//! Python language processor for AST-based code chunking.

use crate::language::common::{
    BaseProcessor, LanguageConfig, LanguageProcessor, NodeExtractionRule, CHUNK_SIZE_PYTHON,
    TS_NODE_FUNCTION_DEFINITION,
};
use mcb_domain::entities::CodeChunk;
use mcb_domain::value_objects::Language;

/// Python language processor with function and class extraction.
pub struct PythonProcessor {
    processor: BaseProcessor,
}

impl Default for PythonProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl PythonProcessor {
    /// Create a new Python language processor
    pub fn new() -> Self {
        let config = LanguageConfig::new(tree_sitter_python::LANGUAGE.into())
            .with_rules(vec![NodeExtractionRule {
                node_types: vec![
                    TS_NODE_FUNCTION_DEFINITION.to_string(),
                    "class_definition".to_string(),
                ],
                min_length: 30,
                min_lines: 2,
                max_depth: 2,
                priority: 5,
                include_context: true,
            }])
            .with_fallback_patterns(vec![r"^def ".to_string(), r"^class ".to_string()])
            .with_chunk_size(CHUNK_SIZE_PYTHON);

        Self {
            processor: BaseProcessor::new(config),
        }
    }
}

impl LanguageProcessor for PythonProcessor {
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
