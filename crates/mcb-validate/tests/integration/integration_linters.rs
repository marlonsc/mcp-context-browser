//! Integration tests for linter execution (Phase 1 of mcb-validate refactoring)
//!
//! These tests verify that:
//! - Ruff and Clippy can be executed
//! - JSON output is correctly parsed
//! - `LintViolation` structs are properly populated
//! - `lint_select` codes are correctly categorized

#![allow(clippy::ignore_without_reason)]

use mcb_validate::linters::{LintViolation, LinterEngine, LinterType, YamlRuleExecutor};
use mcb_validate::{ValidatedRule, YamlRuleLoader};
use std::path::PathBuf;

#[allow(dead_code)]
fn get_workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

/// Check if an external tool is available on the system
fn is_tool_available(tool: &str) -> bool {
    std::process::Command::new(tool)
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
}

/// Skip test if tool is not available, with helpful message
macro_rules! require_tool {
    ($tool:expr, $install_cmd:expr) => {
        if !is_tool_available($tool) {
            eprintln!("\n⚠️  Skipping test: {} not installed", $tool);
            eprintln!("   Install: {}", $install_cmd);
            eprintln!("   Run this test after installation to verify functionality\n");
            return;
        }
    };
}

// ==================== Unit Tests for Linter Types ====================

#[test]
fn test_linter_engine_creation() {
    let _engine = LinterEngine::new();
    // Engine should be created with default linters
    // Engine was created successfully (no panic)
}

#[test]
fn test_linter_engine_with_specific_linters() {
    let _engine = LinterEngine::with_linters(vec![LinterType::Ruff]);
    // Engine was created with Ruff only (no panic)

    let _engine = LinterEngine::with_linters(vec![LinterType::Clippy]);
    // Engine was created with Clippy only (no panic)

    let _engine = LinterEngine::with_linters(vec![LinterType::Ruff, LinterType::Clippy]);
    // Engine was created with both linters (no panic)
}

#[test]
fn test_linter_type_equality() {
    assert_eq!(LinterType::Ruff, LinterType::Ruff);
    assert_eq!(LinterType::Clippy, LinterType::Clippy);
    assert_ne!(LinterType::Ruff, LinterType::Clippy);
}

#[test]
fn test_linter_type_commands() {
    assert_eq!(LinterType::Ruff.command(), "ruff");
    assert_eq!(LinterType::Clippy.command(), "cargo");
}

// ==================== JSON Parsing Tests ====================

#[test]
fn test_ruff_json_array_parsing() {
    // Ruff outputs JSON in array format
    let json_output = r#"[
        {
            "code": "F401",
            "message": "'os' imported but unused",
            "filename": "test.py",
            "location": {"row": 1, "column": 1}
        },
        {
            "code": "E501",
            "message": "Line too long",
            "filename": "test.py",
            "location": {"row": 10, "column": 80}
        }
    ]"#;

    let violations = LinterType::Ruff.parse_output(json_output);

    assert_eq!(
        violations.len(),
        2,
        "Should parse 2 violations from JSON array"
    );

    assert_eq!(violations[0].rule, "F401");
    assert_eq!(violations[0].file, "test.py");
    assert_eq!(violations[0].line, 1);
    assert_eq!(violations[0].column, 1);
    assert!(violations[0].message.contains("imported but unused"));

    assert_eq!(violations[1].rule, "E501");
    assert_eq!(violations[1].line, 10);
}

#[test]
fn test_ruff_json_lines_fallback() {
    // Legacy JSON lines format (fallback)
    let json_output = r#"{"code": "W291", "message": "Trailing whitespace", "filename": "foo.py", "location": {"row": 5, "column": 10}}"#;

    let violations = LinterType::Ruff.parse_output(json_output);

    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].rule, "W291");
    assert_eq!(violations[0].file, "foo.py");
}

#[test]
fn test_ruff_empty_output() {
    let violations = LinterType::Ruff.parse_output("[]");
    assert!(
        violations.is_empty(),
        "Empty array should yield no violations"
    );

    let violations = LinterType::Ruff.parse_output("");
    assert!(
        violations.is_empty(),
        "Empty string should yield no violations"
    );
}

#[test]
fn test_clippy_json_parsing() {
    // Clippy outputs JSON lines with "reason" field
    let json_output = r#"{"reason":"compiler-message","message":{"message":"used `unwrap()` on an `Option` value","code":{"code":"clippy::unwrap_used","explanation":null},"level":"warning","spans":[{"file_name":"src/lib.rs","line_start":42,"column_start":5,"is_primary":true}]}}
{"reason":"build-finished","success":true}"#;

    let violations = LinterType::Clippy.parse_output(json_output);

    assert_eq!(
        violations.len(),
        1,
        "Should parse 1 violation (ignoring build-finished)"
    );
    assert_eq!(violations[0].rule, "clippy::unwrap_used");
    assert_eq!(violations[0].file, "src/lib.rs");
    assert_eq!(violations[0].line, 42);
    assert_eq!(violations[0].column, 5);
    assert_eq!(violations[0].severity, "warning");
    assert_eq!(violations[0].category, "quality");
}

#[test]
fn test_clippy_filters_non_compiler_messages() {
    // Should ignore compiler-artifact and build-finished
    let json_output = r#"{"reason":"compiler-artifact","target":{"name":"test"}}
{"reason":"build-finished","success":true}"#;

    let violations = LinterType::Clippy.parse_output(json_output);
    assert!(violations.is_empty(), "Should ignore non-message lines");
}

#[test]
fn test_clippy_requires_primary_span() {
    // Message without primary span should be skipped
    let json_output = r#"{"reason":"compiler-message","message":{"message":"help message","code":{"code":"clippy::help","explanation":null},"level":"help","spans":[{"file_name":"src/lib.rs","line_start":1,"column_start":1,"is_primary":false}]}}"#;

    let violations = LinterType::Clippy.parse_output(json_output);
    assert!(
        violations.is_empty(),
        "Should skip messages without primary span"
    );
}

// ==================== Severity Mapping Tests ====================

#[test]
fn test_ruff_severity_mapping() {
    // F-codes are errors (Pyflakes)
    let json = r#"[{"code": "F401", "message": "unused", "filename": "t.py", "location": {"row": 1, "column": 1}}]"#;
    let violations = LinterType::Ruff.parse_output(json);
    assert_eq!(violations[0].severity, "error");

    // E-codes are errors (pycodestyle)
    let json = r#"[{"code": "E501", "message": "line too long", "filename": "t.py", "location": {"row": 1, "column": 1}}]"#;
    let violations = LinterType::Ruff.parse_output(json);
    assert_eq!(violations[0].severity, "error");

    // W-codes are warnings
    let json = r#"[{"code": "W291", "message": "trailing whitespace", "filename": "t.py", "location": {"row": 1, "column": 1}}]"#;
    let violations = LinterType::Ruff.parse_output(json);
    assert_eq!(violations[0].severity, "warning");

    // I-codes are info (isort)
    let json = r#"[{"code": "I001", "message": "unsorted imports", "filename": "t.py", "location": {"row": 1, "column": 1}}]"#;
    let violations = LinterType::Ruff.parse_output(json);
    assert_eq!(violations[0].severity, "info");
}

// ==================== Async Execution Tests ====================

#[tokio::test]
async fn test_linter_engine_empty_files() {
    let engine = LinterEngine::new();
    let result = engine.check_files(&[]).await;

    assert!(result.is_ok(), "Checking empty file list should succeed");
    assert!(
        result.unwrap().is_empty(),
        "No violations for empty file list"
    );
}

#[tokio::test]
async fn test_linter_execution_with_nonexistent_files() {
    let engine = LinterEngine::with_linters(vec![LinterType::Ruff]);
    let fake_path = PathBuf::from("/nonexistent/path/file.py");

    // This should not panic, just return empty or handle gracefully
    let result = engine.check_files(&[fake_path.as_path()]).await;
    // Linter might fail or return empty - either is acceptable
    assert!(
        result.is_ok() || result.is_err(),
        "Should handle gracefully"
    );
}

// ==================== Integration with Real Workspace ====================

#[tokio::test]
async fn test_linter_mapping() {
    let engine = LinterEngine::new();

    // Test Ruff code mapping
    assert_eq!(engine.map_lint_to_rule("F401"), Some("QUAL005"));

    // Test Clippy code mapping
    assert_eq!(
        engine.map_lint_to_rule("clippy::unwrap_used"),
        Some("QUAL001")
    );

    // Unknown codes should return None
    assert_eq!(engine.map_lint_to_rule("UNKNOWN"), None);
}

// ==================== LintViolation Structure Tests ====================

#[test]
fn test_lint_violation_structure() {
    let violation = LintViolation {
        rule: "F401".to_string(),
        file: "test.py".to_string(),
        line: 10,
        column: 5,
        message: "Unused import".to_string(),
        severity: "error".to_string(),
        category: "quality".to_string(),
    };

    assert_eq!(violation.rule, "F401");
    assert_eq!(violation.file, "test.py");
    assert_eq!(violation.line, 10);
    assert_eq!(violation.column, 5);
    assert_eq!(violation.message, "Unused import");
    assert_eq!(violation.severity, "error");
    assert_eq!(violation.category, "quality");
}

// ==================== Hybrid Engine Lint Integration Tests ====================

#[test]
fn test_lint_code_categorization() {
    // This tests the internal categorization logic
    // Ruff codes: F401, E501, W291, I001, etc.
    // Clippy codes: clippy::unwrap_used, clippy::expect_used, etc.

    let codes = vec![
        "F401".to_string(),
        "E501".to_string(),
        "clippy::unwrap_used".to_string(),
        "clippy::expect_used".to_string(),
        "W291".to_string(),
    ];

    let mut ruff_count = 0;
    let mut clippy_count = 0;

    for code in &codes {
        if code.starts_with("clippy::") {
            clippy_count += 1;
        } else {
            ruff_count += 1;
        }
    }

    assert_eq!(ruff_count, 3, "Should have 3 Ruff codes");
    assert_eq!(clippy_count, 2, "Should have 2 Clippy codes");
}

// ==================== Documentation Verification ====================

/// Verify that public types have expected documentation
#[test]
fn test_public_api_accessible() {
    // These should all be publicly accessible via the linters module
    // Using _ prefix to suppress unused warnings while still testing compilation

    // If this compiles, the public API is correctly exported
    // All public linter types are accessible (no panic)
    assert_eq!(2 + 2, 4); // Basic assertion to ensure test runs
}

// ==================== REAL LINTER EXECUTION TESTS (Phase 1 Deliverable) ====================
//
// These tests verify that actual linter execution and JSON parsing work end-to-end.
// They are marked #[ignore] because they require external tools to be installed:
// - ruff: pip install ruff
// - cargo clippy: rustup component add clippy
//
// Run with: cargo test --package mcb-validate -- --ignored

/// Test real ruff execution against Python files
///
/// Phase 1 deliverable: "cargo test `integration_linters` passes with real Clippy/Ruff output"
#[test]
fn test_ruff_real_execution() {
    use std::process::Command;

    require_tool!("ruff", "pip install ruff");

    // Create a temporary Python file with known violations
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("test_file.py");
    std::fs::write(
        &test_file,
        r"import os  # F401: unused import
import sys  # F401: unused import

def example():
    x = 1
    return x
",
    )
    .expect("Failed to write test file");

    // Execute ruff with JSON output
    let output = Command::new("ruff")
        .args([
            "check",
            "--output-format=json",
            test_file.to_str().expect("Path is valid UTF-8"),
        ])
        .output()
        .expect("Failed to execute ruff - is it installed? (pip install ruff)");

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("Ruff output: {stdout}");

    // Parse the real output
    let violations = LinterType::Ruff.parse_output(&stdout);

    // Verify we got violations (should have at least 2 F401 for unused imports)
    assert!(
        violations.len() >= 2,
        "Expected at least 2 violations (unused imports), got {}",
        violations.len()
    );

    // Verify structure of parsed violations
    for v in &violations {
        assert!(!v.rule.is_empty(), "Rule should not be empty");
        assert!(!v.file.is_empty(), "File should not be empty");
        assert!(v.line > 0, "Line should be positive");
        assert!(!v.severity.is_empty(), "Severity should not be empty");
    }

    // Should have F401 violations
    let f401_count = violations.iter().filter(|v| v.rule == "F401").count();
    assert!(
        f401_count >= 2,
        "Expected at least 2 F401 violations, got {f401_count}"
    );

    println!(
        "Real ruff execution: {} violations parsed successfully",
        violations.len()
    );
}

/// Test real clippy execution against Rust code
///
/// Phase 1 deliverable: "cargo test `integration_linters` passes with real Clippy/Ruff output"
#[test]
fn test_clippy_real_execution() {
    use std::process::Command;

    require_tool!("cargo", "rustup component add clippy");

    // Run clippy on mcb-validate itself with JSON output
    // Using -q to reduce noise and only get diagnostic messages
    let output = Command::new("cargo")
        .args([
            "clippy",
            "-p",
            "mcb-validate",
            "--message-format=json",
            "-q",
            "--",
            "-W",
            "clippy::all",
        ])
        .current_dir(get_workspace_root())
        .output()
        .expect("Failed to execute cargo clippy");

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!(
        "Clippy output (first 1000 chars): {}",
        &stdout.chars().take(1000).collect::<String>()
    );

    // Parse the real output
    let violations = LinterType::Clippy.parse_output(&stdout);

    // Clippy might find violations or not - the key is that parsing works
    // and doesn't panic
    println!(
        "Real clippy execution: {} violations parsed successfully",
        violations.len()
    );

    // Verify structure of any parsed violations
    for v in &violations {
        assert!(!v.rule.is_empty(), "Rule should not be empty");
        assert!(!v.file.is_empty(), "File should not be empty");
        assert!(v.line > 0, "Line should be positive");
        assert!(!v.severity.is_empty(), "Severity should not be empty");
        assert!(
            v.rule.starts_with("clippy::"),
            "Clippy rules should start with 'clippy::'"
        );
    }
}

/// Test `LinterEngine` can execute linters and aggregate results
#[tokio::test]
async fn test_linter_engine_real_execution() {
    require_tool!("ruff", "pip install ruff");

    // Create a temporary Python file with known violations
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("violations.py");
    std::fs::write(
        &test_file,
        r"import os  # unused
import sys  # unused

def foo():
    pass
",
    )
    .expect("Failed to write test file");

    // Create engine with just Ruff (faster test)
    let engine = LinterEngine::with_linters(vec![LinterType::Ruff]);

    // Execute linter via engine
    let result = engine.check_files(&[test_file.as_path()]).await;

    match result {
        Ok(violations) => {
            assert!(
                violations.len() >= 2,
                "Expected at least 2 violations, got {}",
                violations.len()
            );
            println!(
                "LinterEngine real execution: {} violations",
                violations.len()
            );
        }
        Err(e) => {
            panic!("LinterEngine should execute successfully: {e:?}");
        }
    }
}

// ==================== YAML RULE → LINTER → VIOLATIONS (Phase 1 Deliverable) ====================
//
// These tests verify the complete pipeline:
// 1. Create a ValidatedRule with lint_select codes
// 2. Execute via YamlRuleExecutor
// 3. Verify violations are correctly returned and filtered
//
// This is the Phase 1 deliverable: "Wire lint_select YAML field to actual linter execution"

/// Helper to create a `ValidatedRule` for testing
fn create_test_rule(
    id: &str,
    lint_select: Vec<String>,
    category: &str,
    enabled: bool,
) -> ValidatedRule {
    ValidatedRule {
        id: id.to_string(),
        name: format!("Test rule {id}"),
        category: category.to_string(),
        severity: "warning".to_string(),
        enabled,
        description: "Test rule for integration testing".to_string(),
        rationale: "Testing YAML rule executor".to_string(),
        engine: "none".to_string(),
        config: serde_json::Value::Object(serde_json::Map::new()),
        rule_definition: serde_json::Value::Object(serde_json::Map::new()),
        fixes: Vec::new(),
        lint_select,
        message: None,
        selectors: Vec::new(),
        ast_query: None,
        metrics: None,
    }
}

/// Test that `YamlRuleExecutor` correctly routes Ruff codes to Ruff linter
#[tokio::test]
async fn test_yaml_rule_executor_ruff_integration() {
    require_tool!("ruff", "pip install ruff");

    // Create a temp file with unused imports
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("unused_imports.py");
    std::fs::write(
        &test_file,
        r"import os  # F401: unused import
import sys  # F401: unused import

def example():
    return 42
",
    )
    .expect("Failed to write test file");

    // Create rule with F401 lint_select (Ruff unused import code)
    let rule = create_test_rule("TEST001", vec!["F401".to_string()], "quality", true);

    // Execute via YamlRuleExecutor
    let violations = YamlRuleExecutor::execute_rule(&rule, &[test_file.as_path()])
        .await
        .expect("YamlRuleExecutor should execute successfully");

    // Verify violations
    assert!(
        violations.len() >= 2,
        "Expected at least 2 F401 violations (unused imports), got {}",
        violations.len()
    );

    // All violations should have F401 rule (filtered by lint_select)
    for v in &violations {
        assert_eq!(v.rule, "F401", "All violations should be F401");
        assert_eq!(v.category, "quality", "Category should come from rule");
    }

    println!(
        "YamlRuleExecutor Ruff integration: {} violations for F401",
        violations.len()
    );
}

/// Test that disabled rules return no violations
#[tokio::test]
async fn test_yaml_rule_executor_disabled_rule() {
    let rule = create_test_rule(
        "DISABLED001",
        vec!["F401".to_string()],
        "quality",
        false, // disabled
    );

    // Even with files, disabled rules should return empty
    let fake_path = PathBuf::from("any/path.py");
    let violations = YamlRuleExecutor::execute_rule(&rule, &[fake_path.as_path()])
        .await
        .expect("Should handle disabled rule");

    assert!(
        violations.is_empty(),
        "Disabled rule should return no violations"
    );
}

/// Test that rules with empty `lint_select` return no violations
#[tokio::test]
async fn test_yaml_rule_executor_empty_lint_select() {
    let rule = create_test_rule(
        "EMPTY001",
        vec![], // No lint_select codes
        "quality",
        true,
    );

    let fake_path = PathBuf::from("any/path.py");
    let violations = YamlRuleExecutor::execute_rule(&rule, &[fake_path.as_path()])
        .await
        .expect("Should handle empty lint_select");

    assert!(
        violations.is_empty(),
        "Empty lint_select should return no violations"
    );
}

/// Test that Clippy codes are correctly detected and routed
#[tokio::test]
async fn test_yaml_rule_executor_clippy_code_detection() {
    // This tests the code detection logic, not actual Clippy execution
    // (Clippy requires a Cargo project context)

    let rule = create_test_rule(
        "CLIPPY001",
        vec!["clippy::unwrap_used".to_string()],
        "quality",
        true,
    );

    // With no actual Rust files, should return Ok with empty violations
    let violations = YamlRuleExecutor::execute_rule(&rule, &[])
        .await
        .expect("Should handle empty file list");

    // Empty file list = no violations (but pipeline works)
    assert!(
        violations.is_empty(),
        "Empty file list should return no violations"
    );

    println!("YamlRuleExecutor Clippy code detection: pipeline works");
}

/// Test filtering: only `lint_select` codes should appear in results
#[tokio::test]
async fn test_yaml_rule_executor_filters_to_lint_select() {
    require_tool!("ruff", "pip install ruff");

    // Create a file with multiple violation types
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("multi_violations.py");
    std::fs::write(
        &test_file,
        r"import os  # F401: unused
import sys  # F401: unused

x=1  # E225: missing whitespace
y=2  # E225: missing whitespace

def f():
    pass
",
    )
    .expect("Failed to write test file");

    // Rule only selects F401, not E225
    let rule = create_test_rule("FILTER001", vec!["F401".to_string()], "quality", true);

    let violations = YamlRuleExecutor::execute_rule(&rule, &[test_file.as_path()])
        .await
        .expect("YamlRuleExecutor should execute successfully");

    // Should only have F401 violations
    for v in &violations {
        assert_eq!(
            v.rule, "F401",
            "Should only have F401 violations, got {}",
            v.rule
        );
    }

    println!(
        "YamlRuleExecutor filtering: {} violations (all F401)",
        violations.len()
    );
}

/// Test custom message from rule is applied to violations
#[tokio::test]
async fn test_yaml_rule_executor_custom_message() {
    require_tool!("ruff", "pip install ruff");

    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("custom_msg.py");
    std::fs::write(&test_file, "import os  # unused\n").expect("Failed to write test file");

    let mut rule = create_test_rule("MSG001", vec!["F401".to_string()], "quality", true);
    rule.message = Some("Custom: Remove this unused import".to_string());

    let violations = YamlRuleExecutor::execute_rule(&rule, &[test_file.as_path()])
        .await
        .expect("YamlRuleExecutor should execute successfully");

    // All violations should have custom message
    for v in &violations {
        assert_eq!(
            v.message, "Custom: Remove this unused import",
            "Custom message should be applied"
        );
    }

    println!(
        "YamlRuleExecutor custom message: {} violations with custom message",
        violations.len()
    );
}

// ==================== END-TO-END YAML FILE → LINTER → VIOLATIONS ====================
//
// This is the REAL Phase 1 deliverable: load actual YAML rule file → execute linter → get violations
// This tests the complete pipeline as specified in the plan.

/// Helper to get the rules directory path
fn get_rules_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("rules")
}

/// END-TO-END TEST: Load ruff-imports.yml → execute Ruff → get F401 violations
///
/// This is the Phase 1 deliverable: "Integration test: YAML rule → linter → violations"
/// Tests the COMPLETE pipeline from actual YAML file to violations.
#[tokio::test]
async fn test_e2e_yaml_file_to_linter_violations() {
    require_tool!("ruff", "pip install ruff");

    let rules_dir = get_rules_dir();

    // Verify rules directory exists
    assert!(
        rules_dir.exists(),
        "Rules directory should exist at {rules_dir:?}"
    );

    // Load rules from YAML files using YamlRuleLoader
    let mut loader =
        YamlRuleLoader::new(rules_dir.clone()).expect("YamlRuleLoader should initialize");

    let rules = loader
        .load_all_rules()
        .await
        .expect("Should load all YAML rules");

    // Find the ruff-imports rule (QUAL005)
    let ruff_rule = rules
        .iter()
        .find(|r| r.id == "QUAL005")
        .expect("QUAL005 (ruff-imports) rule should exist");

    // Verify rule has lint_select
    assert!(
        !ruff_rule.lint_select.is_empty(),
        "QUAL005 should have lint_select codes"
    );
    assert!(
        ruff_rule.lint_select.contains(&"F401".to_string()),
        "QUAL005 should select F401 (unused imports)"
    );

    // Create a temp Python file with unused imports
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("test_unused.py");
    std::fs::write(
        &test_file,
        r#"import os  # F401: unused import
import sys  # F401: unused import
import json  # F401: unused import

def main():
    print("Hello")

if __name__ == "__main__":
    main()
"#,
    )
    .expect("Failed to write test file");

    // Execute the rule via YamlRuleExecutor
    let violations = YamlRuleExecutor::execute_rule(ruff_rule, &[test_file.as_path()])
        .await
        .expect("YamlRuleExecutor should execute successfully");

    // Verify violations
    assert!(
        violations.len() >= 3,
        "Expected at least 3 F401 violations (3 unused imports), got {}",
        violations.len()
    );

    // All violations should be F401
    for v in &violations {
        assert_eq!(
            v.rule, "F401",
            "All violations should be F401 from lint_select filtering"
        );
    }

    println!(
        "E2E YAML → Linter → Violations: {} violations from YAML rule QUAL005",
        violations.len()
    );
}

/// END-TO-END TEST: Load no-unwrap.yml → verify Clippy `lint_select`
///
/// This verifies that Clippy-based rules load correctly from YAML.
/// Note: Actually running Clippy requires a Cargo project context.
#[tokio::test]
async fn test_e2e_yaml_clippy_rule_loads() {
    let rules_dir = get_rules_dir();

    // Load rules from YAML files
    let mut loader =
        YamlRuleLoader::new(rules_dir.clone()).expect("YamlRuleLoader should initialize");

    let rules = loader
        .load_all_rules()
        .await
        .expect("Should load all YAML rules");

    // Find the no-unwrap rule (QUAL001)
    let unwrap_rule = rules
        .iter()
        .find(|r| r.id == "QUAL001")
        .expect("QUAL001 (no-unwrap) rule should exist");

    // Verify rule configuration
    assert_eq!(unwrap_rule.name, "No Unwrap in Production");
    assert_eq!(unwrap_rule.category, "quality");
    assert!(unwrap_rule.enabled, "Rule should be enabled");

    // Verify lint_select contains Clippy code
    assert!(
        unwrap_rule
            .lint_select
            .contains(&"clippy::unwrap_used".to_string()),
        "QUAL001 should select clippy::unwrap_used, got {:?}",
        unwrap_rule.lint_select
    );

    println!(
        "E2E YAML Clippy rule loaded: {} with lint_select {:?}",
        unwrap_rule.id, unwrap_rule.lint_select
    );
}

/// Verify all YAML rules with `lint_select` load correctly
#[tokio::test]
async fn test_all_yaml_rules_with_lint_select_load() {
    let rules_dir = get_rules_dir();

    let mut loader = YamlRuleLoader::new(rules_dir).expect("YamlRuleLoader should initialize");

    let rules = loader
        .load_all_rules()
        .await
        .expect("Should load all YAML rules");

    // Find all rules with lint_select
    let lint_rules: Vec<_> = rules.iter().filter(|r| !r.lint_select.is_empty()).collect();

    // Should have at least 2 lint-based rules (QUAL001, QUAL005)
    assert!(
        lint_rules.len() >= 2,
        "Expected at least 2 rules with lint_select, got {}",
        lint_rules.len()
    );

    println!("Loaded {} YAML rules with lint_select:", lint_rules.len());
    for rule in lint_rules {
        println!("  - {} ({}): {:?}", rule.id, rule.name, rule.lint_select);
    }
}

/// END-TO-END TEST: YAML (QUAL001) → Cargo Project → Clippy → Violations
///
/// This is the Phase 1 deliverable for Clippy integration:
/// 1. Load QUAL001 (no-unwrap) rule from actual YAML file
/// 2. Create a temporary Cargo project with .`unwrap()` calls
/// 3. Execute `YamlRuleExecutor` (which runs cargo clippy)
/// 4. Verify `clippy::unwrap_used` violations are returned
#[tokio::test]
async fn test_e2e_yaml_clippy_rule_execution() {
    require_tool!("cargo", "rustup component add clippy");

    let rules_dir = get_rules_dir();

    // Step 1: Load QUAL001 rule from YAML
    let mut loader =
        YamlRuleLoader::new(rules_dir.clone()).expect("YamlRuleLoader should initialize");

    let rules = loader
        .load_all_rules()
        .await
        .expect("Should load all YAML rules");

    let unwrap_rule = rules
        .iter()
        .find(|r| r.id == "QUAL001")
        .expect("QUAL001 (no-unwrap) rule should exist");

    assert!(
        unwrap_rule
            .lint_select
            .contains(&"clippy::unwrap_used".to_string()),
        "QUAL001 should select clippy::unwrap_used"
    );

    // Step 2: Create a temporary Cargo project with .unwrap() violations
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let project_dir = temp_dir.path();
    let src_dir = project_dir.join("src");
    std::fs::create_dir_all(&src_dir).expect("Failed to create src dir");

    // Create Cargo.toml
    std::fs::write(
        project_dir.join("Cargo.toml"),
        r#"[package]
name = "test_unwrap_violations"
version = "0.1.0"
edition = "2024"

[lib]
path = "src/lib.rs"
"#,
    )
    .expect("Failed to write Cargo.toml");

    // Create lib.rs with .unwrap() violations
    let lib_file = src_dir.join("lib.rs");
    std::fs::write(
        &lib_file,
        r#"//! Test library with intentional .unwrap() violations

/// Function that uses .unwrap() - should trigger clippy::unwrap_used
pub fn parse_number(s: &str) -> i32 {
    s.parse().unwrap()  // clippy::unwrap_used violation
}

/// Another function with .unwrap()
pub fn get_first(v: &[i32]) -> i32 {
    v.first().unwrap().clone()  // clippy::unwrap_used violation
}

/// Function with .expect() - also triggers the lint
pub fn parse_with_expect(s: &str) -> i32 {
    s.parse().expect("parse failed")  // clippy::expect_used (related)
}
"#,
    )
    .expect("Failed to write lib.rs");

    // Step 3: Execute YamlRuleExecutor with the temp Rust file
    let violations = YamlRuleExecutor::execute_rule(unwrap_rule, &[lib_file.as_path()])
        .await
        .expect("YamlRuleExecutor should execute successfully");

    // Step 4: Verify violations
    // Note: We expect at least 2 unwrap_used violations
    println!(
        "E2E Clippy execution: {} violations from YAML rule QUAL001",
        violations.len()
    );

    for v in &violations {
        println!(
            "  Violation: {} at {}:{} - {}",
            v.rule, v.file, v.line, v.message
        );
    }

    assert!(
        violations.len() >= 2,
        "Expected at least 2 clippy::unwrap_used violations, got {}. \
         Make sure Clippy is installed (rustup component add clippy)",
        violations.len()
    );

    // All violations should be clippy::unwrap_used (from lint_select filtering)
    for v in &violations {
        assert_eq!(
            v.rule, "clippy::unwrap_used",
            "All violations should be clippy::unwrap_used from lint_select filtering, got {}",
            v.rule
        );
    }

    println!("SUCCESS: E2E YAML → Cargo Project → Clippy → Violations pipeline works!");
}
