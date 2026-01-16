//! Rust language processor for AST-based code chunking.

use crate::adapters::chunking::config::{LanguageConfig, NodeExtractionRule};
use crate::adapters::chunking::constants::CHUNK_SIZE_RUST;
use crate::adapters::chunking::processor::{BaseProcessor, LanguageProcessor};
use mcb_domain::entities::CodeChunk;
use mcb_domain::value_objects::Language;

/// Rust language processor with comprehensive AST extraction rules.
pub struct RustProcessor {
    processor: BaseProcessor,
}

impl Default for RustProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl RustProcessor {
    /// Create a new Rust language processor
    pub fn new() -> Self {
        let config = LanguageConfig::new(tree_sitter_rust::LANGUAGE.into())
            .with_rules(vec![
                NodeExtractionRule {
                    node_types: vec![
                        "function_item".to_string(),
                        "struct_item".to_string(),
                        "enum_item".to_string(),
                        "impl_item".to_string(),
                        "trait_item".to_string(),
                    ],
                    min_length: 40,
                    min_lines: 2,
                    max_depth: 4,
                    priority: 10,
                    include_context: true,
                },
                NodeExtractionRule {
                    node_types: vec![
                        "mod_item".to_string(),
                        "macro_definition".to_string(),
                        "const_item".to_string(),
                        "static_item".to_string(),
                    ],
                    min_length: 25,
                    min_lines: 1,
                    max_depth: 3,
                    priority: 5,
                    include_context: false,
                },
                NodeExtractionRule {
                    node_types: vec!["type_item".to_string(), "use_declaration".to_string()],
                    min_length: 15,
                    min_lines: 1,
                    max_depth: 2,
                    priority: 1,
                    include_context: false,
                },
            ])
            .with_fallback_patterns(vec![
                r"^fn ".to_string(),
                r"^struct ".to_string(),
                r"^impl ".to_string(),
                r"^pub fn ".to_string(),
                r"^pub struct ".to_string(),
                r"^enum ".to_string(),
                r"^trait ".to_string(),
                r"^mod ".to_string(),
                r"^const ".to_string(),
                r"^static ".to_string(),
            ])
            .with_chunk_size(CHUNK_SIZE_RUST);

        Self {
            processor: BaseProcessor::new(config),
        }
    }
}

impl LanguageProcessor for RustProcessor {
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
