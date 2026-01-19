//! Rust language processor for AST-based code chunking.

use crate::language::common::{
    BaseProcessor, CHUNK_SIZE_RUST, LanguageConfig, LanguageProcessor, NodeExtractionRule,
};
use mcb_domain::entities::CodeChunk;
use mcb_domain::value_objects::Language;

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
    /// Create a new Rust language processor
    pub fn new() -> Self {
        let config = LanguageConfig::new(tree_sitter_rust::LANGUAGE.into())
            .with_rules(Self::extraction_rules())
            .with_fallback_patterns(Self::fallback_patterns())
            .with_chunk_size(CHUNK_SIZE_RUST);

        Self {
            processor: BaseProcessor::new(config),
        }
    }

    fn extraction_rules() -> Vec<NodeExtractionRule> {
        vec![
            NodeExtractionRule::primary(&[
                "function_item",
                "struct_item",
                "enum_item",
                "impl_item",
                "trait_item",
            ]),
            NodeExtractionRule::secondary(&[
                "mod_item",
                "macro_definition",
                "const_item",
                "static_item",
            ]),
            NodeExtractionRule::tertiary(&["type_item", "use_declaration"]),
        ]
    }

    fn fallback_patterns() -> Vec<String> {
        [
            "fn ",
            "struct ",
            "impl ",
            "pub fn ",
            "pub struct ",
            "enum ",
            "trait ",
            "mod ",
            "const ",
            "static ",
        ]
        .iter()
        .map(|p| format!("^{}", p))
        .collect()
    }
}

impl LanguageProcessor for RustProcessor {
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
