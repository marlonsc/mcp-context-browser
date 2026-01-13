//! Language processor trait and implementations
//!
//! This module defines the LanguageProcessor trait that provides a common interface
//! for language-specific chunking logic.

use crate::domain::chunking::config::LanguageConfig;
use crate::domain::types::{CodeChunk, Language};
use shaku::Interface;

/// Trait for language-specific processing
pub trait LanguageProcessor: Interface {
    /// Get language configuration
    fn config(&self) -> &LanguageConfig;

    /// Extract chunks using tree-sitter
    fn extract_chunks_with_tree_sitter(
        &self,
        tree: &tree_sitter::Tree,
        content: &str,
        file_name: &str,
        language: &Language,
    ) -> Vec<CodeChunk>;

    /// Extract chunks using fallback method
    fn extract_chunks_fallback(
        &self,
        content: &str,
        file_name: &str,
        language: &Language,
    ) -> Vec<CodeChunk>;

    /// Get the language instance
    fn get_language(&self) -> tree_sitter::Language {
        self.config().get_language()
    }
}

/// Base processor struct that holds configuration
#[derive(Debug)]
pub struct BaseProcessor {
    config: LanguageConfig,
}

impl BaseProcessor {
    /// Create a new base processor with configuration
    pub fn new(config: LanguageConfig) -> Self {
        Self { config }
    }

    /// Get the configuration
    pub fn config(&self) -> &LanguageConfig {
        &self.config
    }
}

impl LanguageProcessor for BaseProcessor {
    fn config(&self) -> &LanguageConfig {
        &self.config
    }

    fn extract_chunks_with_tree_sitter(
        &self,
        tree: &tree_sitter::Tree,
        content: &str,
        file_name: &str,
        language: &Language,
    ) -> Vec<CodeChunk> {
        let mut chunks = Vec::new();
        let mut cursor = tree.walk();

        if cursor.goto_first_child() {
            let traverser = crate::domain::chunking::traverser::AstTraverser::new(
                &self.config().extraction_rules,
                language,
            )
            .with_max_chunks(75); // Limit chunks per file
            traverser.traverse_and_extract(&mut cursor, content, file_name, 0, &mut chunks);
        }

        // Sort chunks by priority (highest first) and then by line number
        chunks.sort_by(|a, b| {
            let a_priority = a
                .metadata
                .get("priority")
                .and_then(|p| p.as_i64())
                .unwrap_or(0);
            let b_priority = b
                .metadata
                .get("priority")
                .and_then(|p| p.as_i64())
                .unwrap_or(0);

            // Sort by priority descending, then by start_line ascending
            b_priority
                .cmp(&a_priority)
                .then(a.start_line.cmp(&b.start_line))
        });

        // Keep only top priority chunks if we have too many
        if chunks.len() > 50 {
            chunks.truncate(50);
        }

        chunks
    }

    fn extract_chunks_fallback(
        &self,
        content: &str,
        file_name: &str,
        language: &Language,
    ) -> Vec<CodeChunk> {
        crate::domain::chunking::fallback::GenericFallbackChunker::new(self.config())
            .chunk_with_patterns(content, file_name, language)
    }
}

/// Macro to implement LanguageProcessor for concrete processors
#[macro_export]
macro_rules! impl_language_processor {
    ($processor:ty) => {
        impl $crate::domain::chunking::LanguageProcessor for $processor {
            fn config(&self) -> &$crate::domain::chunking::config::LanguageConfig {
                &self.processor.config()
            }

            fn extract_chunks_with_tree_sitter(
                &self,
                tree: &tree_sitter::Tree,
                content: &str,
                file_name: &str,
                language: &$crate::domain::types::Language,
            ) -> Vec<$crate::domain::types::CodeChunk> {
                let mut chunks = Vec::new();
                let mut cursor = tree.walk();

                if cursor.goto_first_child() {
                    let traverser = $crate::domain::chunking::traverser::AstTraverser::new(
                        &self.config().extraction_rules,
                        language,
                    )
                    .with_max_chunks(75); // Limit chunks per file
                    traverser.traverse_and_extract(&mut cursor, content, file_name, 0, &mut chunks);
                }

                // Sort chunks by priority (highest first) and then by line number
                chunks.sort_by(|a, b| {
                    let a_priority = a
                        .metadata
                        .get("priority")
                        .and_then(|p| p.as_i64())
                        .unwrap_or(0);
                    let b_priority = b
                        .metadata
                        .get("priority")
                        .and_then(|p| p.as_i64())
                        .unwrap_or(0);

                    // Sort by priority descending, then by start_line ascending
                    b_priority
                        .cmp(&a_priority)
                        .then(a.start_line.cmp(&b.start_line))
                });

                // Keep only top priority chunks if we have too many
                if chunks.len() > 50 {
                    chunks.truncate(50);
                }

                chunks
            }

            fn extract_chunks_fallback(
                &self,
                content: &str,
                file_name: &str,
                language: &$crate::domain::types::Language,
            ) -> Vec<$crate::domain::types::CodeChunk> {
                $crate::domain::chunking::fallback::GenericFallbackChunker::new(self.config())
                    .chunk_with_patterns(content, file_name, language)
            }
        }
    };
}

// Export the macro for use in language-specific modules
pub(crate) use impl_language_processor;

/// Macro to define complete language processor with boilerplate generation
///
/// Generates:
/// - Processor struct
/// - Default impl
/// - new() constructor with full configuration
/// - LanguageProcessor trait implementation
///
/// # Example
/// ```ignore
/// define_language_processor! {
///     PythonProcessor,
///     tree_sitter_python::LANGUAGE,
///     chunk_size: 512,
///     doc: "Python language processor",
///     rules: [
///         {
///             node_types: ["function_definition"],
///             min_length: 30,
///             min_lines: 2,
///             max_depth: 2,
///             priority: 5,
///             include_context: true,
///         },
///     ],
///     fallback_patterns: [r"^def "],
/// }
/// ```
#[macro_export]
macro_rules! define_language_processor {
    (
        $processor_name:ident,
        $ts_language:expr,
        chunk_size: $chunk_size:expr,
        doc: $doc:literal,
        rules: [
            $(
                {
                    node_types: [$($node_type:literal),* $(,)?],
                    min_length: $min_length:expr,
                    min_lines: $min_lines:expr,
                    max_depth: $max_depth:expr,
                    priority: $priority:expr,
                    include_context: $include_context:expr,
                }
            ),* $(,)?
        ],
        fallback_patterns: [$($pattern:literal),* $(,)?],
    ) => {
        #[doc = $doc]
        pub struct $processor_name {
            processor: $crate::domain::chunking::processor::BaseProcessor,
        }

        impl Default for $processor_name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl $processor_name {
            /// Create a new language processor
            pub fn new() -> Self {
                use $crate::domain::chunking::config::{LanguageConfig, NodeExtractionRule};

                let config = LanguageConfig::new($ts_language.into())
                    .with_rules(vec![
                        $(
                            NodeExtractionRule {
                                node_types: vec![$($node_type.to_string()),*],
                                min_length: $min_length,
                                min_lines: $min_lines,
                                max_depth: $max_depth,
                                priority: $priority,
                                include_context: $include_context,
                            },
                        )*
                    ])
                    .with_fallback_patterns(vec![
                        $($pattern.to_string()),*
                    ])
                    .with_chunk_size($chunk_size);

                Self {
                    processor: $crate::domain::chunking::processor::BaseProcessor::new(config),
                }
            }
        }

        // Apply the trait implementation
        $crate::impl_language_processor!($processor_name);
    };
}

/// Generates complete language processor with language parameter support (for dual-language processors like JavaScript/TypeScript)
#[macro_export]
macro_rules! define_language_processor_with_param {
    (
        $processor_name:ident,
        language_fn: $language_fn:expr,
        chunk_size: $chunk_size:expr,
        doc: $doc:literal,
        rules: [
            $(
                {
                    node_types: [$($node_type:literal),* $(,)?],
                    min_length: $min_length:expr,
                    min_lines: $min_lines:expr,
                    max_depth: $max_depth:expr,
                    priority: $priority:expr,
                    include_context: $include_context:expr,
                }
            ),* $(,)?
        ],
        fallback_patterns: [$($pattern:literal),* $(,)?],
    ) => {
        #[doc = $doc]
        pub struct $processor_name {
            processor: $crate::domain::chunking::processor::BaseProcessor,
        }

        impl $processor_name {
            /// Create new processor with language parameter
            pub fn new(language: $crate::domain::types::Language) -> Self {
                use $crate::domain::chunking::config::{LanguageConfig, NodeExtractionRule};

                let language_instance = $language_fn(language);

                let config = LanguageConfig::new(language_instance)
                    .with_rules(vec![
                        $(
                            NodeExtractionRule {
                                node_types: vec![$($node_type.to_string()),*],
                                min_length: $min_length,
                                min_lines: $min_lines,
                                max_depth: $max_depth,
                                priority: $priority,
                                include_context: $include_context,
                            },
                        )*
                    ])
                    .with_fallback_patterns(vec![
                        $($pattern.to_string()),*
                    ])
                    .with_chunk_size($chunk_size);

                Self {
                    processor: $crate::domain::chunking::processor::BaseProcessor::new(config),
                }
            }
        }

        impl Default for $processor_name {
            fn default() -> Self {
                Self::new($crate::domain::types::Language::JavaScript)
            }
        }

        $crate::domain::chunking::processor::impl_language_processor!($processor_name);
    };
}
