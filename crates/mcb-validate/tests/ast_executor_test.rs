//! Tests for AST Query Executor (Phase 2)
//!
//! Integration tests for YAML rule → Tree-sitter → violations pipeline.

#[cfg(test)]
mod ast_executor_tests {
    use mcb_validate::ast::AstQueryExecutor;
    use mcb_validate::rules::yaml_loader::{AstSelector, ValidatedRule};
    use tempfile::TempDir;

    fn create_test_rule(selectors: Vec<AstSelector>, ast_query: Option<String>) -> ValidatedRule {
        ValidatedRule {
            id: "TEST001".to_string(),
            name: "Test Rule".to_string(),
            category: "quality".to_string(),
            severity: "error".to_string(),
            enabled: true,
            description: "Test rule for AST query executor".to_string(),
            rationale: "Testing AST-based detection".to_string(),
            engine: "ast-query".to_string(),
            config: serde_json::json!({}),
            rule_definition: serde_json::json!({}),
            fixes: vec![],
            lint_select: vec![],
            message: Some("Test violation detected".to_string()),
            selectors,
            ast_query,
            metrics: None,
        }
    }

    #[tokio::test]
    async fn test_execute_rust_unwrap_selector() {
        let temp_dir = TempDir::new().unwrap();
        let rust_file = temp_dir.path().join("test.rs");

        // Create a Rust file with unwrap calls
        std::fs::write(
            &rust_file,
            r#"
fn main() {
    let x: Option<i32> = Some(42);
    let y = x.unwrap();  // This should be detected

    let z = Some(1).unwrap();  // This should be detected too
}
"#,
        )
        .unwrap();

        let selector = AstSelector {
            language: "rust".to_string(),
            node_type: "call_expression".to_string(),
            pattern: Some(
                "(call_expression function: (field_expression field: (field_identifier) @method) (#eq? @method \"unwrap\")) @match"
                    .to_string(),
            ),
        };

        let rule = create_test_rule(vec![selector], None);
        let files = vec![rust_file.as_path()];

        let violations = AstQueryExecutor::execute_rule(&rule, &files).await.unwrap();

        // Should find 2 unwrap calls
        assert!(
            violations.len() >= 2,
            "Expected at least 2 violations, found {}",
            violations.len()
        );
        assert!(violations.iter().all(|v| v.rule_id == "TEST001"));
        assert!(violations.iter().all(|v| v.severity == "error"));
    }

    #[tokio::test]
    async fn test_execute_rust_ast_query_directly() {
        let temp_dir = TempDir::new().unwrap();
        let rust_file = temp_dir.path().join("test.rs");

        // Create a Rust file with function definitions
        std::fs::write(
            &rust_file,
            r#"
fn hello() {
    println!("Hello");
}

fn world() {
    println!("World");
}
"#,
        )
        .unwrap();

        // Use ast_query field directly to find all function definitions
        let rule = create_test_rule(
            vec![AstSelector {
                language: "rust".to_string(),
                node_type: "function_item".to_string(),
                pattern: None, // Will use default pattern
            }],
            None,
        );
        let files = vec![rust_file.as_path()];

        let violations = AstQueryExecutor::execute_rule(&rule, &files).await.unwrap();

        // Should find 2 function definitions
        assert_eq!(
            violations.len(),
            2,
            "Expected 2 functions, found {}",
            violations.len()
        );
    }

    #[tokio::test]
    async fn test_execute_python_file() {
        let temp_dir = TempDir::new().unwrap();
        let python_file = temp_dir.path().join("test.py");

        // Create a Python file with function definitions
        std::fs::write(
            &python_file,
            r#"
def greet(name):
    print(f"Hello, {name}!")

def farewell(name):
    print(f"Goodbye, {name}!")

class MyClass:
    def method(self):
        pass
"#,
        )
        .unwrap();

        let selector = AstSelector {
            language: "python".to_string(),
            node_type: "function_definition".to_string(),
            pattern: None, // Default pattern matches all function_definition nodes
        };

        let rule = create_test_rule(vec![selector], None);
        let files = vec![python_file.as_path()];

        let violations = AstQueryExecutor::execute_rule(&rule, &files).await.unwrap();

        // Should find 3 function definitions (greet, farewell, method)
        assert_eq!(
            violations.len(),
            3,
            "Expected 3 function definitions, found {}",
            violations.len()
        );
    }

    #[tokio::test]
    async fn test_disabled_rule_returns_no_violations() {
        let temp_dir = TempDir::new().unwrap();
        let rust_file = temp_dir.path().join("test.rs");

        std::fs::write(&rust_file, "fn main() { let x = Some(1).unwrap(); }").unwrap();

        let selector = AstSelector {
            language: "rust".to_string(),
            node_type: "call_expression".to_string(),
            pattern: Some(
                "(call_expression function: (field_expression field: (field_identifier) @method) (#eq? @method \"unwrap\")) @match"
                    .to_string(),
            ),
        };

        let mut rule = create_test_rule(vec![selector], None);
        rule.enabled = false; // Disable the rule

        let files = vec![rust_file.as_path()];
        let violations = AstQueryExecutor::execute_rule(&rule, &files).await.unwrap();

        assert!(
            violations.is_empty(),
            "Disabled rule should return no violations"
        );
    }

    #[tokio::test]
    async fn test_no_selectors_returns_no_violations() {
        let temp_dir = TempDir::new().unwrap();
        let rust_file = temp_dir.path().join("test.rs");

        std::fs::write(&rust_file, "fn main() { let x = Some(1).unwrap(); }").unwrap();

        let rule = create_test_rule(vec![], None); // No selectors
        let files = vec![rust_file.as_path()];

        let violations = AstQueryExecutor::execute_rule(&rule, &files).await.unwrap();

        assert!(
            violations.is_empty(),
            "Rule with no selectors should return no violations"
        );
    }

    #[tokio::test]
    async fn test_language_mismatch_skips_file() {
        let temp_dir = TempDir::new().unwrap();
        let rust_file = temp_dir.path().join("test.rs");

        std::fs::write(&rust_file, "fn main() { let x = Some(1).unwrap(); }").unwrap();

        // Python selector should not match Rust file
        let selector = AstSelector {
            language: "python".to_string(),
            node_type: "function_definition".to_string(),
            pattern: None,
        };

        let rule = create_test_rule(vec![selector], None);
        let files = vec![rust_file.as_path()];

        let violations = AstQueryExecutor::execute_rule(&rule, &files).await.unwrap();

        assert!(
            violations.is_empty(),
            "Python selector should not match Rust file"
        );
    }

    #[tokio::test]
    async fn test_violation_has_correct_location() {
        let temp_dir = TempDir::new().unwrap();
        let rust_file = temp_dir.path().join("test.rs");

        std::fs::write(
            &rust_file,
            r#"fn main() {
    let x = Some(1).unwrap();
}"#,
        )
        .unwrap();

        let selector = AstSelector {
            language: "rust".to_string(),
            node_type: "call_expression".to_string(),
            pattern: Some(
                "(call_expression function: (field_expression field: (field_identifier) @method) (#eq? @method \"unwrap\")) @match"
                    .to_string(),
            ),
        };

        let rule = create_test_rule(vec![selector], None);
        let files = vec![rust_file.as_path()];

        let violations = AstQueryExecutor::execute_rule(&rule, &files).await.unwrap();

        assert_eq!(violations.len(), 1);
        let v = &violations[0];
        assert_eq!(v.line, 2, "Expected line 2, got {}", v.line);
        assert!(v.column > 0, "Column should be positive");
        assert!(!v.context.is_empty(), "Context should not be empty");
    }

    #[tokio::test]
    async fn test_multiple_files() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("file1.rs");
        let file2 = temp_dir.path().join("file2.rs");

        std::fs::write(&file1, "fn a() { Some(1).unwrap(); }").unwrap();
        std::fs::write(&file2, "fn b() { Some(2).unwrap(); Some(3).unwrap(); }").unwrap();

        let selector = AstSelector {
            language: "rust".to_string(),
            node_type: "call_expression".to_string(),
            pattern: Some(
                "(call_expression function: (field_expression field: (field_identifier) @method) (#eq? @method \"unwrap\")) @match"
                    .to_string(),
            ),
        };

        let rule = create_test_rule(vec![selector], None);
        let files = vec![file1.as_path(), file2.as_path()];

        let violations = AstQueryExecutor::execute_rule(&rule, &files).await.unwrap();

        // 1 in file1 + 2 in file2 = 3 total
        assert_eq!(violations.len(), 3, "Expected 3 violations across 2 files");
    }
}
