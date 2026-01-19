//! Code Quality Validation
//!
//! Validates code quality standards:
//! - No unwrap/expect in production code (AST-based detection)
//! - No panic!() in production code
//! - File size limits (500 lines)
//! - TODO/FIXME comment detection
//!
//! Phase 2 deliverable: QUAL001 (no-unwrap) detects `.unwrap()` calls via AST

use crate::ast::UnwrapDetector;
use crate::violation_trait::{Violation, ViolationCategory};
use crate::{Result, Severity, ValidationConfig};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use walkdir::WalkDir;

/// Maximum allowed lines per file
pub const MAX_FILE_LINES: usize = 500;

/// Maximum allowed function lines (informational)
pub const MAX_FUNCTION_LINES: usize = 50;

/// Quality violation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QualityViolation {
    /// unwrap() found in production code
    UnwrapInProduction {
        file: PathBuf,
        line: usize,
        context: String,
        severity: Severity,
    },
    /// expect() found in production code
    ExpectInProduction {
        file: PathBuf,
        line: usize,
        context: String,
        severity: Severity,
    },
    /// panic!() found in production code
    PanicInProduction {
        file: PathBuf,
        line: usize,
        context: String,
        severity: Severity,
    },
    /// File exceeds line limit
    FileTooLarge {
        file: PathBuf,
        lines: usize,
        max_allowed: usize,
        severity: Severity,
    },
    /// TODO/FIXME comment found
    TodoComment {
        file: PathBuf,
        line: usize,
        content: String,
        severity: Severity,
    },
}

impl QualityViolation {
    pub fn severity(&self) -> Severity {
        match self {
            Self::UnwrapInProduction { severity, .. } => *severity,
            Self::ExpectInProduction { severity, .. } => *severity,
            Self::PanicInProduction { severity, .. } => *severity,
            Self::FileTooLarge { severity, .. } => *severity,
            Self::TodoComment { severity, .. } => *severity,
        }
    }
}

impl std::fmt::Display for QualityViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnwrapInProduction {
                file,
                line,
                context,
                ..
            } => {
                write!(
                    f,
                    "unwrap() in production: {}:{} - {}",
                    file.display(),
                    line,
                    context
                )
            }
            Self::ExpectInProduction {
                file,
                line,
                context,
                ..
            } => {
                write!(
                    f,
                    "expect() in production: {}:{} - {}",
                    file.display(),
                    line,
                    context
                )
            }
            Self::PanicInProduction {
                file,
                line,
                context,
                ..
            } => {
                write!(
                    f,
                    "panic!() in production: {}:{} - {}",
                    file.display(),
                    line,
                    context
                )
            }
            Self::FileTooLarge {
                file,
                lines,
                max_allowed,
                ..
            } => {
                write!(
                    f,
                    "File too large: {} has {} lines (max: {})",
                    file.display(),
                    lines,
                    max_allowed
                )
            }
            Self::TodoComment {
                file,
                line,
                content,
                ..
            } => {
                write!(f, "TODO/FIXME: {}:{} - {}", file.display(), line, content)
            }
        }
    }
}

impl Violation for QualityViolation {
    fn id(&self) -> &str {
        match self {
            Self::UnwrapInProduction { .. } => "QUAL001",
            Self::ExpectInProduction { .. } => "QUAL002",
            Self::PanicInProduction { .. } => "QUAL003",
            Self::FileTooLarge { .. } => "QUAL004",
            Self::TodoComment { .. } => "QUAL005",
        }
    }

    fn category(&self) -> ViolationCategory {
        ViolationCategory::Quality
    }

    fn severity(&self) -> Severity {
        match self {
            Self::UnwrapInProduction { severity, .. } => *severity,
            Self::ExpectInProduction { severity, .. } => *severity,
            Self::PanicInProduction { severity, .. } => *severity,
            Self::FileTooLarge { severity, .. } => *severity,
            Self::TodoComment { severity, .. } => *severity,
        }
    }

    fn file(&self) -> Option<&std::path::PathBuf> {
        match self {
            Self::UnwrapInProduction { file, .. } => Some(file),
            Self::ExpectInProduction { file, .. } => Some(file),
            Self::PanicInProduction { file, .. } => Some(file),
            Self::FileTooLarge { file, .. } => Some(file),
            Self::TodoComment { file, .. } => Some(file),
        }
    }

    fn line(&self) -> Option<usize> {
        match self {
            Self::UnwrapInProduction { line, .. } => Some(*line),
            Self::ExpectInProduction { line, .. } => Some(*line),
            Self::PanicInProduction { line, .. } => Some(*line),
            Self::FileTooLarge { .. } => None,
            Self::TodoComment { line, .. } => Some(*line),
        }
    }

    fn suggestion(&self) -> Option<String> {
        match self {
            Self::UnwrapInProduction { .. } => {
                Some("Use `?` operator or handle the error explicitly".to_string())
            }
            Self::ExpectInProduction { .. } => {
                Some("Use `?` operator or handle the error explicitly".to_string())
            }
            Self::PanicInProduction { .. } => {
                Some("Return an error instead of panicking".to_string())
            }
            Self::FileTooLarge { max_allowed, .. } => Some(format!(
                "Split file into smaller modules (max {} lines)",
                max_allowed
            )),
            Self::TodoComment { .. } => {
                Some("Address the TODO/FIXME or create an issue to track it".to_string())
            }
        }
    }
}

/// Quality validator
pub struct QualityValidator {
    config: ValidationConfig,
    max_file_lines: usize,
}

impl QualityValidator {
    /// Create a new quality validator
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self::with_config(ValidationConfig::new(workspace_root))
    }

    /// Create a validator with custom configuration for multi-directory support
    pub fn with_config(config: ValidationConfig) -> Self {
        Self {
            config,
            max_file_lines: MAX_FILE_LINES,
        }
    }

    /// Set custom max file lines
    pub fn with_max_file_lines(mut self, max: usize) -> Self {
        self.max_file_lines = max;
        self
    }

    /// Run all quality validations
    pub fn validate_all(&self) -> Result<Vec<QualityViolation>> {
        let mut violations = Vec::new();
        violations.extend(self.validate_no_unwrap_expect()?);
        violations.extend(self.validate_no_panic()?);
        violations.extend(self.validate_file_sizes()?);
        violations.extend(self.find_todo_comments()?);
        Ok(violations)
    }

    /// Check for unwrap/expect in src/ (not tests/)
    ///
    /// This uses AST-based detection via Tree-sitter for accurate detection
    /// of `.unwrap()` and `.expect()` method calls.
    ///
    /// Phase 2 deliverable: "QUAL001 (no-unwrap) detects `.unwrap()` calls via AST"
    pub fn validate_no_unwrap_expect(&self) -> Result<Vec<QualityViolation>> {
        let mut violations = Vec::new();
        let mut detector = UnwrapDetector::new()?;

        // Use get_scan_dirs() for proper handling of both crate-style and flat directories
        for src_dir in self.config.get_scan_dirs()? {
            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                // Use AST-based detection
                let detections = detector.detect_in_file(entry.path())?;

                for detection in detections {
                    // Skip detections in test modules
                    if detection.in_test {
                        continue;
                    }

                    // Skip if context contains SAFETY justification
                    // (checked via Regex since AST doesn't capture comments easily)
                    let content = std::fs::read_to_string(entry.path())?;
                    let lines: Vec<&str> = content.lines().collect();
                    if detection.line > 0 && detection.line <= lines.len() {
                        let line_content = lines[detection.line - 1];
                        if line_content.contains("// SAFETY:")
                            || line_content.contains("// safety:")
                        {
                            continue;
                        }
                    }

                    // Create appropriate violation based on method type
                    match detection.method.as_str() {
                        "unwrap" => {
                            violations.push(QualityViolation::UnwrapInProduction {
                                file: entry.path().to_path_buf(),
                                line: detection.line,
                                context: detection.context,
                                severity: Severity::Error,
                            });
                        }
                        "expect" => {
                            violations.push(QualityViolation::ExpectInProduction {
                                file: entry.path().to_path_buf(),
                                line: detection.line,
                                context: detection.context,
                                severity: Severity::Error,
                            });
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Check for panic!() macros in production code
    pub fn validate_no_panic(&self) -> Result<Vec<QualityViolation>> {
        let mut violations = Vec::new();
        let panic_pattern = Regex::new(r"panic!\s*\(").expect("Invalid regex");

        // Use get_scan_dirs() for proper handling of both crate-style and flat directories
        for src_dir in self.config.get_scan_dirs()? {
            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
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

                    // Check for panic!
                    if panic_pattern.is_match(line) {
                        violations.push(QualityViolation::PanicInProduction {
                            file: entry.path().to_path_buf(),
                            line: line_num + 1,
                            context: trimmed.to_string(),
                            severity: Severity::Error,
                        });
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Validate all source files under line limit
    pub fn validate_file_sizes(&self) -> Result<Vec<QualityViolation>> {
        let mut violations = Vec::new();

        // Use get_scan_dirs() for proper handling of both crate-style and flat directories
        for src_dir in self.config.get_scan_dirs()? {
            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.path().extension().is_some_and(|ext| ext == "rs")
                        && !self.config.should_exclude(e.path())
                        && !e.path().to_string_lossy().contains("/tests/")
                        && !e.path().to_string_lossy().contains("/target/")
                        && !e.path().to_string_lossy().ends_with("_test.rs")
                })
            {
                let content = std::fs::read_to_string(entry.path())?;
                let line_count = content.lines().count();

                if line_count > self.max_file_lines {
                    violations.push(QualityViolation::FileTooLarge {
                        file: entry.path().to_path_buf(),
                        lines: line_count,
                        max_allowed: self.max_file_lines,
                        severity: Severity::Warning,
                    });
                }
            }
        }

        Ok(violations)
    }

    /// Find TODO/FIXME comments
    pub fn find_todo_comments(&self) -> Result<Vec<QualityViolation>> {
        let mut violations = Vec::new();
        let todo_pattern =
            Regex::new(r"(?i)(TODO|FIXME|XXX|HACK):?\s*(.*)").expect("Invalid regex");

        // Use get_scan_dirs() for proper handling of both crate-style and flat directories
        for src_dir in self.config.get_scan_dirs()? {
            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                let content = std::fs::read_to_string(entry.path())?;

                for (line_num, line) in content.lines().enumerate() {
                    if let Some(cap) = todo_pattern.captures(line) {
                        let todo_type = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                        let message = cap.get(2).map(|m| m.as_str()).unwrap_or("").trim();

                        violations.push(QualityViolation::TodoComment {
                            file: entry.path().to_path_buf(),
                            line: line_num + 1,
                            content: format!("{}: {}", todo_type.to_uppercase(), message),
                            severity: Severity::Info,
                        });
                    }
                }
            }
        }

        Ok(violations)
    }
}

impl crate::validator_trait::Validator for QualityValidator {
    fn name(&self) -> &'static str {
        "quality"
    }

    fn description(&self) -> &'static str {
        "Validates code quality (no unwrap/expect)"
    }

    fn validate(&self, _config: &ValidationConfig) -> anyhow::Result<Vec<Box<dyn Violation>>> {
        let violations = self.validate_all()?;
        Ok(violations
            .into_iter()
            .map(|v| Box::new(v) as Box<dyn Violation>)
            .collect())
    }
}
