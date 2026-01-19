//! Linter Integration Module
//!
//! Integrates external linters (Ruff, Clippy) as first-layer validation
//! that feeds into the unified violation reporting system.

use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;

use crate::Result;

/// Unified linter violation format
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct LintViolation {
    pub rule: String,
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub message: String,
    pub severity: String,
    pub category: String,
}

/// Supported linter types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinterType {
    Ruff,
    Clippy,
}

impl LinterType {
    pub fn command(&self) -> &'static str {
        match self {
            LinterType::Ruff => "ruff",
            LinterType::Clippy => "cargo",
        }
    }

    pub fn args(&self, files: &[&Path]) -> Vec<String> {
        match self {
            LinterType::Ruff => {
                let mut args = vec!["check".to_string(), "--output-format=json".to_string()];
                for file in files {
                    args.push(file.to_string_lossy().to_string());
                }
                args
            }
            LinterType::Clippy => {
                vec![
                    "clippy".to_string(),
                    "--message-format=json".to_string(),
                    "--".to_string(),
                ]
            }
        }
    }

    pub fn parse_output(&self, output: &str) -> Vec<LintViolation> {
        match self {
            LinterType::Ruff => parse_ruff_output(output),
            LinterType::Clippy => parse_clippy_output(output),
        }
    }
}

/// Execute Ruff linter on files
pub struct RuffLinter;

impl RuffLinter {
    pub async fn check_files(files: &[&Path]) -> Result<Vec<LintViolation>> {
        let linter = LinterType::Ruff;
        let output = run_linter_command(linter, files).await?;
        Ok(linter.parse_output(&output))
    }

    pub async fn check_file(file: &Path) -> Result<Vec<LintViolation>> {
        Self::check_files(&[file]).await
    }
}

/// Execute Clippy linter on Rust project
pub struct ClippyLinter;

impl ClippyLinter {
    pub async fn check_project(project_root: &Path) -> Result<Vec<LintViolation>> {
        // Change to project directory and run clippy
        let output = Command::new("cargo")
            .args(["clippy", "--message-format=json"])
            .current_dir(project_root)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(LinterType::Clippy.parse_output(&stdout))
    }
}

/// Unified linter interface
pub struct LinterEngine {
    enabled_linters: Vec<LinterType>,
}

impl LinterEngine {
    pub fn new() -> Self {
        Self {
            enabled_linters: vec![LinterType::Ruff, LinterType::Clippy],
        }
    }

    pub fn with_linters(linters: Vec<LinterType>) -> Self {
        Self {
            enabled_linters: linters,
        }
    }

    pub async fn check_files(&self, files: &[&Path]) -> Result<Vec<LintViolation>> {
        let mut all_violations = Vec::new();

        // Check if Ruff is available and run it
        if self.enabled_linters.contains(&LinterType::Ruff) {
            if let Ok(violations) = RuffLinter::check_files(files).await {
                all_violations.extend(violations);
            }
        }

        // For Clippy, we need to check if any Rust files are present
        if self.enabled_linters.contains(&LinterType::Clippy) {
            let has_rust_files = files
                .iter()
                .any(|f| f.extension().is_some_and(|ext| ext == "rs"));
            if has_rust_files {
                // Find project root (simplified - assumes files are in a Cargo project)
                if let Some(project_root) = find_project_root(files) {
                    if let Ok(violations) = ClippyLinter::check_project(&project_root).await {
                        all_violations.extend(violations);
                    }
                }
            }
        }

        Ok(all_violations)
    }

    pub fn map_lint_to_rule(&self, lint_code: &str) -> Option<&'static str> {
        match lint_code {
            // Ruff mappings
            "F401" => Some("QUAL005"), // Unused imports

            // Clippy mappings
            "clippy::unwrap_used" => Some("QUAL001"), // Unwrap usage

            _ => None,
        }
    }
}

impl Default for LinterEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Execute linter command
async fn run_linter_command(linter: LinterType, files: &[&Path]) -> Result<String> {
    let output = Command::new(linter.command())
        .args(linter.args(files))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        // Even if command fails, we might have partial output
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

/// Parse Ruff JSON output
///
/// Ruff outputs JSON in array format when using `--output-format=json`:
/// ```json
/// [
///   {
///     "code": "F401",
///     "message": "...",
///     "filename": "...",
///     "location": {"row": 1, "column": 1},
///     ...
///   }
/// ]
/// ```
fn parse_ruff_output(output: &str) -> Vec<LintViolation> {
    let mut violations = Vec::new();

    // Try parsing as JSON array first (ruff's default format)
    if let Ok(ruff_violations) = serde_json::from_str::<Vec<RuffViolation>>(output) {
        for ruff_violation in ruff_violations {
            violations.push(LintViolation {
                rule: ruff_violation.code.clone(),
                file: ruff_violation.filename,
                line: ruff_violation.location.row,
                column: ruff_violation.location.column,
                message: ruff_violation.message,
                severity: map_ruff_severity(&ruff_violation.code),
                category: "quality".to_string(),
            });
        }
        return violations;
    }

    // Fallback: try parsing as JSON lines (legacy/alternative format)
    for line in output.lines() {
        if let Ok(ruff_violation) = serde_json::from_str::<RuffViolation>(line) {
            violations.push(LintViolation {
                rule: ruff_violation.code.clone(),
                file: ruff_violation.filename,
                line: ruff_violation.location.row,
                column: ruff_violation.location.column,
                message: ruff_violation.message,
                severity: map_ruff_severity(&ruff_violation.code),
                category: "quality".to_string(),
            });
        }
    }

    violations
}

/// Parse Clippy JSON output
///
/// Clippy outputs JSON lines with different "reason" types:
/// - "compiler-message": contains lint/warning/error messages
/// - "compiler-artifact": build artifacts (ignore)
/// - "build-finished": build completion (ignore)
///
/// The message structure for compiler-message:
/// ```json
/// {
///   "reason": "compiler-message",
///   "message": {
///     "code": {"code": "clippy::unwrap_used", "explanation": null},
///     "level": "warning",
///     "message": "...",
///     "spans": [{"file_name": "...", "line_start": 1, "column_start": 1, ...}]
///   }
/// }
/// ```
fn parse_clippy_output(output: &str) -> Vec<LintViolation> {
    let mut violations = Vec::new();

    // Parse JSON lines
    for line in output.lines() {
        // Skip empty lines
        if line.trim().is_empty() {
            continue;
        }

        // Try to parse as ClippyOutput (with reason field)
        if let Ok(clippy_output) = serde_json::from_str::<ClippyOutput>(line) {
            // Only process compiler-message reason
            if clippy_output.reason != "compiler-message" {
                continue;
            }

            let msg = &clippy_output.message;

            // Skip messages without primary spans
            let Some(span) = msg.spans.iter().find(|s| s.is_primary) else {
                continue;
            };

            // Extract the code (either from nested structure or direct)
            let rule_code = msg
                .code
                .as_ref()
                .map(|c| c.code.clone())
                .unwrap_or_default();

            // Skip if no rule code (likely a build error, not a lint)
            if rule_code.is_empty() {
                continue;
            }

            violations.push(LintViolation {
                rule: rule_code.clone(),
                file: span.file_name.clone(),
                line: span.line_start,
                column: span.column_start,
                message: msg.message.clone(),
                severity: map_clippy_level(&msg.level),
                category: if rule_code.contains("clippy") {
                    "quality"
                } else {
                    "correctness"
                }
                .to_string(),
            });
        }
    }

    violations
}

/// Find project root from files (simplified)
fn find_project_root(files: &[&Path]) -> Option<std::path::PathBuf> {
    // Simple heuristic: go up until we find Cargo.toml
    if let Some(first_file) = files.first() {
        let mut current = first_file.parent()?;
        loop {
            if current.join("Cargo.toml").exists() {
                return Some(current.to_path_buf());
            }
            current = current.parent()?;
        }
    }
    None
}

/// Map Ruff severity
fn map_ruff_severity(code: &str) -> String {
    match code.chars().next() {
        Some('F') => "error".to_string(),   // Pyflakes
        Some('E') => "error".to_string(),   // pycodestyle errors
        Some('W') => "warning".to_string(), // pycodestyle warnings
        _ => "info".to_string(),
    }
}

/// Map Clippy level
fn map_clippy_level(level: &str) -> String {
    match level {
        "error" => "error".to_string(),
        "warning" => "warning".to_string(),
        "note" => "info".to_string(),
        "help" => "info".to_string(),
        _ => "info".to_string(),
    }
}

/// Ruff violation format
#[derive(serde::Deserialize)]
struct RuffViolation {
    pub code: String,
    pub message: String,
    pub filename: String,
    pub location: RuffLocation,
}

#[derive(serde::Deserialize)]
struct RuffLocation {
    pub row: usize,
    pub column: usize,
}

/// Clippy output format (JSON lines with reason field)
#[derive(serde::Deserialize)]
struct ClippyOutput {
    pub reason: String,
    pub message: ClippyMessageContent,
}

#[derive(serde::Deserialize)]
struct ClippyMessageContent {
    pub message: String,
    pub code: Option<ClippyCode>,
    pub level: String,
    pub spans: Vec<ClippySpan>,
}

/// Clippy code is nested: {"code": "clippy::unwrap_used", "explanation": null}
#[derive(serde::Deserialize)]
struct ClippyCode {
    pub code: String,
    #[allow(dead_code)]
    pub explanation: Option<String>,
}

#[derive(serde::Deserialize)]
struct ClippySpan {
    pub file_name: String,
    pub line_start: usize,
    pub column_start: usize,
    #[serde(default)]
    pub is_primary: bool,
}

// ==================== YAML Rule Executor (Phase 1 Deliverable) ====================
//
// This module wires the `lint_select` YAML field to actual linter execution.
// Plan requirement: "Wire lint_select YAML field to actual linter execution"

use crate::rules::yaml_loader::ValidatedRule;
use std::path::PathBuf;

/// Execute a YAML rule that uses lint_select for linter-based validation
///
/// This is the Phase 1 deliverable: YAML rule → linter → violations pipeline
pub struct YamlRuleExecutor;

impl YamlRuleExecutor {
    /// Execute a rule's lint_select codes against files
    ///
    /// # Arguments
    /// * `rule` - The validated YAML rule with lint_select codes
    /// * `files` - Files to check (Python for Ruff, Rust for Clippy)
    ///
    /// # Returns
    /// Violations that match the rule's lint_select codes
    pub async fn execute_rule(
        rule: &ValidatedRule,
        files: &[&Path],
    ) -> Result<Vec<LintViolation>> {
        // Skip if no lint_select codes
        if rule.lint_select.is_empty() {
            return Ok(vec![]);
        }

        // Skip if rule is disabled
        if !rule.enabled {
            return Ok(vec![]);
        }

        // Determine which linters to use based on lint_select codes
        let linters = Self::detect_linters_from_codes(&rule.lint_select);

        if linters.is_empty() {
            return Ok(vec![]);
        }

        // Execute linters
        let engine = LinterEngine::with_linters(linters);
        let all_violations = engine.check_files(files).await?;

        // Filter violations to only include those matching lint_select codes
        let filtered: Vec<LintViolation> = all_violations
            .into_iter()
            .filter(|v| rule.lint_select.iter().any(|code| v.rule == *code))
            .map(|mut v| {
                // Apply rule's custom message if provided
                if let Some(ref msg) = rule.message {
                    v.message = msg.clone();
                }
                // Set category from rule
                v.category = rule.category.clone();
                v
            })
            .collect();

        Ok(filtered)
    }

    /// Execute a rule against a directory (scans for appropriate files)
    pub async fn execute_rule_on_dir(
        rule: &ValidatedRule,
        dir: &Path,
    ) -> Result<Vec<LintViolation>> {
        // Collect files based on linter type
        let linters = Self::detect_linters_from_codes(&rule.lint_select);
        let mut files: Vec<PathBuf> = Vec::new();

        for entry in walkdir::WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str());

            // Collect Python files for Ruff
            if linters.contains(&LinterType::Ruff) && ext == Some("py") {
                files.push(path.to_path_buf());
            }

            // Collect Rust files for Clippy
            if linters.contains(&LinterType::Clippy) && ext == Some("rs") {
                files.push(path.to_path_buf());
            }
        }

        let file_refs: Vec<&Path> = files.iter().map(|p: &PathBuf| p.as_path()).collect();
        Self::execute_rule(rule, &file_refs).await
    }

    /// Detect which linters to use based on lint_select codes
    fn detect_linters_from_codes(codes: &[String]) -> Vec<LinterType> {
        let mut linters = Vec::new();

        for code in codes {
            if code.starts_with("clippy::") {
                if !linters.contains(&LinterType::Clippy) {
                    linters.push(LinterType::Clippy);
                }
            } else {
                // Ruff codes: F*, E*, W*, I*, B*, C*, D*, N*, S*, etc.
                if !linters.contains(&LinterType::Ruff) {
                    linters.push(LinterType::Ruff);
                }
            }
        }

        linters
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linter_engine_creation() {
        let engine = LinterEngine::new();
        assert!(!engine.enabled_linters.is_empty());
    }

    #[test]
    fn test_ruff_severity_mapping() {
        assert_eq!(map_ruff_severity("F401"), "error");
        assert_eq!(map_ruff_severity("W291"), "warning");
        assert_eq!(map_ruff_severity("I001"), "info");
    }

    #[test]
    fn test_clippy_level_mapping() {
        assert_eq!(map_clippy_level("error"), "error");
        assert_eq!(map_clippy_level("warning"), "warning");
        assert_eq!(map_clippy_level("note"), "info");
    }

    #[tokio::test]
    async fn test_linter_engine_execution() {
        let engine = LinterEngine::new();

        // Test with non-existent files (should not panic)
        let result = engine.check_files(&[]).await;
        assert!(result.is_ok());
    }
}
