//! Generic Reporter
//!
//! Generates reports from violations implementing the Violation trait.
//! Supports multiple output formats: human-readable, JSON, and CI (GitHub Actions).

use crate::violation_trait::{Violation, ViolationCategory};
use crate::Severity;
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;

/// Report containing all violations with summary
#[derive(Debug, Clone, Serialize)]
pub struct GenericReport {
    /// Timestamp of the validation run
    pub timestamp: String,
    /// Workspace root path
    pub workspace_root: PathBuf,
    /// Summary statistics
    pub summary: GenericSummary,
    /// All violations grouped by category
    pub violations_by_category: HashMap<String, Vec<ViolationEntry>>,
}

/// Summary of validation results
#[derive(Debug, Clone, Serialize)]
pub struct GenericSummary {
    /// Total number of violations
    pub total_violations: usize,
    /// Number of errors
    pub errors: usize,
    /// Number of warnings
    pub warnings: usize,
    /// Number of info messages
    pub infos: usize,
    /// Violations per category
    pub by_category: HashMap<String, usize>,
    /// Whether validation passed (no error-level violations)
    pub passed: bool,
}

/// Serializable violation entry
#[derive(Debug, Clone, Serialize)]
pub struct ViolationEntry {
    /// Unique violation ID
    pub id: String,
    /// Category
    pub category: String,
    /// Severity
    pub severity: String,
    /// File path (if applicable)
    pub file: Option<PathBuf>,
    /// Line number (if applicable)
    pub line: Option<usize>,
    /// Human-readable message
    pub message: String,
    /// Suggested fix (if applicable)
    pub suggestion: Option<String>,
}

impl ViolationEntry {
    /// Create from a Violation trait object
    pub fn from_violation(v: &dyn Violation) -> Self {
        Self {
            id: v.id().to_string(),
            category: v.category().to_string(),
            severity: v.severity().to_string(),
            file: v.file().cloned(),
            line: v.line(),
            message: v.message(),
            suggestion: v.suggestion(),
        }
    }
}

/// Generic reporter for violations
pub struct GenericReporter;

impl GenericReporter {
    /// Create a report from violations
    pub fn create_report(
        violations: &[Box<dyn Violation>],
        workspace_root: PathBuf,
    ) -> GenericReport {
        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();

        // Count by severity
        let errors = violations
            .iter()
            .filter(|v| v.severity() == Severity::Error)
            .count();
        let warnings = violations
            .iter()
            .filter(|v| v.severity() == Severity::Warning)
            .count();
        let infos = violations
            .iter()
            .filter(|v| v.severity() == Severity::Info)
            .count();

        // Group by category
        let mut by_category: HashMap<String, Vec<ViolationEntry>> = HashMap::new();
        let mut category_counts: HashMap<String, usize> = HashMap::new();

        for v in violations {
            let category_name = v.category().to_string();
            let entry = ViolationEntry::from_violation(v.as_ref());

            by_category
                .entry(category_name.clone())
                .or_default()
                .push(entry);

            *category_counts.entry(category_name).or_default() += 1;
        }

        GenericReport {
            timestamp,
            workspace_root,
            summary: GenericSummary {
                total_violations: violations.len(),
                errors,
                warnings,
                infos,
                by_category: category_counts,
                passed: errors == 0,
            },
            violations_by_category: by_category,
        }
    }

    /// Generate JSON report
    pub fn to_json(violations: &[Box<dyn Violation>], workspace_root: PathBuf) -> String {
        let report = Self::create_report(violations, workspace_root);
        serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_string())
    }

    /// Generate human-readable report
    pub fn to_human_readable(violations: &[Box<dyn Violation>], workspace_root: PathBuf) -> String {
        let report = Self::create_report(violations, workspace_root);
        let mut output = String::new();

        output.push_str("=== Architecture Validation Report ===\n\n");
        output.push_str(&format!("Timestamp: {}\n", report.timestamp));
        output.push_str(&format!("Workspace: {}\n\n", report.workspace_root.display()));

        // Summary
        output.push_str("--- Summary ---\n");
        output.push_str(&format!(
            "Total Violations: {} ({} errors, {} warnings, {} info)\n",
            report.summary.total_violations,
            report.summary.errors,
            report.summary.warnings,
            report.summary.infos
        ));
        output.push_str(&format!(
            "Status: {}\n\n",
            if report.summary.passed {
                "PASSED"
            } else {
                "FAILED"
            }
        ));

        // By category
        if !report.violations_by_category.is_empty() {
            output.push_str("--- Violations by Category ---\n\n");

            // Sort categories for consistent output
            let mut categories: Vec<_> = report.violations_by_category.keys().collect();
            categories.sort();

            for category in categories {
                let violations = &report.violations_by_category[category];
                if violations.is_empty() {
                    continue;
                }

                output.push_str(&format!("=== {} ({}) ===\n", category, violations.len()));

                for v in violations {
                    let location = match (&v.file, v.line) {
                        (Some(f), Some(l)) => format!("{}:{}", f.display(), l),
                        (Some(f), None) => f.display().to_string(),
                        _ => "unknown".to_string(),
                    };

                    output.push_str(&format!(
                        "  [{:>7}] [{}] {} - {}\n",
                        v.severity, v.id, location, v.message
                    ));

                    if let Some(ref suggestion) = v.suggestion {
                        output.push_str(&format!("            -> {}\n", suggestion));
                    }
                }
                output.push('\n');
            }
        }

        output
    }

    /// Generate CI summary (GitHub Actions format)
    pub fn to_ci_summary(violations: &[Box<dyn Violation>]) -> String {
        let mut output = String::new();

        for v in violations {
            match v.severity() {
                Severity::Error => {
                    if let (Some(file), Some(line)) = (v.file(), v.line()) {
                        output.push_str(&format!(
                            "::error file={},line={}::[{}] {}\n",
                            file.display(),
                            line,
                            v.id(),
                            v.message()
                        ));
                    } else if let Some(file) = v.file() {
                        output.push_str(&format!(
                            "::error file={}::[{}] {}\n",
                            file.display(),
                            v.id(),
                            v.message()
                        ));
                    } else {
                        output.push_str(&format!("::error ::[{}] {}\n", v.id(), v.message()));
                    }
                }
                Severity::Warning => {
                    if let (Some(file), Some(line)) = (v.file(), v.line()) {
                        output.push_str(&format!(
                            "::warning file={},line={}::[{}] {}\n",
                            file.display(),
                            line,
                            v.id(),
                            v.message()
                        ));
                    } else if let Some(file) = v.file() {
                        output.push_str(&format!(
                            "::warning file={}::[{}] {}\n",
                            file.display(),
                            v.id(),
                            v.message()
                        ));
                    } else {
                        output.push_str(&format!("::warning ::[{}] {}\n", v.id(), v.message()));
                    }
                }
                Severity::Info => {
                    // Info messages are not reported in CI
                }
            }
        }

        output
    }

    /// Count violations by severity
    pub fn count_by_severity(violations: &[Box<dyn Violation>]) -> (usize, usize, usize) {
        let errors = violations
            .iter()
            .filter(|v| v.severity() == Severity::Error)
            .count();
        let warnings = violations
            .iter()
            .filter(|v| v.severity() == Severity::Warning)
            .count();
        let infos = violations
            .iter()
            .filter(|v| v.severity() == Severity::Info)
            .count();
        (errors, warnings, infos)
    }

    /// Filter violations by category
    pub fn filter_by_category(
        violations: Vec<Box<dyn Violation>>,
        category: ViolationCategory,
    ) -> Vec<Box<dyn Violation>> {
        violations
            .into_iter()
            .filter(|v| v.category() == category)
            .collect()
    }

    /// Filter violations by severity
    pub fn filter_by_severity(
        violations: Vec<Box<dyn Violation>>,
        severity: Severity,
    ) -> Vec<Box<dyn Violation>> {
        violations
            .into_iter()
            .filter(|v| v.severity() == severity)
            .collect()
    }
}
