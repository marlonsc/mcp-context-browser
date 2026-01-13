//! Swift language processor for AST-based code chunking.

use crate::chunking::config::{LanguageConfig, NodeExtractionRule};
use crate::chunking::processor::BaseProcessor;
use crate::chunking::LanguageProcessor;
use crate::domain::types::{CodeChunk, Language};
use crate::infrastructure::constants::CHUNK_SIZE_SWIFT;

/// Swift language processor with function, class, and protocol extraction.
pub struct SwiftProcessor {
    processor: BaseProcessor,
}

impl Default for SwiftProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl SwiftProcessor {
    pub fn new() -> Self {
        let config = LanguageConfig::new(tree_sitter_swift::LANGUAGE.into())
            .with_rules(vec![
                NodeExtractionRule {
                    node_types: vec![
                        "function_declaration".to_string(),
                        "class_declaration".to_string(),
                        "struct_declaration".to_string(),
                        "protocol_declaration".to_string(),
                        "enum_declaration".to_string(),
                    ],
                    min_length: 35,
                    min_lines: 2,
                    max_depth: 3,
                    priority: 9,
                    include_context: true,
                },
                NodeExtractionRule {
                    node_types: vec![
                        "computed_property".to_string(),
                        "subscript_declaration".to_string(),
                        "initializer_declaration".to_string(),
                    ],
                    min_length: 25,
                    min_lines: 2,
                    max_depth: 2,
                    priority: 7,
                    include_context: true,
                },
            ])
            .with_fallback_patterns(vec![
                r"^func ".to_string(),
                r"^class ".to_string(),
                r"^struct ".to_string(),
                r"^protocol ".to_string(),
                r"^enum ".to_string(),
                r"^extension ".to_string(),
            ])
            .with_chunk_size(CHUNK_SIZE_SWIFT);

        Self {
            processor: BaseProcessor::new(config),
        }
    }
}

crate::chunking::processor::impl_language_processor!(SwiftProcessor);
