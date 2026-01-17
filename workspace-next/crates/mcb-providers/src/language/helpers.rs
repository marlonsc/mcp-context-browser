//! Language detection and helper utilities
//!
//! Provides functions to detect programming languages from file extensions
//! and other utility functions for working with language identifiers.

use super::common::constants::*;

/// Extension to language mapping table
const EXTENSION_LANG_MAP: &[(&[&str], &str)] = &[
    (&["rs"], LANG_RUST),
    (&["py", "pyw", "pyi"], LANG_PYTHON),
    (&["js", "mjs", "cjs", "jsx"], LANG_JAVASCRIPT),
    (&["ts", "tsx", "mts", "cts"], LANG_TYPESCRIPT),
    (&["go"], LANG_GO),
    (&["java"], LANG_JAVA),
    (&["c", "h"], LANG_C),
    (&["cpp", "cc", "cxx", "hpp", "hxx", "hh"], LANG_CPP),
    (&["cs"], LANG_CSHARP),
    (&["rb", "rake", "gemspec"], LANG_RUBY),
    (&["php", "phtml"], LANG_PHP),
    (&["swift"], LANG_SWIFT),
    (&["kt", "kts"], LANG_KOTLIN),
];

/// Detect language from file extension
///
/// Returns a string identifier for the programming language based on the file extension.
/// Returns "unknown" for unsupported or unrecognized extensions.
pub fn language_from_extension(ext: &str) -> String {
    let ext_lower = ext.to_lowercase();
    EXTENSION_LANG_MAP
        .iter()
        .find(|(exts, _)| exts.iter().any(|e| *e == ext_lower))
        .map(|(_, lang)| (*lang).to_string())
        .unwrap_or_else(|| LANG_UNKNOWN.to_string())
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

/// Language to chunk size mapping table
const LANG_CHUNK_SIZE_MAP: &[(&[&str], usize)] = &[
    (&[LANG_RUST], CHUNK_SIZE_RUST),
    (&[LANG_PYTHON], CHUNK_SIZE_PYTHON),
    (&[LANG_JAVASCRIPT, LANG_TYPESCRIPT], CHUNK_SIZE_JAVASCRIPT),
    (&[LANG_GO], CHUNK_SIZE_GO),
    (&[LANG_JAVA], CHUNK_SIZE_JAVA),
    (&[LANG_C], CHUNK_SIZE_C),
    (&[LANG_CPP], CHUNK_SIZE_CPP),
    (&[LANG_CSHARP], CHUNK_SIZE_CSHARP),
    (&[LANG_RUBY], CHUNK_SIZE_RUBY),
    (&[LANG_PHP], CHUNK_SIZE_PHP),
    (&[LANG_SWIFT], CHUNK_SIZE_SWIFT),
    (&[LANG_KOTLIN], CHUNK_SIZE_KOTLIN),
];

/// Get the chunk size for a specific language
pub fn get_chunk_size(language: &str) -> usize {
    LANG_CHUNK_SIZE_MAP
        .iter()
        .find(|(langs, _)| langs.contains(&language))
        .map(|(_, size)| *size)
        .unwrap_or(CHUNK_SIZE_GENERIC)
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
