//! Ruby language processor for AST-based code chunking.

use crate::chunking::config::{LanguageConfig, NodeExtractionRule};
use crate::chunking::processor::BaseProcessor;
use crate::chunking::LanguageProcessor;
use crate::domain::types::{CodeChunk, Language};
use crate::infrastructure::constants::CHUNK_SIZE_RUBY;

/// Ruby language processor with method, class, and module extraction.
pub struct RubyProcessor {
    processor: BaseProcessor,
}

impl Default for RubyProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl RubyProcessor {
    pub fn new() -> Self {
        let config = LanguageConfig::new(tree_sitter_ruby::LANGUAGE.into())
            .with_rules(vec![
                NodeExtractionRule {
                    node_types: vec![
                        "method".to_string(),
                        "class".to_string(),
                        "module".to_string(),
                        "singleton_method".to_string(),
                    ],
                    min_length: 30,
                    min_lines: 2,
                    max_depth: 3,
                    priority: 9,
                    include_context: true,
                },
                NodeExtractionRule {
                    node_types: vec!["block".to_string(), "lambda".to_string()],
                    min_length: 20,
                    min_lines: 2,
                    max_depth: 2,
                    priority: 5,
                    include_context: false,
                },
            ])
            .with_fallback_patterns(vec![
                r"^def ".to_string(),
                r"^class ".to_string(),
                r"^module ".to_string(),
                r"^attr_".to_string(),
            ])
            .with_chunk_size(CHUNK_SIZE_RUBY);

        Self {
            processor: BaseProcessor::new(config),
        }
    }
}

crate::chunking::processor::impl_language_processor!(RubyProcessor);
