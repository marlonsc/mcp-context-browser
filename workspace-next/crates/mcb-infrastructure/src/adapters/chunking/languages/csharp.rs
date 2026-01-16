//! C# language processor for AST-based code chunking.

use crate::adapters::chunking::config::{LanguageConfig, NodeExtractionRule};
use crate::adapters::chunking::constants::CHUNK_SIZE_CSHARP;
use crate::adapters::chunking::processor::{BaseProcessor, LanguageProcessor};
use mcb_domain::entities::CodeChunk;
use mcb_domain::value_objects::Language;

/// C# language processor.
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
                ],
                min_length: 30,
                min_lines: 2,
                max_depth: 3,
                priority: 5,
                include_context: true,
            }])
            .with_fallback_patterns(vec![
                r"^\s*public ".to_string(),
                r"^\s*private ".to_string(),
                r"^\s*class ".to_string(),
                r"^\s*interface ".to_string(),
            ])
            .with_chunk_size(CHUNK_SIZE_CSHARP);

        Self {
            processor: BaseProcessor::new(config),
        }
    }
}

impl LanguageProcessor for CSharpProcessor {
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
