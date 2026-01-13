//! C# language processor for AST-based code chunking.

use crate::chunking::config::{LanguageConfig, NodeExtractionRule};
use crate::chunking::processor::BaseProcessor;
use crate::chunking::LanguageProcessor;
use crate::domain::types::{CodeChunk, Language};
use crate::infrastructure::constants::CHUNK_SIZE_CSHARP;

/// C# language processor with method and class extraction.
pub struct CSharpProcessor {
    processor: BaseProcessor,
}

impl Default for CSharpProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl CSharpProcessor {
    pub fn new() -> Self {
        let config = LanguageConfig::new(tree_sitter_c_sharp::LANGUAGE.into())
            .with_rules(vec![NodeExtractionRule {
                node_types: vec![
                    "method_declaration".to_string(),
                    "class_declaration".to_string(),
                    "interface_declaration".to_string(),
                    "property_declaration".to_string(),
                ],
                min_length: 40,
                min_lines: 3,
                max_depth: 3,
                priority: 9,
                include_context: true,
            }])
            .with_fallback_patterns(vec![
                r"^public .*\(.*\)".to_string(),
                r"^private .*\(.*\)".to_string(),
                r"^protected .*\(.*\)".to_string(),
                r"^class ".to_string(),
                r"^interface ".to_string(),
            ])
            .with_chunk_size(CHUNK_SIZE_CSHARP);

        Self {
            processor: BaseProcessor::new(config),
        }
    }
}

crate::chunking::processor::impl_language_processor!(CSharpProcessor);
