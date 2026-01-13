//! Intelligent code chunking using tree-sitter for structural parsing
//!
//! Provides language-aware chunking that respects code structure rather than
//! naive line-based or character-based splitting.

// Public API re-exports
pub use config::{LanguageConfig, NodeExtractionRule, NodeExtractionRuleBuilder};
pub use engine::IntelligentChunker;
pub use processor::LanguageProcessor;

// Module declarations
pub mod config;
pub mod engine;
pub mod fallback;
pub mod languages;
pub mod processor;
pub mod traverser;

// Re-export language processors
pub use languages::*;

// Language configurations registry
use crate::domain::types::Language;
use std::collections::HashMap;
use std::sync::LazyLock;

pub(crate) static LANGUAGE_CONFIGS: LazyLock<
    HashMap<Language, Box<dyn LanguageProcessor + Send + Sync>>,
> = LazyLock::new(|| {
    let mut configs = HashMap::new();

    // Register all supported languages
    configs.insert(
        Language::Rust,
        Box::new(RustProcessor::new()) as Box<dyn LanguageProcessor + Send + Sync>,
    );
    configs.insert(
        Language::Python,
        Box::new(PythonProcessor::new()) as Box<dyn LanguageProcessor + Send + Sync>,
    );
    configs.insert(
        Language::JavaScript,
        Box::new(JavaScriptProcessor::new(Language::JavaScript))
            as Box<dyn LanguageProcessor + Send + Sync>,
    );
    configs.insert(
        Language::TypeScript,
        Box::new(JavaScriptProcessor::new(Language::TypeScript))
            as Box<dyn LanguageProcessor + Send + Sync>,
    );
    configs.insert(
        Language::Java,
        Box::new(JavaProcessor::new()) as Box<dyn LanguageProcessor + Send + Sync>,
    );
    configs.insert(
        Language::Go,
        Box::new(GoProcessor::new()) as Box<dyn LanguageProcessor + Send + Sync>,
    );
    configs.insert(
        Language::C,
        Box::new(CProcessor::new()) as Box<dyn LanguageProcessor + Send + Sync>,
    );
    configs.insert(
        Language::Cpp,
        Box::new(CppProcessor::new()) as Box<dyn LanguageProcessor + Send + Sync>,
    );
    configs.insert(
        Language::CSharp,
        Box::new(CSharpProcessor::new()) as Box<dyn LanguageProcessor + Send + Sync>,
    );
    configs.insert(
        Language::Ruby,
        Box::new(RubyProcessor::new()) as Box<dyn LanguageProcessor + Send + Sync>,
    );
    configs.insert(
        Language::Php,
        Box::new(PhpProcessor::new()) as Box<dyn LanguageProcessor + Send + Sync>,
    );
    configs.insert(
        Language::Swift,
        Box::new(SwiftProcessor::new()) as Box<dyn LanguageProcessor + Send + Sync>,
    );
    configs.insert(
        Language::Kotlin,
        Box::new(KotlinProcessor::new()) as Box<dyn LanguageProcessor + Send + Sync>,
    );

    configs
});
