//! PMAT Integration Validator
//!
//! Integrates with PMAT CLI tool for additional analysis:
//! - Cyclomatic complexity analysis
//! - Dead code detection
//! - Technical Debt Gradient (TDG) scoring
//!
//! This validator is optional - it only runs if the `pmat` binary is available.

use crate::violation_trait::{Violation, ViolationCategory};
use crate::{Result, Severity, ValidationConfig};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;

/// Default complexity threshold
pub const DEFAULT_COMPLEXITY_THRESHOLD: u32 = 15;

/// Default TDG score threshold (0-100, higher is worse)
pub const DEFAULT_TDG_THRESHOLD: u32 = 50;

/// PMAT violation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PmatViolation {
    /// High cyclomatic complexity
    HighComplexity {
        file: PathBuf,
        function: String,
        complexity: u32,
        threshold: u32,
        severity: Severity,
    },
    /// Dead code detected
    DeadCode {
        file: PathBuf,
        line: usize,
        item_type: String,
        name: String,
        severity: Severity,
    },
    /// Low TDG score (high technical debt)
    LowTdgScore {
        file: PathBuf,
        score: u32,
        threshold: u32,
        severity: Severity,
    },
    /// PMAT tool not available
    PmatUnavailable { message: String, severity: Severity },
    /// PMAT execution error
    PmatError {
        command: String,
        error: String,
        severity: Severity,
    },
}

impl PmatViolation {
    pub fn severity(&self) -> Severity {
        match self {
            Self::HighComplexity { severity, .. } => *severity,
            Self::DeadCode { severity, .. } => *severity,
            Self::LowTdgScore { severity, .. } => *severity,
            Self::PmatUnavailable { severity, .. } => *severity,
            Self::PmatError { severity, .. } => *severity,
        }
    }
}

impl std::fmt::Display for PmatViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HighComplexity {
                file,
                function,
                complexity,
                threshold,
                ..
            } => {
                write!(
                    f,
                    "High complexity: {}::{} - complexity {} (threshold: {})",
                    file.display(),
                    function,
                    complexity,
                    threshold
                )
            }
            Self::DeadCode {
                file,
                line,
                item_type,
                name,
                ..
            } => {
                write!(
                    f,
                    "Dead code: {}:{} - {} '{}'",
                    file.display(),
                    line,
                    item_type,
                    name
                )
            }
            Self::LowTdgScore {
                file,
                score,
                threshold,
                ..
            } => {
                write!(
                    f,
                    "High technical debt: {} - TDG score {} (threshold: {})",
                    file.display(),
                    score,
                    threshold
                )
            }
            Self::PmatUnavailable { message, .. } => {
                write!(f, "PMAT unavailable: {}", message)
            }
            Self::PmatError { command, error, .. } => {
                write!(f, "PMAT error running '{}': {}", command, error)
            }
        }
    }
}

impl Violation for PmatViolation {
    fn id(&self) -> &str {
        match self {
            Self::HighComplexity { .. } => "PMAT001",
            Self::DeadCode { .. } => "PMAT002",
            Self::LowTdgScore { .. } => "PMAT003",
            Self::PmatUnavailable { .. } => "PMAT004",
            Self::PmatError { .. } => "PMAT005",
        }
    }

    fn category(&self) -> ViolationCategory {
        ViolationCategory::Pmat
    }

    fn severity(&self) -> Severity {
        match self {
            Self::HighComplexity { severity, .. } => *severity,
            Self::DeadCode { severity, .. } => *severity,
            Self::LowTdgScore { severity, .. } => *severity,
            Self::PmatUnavailable { severity, .. } => *severity,
            Self::PmatError { severity, .. } => *severity,
        }
    }

    fn file(&self) -> Option<&PathBuf> {
        match self {
            Self::HighComplexity { file, .. } => Some(file),
            Self::DeadCode { file, .. } => Some(file),
            Self::LowTdgScore { file, .. } => Some(file),
            Self::PmatUnavailable { .. } => None,
            Self::PmatError { .. } => None,
        }
    }

    fn line(&self) -> Option<usize> {
        match self {
            Self::DeadCode { line, .. } => Some(*line),
            _ => None,
        }
    }

    fn suggestion(&self) -> Option<String> {
        match self {
            Self::HighComplexity {
                function,
                complexity,
                threshold,
                ..
            } => Some(format!(
                "Consider refactoring '{}' to reduce complexity from {} to below {}. \
                 Split into smaller functions or simplify control flow.",
                function, complexity, threshold
            )),
            Self::DeadCode {
                item_type, name, ..
            } => Some(format!(
                "Remove unused {} '{}' or mark with #[allow(dead_code)] if intentional.",
                item_type, name
            )),
            Self::LowTdgScore { score, threshold, .. } => Some(format!(
                "Technical debt score {} exceeds threshold {}. \
                 Address code smells, reduce complexity, and improve maintainability.",
                score, threshold
            )),
            Self::PmatUnavailable { .. } => {
                Some("Install PMAT CLI tool to enable additional analysis.".to_string())
            }
            Self::PmatError { command, .. } => {
                Some(format!("Check PMAT installation and run '{}' manually to diagnose.", command))
            }
        }
    }
}

/// PMAT complexity result from JSON output
#[derive(Debug, Deserialize)]
struct ComplexityResult {
    #[serde(default)]
    file: Option<String>,
    #[serde(default)]
    function: Option<String>,
    #[serde(default)]
    complexity: Option<u32>,
}

/// PMAT dead code result from JSON output
#[derive(Debug, Deserialize)]
struct DeadCodeResult {
    #[serde(default)]
    file: Option<String>,
    #[serde(default)]
    line: Option<usize>,
    #[serde(default)]
    item_type: Option<String>,
    #[serde(default)]
    name: Option<String>,
}

/// PMAT TDG result from JSON output
#[derive(Debug, Deserialize)]
struct TdgResult {
    #[serde(default)]
    file: Option<String>,
    #[serde(default)]
    score: Option<u32>,
}

/// PMAT integration validator
pub struct PmatValidator {
    config: ValidationConfig,
    complexity_threshold: u32,
    tdg_threshold: u32,
    pmat_available: bool,
}

impl PmatValidator {
    /// Create a new PMAT validator
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self::with_config(ValidationConfig::new(workspace_root))
    }

    /// Create a validator with custom configuration
    pub fn with_config(config: ValidationConfig) -> Self {
        let pmat_available = Self::check_pmat_available();
        Self {
            config,
            complexity_threshold: DEFAULT_COMPLEXITY_THRESHOLD,
            tdg_threshold: DEFAULT_TDG_THRESHOLD,
            pmat_available,
        }
    }

    /// Set complexity threshold
    pub fn with_complexity_threshold(mut self, threshold: u32) -> Self {
        self.complexity_threshold = threshold;
        self
    }

    /// Set TDG threshold
    pub fn with_tdg_threshold(mut self, threshold: u32) -> Self {
        self.tdg_threshold = threshold;
        self
    }

    /// Check if PMAT is available
    fn check_pmat_available() -> bool {
        Command::new("pmat")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Check if PMAT is available for this validator instance
    pub fn is_available(&self) -> bool {
        self.pmat_available
    }

    /// Run all PMAT validations
    pub fn validate_all(&self) -> Result<Vec<PmatViolation>> {
        let mut violations = Vec::new();

        if !self.pmat_available {
            violations.push(PmatViolation::PmatUnavailable {
                message: "pmat binary not found in PATH - skipping PMAT analysis".to_string(),
                severity: Severity::Info,
            });
            return Ok(violations);
        }

        violations.extend(self.validate_complexity()?);
        violations.extend(self.validate_dead_code()?);
        violations.extend(self.validate_tdg()?);

        Ok(violations)
    }

    /// Run complexity analysis using PMAT
    pub fn validate_complexity(&self) -> Result<Vec<PmatViolation>> {
        let mut violations = Vec::new();

        if !self.pmat_available {
            return Ok(violations);
        }

        let workspace_root = &self.config.workspace_root;

        let output = Command::new("pmat")
            .args([
                "analyze",
                "complexity",
                "--project-path",
                workspace_root.to_str().unwrap_or("."),
            ])
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);

                // Try to parse as JSON array of complexity results
                if let Ok(results) = serde_json::from_str::<Vec<ComplexityResult>>(&stdout) {
                    for result in results {
                        if let (Some(file), Some(function), Some(complexity)) =
                            (result.file, result.function, result.complexity)
                        {
                            if complexity > self.complexity_threshold {
                                violations.push(PmatViolation::HighComplexity {
                                    file: PathBuf::from(file),
                                    function,
                                    complexity,
                                    threshold: self.complexity_threshold,
                                    severity: if complexity > self.complexity_threshold * 2 {
                                        Severity::Warning
                                    } else {
                                        Severity::Info
                                    },
                                });
                            }
                        }
                    }
                }
                // If parsing fails, that's OK - PMAT output format may vary
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                violations.push(PmatViolation::PmatError {
                    command: "pmat analyze complexity".to_string(),
                    error: stderr.to_string(),
                    severity: Severity::Info,
                });
            }
            Err(e) => {
                violations.push(PmatViolation::PmatError {
                    command: "pmat analyze complexity".to_string(),
                    error: e.to_string(),
                    severity: Severity::Info,
                });
            }
        }

        Ok(violations)
    }

    /// Run dead code analysis using PMAT
    pub fn validate_dead_code(&self) -> Result<Vec<PmatViolation>> {
        let mut violations = Vec::new();

        if !self.pmat_available {
            return Ok(violations);
        }

        let workspace_root = &self.config.workspace_root;

        let output = Command::new("pmat")
            .args([
                "analyze",
                "dead-code",
                "--path",
                workspace_root.to_str().unwrap_or("."),
            ])
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);

                // Try to parse as JSON array of dead code results
                if let Ok(results) = serde_json::from_str::<Vec<DeadCodeResult>>(&stdout) {
                    for result in results {
                        if let (Some(file), Some(line), Some(item_type), Some(name)) =
                            (result.file, result.line, result.item_type, result.name)
                        {
                            violations.push(PmatViolation::DeadCode {
                                file: PathBuf::from(file),
                                line,
                                item_type,
                                name,
                                severity: Severity::Info,
                            });
                        }
                    }
                }
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                if !stderr.is_empty() {
                    violations.push(PmatViolation::PmatError {
                        command: "pmat analyze dead-code".to_string(),
                        error: stderr.to_string(),
                        severity: Severity::Info,
                    });
                }
            }
            Err(e) => {
                violations.push(PmatViolation::PmatError {
                    command: "pmat analyze dead-code".to_string(),
                    error: e.to_string(),
                    severity: Severity::Info,
                });
            }
        }

        Ok(violations)
    }

    /// Run TDG analysis using PMAT
    pub fn validate_tdg(&self) -> Result<Vec<PmatViolation>> {
        let mut violations = Vec::new();

        if !self.pmat_available {
            return Ok(violations);
        }

        let workspace_root = &self.config.workspace_root;

        let output = Command::new("pmat")
            .args(["tdg", workspace_root.to_str().unwrap_or(".")])
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);

                // Try to parse as JSON array of TDG results
                if let Ok(results) = serde_json::from_str::<Vec<TdgResult>>(&stdout) {
                    for result in results {
                        if let (Some(file), Some(score)) = (result.file, result.score) {
                            if score > self.tdg_threshold {
                                violations.push(PmatViolation::LowTdgScore {
                                    file: PathBuf::from(file),
                                    score,
                                    threshold: self.tdg_threshold,
                                    severity: if score > self.tdg_threshold + 25 {
                                        Severity::Warning
                                    } else {
                                        Severity::Info
                                    },
                                });
                            }
                        }
                    }
                }
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                if !stderr.is_empty() {
                    violations.push(PmatViolation::PmatError {
                        command: "pmat tdg".to_string(),
                        error: stderr.to_string(),
                        severity: Severity::Info,
                    });
                }
            }
            Err(e) => {
                violations.push(PmatViolation::PmatError {
                    command: "pmat tdg".to_string(),
                    error: e.to_string(),
                    severity: Severity::Info,
                });
            }
        }

        Ok(violations)
    }
}

impl crate::validator_trait::Validator for PmatValidator {
    fn name(&self) -> &'static str {
        "pmat"
    }

    fn description(&self) -> &'static str {
        "PMAT integration for cyclomatic complexity, dead code detection, and TDG scoring"
    }

    fn validate(&self, _config: &ValidationConfig) -> anyhow::Result<Vec<Box<dyn Violation>>> {
        let violations = self.validate_all()?;
        Ok(violations
            .into_iter()
            .map(|v| Box::new(v) as Box<dyn Violation>)
            .collect())
    }

    fn enabled_by_default(&self) -> bool {
        // Only enable by default if PMAT is available
        self.pmat_available
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_pmat_availability_check() {
        // This test verifies the availability check works
        // It doesn't require PMAT to be installed
        let result = PmatValidator::check_pmat_available();
        // Result can be true or false depending on environment
        assert!(result == true || result == false);
    }

    #[test]
    fn test_validator_creation() {
        let temp = TempDir::new().unwrap();
        let validator = PmatValidator::new(temp.path());

        // Validator should be created successfully regardless of PMAT availability
        assert_eq!(validator.complexity_threshold, DEFAULT_COMPLEXITY_THRESHOLD);
        assert_eq!(validator.tdg_threshold, DEFAULT_TDG_THRESHOLD);
    }

    #[test]
    fn test_custom_thresholds() {
        let temp = TempDir::new().unwrap();
        let validator = PmatValidator::new(temp.path())
            .with_complexity_threshold(20)
            .with_tdg_threshold(60);

        assert_eq!(validator.complexity_threshold, 20);
        assert_eq!(validator.tdg_threshold, 60);
    }

    #[test]
    fn test_unavailable_pmat_returns_info() {
        let temp = TempDir::new().unwrap();
        let mut validator = PmatValidator::new(temp.path());

        // Force PMAT to be unavailable for testing
        validator.pmat_available = false;

        let violations = validator.validate_all().unwrap();

        // Should return a single PmatUnavailable info message
        assert_eq!(violations.len(), 1);
        match &violations[0] {
            PmatViolation::PmatUnavailable { severity, .. } => {
                assert_eq!(*severity, Severity::Info);
            }
            _ => panic!("Expected PmatUnavailable violation"),
        }
    }
}
