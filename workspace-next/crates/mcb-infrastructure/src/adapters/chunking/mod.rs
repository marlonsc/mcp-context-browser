//! Intelligent code chunking adapter using tree-sitter for structural parsing
//!
//! Provides language-aware chunking that respects code structure rather than
//! naive line-based or character-based splitting.
//!
//! ## Overview
//!
//! The chunking adapter breaks source code into semantically meaningful segments
//! for embedding and indexing. Each chunk represents a logical unit (function,
//! class, method) extracted via Abstract Syntax Tree (AST) parsing.
//!
//! ## Supported Languages
//!
//! - Rust, Python, JavaScript, TypeScript
//! - Go, Java, C, C++, C#
//! - Ruby, PHP, Swift, Kotlin
//!
//! ## Architecture
//!
//! This adapter implements the `CodeChunker` port trait from mcb-domain,
//! providing the actual AST-based chunking implementation using tree-sitter.

// Configuration and constants
pub mod config;
pub mod constants;

// Language detection utilities
pub mod language_helpers;

// Core chunking components
pub mod engine;
pub mod fallback;
pub mod processor;
pub mod traverser;

// Language-specific processors
pub mod languages;

// Public re-exports
pub use config::{LanguageConfig, NodeExtractionRule, NodeExtractionRuleBuilder};
pub use constants::*;
pub use engine::IntelligentChunker;
pub use language_helpers::{get_chunk_size, is_language_supported, language_from_extension, supported_languages};
pub use processor::LanguageProcessor;

// Re-export language processors
pub use languages::*;
