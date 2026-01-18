//! Linter Integration Module
//!
//! Integrates external linters (Ruff, Clippy) as first-layer validation
//! that feeds into the unified violation reporting system.

use std::collections::HashMap;
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
#[derive(Debug, Clone, Copy)]
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
                let mut args = vec!["check".to_string(), "--format=json".to_string()];
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
            .args(&["clippy", "--message-format=json"])
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
            let has_rust_files = files.iter().any(|f| f.extension().map_or(false, |ext| ext == "rs"));
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
        .args(&linter.args(files))
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
fn parse_ruff_output(output: &str) -> Vec<LintViolation> {
    let mut violations = Vec::new();

    // Parse JSON lines
    for line in output.lines() {
        if let Ok(ruff_violation) = serde_json::from_str::<RuffViolation>(line) {
            violations.push(LintViolation {
                rule: ruff_violation.code,
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
fn parse_clippy_output(output: &str) -> Vec<LintViolation> {
    let mut violations = Vec::new();

    // Parse JSON lines
    for line in output.lines() {
        if let Ok(clippy_message) = serde_json::from_str::<ClippyMessage>(line) {
            if let Some(span) = &clippy_message.message.spans.first() {
                violations.push(LintViolation {
                    rule: clippy_message.message.code.clone().unwrap_or_default(),
                    file: span.file_name.clone(),
                    line: span.line_start,
                    column: span.column_start,
                    message: clippy_message.message.message,
                    severity: map_clippy_level(&clippy_message.message.level),
                    category: clippy_message.message.code
                        .as_ref()
                        .map(|code| if code.contains("clippy") { "quality" } else { "correctness" })
                        .unwrap_or("quality")
                        .to_string(),
                });
            }
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

/// Clippy message format
#[derive(serde::Deserialize)]
struct ClippyMessage {
    pub message: ClippyMessageContent,
}

#[derive(serde::Deserialize)]
struct ClippyMessageContent {
    pub message: String,
    pub code: Option<String>,
    pub level: String,
    pub spans: Vec<ClippySpan>,
}

#[derive(serde::Deserialize)]
struct ClippySpan {
    pub file_name: String,
    pub line_start: usize,
    pub column_start: usize,
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