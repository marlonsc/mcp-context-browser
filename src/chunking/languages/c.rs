//! C language processor for AST-based code chunking.

use crate::chunking::config::{LanguageConfig, NodeExtractionRule};
use crate::chunking::processor::BaseProcessor;
use crate::chunking::LanguageProcessor;
use crate::domain::types::{CodeChunk, Language};
use crate::infrastructure::constants::CHUNK_SIZE_C;

/// C language processor with function and struct extraction.
pub struct CProcessor {
    processor: BaseProcessor,
}

impl Default for CProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl CProcessor {
    pub fn new() -> Self {
        let config = LanguageConfig::new(tree_sitter_c::LANGUAGE.into())
            .with_rules(vec![NodeExtractionRule {
                node_types: vec![
                    "function_definition".to_string(),
                    "struct_specifier".to_string(),
                    "enum_specifier".to_string(),
                ],
                min_length: 30,
                min_lines: 2,
                max_depth: 2,
                priority: 8,
                include_context: false,
            }])
            .with_fallback_patterns(vec![
                r"^[a-zA-Z_][a-zA-Z0-9_]*\s*\(".to_string(),
                r"^struct ".to_string(),
                r"^enum ".to_string(),
                r"^typedef ".to_string(),
            ])
            .with_chunk_size(CHUNK_SIZE_C);

        Self {
            processor: BaseProcessor::new(config),
        }
    }
}

crate::chunking::processor::impl_language_processor!(CProcessor);
