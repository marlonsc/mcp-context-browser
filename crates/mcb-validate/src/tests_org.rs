//! Test Organization Validation
//!
//! Validates test organization:
//! - No inline test modules in src/ (should be in tests/)
//! - Test file naming conventions
//! - Test function naming conventions

use crate::violation_trait::{Violation, ViolationCategory};
use crate::{Result, Severity, ValidationConfig};
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
    /// Trivial assertion that always passes (assert!(true), assert_eq!(1, 1))
    TrivialAssertion {
        file: PathBuf,
        line: usize,
        function_name: String,
        assertion: String,
        severity: Severity,
    },
    /// Test only uses .unwrap() as assertion (crash = pass)
    UnwrapOnlyAssertion {
        file: PathBuf,
        line: usize,
        function_name: String,
        severity: Severity,
    },
    /// Test body is only comments (no code)
    CommentOnlyTest {
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
            Self::TrivialAssertion { severity, .. } => *severity,
            Self::UnwrapOnlyAssertion { severity, .. } => *severity,
            Self::CommentOnlyTest { severity, .. } => *severity,
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
            Self::BadTestFileName {
                file, suggestion, ..
            } => {
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
            Self::TrivialAssertion {
                file,
                line,
                function_name,
                assertion,
                ..
            } => {
                write!(
                    f,
                    "Trivial assertion: {}:{} - {} uses '{}' (always passes)",
                    file.display(),
                    line,
                    function_name,
                    assertion
                )
            }
            Self::UnwrapOnlyAssertion {
                file,
                line,
                function_name,
                ..
            } => {
                write!(
                    f,
                    "Unwrap-only test: {}:{} - {} has no real assertion, only .unwrap()",
                    file.display(),
                    line,
                    function_name
                )
            }
            Self::CommentOnlyTest {
                file,
                line,
                function_name,
                ..
            } => {
                write!(
                    f,
                    "Comment-only test: {}:{} - {} has no executable code",
                    file.display(),
                    line,
                    function_name
                )
            }
        }
    }
}

impl Violation for TestViolation {
    fn id(&self) -> &str {
        match self {
            Self::InlineTestModule { .. } => "TEST001",
            Self::BadTestFileName { .. } => "TEST002",
            Self::BadTestFunctionName { .. } => "TEST003",
            Self::TestWithoutAssertion { .. } => "TEST004",
            Self::TrivialAssertion { .. } => "TEST005",
            Self::UnwrapOnlyAssertion { .. } => "TEST006",
            Self::CommentOnlyTest { .. } => "TEST007",
        }
    }

    fn category(&self) -> ViolationCategory {
        ViolationCategory::Testing
    }

    fn severity(&self) -> Severity {
        match self {
            Self::InlineTestModule { severity, .. } => *severity,
            Self::BadTestFileName { severity, .. } => *severity,
            Self::BadTestFunctionName { severity, .. } => *severity,
            Self::TestWithoutAssertion { severity, .. } => *severity,
            Self::TrivialAssertion { severity, .. } => *severity,
            Self::UnwrapOnlyAssertion { severity, .. } => *severity,
            Self::CommentOnlyTest { severity, .. } => *severity,
        }
    }

    fn file(&self) -> Option<&std::path::PathBuf> {
        match self {
            Self::InlineTestModule { file, .. } => Some(file),
            Self::BadTestFileName { file, .. } => Some(file),
            Self::BadTestFunctionName { file, .. } => Some(file),
            Self::TestWithoutAssertion { file, .. } => Some(file),
            Self::TrivialAssertion { file, .. } => Some(file),
            Self::UnwrapOnlyAssertion { file, .. } => Some(file),
            Self::CommentOnlyTest { file, .. } => Some(file),
        }
    }

    fn line(&self) -> Option<usize> {
        match self {
            Self::InlineTestModule { line, .. } => Some(*line),
            Self::BadTestFileName { .. } => None,
            Self::BadTestFunctionName { line, .. } => Some(*line),
            Self::TestWithoutAssertion { line, .. } => Some(*line),
            Self::TrivialAssertion { line, .. } => Some(*line),
            Self::UnwrapOnlyAssertion { line, .. } => Some(*line),
            Self::CommentOnlyTest { line, .. } => Some(*line),
        }
    }

    fn suggestion(&self) -> Option<String> {
        match self {
            Self::InlineTestModule { .. } => {
                Some("Move test module to tests/ directory".to_string())
            }
            Self::BadTestFileName { suggestion, .. } => Some(format!("Rename to {}", suggestion)),
            Self::BadTestFunctionName { suggestion, .. } => {
                Some(format!("Rename to {}", suggestion))
            }
            Self::TestWithoutAssertion { function_name, .. } => Some(format!(
                "Add assertion to {} or use smoke test suffix",
                function_name
            )),
            Self::TrivialAssertion { assertion, .. } => {
                Some(format!("Replace {} with meaningful assertion", assertion))
            }
            Self::UnwrapOnlyAssertion { .. } => Some(
                "Add explicit assert! or assert_eq! instead of relying on .unwrap()".to_string(),
            ),
            Self::CommentOnlyTest { .. } => {
                Some("Add executable test code or remove the test".to_string())
            }
        }
    }
}

/// Test organization validator
pub struct TestValidator {
    config: ValidationConfig,
}

impl TestValidator {
    /// Create a new test validator
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self::with_config(ValidationConfig::new(workspace_root))
    }

    /// Create a validator with custom configuration for multi-directory support
    pub fn with_config(config: ValidationConfig) -> Self {
        Self { config }
    }

    /// Run all test organization validations
    pub fn validate_all(&self) -> Result<Vec<TestViolation>> {
        let mut violations = Vec::new();
        violations.extend(self.validate_no_inline_tests()?);
        violations.extend(self.validate_test_directory_structure()?);
        violations.extend(self.validate_test_naming()?);
        violations.extend(self.validate_test_function_naming()?);
        violations.extend(self.validate_test_quality()?);
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
                .filter_map(std::result::Result::ok)
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
                                severity: Severity::Error,
                            });
                        }
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Validate that tests are properly organized in subdirectories
    pub fn validate_test_directory_structure(&self) -> Result<Vec<TestViolation>> {
        let mut violations = Vec::new();

        for crate_dir in self.get_crate_dirs()? {
            let tests_dir = crate_dir.join("tests");
            if !tests_dir.exists() {
                continue;
            }

            // Check that at least unit/ or integration/ exists (e2e/ is optional)
            let unit_exists = tests_dir.join("unit").exists();
            let integration_exists = tests_dir.join("integration").exists();

            // Only flag if NEITHER unit/ nor integration/ exist and there are test files
            if !unit_exists && !integration_exists {
                let has_test_files = std::fs::read_dir(&tests_dir)
                    .map(|entries| {
                        entries.filter_map(|e| e.ok()).any(|e| {
                            let path = e.path();
                            path.is_file()
                                && path.extension().and_then(|x| x.to_str()) == Some("rs")
                                && !matches!(
                                    path.file_name().and_then(|n| n.to_str()).unwrap_or(""),
                                    "lib.rs" | "mod.rs" | "test_utils.rs"
                                )
                        })
                    })
                    .unwrap_or(false);

                if has_test_files {
                    violations.push(TestViolation::BadTestFileName {
                        file: tests_dir.clone(),
                        suggestion: "Create tests/unit/ or tests/integration/ directory"
                            .to_string(),
                        severity: Severity::Warning,
                    });
                }
            }

            // Check for test files directly in tests/ directory (not in subdirs)
            for entry in std::fs::read_dir(&tests_dir)? {
                let entry = entry?;
                let path = entry.path();

                // Skip directories
                if path.is_dir() {
                    continue;
                }

                // Skip non-Rust files
                if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                    continue;
                }

                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                // Skip allowed files in root tests directory
                // These are: lib.rs, mod.rs, test_utils.rs, and entry points for test subdirectories
                if matches!(
                    file_name,
                    "lib.rs" | "mod.rs" | "test_utils.rs" | "unit.rs" | "integration.rs" | "e2e.rs"
                ) {
                    continue;
                }

                // Any other .rs file directly in tests/ is a violation
                violations.push(TestViolation::BadTestFileName {
                    file: path,
                    suggestion: "Move to tests/unit/, tests/integration/, or tests/e2e/ directory"
                        .to_string(),
                    severity: Severity::Error,
                });
            }
        }

        Ok(violations)
    }

    /// Check test file naming conventions and directory structure
    pub fn validate_test_naming(&self) -> Result<Vec<TestViolation>> {
        let mut violations = Vec::new();

        for crate_dir in self.get_crate_dirs()? {
            let tests_dir = crate_dir.join("tests");
            if !tests_dir.exists() {
                continue;
            }

            // We don't require specific directories - tests can be organized
            // in any subdirectory structure as long as they have entry points

            for entry in WalkDir::new(&tests_dir)
                .into_iter()
                .filter_map(std::result::Result::ok)
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                let path = entry.path();
                let file_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");

                // Skip lib.rs and mod.rs
                if file_name == "lib" || file_name == "mod" {
                    continue;
                }

                // Skip test utility files (mocks, fixtures, helpers)
                let path_str = path.to_string_lossy();
                if path_str.contains("test_utils")
                    || file_name.contains("mock")
                    || file_name.contains("fixture")
                    || file_name.contains("helper")
                {
                    continue;
                }

                // Check directory-based naming conventions
                let parent_dir = path
                    .parent()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("");

                match parent_dir {
                    "unit" => {
                        // Unit tests must follow [module]_tests.rs pattern
                        if !file_name.ends_with("_tests") {
                            violations.push(TestViolation::BadTestFileName {
                                file: path.to_path_buf(),
                                suggestion: format!(
                                    "{}_tests.rs (unit tests must end with _tests)",
                                    file_name
                                ),
                                severity: Severity::Warning,
                            });
                        }
                    }
                    "integration" => {
                        // Integration tests can be more flexible but should indicate their purpose
                        let is_valid_integration = file_name.contains("integration")
                            || file_name.contains("workflow")
                            || file_name.ends_with("_integration")
                            || file_name.ends_with("_workflow");

                        if !is_valid_integration {
                            violations.push(TestViolation::BadTestFileName {
                                file: path.to_path_buf(),
                                suggestion: format!("{}_integration.rs or {}_workflow.rs (integration tests should indicate their purpose)", file_name, file_name),
                                severity: Severity::Info,
                            });
                        }
                    }
                    "e2e" => {
                        // E2E tests should clearly indicate they're end-to-end
                        let is_valid_e2e = file_name.contains("e2e")
                            || file_name.contains("end_to_end")
                            || file_name.starts_with("test_");

                        if !is_valid_e2e {
                            violations.push(TestViolation::BadTestFileName {
                                file: path.to_path_buf(),
                                suggestion: format!("{}_e2e.rs or test_{}.rs (e2e tests should indicate they're end-to-end)", file_name, file_name),
                                severity: Severity::Info,
                            });
                        }
                    }
                    "tests" => {
                        // Files directly in tests/ directory (not in any subdirectory)
                        // are violations UNLESS they are entry points
                        let file_full = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                        if !matches!(
                            file_full,
                            "lib.rs"
                                | "mod.rs"
                                | "test_utils.rs"
                                | "unit.rs"
                                | "integration.rs"
                                | "e2e.rs"
                        ) {
                            violations.push(TestViolation::BadTestFileName {
                                file: path.to_path_buf(),
                                suggestion: "Move to a subdirectory (e.g., tests/unit/)"
                                    .to_string(),
                                severity: Severity::Warning,
                            });
                        }
                    }
                    _ => {
                        // Files in subdirectories are allowed (module structure)
                        // No violation
                    }
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
        let fn_pattern =
            Regex::new(r"(?:async\s+)?fn\s+([a-z_][a-z0-9_]*)\s*\(").expect("Invalid regex");
        // Standard assertions plus implicit assertions
        let assert_pattern = Regex::new(
            r"assert!|assert_eq!|assert_ne!|panic!|should_panic|\.unwrap\(|\.expect\(|Box<dyn\s|type_name::",
        )
        .expect("Invalid regex");

        // Smoke test patterns - these verify compilation, not runtime behavior
        let smoke_test_patterns = [
            "_trait_object", // Tests that verify trait object construction compiles
            "_exists",       // Tests that verify types exist
            "_creation",     // Constructor tests with implicit unwrap assertions
            "_compiles",     // Explicit compile-time tests
            "_factory",      // Factory pattern tests (often smoke tests)
        ];

        for crate_dir in self.get_crate_dirs()? {
            let tests_dir = crate_dir.join("tests");
            if !tests_dir.exists() {
                continue;
            }

            for entry in WalkDir::new(&tests_dir)
                .into_iter()
                .filter_map(std::result::Result::ok)
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
                                let fn_name = cap.get(1).map_or("", |m| m.as_str());

                                // Check naming convention - must start with test_
                                if !fn_name.starts_with("test_") {
                                    violations.push(TestViolation::BadTestFunctionName {
                                        file: entry.path().to_path_buf(),
                                        line: fn_line_idx + 1,
                                        function_name: fn_name.to_string(),
                                        suggestion: format!("test_{}", fn_name),
                                        severity: Severity::Warning,
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

    /// Validate test quality (trivial assertions, unwrap-only, comment-only)
    pub fn validate_test_quality(&self) -> Result<Vec<TestViolation>> {
        let mut violations = Vec::new();

        // Trivial assertion patterns
        let trivial_patterns = [
            (r"assert!\s*\(\s*true\s*\)", "assert!(true)"),
            (r"assert!\s*\(\s*!false\s*\)", "assert!(!false)"),
            (
                r"assert_eq!\s*\(\s*true\s*,\s*true\s*\)",
                "assert_eq!(true, true)",
            ),
            (r"assert_eq!\s*\(\s*1\s*,\s*1\s*\)", "assert_eq!(1, 1)"),
            (r"assert_ne!\s*\(\s*1\s*,\s*2\s*\)", "assert_ne!(1, 2)"),
            (
                r"assert_ne!\s*\(\s*true\s*,\s*false\s*\)",
                "assert_ne!(true, false)",
            ),
        ];

        let compiled_trivial: Vec<_> = trivial_patterns
            .iter()
            .filter_map(|(p, desc)| Regex::new(p).ok().map(|r| (r, *desc)))
            .collect();

        let test_attr_pattern = Regex::new(r"#\[(?:tokio::)?test\]").ok();
        let fn_pattern = Regex::new(r"(?:async\s+)?fn\s+([a-z_][a-z0-9_]*)\s*\(").ok();
        // Match common assertion macros (including word boundaries to avoid false positives)
        let real_assert_pattern = Regex::new(
            r"(?:^|\s|;|\()(assert!|assert_eq!|assert_ne!|assert_matches!|debug_assert!|debug_assert_eq!|debug_assert_ne!|panic!)",
        )
        .ok();
        let unwrap_pattern = Regex::new(r"\.unwrap\(|\.expect\(").ok();

        for crate_dir in self.get_crate_dirs()? {
            let tests_dir = crate_dir.join("tests");
            if !tests_dir.exists() {
                continue;
            }

            for entry in WalkDir::new(&tests_dir)
                .into_iter()
                .filter_map(std::result::Result::ok)
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                let content = std::fs::read_to_string(entry.path())?;
                let lines: Vec<&str> = content.lines().collect();

                let mut i = 0;
                while i < lines.len() {
                    let line = lines[i];

                    // Check for test attribute
                    let is_test_attr = test_attr_pattern.as_ref().is_some_and(|p| p.is_match(line));

                    if is_test_attr {
                        // Find the function definition
                        let mut fn_line_idx = i + 1;
                        while fn_line_idx < lines.len() {
                            let potential_fn = lines[fn_line_idx];
                            let fn_cap = fn_pattern.as_ref().and_then(|p| p.captures(potential_fn));

                            if let Some(cap) = fn_cap {
                                let fn_name = cap.get(1).map_or("", |m| m.as_str());
                                let fn_start = fn_line_idx;

                                // Collect function body
                                let mut body_lines: Vec<(usize, &str)> = Vec::new();
                                let mut brace_depth = 0;
                                let mut started = false;

                                for (idx, check_line) in lines.iter().enumerate().skip(fn_line_idx)
                                {
                                    if check_line.contains('{') {
                                        started = true;
                                    }
                                    if started {
                                        brace_depth +=
                                            check_line.chars().filter(|c| *c == '{').count() as i32;
                                        brace_depth -=
                                            check_line.chars().filter(|c| *c == '}').count() as i32;
                                        body_lines.push((idx, check_line));
                                        if brace_depth <= 0 {
                                            break;
                                        }
                                    }
                                }

                                // Check for trivial assertions
                                for (line_idx, body_line) in &body_lines {
                                    for (pattern, desc) in &compiled_trivial {
                                        if pattern.is_match(body_line) {
                                            violations.push(TestViolation::TrivialAssertion {
                                                file: entry.path().to_path_buf(),
                                                line: line_idx + 1,
                                                function_name: fn_name.to_string(),
                                                assertion: desc.to_string(),
                                                severity: Severity::Warning,
                                            });
                                        }
                                    }
                                }

                                // Check for unwrap-only tests (has unwrap but no real assertion)
                                let has_unwrap = unwrap_pattern
                                    .as_ref()
                                    .is_some_and(|p| body_lines.iter().any(|(_, l)| p.is_match(l)));
                                let has_real_assert = real_assert_pattern
                                    .as_ref()
                                    .is_some_and(|p| body_lines.iter().any(|(_, l)| p.is_match(l)));

                                if has_unwrap && !has_real_assert {
                                    // Use Warning severity since this is heuristic-based detection
                                    // Tests may have valid assertions that aren't detected by the pattern
                                    violations.push(TestViolation::UnwrapOnlyAssertion {
                                        file: entry.path().to_path_buf(),
                                        line: fn_start + 1,
                                        function_name: fn_name.to_string(),
                                        severity: Severity::Warning,
                                    });
                                }

                                // Check for comment-only tests
                                let _meaningful_lines: Vec<_> = body_lines
                                    .iter()
                                    .filter(|(_, l)| {
                                        let trimmed = l.trim();
                                        !trimmed.is_empty()
                                            && !trimmed.starts_with("//")
                                            && !trimmed.starts_with('{')
                                            && !trimmed.starts_with('}')
                                            && trimmed != "{"
                                            && trimmed != "}"
                                    })
                                    .collect();

                                // If no meaningful lines (only comments/braces), it's comment-only
                                let all_comments = body_lines.iter().all(|(_, l)| {
                                    let trimmed = l.trim();
                                    trimmed.is_empty()
                                        || trimmed.starts_with("//")
                                        || trimmed == "{"
                                        || trimmed == "}"
                                        || trimmed.starts_with("fn ")
                                        || trimmed.starts_with("async fn ")
                                });

                                if all_comments && body_lines.len() > 2 {
                                    violations.push(TestViolation::CommentOnlyTest {
                                        file: entry.path().to_path_buf(),
                                        line: fn_start + 1,
                                        function_name: fn_name.to_string(),
                                        severity: Severity::Error,
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
        self.config.get_source_dirs()
    }

    /// Check if a path is from legacy/additional source directories
    #[allow(dead_code)]
    fn is_legacy_path(&self, path: &std::path::Path) -> bool {
        self.config.is_legacy_path(path)
    }
}

impl crate::validator_trait::Validator for TestValidator {
    fn name(&self) -> &'static str {
        "tests_org"
    }

    fn description(&self) -> &'static str {
        "Validates test organization and quality"
    }

    fn validate(&self, _config: &ValidationConfig) -> anyhow::Result<Vec<Box<dyn Violation>>> {
        let violations = self.validate_all()?;
        Ok(violations
            .into_iter()
            .map(|v| Box::new(v) as Box<dyn Violation>)
            .collect())
    }
}
