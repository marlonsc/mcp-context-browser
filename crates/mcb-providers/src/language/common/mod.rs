//! Common utilities for language chunking providers
//!
//! This module contains shared code used by all language-specific processors.

pub mod config;
pub mod constants;
pub mod fallback;
pub mod processor;
pub mod traverser;

// Re-export commonly used types
pub use config::{LanguageConfig, NodeExtractionRule, NodeExtractionRuleBuilder};
pub use constants::*;
pub use fallback::GenericFallbackChunker;
pub use processor::{BaseProcessor, LanguageProcessor};
pub use traverser::AstTraverser;
