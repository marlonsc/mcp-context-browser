//! Go language processor for AST-based code chunking.

use crate::chunking::config::{LanguageConfig, NodeExtractionRule};
use crate::chunking::processor::BaseProcessor;
use crate::chunking::LanguageProcessor;
use crate::domain::types::{CodeChunk, Language};
use crate::infrastructure::constants::CHUNK_SIZE_GO;

/// Go language processor with function and type extraction.
pub struct GoProcessor {
    processor: BaseProcessor,
}

impl Default for GoProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl GoProcessor {
    pub fn new() -> Self {
        let config = LanguageConfig::new(tree_sitter_go::LANGUAGE.into())
            .with_rules(vec![NodeExtractionRule {
                node_types: vec![
                    "function_declaration".to_string(),
                    "method_declaration".to_string(),
                    "type_declaration".to_string(),
                    "struct_type".to_string(),
                ],
                min_length: 35,
                min_lines: 2,
                max_depth: 3,
                priority: 8,
                include_context: false,
            }])
            .with_fallback_patterns(vec![r"^func ".to_string(), r"^type ".to_string()])
            .with_chunk_size(CHUNK_SIZE_GO);

        Self {
            processor: BaseProcessor::new(config),
        }
    }
}

crate::chunking::processor::impl_language_processor!(GoProcessor);
