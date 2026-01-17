//! Language Chunking Provider Implementations
//!
//! Provides AST-based code parsing for various programming languages.
//! This is the single source of truth for all language-related
//! chunking functionality, including processors and the intelligent chunker.
//!
//! ## Architecture
//!
//! - `common/` - Shared utilities (BaseProcessor, LanguageConfig, constants)
//! - `helpers` - Language detection and utility functions
//! - `engine` - IntelligentChunker that orchestrates language processors
//! - Language-specific processors (rust.rs, python.rs, etc.)
//!
//! ## Available Processors
//!
//! | Processor | Language | Status |
//! |-----------|----------|--------|
//! | [`RustProcessor`] | Rust | Complete |
//! | [`PythonProcessor`] | Python | Complete |
//! | [`JavaScriptProcessor`] | JavaScript/TypeScript | Complete |
//! | [`GoProcessor`] | Go | Complete |
//! | [`JavaProcessor`] | Java | Complete |
//! | [`CProcessor`] | C | Complete |
//! | [`CppProcessor`] | C++ | Complete |
//! | [`CSharpProcessor`] | C# | Complete |
//! | [`RubyProcessor`] | Ruby | Complete |
//! | [`PhpProcessor`] | PHP | Complete |
//! | [`SwiftProcessor`] | Swift | Complete |
//! | [`KotlinProcessor`] | Kotlin | Complete |

/// Common utilities and base types for language processors
pub mod common;

/// Language detection and helper utilities
pub mod helpers;

/// Intelligent chunking engine using tree-sitter
pub mod engine;

// Language-specific processors
pub mod c;
pub mod cpp;
pub mod csharp;
pub mod go;
pub mod java;
pub mod javascript;
pub mod kotlin;
pub mod php;
pub mod python;
pub mod ruby;
pub mod rust;
pub mod swift;

// Re-export processors for convenience
pub use c::CProcessor;
pub use common::{BaseProcessor, LanguageConfig, LanguageProcessor, NodeExtractionRule};
pub use cpp::CppProcessor;
pub use csharp::CSharpProcessor;
pub use go::GoProcessor;
pub use java::JavaProcessor;
pub use javascript::JavaScriptProcessor;
pub use kotlin::KotlinProcessor;
pub use php::PhpProcessor;
pub use python::PythonProcessor;
pub use ruby::RubyProcessor;
pub use rust::RustProcessor;
pub use swift::SwiftProcessor;

// Re-export engine and helpers
pub use engine::IntelligentChunker;
pub use helpers::{get_chunk_size, is_language_supported, language_from_extension, supported_languages};
