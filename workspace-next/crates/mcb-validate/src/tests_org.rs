//! Test Organization Validation
//!
//! Validates test organization:
//! - No inline test modules in src/ (should be in tests/)
//! - Test file naming conventions
//! - Test function naming conventions

use crate::{Result, Severity};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use walkdir::WalkDir;

/// Test organization violation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestViolation {
    /// Inline test module found in src/
    InlineTestModule {
        file: PathBuf,
        line: usize,
        severity: Severity,
    },
    /// Test file with incorrect naming
    BadTestFileName {
        file: PathBuf,
        suggestion: String,
        severity: Severity,
    },
    /// Test function with incorrect naming
    BadTestFunctionName {
        file: PathBuf,
        line: usize,
        function_name: String,
        suggestion: String,
        severity: Severity,
    },
    /// Test without assertion
    TestWithoutAssertion {
        file: PathBuf,
        line: usize,
        function_name: String,
        severity: Severity,
    },
}

impl TestViolation {
    pub fn severity(&self) -> Severity {
        match self {
            Self::InlineTestModule { severity, .. } => *severity,
            Self::BadTestFileName { severity, .. } => *severity,
            Self::BadTestFunctionName { severity, .. } => *severity,
            Self::TestWithoutAssertion { severity, .. } => *severity,
        }
    }
}

impl std::fmt::Display for TestViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InlineTestModule { file, line, .. } => {
                write!(
                    f,
                    "Inline test module: {}:{} - move to tests/ directory",
                    file.display(),
                    line
                )
            }
            Self::BadTestFileName { file, suggestion, .. } => {
                write!(
                    f,
                    "Bad test file name: {} (use {})",
                    file.display(),
                    suggestion
                )
            }
            Self::BadTestFunctionName {
                file,
                line,
                function_name,
                suggestion,
                ..
            } => {
                write!(
                    f,
                    "Bad test function name: {}:{} - {} (use {})",
                    file.display(),
                    line,
                    function_name,
                    suggestion
                )
            }
            Self::TestWithoutAssertion {
                file,
                line,
                function_name,
                ..
            } => {
                write!(
                    f,
                    "Test without assertion: {}:{} - {}",
                    file.display(),
                    line,
                    function_name
                )
            }
        }
    }
}

/// Test organization validator
pub struct TestValidator {
    workspace_root: PathBuf,
}

impl TestValidator {
    /// Create a new test validator
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self {
            workspace_root: workspace_root.into(),
        }
    }

    /// Run all test organization validations
    pub fn validate_all(&self) -> Result<Vec<TestViolation>> {
        let mut violations = Vec::new();
        violations.extend(self.validate_no_inline_tests()?);
        violations.extend(self.validate_test_naming()?);
        violations.extend(self.validate_test_function_naming()?);
        Ok(violations)
    }

    /// Verify no #[cfg(test)] mod tests {} in src/
    pub fn validate_no_inline_tests(&self) -> Result<Vec<TestViolation>> {
        let mut violations = Vec::new();
        let cfg_test_pattern = Regex::new(r"#\[cfg\(test\)\]").expect("Invalid regex");
        let mod_tests_pattern = Regex::new(r"mod\s+tests\s*\{").expect("Invalid regex");

        for crate_dir in self.get_crate_dirs()? {
            let src_dir = crate_dir.join("src");
            if !src_dir.exists() {
                continue;
            }

            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                let content = std::fs::read_to_string(entry.path())?;
                let lines: Vec<&str> = content.lines().collect();

                for (line_num, line) in lines.iter().enumerate() {
                    // Check for #[cfg(test)] followed by mod tests
                    if cfg_test_pattern.is_match(line) {
                        // Look ahead for mod tests
                        let lookahead = lines
                            .iter()
                            .skip(line_num)
                            .take(5)
                            .copied()
                            .collect::<Vec<_>>()
                            .join("\n");

                        if mod_tests_pattern.is_match(&lookahead) {
                            violations.push(TestViolation::InlineTestModule {
                                file: entry.path().to_path_buf(),
                                line: line_num + 1,
                                severity: Severity::Warning,
                            });
                        }
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Check test file naming conventions
    pub fn validate_test_naming(&self) -> Result<Vec<TestViolation>> {
        let mut violations = Vec::new();

        for crate_dir in self.get_crate_dirs()? {
            let tests_dir = crate_dir.join("tests");
            if !tests_dir.exists() {
                continue;
            }

            for entry in WalkDir::new(&tests_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                let file_name = entry
                    .path()
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("");

                // Skip lib.rs and mod.rs
                if file_name == "lib" || file_name == "mod" {
                    continue;
                }

                // Skip test utility files (mocks, fixtures, helpers)
                let path_str = entry.path().to_string_lossy();
                if path_str.contains("test_utils")
                    || file_name.contains("mock")
                    || file_name.contains("fixture")
                    || file_name.contains("helper")
                {
                    continue;
                }

                // Test files should end with _test or _tests, or use integration test patterns
                let is_valid_test_name = file_name.ends_with("_test")
                    || file_name.ends_with("_tests")
                    || file_name.contains("integration")
                    || file_name.contains("workflow")
                    || file_name.contains("e2e")
                    || file_name.contains("benchmark");

                if !is_valid_test_name {
                    violations.push(TestViolation::BadTestFileName {
                        file: entry.path().to_path_buf(),
                        suggestion: format!("{}_test.rs or {}_tests.rs", file_name, file_name),
                        severity: Severity::Info,
                    });
                }
            }
        }

        Ok(violations)
    }

    /// Verify test functions follow naming pattern
    pub fn validate_test_function_naming(&self) -> Result<Vec<TestViolation>> {
        let mut violations = Vec::new();
        let test_attr_pattern = Regex::new(r"#\[test\]").expect("Invalid regex");
        let tokio_test_pattern = Regex::new(r"#\[tokio::test\]").expect("Invalid regex");
        let fn_pattern = Regex::new(r"(?:async\s+)?fn\s+([a-z_][a-z0-9_]*)\s*\(").expect("Invalid regex");
        // Standard assertions plus implicit assertions
        let assert_pattern = Regex::new(
            r"assert!|assert_eq!|assert_ne!|panic!|should_panic|\.unwrap\(|\.expect\(|Box<dyn\s|type_name::",
        )
        .expect("Invalid regex");

        // Smoke test patterns - these verify compilation, not runtime behavior
        let smoke_test_patterns = [
            "_trait_object",  // Tests that verify trait object construction compiles
            "_exists",        // Tests that verify types exist
            "_creation",      // Constructor tests with implicit unwrap assertions
            "_compiles",      // Explicit compile-time tests
            "_factory",       // Factory pattern tests (often smoke tests)
        ];

        for crate_dir in self.get_crate_dirs()? {
            let tests_dir = crate_dir.join("tests");
            if !tests_dir.exists() {
                continue;
            }

            for entry in WalkDir::new(&tests_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                let content = std::fs::read_to_string(entry.path())?;
                let lines: Vec<&str> = content.lines().collect();

                let mut i = 0;
                while i < lines.len() {
                    let line = lines[i];

                    // Check for #[test] or #[tokio::test]
                    if test_attr_pattern.is_match(line) || tokio_test_pattern.is_match(line) {
                        // Find the function definition
                        let mut fn_line_idx = i + 1;
                        while fn_line_idx < lines.len() {
                            let potential_fn = lines[fn_line_idx];
                            if let Some(cap) = fn_pattern.captures(potential_fn) {
                                let fn_name = cap.get(1).map(|m| m.as_str()).unwrap_or("");

                                // Check naming convention
                                if !fn_name.starts_with("test_") && !fn_name.starts_with("it_") {
                                    violations.push(TestViolation::BadTestFunctionName {
                                        file: entry.path().to_path_buf(),
                                        line: fn_line_idx + 1,
                                        function_name: fn_name.to_string(),
                                        suggestion: format!("test_{}", fn_name),
                                        severity: Severity::Info,
                                    });
                                }

                                // Check for assertions in the function body
                                let mut has_assertion = false;
                                let mut brace_depth = 0;
                                let mut started = false;

                                for check_line in &lines[fn_line_idx..] {
                                    if check_line.contains('{') {
                                        started = true;
                                    }
                                    if started {
                                        brace_depth +=
                                            check_line.chars().filter(|c| *c == '{').count();
                                        brace_depth -=
                                            check_line.chars().filter(|c| *c == '}').count();

                                        if assert_pattern.is_match(check_line) {
                                            has_assertion = true;
                                            break;
                                        }

                                        if brace_depth == 0 {
                                            break;
                                        }
                                    }
                                }

                                // Skip smoke tests - they verify compilation, not behavior
                                let is_smoke_test = smoke_test_patterns
                                    .iter()
                                    .any(|pattern| fn_name.ends_with(pattern));

                                if !has_assertion && !is_smoke_test {
                                    violations.push(TestViolation::TestWithoutAssertion {
                                        file: entry.path().to_path_buf(),
                                        line: fn_line_idx + 1,
                                        function_name: fn_name.to_string(),
                                        severity: Severity::Warning,
                                    });
                                }

                                break;
                            }
                            fn_line_idx += 1;
                        }
                    }
                    i += 1;
                }
            }
        }

        Ok(violations)
    }

    fn get_crate_dirs(&self) -> Result<Vec<PathBuf>> {
        let crates_dir = self.workspace_root.join("crates");
        if !crates_dir.exists() {
            return Ok(Vec::new());
        }

        let mut dirs = Vec::new();
        for entry in std::fs::read_dir(&crates_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if name_str != "mcb-validate" {
                    dirs.push(entry.path());
                }
            }
        }
        Ok(dirs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_crate_with_tests(temp: &TempDir, name: &str, src_content: &str, test_content: &str) {
        let crate_dir = temp.path().join("crates").join(name);
        let src_dir = crate_dir.join("src");
        let tests_dir = crate_dir.join("tests");

        fs::create_dir_all(&src_dir).unwrap();
        fs::create_dir_all(&tests_dir).unwrap();

        fs::write(src_dir.join("lib.rs"), src_content).unwrap();
        fs::write(tests_dir.join("integration_test.rs"), test_content).unwrap();

        fs::write(
            crate_dir.join("Cargo.toml"),
            format!(
                r#"
[package]
name = "{}"
version = "0.1.0"
"#,
                name
            ),
        )
        .unwrap();
    }

    #[test]
    fn test_inline_test_detection() {
        let temp = TempDir::new().unwrap();
        let crate_dir = temp.path().join("crates").join("mcb-test").join("src");
        fs::create_dir_all(&crate_dir).unwrap();

        fs::write(
            crate_dir.join("lib.rs"),
            r#"
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(1, 2), 3);
    }
}
"#,
        )
        .unwrap();

        fs::write(
            temp.path().join("crates").join("mcb-test").join("Cargo.toml"),
            r#"
[package]
name = "mcb-test"
version = "0.1.0"
"#,
        )
        .unwrap();

        let validator = TestValidator::new(temp.path());
        let violations = validator.validate_no_inline_tests().unwrap();

        assert_eq!(violations.len(), 1);
        match &violations[0] {
            TestViolation::InlineTestModule { .. } => {}
            _ => panic!("Expected InlineTestModule"),
        }
    }

    #[test]
    fn test_function_naming_validation() {
        let temp = TempDir::new().unwrap();
        create_test_crate_with_tests(
            &temp,
            "mcb-test",
            "pub fn add(a: i32, b: i32) -> i32 { a + b }",
            r#"
#[test]
fn bad_name() {
    assert!(true);
}

#[test]
fn test_good_name() {
    assert!(true);
}
"#,
        );

        let validator = TestValidator::new(temp.path());
        let violations = validator.validate_test_function_naming().unwrap();

        // Should find 1 bad name
        let bad_names: Vec<_> = violations
            .iter()
            .filter(|v| matches!(v, TestViolation::BadTestFunctionName { .. }))
            .collect();
        assert_eq!(bad_names.len(), 1);
    }
}
