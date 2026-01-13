//! Rust language processor for AST-based code chunking.

use crate::chunking::config::{LanguageConfig, NodeExtractionRule};
use crate::chunking::processor::BaseProcessor;
use crate::chunking::LanguageProcessor;
use crate::domain::types::{CodeChunk, Language};
use crate::infrastructure::constants::CHUNK_SIZE_RUST;

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
    pub fn new() -> Self {
        let config = LanguageConfig::new(tree_sitter_rust::LANGUAGE.into())
            .with_rules(vec![
                // High priority: Main constructs
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
                // Medium priority: Modules and macros
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
                // Low priority: Type aliases and use statements
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

crate::chunking::processor::impl_language_processor!(RustProcessor);
