//! Python language processor for AST-based code chunking.

use crate::chunking::config::{LanguageConfig, NodeExtractionRule};
use crate::chunking::processor::BaseProcessor;
use crate::chunking::LanguageProcessor;
use crate::domain::types::{CodeChunk, Language};
use crate::infrastructure::constants::CHUNK_SIZE_PYTHON;

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
    pub fn new() -> Self {
        let config = LanguageConfig::new(tree_sitter_python::LANGUAGE.into())
            .with_rules(vec![NodeExtractionRule {
                node_types: vec![
                    "function_definition".to_string(),
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

crate::chunking::processor::impl_language_processor!(PythonProcessor);
