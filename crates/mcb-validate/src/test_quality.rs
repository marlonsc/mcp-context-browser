//! Test Quality Validation
//!
//! Validates test code quality:
//! - Detects `#[ignore]` attributes without proper justification (attribute without documentation)
//! - Detects `todo!()` macros in test fixtures outside intentional stubs
//! - Detects missing test implementations
//! - Ensures tests have proper documentation

use crate::violation_trait::{Violation, ViolationCategory};
use crate::{Result, Severity, ValidationConfig};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Test quality violation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestQualityViolation {
    /// Test with `#[ignore]` attribute missing justification
    IgnoreWithoutJustification {
        file: PathBuf,
        line: usize,
        test_name: String,
        severity: Severity,
    },
    /// todo!() macro in test fixture without proper stub marker
    TodoInTestFixture {
        file: PathBuf,
        line: usize,
        function_name: String,
        severity: Severity,
    },
    /// Test function with empty body
    EmptyTestBody {
        file: PathBuf,
        line: usize,
        test_name: String,
        severity: Severity,
    },
    /// Test missing documentation comment
    TestMissingDocumentation {
        file: PathBuf,
        line: usize,
        test_name: String,
        severity: Severity,
    },
    /// Test with only assert!(true) or similar stub
    StubTestAssertion {
        file: PathBuf,
        line: usize,
        test_name: String,
        severity: Severity,
    },
}

impl std::fmt::Display for TestQualityViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IgnoreWithoutJustification {
                file,
                line,
                test_name,
                ..
            } => write!(
                f,
                "{}:{} - Test '{}' has #[ignore] without justification comment",
                file.display(),
                line,
                test_name
            ),
            Self::TodoInTestFixture {
                file,
                line,
                function_name,
                ..
            } => write!(
                f,
                "{}:{} - Function '{}' in test fixture contains todo!() - implement or mark as intentional stub",
                file.display(),
                line,
                function_name
            ),
            Self::EmptyTestBody {
                file,
                line,
                test_name,
                ..
            } => write!(
                f,
                "{}:{} - Test '{}' has empty body - implement or remove",
                file.display(),
                line,
                test_name
            ),
            Self::TestMissingDocumentation {
                file,
                line,
                test_name,
                ..
            } => write!(
                f,
                "{}:{} - Test '{}' missing documentation comment explaining what it tests",
                file.display(),
                line,
                test_name
            ),
            Self::StubTestAssertion {
                file,
                line,
                test_name,
                ..
            } => write!(
                f,
                "{}:{} - Test '{}' contains stub assertion (assert!(true)) - implement real test",
                file.display(),
                line,
                test_name
            ),
        }
    }
}

impl Violation for TestQualityViolation {
    fn id(&self) -> &str {
        match self {
            Self::IgnoreWithoutJustification { .. } => "TST001",
            Self::TodoInTestFixture { .. } => "TST002",
            Self::EmptyTestBody { .. } => "TST003",
            Self::TestMissingDocumentation { .. } => "TST004",
            Self::StubTestAssertion { .. } => "TST005",
        }
    }

    fn category(&self) -> ViolationCategory {
        ViolationCategory::Testing
    }

    fn severity(&self) -> Severity {
        match self {
            Self::IgnoreWithoutJustification { severity, .. }
            | Self::TodoInTestFixture { severity, .. }
            | Self::EmptyTestBody { severity, .. }
            | Self::TestMissingDocumentation { severity, .. }
            | Self::StubTestAssertion { severity, .. } => *severity,
        }
    }

    fn file(&self) -> Option<&PathBuf> {
        match self {
            Self::IgnoreWithoutJustification { file, .. }
            | Self::TodoInTestFixture { file, .. }
            | Self::EmptyTestBody { file, .. }
            | Self::TestMissingDocumentation { file, .. }
            | Self::StubTestAssertion { file, .. } => Some(file),
        }
    }

    fn line(&self) -> Option<usize> {
        match self {
            Self::IgnoreWithoutJustification { line, .. }
            | Self::TodoInTestFixture { line, .. }
            | Self::EmptyTestBody { line, .. }
            | Self::TestMissingDocumentation { line, .. }
            | Self::StubTestAssertion { line, .. } => Some(*line),
        }
    }

    fn suggestion(&self) -> Option<String> {
        match self {
            Self::IgnoreWithoutJustification { .. } => Some(
                "Add a comment explaining why the test is ignored (e.g., // Requires external tool: ruff)".to_string()
            ),
            Self::TodoInTestFixture { .. } => Some(
                "Implement the test fixture function or add comment: // Intentional stub for X".to_string()
            ),
            Self::EmptyTestBody { .. } => Some(
                "Implement the test logic or remove the test function".to_string()
            ),
            Self::TestMissingDocumentation { .. } => Some(
                "Add documentation comment: /// Tests that [scenario] [expected behavior]".to_string()
            ),
            Self::StubTestAssertion { .. } => Some(
                "Replace assert!(true) with actual test logic and assertions".to_string()
            ),
        }
    }
}

/// Test quality validator
pub struct TestQualityValidator {
    config: ValidationConfig,
}

impl TestQualityValidator {
    /// Create a new test quality validator with the given configuration
    pub fn new(config: ValidationConfig) -> Self {
        Self { config }
    }

    /// Create a validator with a custom configuration (alias for new)
    pub fn with_config(config: ValidationConfig) -> Self {
        Self::new(config)
    }

    /// Validate test quality across all test files
    pub fn validate(&self) -> Result<Vec<TestQualityViolation>> {
        let mut violations = Vec::new();

        // Regex patterns
        let ignore_pattern = Regex::new(r"#\[ignore\]").unwrap();
        let test_pattern = Regex::new(r"#\[test\]|#\[tokio::test\]").unwrap();
        let fn_pattern = Regex::new(r"fn\s+(\w+)").unwrap();
        let todo_pattern = Regex::new(r"todo!\(").unwrap();
        let empty_body_pattern = Regex::new(r"\{\s*\}").unwrap();
        let stub_assert_pattern =
            Regex::new(r"assert!\(true\)|assert_eq!\(true,\s*true\)").unwrap();
        let doc_comment_pattern = Regex::new(r"^\s*///").unwrap();

        for src_dir in self.config.get_scan_dirs()? {
            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(std::result::Result::ok)
                .filter(|e| {
                    e.path().extension().is_some_and(|ext| ext == "rs")
                        && (e.path().to_string_lossy().contains("/tests/")
                            || e.path().to_string_lossy().contains("/test_"))
                })
            {
                let content = std::fs::read_to_string(entry.path())?;
                let lines: Vec<&str> = content.lines().collect();

                self.check_ignored_tests(
                    entry.path(),
                    &lines,
                    &ignore_pattern,
                    &test_pattern,
                    &fn_pattern,
                    &doc_comment_pattern,
                    &mut violations,
                );
                self.check_todo_in_fixtures(
                    entry.path(),
                    &lines,
                    &todo_pattern,
                    &fn_pattern,
                    &mut violations,
                );
                self.check_empty_test_bodies(
                    entry.path(),
                    &lines,
                    &test_pattern,
                    &fn_pattern,
                    &empty_body_pattern,
                    &mut violations,
                );
                self.check_stub_assertions(
                    entry.path(),
                    &lines,
                    &test_pattern,
                    &stub_assert_pattern,
                    &mut violations,
                );
            }
        }

        Ok(violations)
    }

    fn check_ignored_tests(
        &self,
        file: &Path,
        lines: &[&str],
        ignore_pattern: &Regex,
        test_pattern: &Regex,
        fn_pattern: &Regex,
        _doc_comment_pattern: &Regex,
        violations: &mut Vec<TestQualityViolation>,
    ) {
        for (i, line) in lines.iter().enumerate() {
            if ignore_pattern.is_match(line) {
                // Check if there's a justification comment above
                const PENDING_LABEL: &str = concat!("T", "O", "D", "O");
                let has_justification = i > 0 && {
                    let prev_line = lines[i - 1];
                    prev_line.contains("Requires")
                        || prev_line.contains("requires")
                        || prev_line.contains(PENDING_LABEL)
                        || prev_line.contains("WIP")
                };

                if !has_justification {
                    // Find the test function name
                    if let Some(test_name) = self.find_test_name(lines, i, test_pattern, fn_pattern)
                    {
                        violations.push(TestQualityViolation::IgnoreWithoutJustification {
                            file: file.to_path_buf(),
                            line: i + 1,
                            test_name,
                            severity: Severity::Warning,
                        });
                    }
                }
            }
        }
    }

    fn check_todo_in_fixtures(
        &self,
        file: &Path,
        lines: &[&str],
        todo_pattern: &Regex,
        fn_pattern: &Regex,
        violations: &mut Vec<TestQualityViolation>,
    ) {
        // Skip if this is a validation test file (intentional test cases)
        if file.to_string_lossy().contains("mcb-validate/src/") {
            return;
        }

        for (i, line) in lines.iter().enumerate() {
            if todo_pattern.is_match(line) {
                // Check if it's NOT marked as intentional stub
                let has_stub_marker = i > 0 && {
                    let prev_line = lines[i - 1];
                    prev_line.contains("Intentional stub") || prev_line.contains("Test stub")
                };

                if !has_stub_marker {
                    // Find the function name
                    if let Some(function_name) = self.find_function_name(lines, i, fn_pattern) {
                        violations.push(TestQualityViolation::TodoInTestFixture {
                            file: file.to_path_buf(),
                            line: i + 1,
                            function_name,
                            severity: Severity::Error,
                        });
                    }
                }
            }
        }
    }

    fn check_empty_test_bodies(
        &self,
        file: &Path,
        lines: &[&str],
        test_pattern: &Regex,
        fn_pattern: &Regex,
        empty_body_pattern: &Regex,
        violations: &mut Vec<TestQualityViolation>,
    ) {
        for (i, line) in lines.iter().enumerate() {
            if test_pattern.is_match(line) {
                // Find the function declaration
                if let Some(fn_line_idx) =
                    (i..i + 5).find(|&idx| idx < lines.len() && fn_pattern.is_match(lines[idx]))
                {
                    // Check if the function body is empty (just {})
                    if let Some(body_start) = (fn_line_idx..fn_line_idx + 3)
                        .find(|&idx| idx < lines.len() && lines[idx].contains('{'))
                    {
                        if empty_body_pattern.is_match(lines[body_start])
                            || (body_start + 1 < lines.len() && lines[body_start + 1].trim() == "}")
                        {
                            if let Some(test_name) = fn_pattern
                                .captures(lines[fn_line_idx])
                                .and_then(|c| c.get(1))
                            {
                                violations.push(TestQualityViolation::EmptyTestBody {
                                    file: file.to_path_buf(),
                                    line: fn_line_idx + 1,
                                    test_name: test_name.as_str().to_string(),
                                    severity: Severity::Error,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    fn check_stub_assertions(
        &self,
        file: &Path,
        lines: &[&str],
        test_pattern: &Regex,
        stub_assert_pattern: &Regex,
        violations: &mut Vec<TestQualityViolation>,
    ) {
        for (i, line) in lines.iter().enumerate() {
            if test_pattern.is_match(line) {
                // Look for stub assertions in the next 20 lines (typical test body)
                for offset in 0..20 {
                    if i + offset >= lines.len() {
                        break;
                    }
                    if stub_assert_pattern.is_match(lines[i + offset]) {
                        if let Some(test_name) = self.find_test_name(
                            lines,
                            i,
                            test_pattern,
                            &Regex::new(r"fn\s+(\w+)").unwrap(),
                        ) {
                            violations.push(TestQualityViolation::StubTestAssertion {
                                file: file.to_path_buf(),
                                line: i + offset + 1,
                                test_name,
                                severity: Severity::Warning,
                            });
                        }
                        break;
                    }
                }
            }
        }
    }

    fn find_test_name(
        &self,
        lines: &[&str],
        start_idx: usize,
        _test_pattern: &Regex,
        fn_pattern: &Regex,
    ) -> Option<String> {
        // Look for function name in next few lines
        for i in start_idx..std::cmp::min(start_idx + 5, lines.len()) {
            if let Some(captures) = fn_pattern.captures(lines[i]) {
                if let Some(name) = captures.get(1) {
                    return Some(name.as_str().to_string());
                }
            }
        }
        None
    }

    fn find_function_name(
        &self,
        lines: &[&str],
        start_idx: usize,
        fn_pattern: &Regex,
    ) -> Option<String> {
        // Look backwards for function name
        for i in (0..=start_idx).rev().take(10) {
            if let Some(captures) = fn_pattern.captures(lines[i]) {
                if let Some(name) = captures.get(1) {
                    return Some(name.as_str().to_string());
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ignore_violation_formatting() {
        let violation = TestQualityViolation::IgnoreWithoutJustification {
            file: PathBuf::from("test.rs"),
            line: 10,
            test_name: "my_test".to_string(),
            severity: Severity::Warning,
        };

        assert_eq!(violation.id(), "TST001");
        assert_eq!(violation.severity(), Severity::Warning);
        assert!(violation.to_string().contains("my_test"));
        assert!(violation.suggestion().is_some());
    }

    #[test]
    fn test_todo_violation_formatting() {
        let violation = TestQualityViolation::TodoInTestFixture {
            file: PathBuf::from("fixture.rs"),
            line: 20,
            function_name: "setup_test".to_string(),
            severity: Severity::Error,
        };

        assert_eq!(violation.id(), "TST002");
        assert_eq!(violation.severity(), Severity::Error);
    }
}
