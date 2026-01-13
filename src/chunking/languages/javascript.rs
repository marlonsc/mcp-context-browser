//! JavaScript/TypeScript language processor for AST-based code chunking.

use crate::chunking::config::{LanguageConfig, NodeExtractionRule};
use crate::chunking::processor::BaseProcessor;
use crate::chunking::LanguageProcessor;
use crate::domain::types::{CodeChunk, Language};
use crate::infrastructure::constants::CHUNK_SIZE_JAVASCRIPT;

/// JavaScript/TypeScript language processor supporting both languages.
pub struct JavaScriptProcessor {
    processor: BaseProcessor,
}

impl Default for JavaScriptProcessor {
    fn default() -> Self {
        Self::new(Language::JavaScript)
    }
}

impl JavaScriptProcessor {
    pub fn new(language: Language) -> Self {
        let language_instance = match language {
            Language::TypeScript => tree_sitter_typescript::LANGUAGE_TSX.into(),
            _ => tree_sitter_javascript::LANGUAGE.into(),
        };
        let config = LanguageConfig::new(language_instance)
            .with_rules(vec![NodeExtractionRule {
                node_types: vec![
                    "function_declaration".to_string(),
                    "function".to_string(),
                    "class_declaration".to_string(),
                    "method_definition".to_string(),
                    "arrow_function".to_string(),
                ],
                min_length: 30,
                min_lines: 2,
                max_depth: 2,
                priority: 9,
                include_context: true,
            }])
            .with_fallback_patterns(vec![
                r"^function ".to_string(),
                r"^const .*=>\s*\{".to_string(),
                r"^class ".to_string(),
            ])
            .with_chunk_size(CHUNK_SIZE_JAVASCRIPT);

        Self {
            processor: BaseProcessor::new(config),
        }
    }
}

crate::chunking::processor::impl_language_processor!(JavaScriptProcessor);
