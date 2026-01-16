//! Language Helpers Tests
//!
//! Tests for language detection and helper utilities.

use mcb_infrastructure::adapters::chunking::{
    get_chunk_size, is_language_supported, language_from_extension,
    CHUNK_SIZE_GENERIC, CHUNK_SIZE_RUST, LANG_C, LANG_CPP, LANG_CSHARP, LANG_GO, LANG_JAVA,
    LANG_JAVASCRIPT, LANG_KOTLIN, LANG_PHP, LANG_PYTHON, LANG_RUBY, LANG_RUST, LANG_SWIFT,
    LANG_TYPESCRIPT, LANG_UNKNOWN,
};

#[test]
fn test_language_from_extension() {
    assert_eq!(language_from_extension("rs"), LANG_RUST);
    assert_eq!(language_from_extension("py"), LANG_PYTHON);
    assert_eq!(language_from_extension("js"), LANG_JAVASCRIPT);
    assert_eq!(language_from_extension("ts"), LANG_TYPESCRIPT);
    assert_eq!(language_from_extension("tsx"), LANG_TYPESCRIPT);
    assert_eq!(language_from_extension("go"), LANG_GO);
    assert_eq!(language_from_extension("java"), LANG_JAVA);
    assert_eq!(language_from_extension("c"), LANG_C);
    assert_eq!(language_from_extension("cpp"), LANG_CPP);
    assert_eq!(language_from_extension("cs"), LANG_CSHARP);
    assert_eq!(language_from_extension("rb"), LANG_RUBY);
    assert_eq!(language_from_extension("php"), LANG_PHP);
    assert_eq!(language_from_extension("swift"), LANG_SWIFT);
    assert_eq!(language_from_extension("kt"), LANG_KOTLIN);
    assert_eq!(language_from_extension("unknown"), LANG_UNKNOWN);
}

#[test]
fn test_is_language_supported() {
    assert!(is_language_supported(LANG_RUST));
    assert!(is_language_supported(LANG_PYTHON));
    assert!(!is_language_supported(LANG_UNKNOWN));
    assert!(!is_language_supported("haskell"));
}

#[test]
fn test_get_chunk_size() {
    assert_eq!(get_chunk_size(LANG_RUST), CHUNK_SIZE_RUST);
    assert_eq!(get_chunk_size(LANG_UNKNOWN), CHUNK_SIZE_GENERIC);
}
