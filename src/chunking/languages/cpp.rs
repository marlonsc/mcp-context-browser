//! C++ language processor for AST-based code chunking.

use crate::chunking::config::{LanguageConfig, NodeExtractionRule};
use crate::chunking::processor::BaseProcessor;
use crate::chunking::LanguageProcessor;
use crate::domain::types::{CodeChunk, Language};
use crate::infrastructure::constants::CHUNK_SIZE_CPP;

/// C++ language processor with class and template extraction.
pub struct CppProcessor {
    processor: BaseProcessor,
}

impl Default for CppProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl CppProcessor {
    pub fn new() -> Self {
        let config = LanguageConfig::new(tree_sitter_cpp::LANGUAGE.into())
            .with_rules(vec![NodeExtractionRule {
                node_types: vec![
                    "function_definition".to_string(),
                    "class_specifier".to_string(),
                    "struct_specifier".to_string(),
                    "template_declaration".to_string(),
                ],
                min_length: 40,
                min_lines: 3,
                max_depth: 3,
                priority: 9,
                include_context: true,
            }])
            .with_fallback_patterns(vec![
                r"^template ".to_string(),
                r"^class ".to_string(),
                r"^struct ".to_string(),
                r"^[a-zA-Z_][a-zA-Z0-9_]*\s*\(".to_string(),
            ])
            .with_chunk_size(CHUNK_SIZE_CPP);

        Self {
            processor: BaseProcessor::new(config),
        }
    }
}

crate::chunking::processor::impl_language_processor!(CppProcessor);
