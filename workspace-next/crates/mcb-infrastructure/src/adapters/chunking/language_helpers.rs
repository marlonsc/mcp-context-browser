//! Language detection and helper utilities
//!
//! Provides functions to detect programming languages from file extensions
//! and other utility functions for working with language identifiers.

use super::constants::*;

/// Detect language from file extension
///
/// Returns a string identifier for the programming language based on the file extension.
/// Returns "unknown" for unsupported or unrecognized extensions.
pub fn language_from_extension(ext: &str) -> String {
    match ext.to_lowercase().as_str() {
        "rs" => LANG_RUST.to_string(),
        "py" | "pyw" | "pyi" => LANG_PYTHON.to_string(),
        "js" | "mjs" | "cjs" => LANG_JAVASCRIPT.to_string(),
        "ts" | "tsx" | "mts" | "cts" => LANG_TYPESCRIPT.to_string(),
        "jsx" => LANG_JAVASCRIPT.to_string(),
        "go" => LANG_GO.to_string(),
        "java" => LANG_JAVA.to_string(),
        "c" | "h" => LANG_C.to_string(),
        "cpp" | "cc" | "cxx" | "hpp" | "hxx" | "hh" => LANG_CPP.to_string(),
        "cs" => LANG_CSHARP.to_string(),
        "rb" | "rake" | "gemspec" => LANG_RUBY.to_string(),
        "php" | "phtml" => LANG_PHP.to_string(),
        "swift" => LANG_SWIFT.to_string(),
        "kt" | "kts" => LANG_KOTLIN.to_string(),
        _ => LANG_UNKNOWN.to_string(),
    }
}

/// Check if a language is supported for AST-based chunking
pub fn is_language_supported(language: &str) -> bool {
    matches!(
        language,
        LANG_RUST
            | LANG_PYTHON
            | LANG_JAVASCRIPT
            | LANG_TYPESCRIPT
            | LANG_GO
            | LANG_JAVA
            | LANG_C
            | LANG_CPP
            | LANG_CSHARP
            | LANG_RUBY
            | LANG_PHP
            | LANG_SWIFT
            | LANG_KOTLIN
    )
}

/// Get the chunk size for a specific language
pub fn get_chunk_size(language: &str) -> usize {
    match language {
        LANG_RUST => CHUNK_SIZE_RUST,
        LANG_PYTHON => CHUNK_SIZE_PYTHON,
        LANG_JAVASCRIPT | LANG_TYPESCRIPT => CHUNK_SIZE_JAVASCRIPT,
        LANG_GO => CHUNK_SIZE_GO,
        LANG_JAVA => CHUNK_SIZE_JAVA,
        LANG_C => CHUNK_SIZE_C,
        LANG_CPP => CHUNK_SIZE_CPP,
        LANG_CSHARP => CHUNK_SIZE_CSHARP,
        LANG_RUBY => CHUNK_SIZE_RUBY,
        LANG_PHP => CHUNK_SIZE_PHP,
        LANG_SWIFT => CHUNK_SIZE_SWIFT,
        LANG_KOTLIN => CHUNK_SIZE_KOTLIN,
        _ => CHUNK_SIZE_GENERIC,
    }
}

/// Get a list of all supported languages
pub fn supported_languages() -> Vec<String> {
    vec![
        LANG_RUST.to_string(),
        LANG_PYTHON.to_string(),
        LANG_JAVASCRIPT.to_string(),
        LANG_TYPESCRIPT.to_string(),
        LANG_GO.to_string(),
        LANG_JAVA.to_string(),
        LANG_C.to_string(),
        LANG_CPP.to_string(),
        LANG_CSHARP.to_string(),
        LANG_RUBY.to_string(),
        LANG_PHP.to_string(),
        LANG_SWIFT.to_string(),
        LANG_KOTLIN.to_string(),
    ]
}

// Tests moved to tests/adapters/chunking/language_helpers_test.rs
