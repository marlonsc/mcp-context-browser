//! Intelligent code chunking adapter using tree-sitter for structural parsing
//!
//! This module re-exports the chunking functionality from `crate::language`.
//! The `language` module is the single source of truth for all code chunking.
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
//! All implementations live in `crate::language`. This module provides
//! re-exports for backward compatibility and clearer API.

// Re-export everything from the language module (single source of truth)
pub use crate::language::{
    // Helpers
    get_chunk_size,
    is_language_supported,
    language_from_extension,
    supported_languages,
    // Base types
    BaseProcessor,
    // Language processors
    CProcessor,
    CSharpProcessor,
    CppProcessor,
    GoProcessor,
    // Engine
    IntelligentChunker,
    JavaProcessor,
    JavaScriptProcessor,
    KotlinProcessor,
    LanguageConfig,
    LanguageProcessor,
    NodeExtractionRule,
    PhpProcessor,
    PythonProcessor,
    RubyProcessor,
    RustProcessor,
    SwiftProcessor,
};

// Re-export constants from language/common
pub use crate::language::common::constants::*;
