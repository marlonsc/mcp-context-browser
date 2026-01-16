//! JavaScript/TypeScript language processor for AST-based code chunking.

use crate::adapters::chunking::config::{LanguageConfig, NodeExtractionRule};
use crate::adapters::chunking::constants::CHUNK_SIZE_JAVASCRIPT;
use crate::adapters::chunking::processor::{BaseProcessor, LanguageProcessor};
use mcb_domain::entities::CodeChunk;
use mcb_domain::value_objects::Language;

/// JavaScript/TypeScript language processor.
pub struct JavaScriptProcessor {
    processor: BaseProcessor,
}

impl Default for JavaScriptProcessor {
    fn default() -> Self {
        Self::new(false)
    }
}

impl JavaScriptProcessor {
    /// Create a new JavaScript/TypeScript language processor
    pub fn new(is_typescript: bool) -> Self {
        let ts_language = if is_typescript {
            tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()
        } else {
            tree_sitter_javascript::LANGUAGE.into()
        };

        let config = LanguageConfig::new(ts_language)
            .with_rules(vec![NodeExtractionRule {
                node_types: vec![
                    "function_declaration".to_string(),
                    "class_declaration".to_string(),
                    "method_definition".to_string(),
                    "arrow_function".to_string(),
                    "interface_declaration".to_string(),
                    "type_alias_declaration".to_string(),
                ],
                min_length: 30,
                min_lines: 2,
                max_depth: 3,
                priority: 5,
                include_context: true,
            }])
            .with_fallback_patterns(vec![
                r"^function ".to_string(),
                r"^class ".to_string(),
                r"^const .* = ".to_string(),
                r"^export ".to_string(),
                r"^interface ".to_string(),
                r"^type ".to_string(),
            ])
            .with_chunk_size(CHUNK_SIZE_JAVASCRIPT);

        Self {
            processor: BaseProcessor::new(config),
        }
    }
}

impl LanguageProcessor for JavaScriptProcessor {
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
