//! Test the new define_language_processor! macro

#[allow(unused_imports)]
use mcp_context_browser::domain::chunking::LanguageProcessor;

mcp_context_browser::define_language_processor! {
    TestProcessorSimple,
    tree_sitter_rust::LANGUAGE,
    chunk_size: 512,
    doc: "Simple test processor with one rule",
    rules: [
        {
            node_types: ["function_item"],
            min_length: 40,
            min_lines: 2,
            max_depth: 4,
            priority: 10,
            include_context: true,
        },
    ],
    fallback_patterns: [r"^fn "],
}

mcp_context_browser::define_language_processor! {
    TestProcessorComplex,
    tree_sitter_python::LANGUAGE,
    chunk_size: 1024,
    doc: "Complex test processor with multiple rules",
    rules: [
        {
            node_types: ["function_definition", "class_definition"],
            min_length: 30,
            min_lines: 2,
            max_depth: 2,
            priority: 5,
            include_context: true,
        },
        {
            node_types: ["async_function_def"],
            min_length: 25,
            min_lines: 1,
            max_depth: 3,
            priority: 4,
            include_context: false,
        },
    ],
    fallback_patterns: [r"^def ", r"^class ", r"^async def "],
}

#[test]
fn test_macro_generates_valid_simple_processor() {
    let processor = TestProcessorSimple::new();
    let config = processor.config();

    // Verify configuration is correct
    assert_eq!(config.chunk_size, 512);
    assert_eq!(config.extraction_rules.len(), 1);
    assert_eq!(config.fallback_patterns.len(), 1);
}

#[test]
fn test_macro_generates_default_impl() {
    let processor = TestProcessorSimple::default();
    assert_eq!(processor.config().chunk_size, 512);
}

#[test]
fn test_macro_generates_complex_processor() {
    let processor = TestProcessorComplex::new();
    let config = processor.config();

    // Verify multiple rules
    assert_eq!(config.extraction_rules.len(), 2);
    assert_eq!(config.fallback_patterns.len(), 3);
    assert_eq!(config.chunk_size, 1024);

    // Verify first rule
    assert_eq!(config.extraction_rules[0].priority, 5);
    assert_eq!(config.extraction_rules[0].node_types.len(), 2);

    // Verify second rule
    assert_eq!(config.extraction_rules[1].priority, 4);
    assert!(!config.extraction_rules[1].include_context);
}

#[test]
fn test_processor_implements_trait() {
    let processor = TestProcessorSimple::new();

    // Verify it implements LanguageProcessor trait
    let config = processor.config();
    assert!(!config.extraction_rules.is_empty());
}
