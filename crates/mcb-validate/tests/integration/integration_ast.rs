//! Integration tests for AST parsing and query execution (Phase 2 of mcb-validate refactoring)
//!
//! These tests verify that:
//! - Tree-sitter parsers can parse multiple languages
//! - AST queries can match patterns in code
//! - `AstQuery` can detect violations like .`unwrap()` usage
//! - The unified AST format works correctly

#![allow(clippy::items_after_statements)]
#![allow(clippy::similar_names)]

use mcb_validate::ast::languages::{
    GoParser, JavaScriptParser, PythonParser, RustParser, TypeScriptParser,
};
use mcb_validate::ast::{
    AstEngine, AstNode, AstParser, AstQuery, AstQueryBuilder, AstQueryPatterns, QueryCondition,
};

// ==================== Parser Creation Tests ====================

#[test]
fn test_ast_engine_creation() {
    let engine = AstEngine::new();
    let languages = engine.supported_languages();

    assert!(languages.contains(&"rust"), "Should support Rust");
    assert!(languages.contains(&"python"), "Should support Python");
    assert!(
        languages.contains(&"javascript"),
        "Should support JavaScript"
    );
    assert!(
        languages.contains(&"typescript"),
        "Should support TypeScript"
    );
    assert!(languages.contains(&"go"), "Should support Go");
}

#[test]
fn test_language_detection() {
    let engine = AstEngine::new();

    assert_eq!(
        engine.detect_language(std::path::Path::new("main.rs")),
        Some("rust")
    );
    assert_eq!(
        engine.detect_language(std::path::Path::new("script.py")),
        Some("python")
    );
    assert_eq!(
        engine.detect_language(std::path::Path::new("app.js")),
        Some("javascript")
    );
    assert_eq!(
        engine.detect_language(std::path::Path::new("component.ts")),
        Some("typescript")
    );
    assert_eq!(
        engine.detect_language(std::path::Path::new("main.go")),
        Some("go")
    );
    assert_eq!(
        engine.detect_language(std::path::Path::new("unknown.xyz")),
        None
    );
}

// ==================== Rust Parser Tests ====================

#[test]
fn test_rust_parser_simple_function() {
    let mut parser = RustParser::new();
    let code = r#"
fn hello_world() {
    println!("Hello, World!");
}
"#;

    let result = parser
        .parse_content(code, "test.rs")
        .expect("Should parse Rust code");
    assert_eq!(result.root.kind, "source_file");
    assert!(
        !result.root.children.is_empty(),
        "Should have children nodes"
    );
}

#[test]
fn test_rust_parser_with_unwrap() {
    let mut parser = RustParser::new();
    let code = r#"
fn risky_function() {
    let value = Some(42);
    let unwrapped = value.unwrap();
    println!("{}", unwrapped);
}
"#;

    let result = parser
        .parse_content(code, "test.rs")
        .expect("Should parse Rust code with unwrap");
    assert_eq!(result.root.kind, "source_file");

    // Verify the AST contains the function
    fn find_node_by_kind<'a>(node: &'a AstNode, kind: &str) -> Option<&'a AstNode> {
        if node.kind == kind {
            return Some(node);
        }
        for child in &node.children {
            if let Some(found) = find_node_by_kind(child, kind) {
                return Some(found);
            }
        }
        None
    }

    assert!(
        find_node_by_kind(&result.root, "function_item").is_some(),
        "Should find function_item node"
    );
}

#[test]
fn test_rust_parser_struct() {
    let mut parser = RustParser::new();
    let code = r"
pub struct MyService {
    name: String,
    value: i32,
}

impl MyService {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            value: 0,
        }
    }
}
";

    let result = parser
        .parse_content(code, "test.rs")
        .expect("Should parse Rust struct");
    assert_eq!(result.root.kind, "source_file");
    assert!(!result.root.children.is_empty());
}

// ==================== Python Parser Tests ====================

#[test]
fn test_python_parser_simple_function() {
    let mut parser = PythonParser::new();
    let code = r#"
def hello_world():
    print("Hello, World!")
"#;

    let result = parser
        .parse_content(code, "test.py")
        .expect("Should parse Python code");
    assert_eq!(result.root.kind, "module");
    assert!(!result.root.children.is_empty());
}

#[test]
fn test_python_parser_class() {
    let mut parser = PythonParser::new();
    let code = r"
class MyService:
    def __init__(self, name: str):
        self.name = name
        self.value = 0

    def get_name(self) -> str:
        return self.name
";

    let result = parser
        .parse_content(code, "test.py")
        .expect("Should parse Python class");
    assert_eq!(result.root.kind, "module");
    assert!(!result.root.children.is_empty());
}

// ==================== JavaScript Parser Tests ====================

#[test]
fn test_javascript_parser_simple_function() {
    let mut parser = JavaScriptParser::new();
    let code = r#"
function helloWorld() {
    console.log("Hello, World!");
}
"#;

    let result = parser
        .parse_content(code, "test.js")
        .expect("Should parse JavaScript code");
    assert_eq!(result.root.kind, "program");
    assert!(!result.root.children.is_empty());
}

#[test]
fn test_javascript_parser_arrow_function() {
    let mut parser = JavaScriptParser::new();
    let code = r"
const add = (a, b) => a + b;
const multiply = (a, b) => {
    return a * b;
};
";

    let result = parser
        .parse_content(code, "test.js")
        .expect("Should parse JS arrow functions");
    assert_eq!(result.root.kind, "program");
    assert!(!result.root.children.is_empty());
}

// ==================== TypeScript Parser Tests ====================

#[test]
fn test_typescript_parser_typed_function() {
    let mut parser = TypeScriptParser::new();
    let code = r"
function greet(name: string): string {
    return `Hello, ${name}!`;
}

interface Person {
    name: string;
    age: number;
}
";

    let result = parser
        .parse_content(code, "test.ts")
        .expect("Should parse TypeScript code");
    // TSX parser reports as "program"
    assert!(!result.root.children.is_empty());
}

// ==================== Go Parser Tests ====================

#[test]
fn test_go_parser_simple_function() {
    let mut parser = GoParser::new();
    let code = r#"
package main

import "fmt"

func main() {
    fmt.Println("Hello, World!")
}
"#;

    let result = parser
        .parse_content(code, "test.go")
        .expect("Should parse Go code");
    assert_eq!(result.root.kind, "source_file");
    assert!(!result.root.children.is_empty());
}

// ==================== AST Query Tests ====================

#[test]
fn test_ast_query_builder() {
    let query = AstQueryBuilder::new("rust", "function_item")
        .with_condition(QueryCondition::Custom {
            name: "has_no_docstring".to_string(),
        })
        .message("Function needs documentation")
        .severity("warning")
        .build();

    assert_eq!(query.language, "rust");
    assert_eq!(query.node_type, "function_item");
    assert_eq!(query.message, "Function needs documentation");
    assert_eq!(query.severity, "warning");
    assert_eq!(query.conditions.len(), 1);
}

#[test]
fn test_ast_query_patterns_undocumented_functions() {
    let query = AstQueryPatterns::undocumented_functions("rust");
    assert_eq!(query.language, "rust");
    assert_eq!(query.node_type, "function_item");
    assert_eq!(query.message, "Functions must be documented");
    assert_eq!(query.severity, "warning");
}

#[test]
fn test_ast_query_patterns_unwrap_usage() {
    let query = AstQueryPatterns::unwrap_usage("rust");
    assert_eq!(query.language, "rust");
    assert_eq!(query.node_type, "call_expression");
    assert_eq!(query.message, "Avoid unwrap() in production code");
    assert_eq!(query.severity, "error");
}

#[test]
fn test_ast_query_patterns_async_functions() {
    let query = AstQueryPatterns::async_functions("rust");
    assert_eq!(query.language, "rust");
    assert_eq!(query.node_type, "function_item");
    assert_eq!(query.message, "Async function detected");
    assert_eq!(query.severity, "info");
}

#[test]
fn test_ast_query_node_type_matching() {
    let query = AstQuery::new("rust", "identifier", "Found identifier", "info");

    let node = AstNode {
        kind: "identifier".to_string(),
        name: Some("test".to_string()),
        span: mcb_validate::ast::Span {
            start: mcb_validate::ast::Position {
                line: 1,
                column: 1,
                byte_offset: 0,
            },
            end: mcb_validate::ast::Position {
                line: 1,
                column: 5,
                byte_offset: 4,
            },
        },
        children: vec![],
        metadata: std::collections::HashMap::new(),
    };

    let violations = query.execute(&node);
    assert_eq!(violations.len(), 1, "Should match identifier node");
    assert!(violations[0].message.contains("Found identifier"));
}

#[test]
fn test_ast_query_no_match() {
    let query = AstQuery::new("rust", "function_item", "Found function", "info");

    let node = AstNode {
        kind: "identifier".to_string(),
        name: Some("test".to_string()),
        span: mcb_validate::ast::Span {
            start: mcb_validate::ast::Position {
                line: 1,
                column: 1,
                byte_offset: 0,
            },
            end: mcb_validate::ast::Position {
                line: 1,
                column: 5,
                byte_offset: 4,
            },
        },
        children: vec![],
        metadata: std::collections::HashMap::new(),
    };

    let violations = query.execute(&node);
    assert_eq!(violations.len(), 0, "Should not match non-function node");
}

#[test]
fn test_ast_query_recursive_matching() {
    let query = AstQuery::new("rust", "identifier", "Found identifier", "info");

    let child_node = AstNode {
        kind: "identifier".to_string(),
        name: Some("inner".to_string()),
        span: mcb_validate::ast::Span {
            start: mcb_validate::ast::Position {
                line: 2,
                column: 1,
                byte_offset: 10,
            },
            end: mcb_validate::ast::Position {
                line: 2,
                column: 6,
                byte_offset: 15,
            },
        },
        children: vec![],
        metadata: std::collections::HashMap::new(),
    };

    let root_node = AstNode {
        kind: "source_file".to_string(),
        name: None,
        span: mcb_validate::ast::Span {
            start: mcb_validate::ast::Position {
                line: 1,
                column: 1,
                byte_offset: 0,
            },
            end: mcb_validate::ast::Position {
                line: 3,
                column: 1,
                byte_offset: 20,
            },
        },
        children: vec![child_node],
        metadata: std::collections::HashMap::new(),
    };

    let violations = query.execute(&root_node);
    assert_eq!(violations.len(), 1, "Should find identifier in children");
}

// ==================== Query Condition Tests ====================

#[test]
fn test_query_condition_has_child() {
    let query = AstQueryBuilder::new("rust", "function_item")
        .with_condition(QueryCondition::HasChild {
            child_type: "block".to_string(),
        })
        .message("Function has block")
        .severity("info")
        .build();

    let block_node = AstNode {
        kind: "block".to_string(),
        name: None,
        span: mcb_validate::ast::Span {
            start: mcb_validate::ast::Position {
                line: 1,
                column: 20,
                byte_offset: 20,
            },
            end: mcb_validate::ast::Position {
                line: 3,
                column: 1,
                byte_offset: 50,
            },
        },
        children: vec![],
        metadata: std::collections::HashMap::new(),
    };

    let func_node = AstNode {
        kind: "function_item".to_string(),
        name: Some("test_fn".to_string()),
        span: mcb_validate::ast::Span {
            start: mcb_validate::ast::Position {
                line: 1,
                column: 1,
                byte_offset: 0,
            },
            end: mcb_validate::ast::Position {
                line: 3,
                column: 1,
                byte_offset: 50,
            },
        },
        children: vec![block_node],
        metadata: std::collections::HashMap::new(),
    };

    let violations = query.execute(&func_node);
    assert_eq!(
        violations.len(),
        1,
        "Should match function with block child"
    );
}

#[test]
fn test_query_condition_no_child() {
    let query = AstQueryBuilder::new("rust", "function_item")
        .with_condition(QueryCondition::NoChild {
            child_type: "unsafe_block".to_string(),
        })
        .message("Safe function")
        .severity("info")
        .build();

    let func_node = AstNode {
        kind: "function_item".to_string(),
        name: Some("safe_fn".to_string()),
        span: mcb_validate::ast::Span {
            start: mcb_validate::ast::Position {
                line: 1,
                column: 1,
                byte_offset: 0,
            },
            end: mcb_validate::ast::Position {
                line: 3,
                column: 1,
                byte_offset: 50,
            },
        },
        children: vec![],
        metadata: std::collections::HashMap::new(),
    };

    let violations = query.execute(&func_node);
    assert_eq!(
        violations.len(),
        1,
        "Should match function without unsafe block"
    );
}

// ==================== Multi-Language Query Tests ====================

#[test]
fn test_python_query_patterns() {
    let query = AstQueryPatterns::undocumented_functions("python");
    assert_eq!(query.language, "python");
    assert_eq!(query.node_type, "function_definition");
}

#[test]
fn test_javascript_query_patterns() {
    let query = AstQueryPatterns::undocumented_functions("javascript");
    assert_eq!(query.language, "javascript");
    assert_eq!(query.node_type, "function_declaration");
}

#[test]
fn test_go_query_patterns() {
    let query = AstQueryPatterns::undocumented_functions("go");
    assert_eq!(query.language, "go");
    assert_eq!(query.node_type, "function_declaration");
}

// ==================== End-to-End Tests ====================

#[test]
fn test_parse_and_query_rust_code() {
    let mut parser = RustParser::new();
    let code = r"
fn documented_function() {
    // This is documented
}

fn undocumented() {
}
";

    let result = parser.parse_content(code, "test.rs").expect("Should parse");
    assert_eq!(result.root.kind, "source_file");

    // Verify we can query the AST
    let query = AstQuery::new("rust", "function_item", "Found function", "info");
    let violations = query.execute(&result.root);

    // Should find the function_item nodes
    assert!(!violations.is_empty(), "Should find at least one function");
}

#[test]
fn test_ast_engine_get_parser_and_parse() {
    let engine = AstEngine::new();

    // Get the Rust parser
    if let Some(parser_arc) = engine.get_parser("rust") {
        let mut parser = parser_arc.lock().expect("Should lock parser");
        let code = "fn main() {}";
        let result = parser.parse_content(code, "main.rs").expect("Should parse");
        assert_eq!(result.root.kind, "source_file");
    } else {
        panic!("Should have Rust parser");
    }
}

#[test]
fn test_ast_engine_register_query() {
    let mut engine = AstEngine::new();

    let query = AstQueryPatterns::unwrap_usage("rust");
    engine.register_query("QUAL001".to_string(), query);

    // Engine should now have the query registered
    // (internal state, but we verify no panic occurs)
}
