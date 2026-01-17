//! Error Boundary Validation
//!
//! Validates Clean Architecture error handling patterns:
//! - Layer error wrapping (domain wraps infrastructure errors)
//! - Context preservation across layers
//! - Error type placement (right layer)

use crate::{Result, Severity, ValidationConfig};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use walkdir::WalkDir;

/// Error boundary violation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorBoundaryViolation {
    /// Error crossing layer without context
    MissingErrorContext {
        file: PathBuf,
        line: usize,
        error_pattern: String,
        suggestion: String,
        severity: Severity,
    },
    /// Infrastructure error type used in domain layer
    WrongLayerError {
        file: PathBuf,
        line: usize,
        error_type: String,
        layer: String,
        severity: Severity,
    },
    /// Internal error details leaked to external API
    LeakedInternalError {
        file: PathBuf,
        line: usize,
        pattern: String,
        severity: Severity,
    },
}

impl ErrorBoundaryViolation {
    pub fn severity(&self) -> Severity {
        match self {
            Self::MissingErrorContext { severity, .. } => *severity,
            Self::WrongLayerError { severity, .. } => *severity,
            Self::LeakedInternalError { severity, .. } => *severity,
        }
    }
}

impl std::fmt::Display for ErrorBoundaryViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingErrorContext {
                file,
                line,
                error_pattern,
                suggestion,
                ..
            } => {
                write!(
                    f,
                    "Missing error context: {}:{} - {} ({})",
                    file.display(),
                    line,
                    error_pattern,
                    suggestion
                )
            }
            Self::WrongLayerError {
                file,
                line,
                error_type,
                layer,
                ..
            } => {
                write!(
                    f,
                    "Wrong layer error: {}:{} - {} in {}",
                    file.display(),
                    line,
                    error_type,
                    layer
                )
            }
            Self::LeakedInternalError {
                file,
                line,
                pattern,
                ..
            } => {
                write!(
                    f,
                    "Leaked internal error: {}:{} - {}",
                    file.display(),
                    line,
                    pattern
                )
            }
        }
    }
}

/// Error boundary validator
pub struct ErrorBoundaryValidator {
    config: ValidationConfig,
}

impl ErrorBoundaryValidator {
    /// Create a new error boundary validator
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self::with_config(ValidationConfig::new(workspace_root))
    }

    /// Create a validator with custom configuration
    pub fn with_config(config: ValidationConfig) -> Self {
        Self { config }
    }

    /// Run all error boundary validations
    pub fn validate_all(&self) -> Result<Vec<ErrorBoundaryViolation>> {
        let mut violations = Vec::new();
        violations.extend(self.validate_error_context()?);
        violations.extend(self.validate_layer_error_types()?);
        violations.extend(self.validate_leaked_errors()?);
        Ok(violations)
    }

    /// Detect error propagation without context
    pub fn validate_error_context(&self) -> Result<Vec<ErrorBoundaryViolation>> {
        let mut violations = Vec::new();

        // Pattern: ? operator without .context() or .with_context()
        // This is a heuristic - we look for lines with ? but no context method
        let question_mark_pattern = Regex::new(r"\?\s*;?\s*$").expect("Invalid regex");
        let context_pattern = Regex::new(r"\.(context|with_context|map_err)\s*\(").expect("Invalid regex");

        // Files that are likely error boundary crossing points
        let boundary_paths = ["handlers/", "adapters/", "services/"];

        for src_dir in self.config.get_scan_dirs()? {
            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                let path_str = entry.path().to_string_lossy();

                // Skip test files
                if path_str.contains("/tests/") {
                    continue;
                }

                // Only check boundary files
                let is_boundary = boundary_paths.iter().any(|p| path_str.contains(p));
                if !is_boundary {
                    continue;
                }

                let content = std::fs::read_to_string(entry.path())?;
                let mut in_test_module = false;

                for (line_num, line) in content.lines().enumerate() {
                    let trimmed = line.trim();

                    // Skip comments
                    if trimmed.starts_with("//") {
                        continue;
                    }

                    // Track test modules
                    if trimmed.contains("#[cfg(test)]") {
                        in_test_module = true;
                        continue;
                    }

                    if in_test_module {
                        continue;
                    }

                    // Check for ? without context
                    if question_mark_pattern.is_match(trimmed) && !context_pattern.is_match(trimmed) {
                        // Skip simple Result propagation
                        if trimmed.starts_with("return ") || trimmed.contains("Ok(") {
                            continue;
                        }

                        violations.push(ErrorBoundaryViolation::MissingErrorContext {
                            file: entry.path().to_path_buf(),
                            line: line_num + 1,
                            error_pattern: trimmed.chars().take(60).collect(),
                            suggestion: "Add .context() or .map_err() for better error messages"
                                .to_string(),
                            severity: Severity::Info,
                        });
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Detect infrastructure error types in domain layer
    pub fn validate_layer_error_types(&self) -> Result<Vec<ErrorBoundaryViolation>> {
        let mut violations = Vec::new();

        // Infrastructure error types that shouldn't appear in domain
        let infra_errors = [
            (r"std::io::Error", "std::io::Error"),
            (r"reqwest::Error", "reqwest::Error"),
            (r"sqlx::Error", "sqlx::Error"),
            (r"tokio::.*Error", "tokio Error"),
            (r"hyper::Error", "hyper::Error"),
            (r"serde_json::Error", "serde_json::Error"),
        ];

        let compiled_errors: Vec<_> = infra_errors
            .iter()
            .filter_map(|(p, desc)| Regex::new(p).ok().map(|r| (r, *desc)))
            .collect();

        for src_dir in self.config.get_scan_dirs()? {
            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                let path_str = entry.path().to_string_lossy();

                // Skip test files
                if path_str.contains("/tests/") {
                    continue;
                }

                // Only check domain layer files
                let is_domain = path_str.contains("/domain/") || path_str.contains("mcb-domain");
                if !is_domain {
                    continue;
                }

                // Skip error definition files
                let file_name = entry.path().file_name().and_then(|n| n.to_str()).unwrap_or("");
                if file_name == "error.rs" || file_name.starts_with("error") {
                    continue;
                }

                let content = std::fs::read_to_string(entry.path())?;
                let mut in_test_module = false;

                for (line_num, line) in content.lines().enumerate() {
                    let trimmed = line.trim();

                    // Skip comments
                    if trimmed.starts_with("//") {
                        continue;
                    }

                    // Track test modules
                    if trimmed.contains("#[cfg(test)]") {
                        in_test_module = true;
                        continue;
                    }

                    if in_test_module {
                        continue;
                    }

                    // Check for infrastructure error types
                    for (pattern, desc) in &compiled_errors {
                        if pattern.is_match(line) {
                            violations.push(ErrorBoundaryViolation::WrongLayerError {
                                file: entry.path().to_path_buf(),
                                line: line_num + 1,
                                error_type: desc.to_string(),
                                layer: "domain".to_string(),
                                severity: Severity::Warning,
                            });
                        }
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Detect internal error details leaked to API responses
    pub fn validate_leaked_errors(&self) -> Result<Vec<ErrorBoundaryViolation>> {
        let mut violations = Vec::new();

        // Patterns that indicate internal errors being exposed
        let leak_patterns = [
            (r#"format!\s*\(\s*"\{\:?\?\}""#, "Debug formatting in response"),
            (r#"\.to_string\(\)\s*\)"#, "Error .to_string() in response"),
            (r#"serde_json::json!\s*\(\s*\{\s*"error"\s*:\s*format!"#, "Internal error in JSON response"),
        ];

        let compiled_leaks: Vec<_> = leak_patterns
            .iter()
            .filter_map(|(p, desc)| Regex::new(p).ok().map(|r| (r, *desc)))
            .collect();

        // Only check handler files (API boundary)
        for src_dir in self.config.get_scan_dirs()? {
            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                let path_str = entry.path().to_string_lossy();

                // Skip test files
                if path_str.contains("/tests/") {
                    continue;
                }

                // Only check handler files
                let is_handler = path_str.contains("/handlers/") || path_str.contains("_handler.rs");
                if !is_handler {
                    continue;
                }

                let content = std::fs::read_to_string(entry.path())?;
                let mut in_test_module = false;

                for (line_num, line) in content.lines().enumerate() {
                    let trimmed = line.trim();

                    // Skip comments
                    if trimmed.starts_with("//") {
                        continue;
                    }

                    // Track test modules
                    if trimmed.contains("#[cfg(test)]") {
                        in_test_module = true;
                        continue;
                    }

                    if in_test_module {
                        continue;
                    }

                    // Check for leak patterns
                    for (pattern, desc) in &compiled_leaks {
                        if pattern.is_match(line) {
                            violations.push(ErrorBoundaryViolation::LeakedInternalError {
                                file: entry.path().to_path_buf(),
                                line: line_num + 1,
                                pattern: desc.to_string(),
                                severity: Severity::Info,
                            });
                        }
                    }
                }
            }
        }

        Ok(violations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_crate_structure(temp: &TempDir, crate_name: &str, path: &str, content: &str) {
        let file_path = temp.path().join("crates").join(crate_name).join("src").join(path);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&file_path, content).unwrap();

        // Create Cargo.toml
        let cargo_path = temp.path().join("crates").join(crate_name).join("Cargo.toml");
        if !cargo_path.exists() {
            fs::write(
                cargo_path,
                format!(
                    r#"
[package]
name = "{}"
version = "0.1.1"
"#,
                    crate_name
                ),
            )
            .unwrap();
        }
    }

    #[test]
    fn test_missing_error_context_detection() {
        let temp = TempDir::new().unwrap();
        create_test_crate_structure(
            &temp,
            "mcb-server",
            "handlers/test.rs",
            r#"
pub async fn handle_request() -> Result<(), Error> {
    let data = fetch_data()?;
    process_data(data)?;
    Ok(())
}
"#,
        );

        let validator = ErrorBoundaryValidator::new(temp.path());
        let violations = validator.validate_error_context().unwrap();

        assert!(!violations.is_empty(), "Should detect missing error context");
    }

    #[test]
    fn test_wrong_layer_error_detection() {
        let temp = TempDir::new().unwrap();
        create_test_crate_structure(
            &temp,
            "mcb-domain",
            "services/test.rs",
            r#"
use std::io::Error;

pub fn domain_function() -> Result<(), std::io::Error> {
    Ok(())
}
"#,
        );

        let validator = ErrorBoundaryValidator::new(temp.path());
        let violations = validator.validate_layer_error_types().unwrap();

        assert!(!violations.is_empty(), "Should detect infrastructure error in domain");
    }

    #[test]
    fn test_error_rs_exempt() {
        let temp = TempDir::new().unwrap();
        create_test_crate_structure(
            &temp,
            "mcb-domain",
            "error.rs",
            r#"
use std::io::Error;

#[derive(Debug)]
pub enum DomainError {
    Io(std::io::Error),
}
"#,
        );

        let validator = ErrorBoundaryValidator::new(temp.path());
        let violations = validator.validate_layer_error_types().unwrap();

        assert!(
            violations.is_empty(),
            "error.rs files should be exempt: {:?}",
            violations
        );
    }
}
