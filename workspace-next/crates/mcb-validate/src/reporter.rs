//! Validation Report Generation
//!
//! Generates reports in multiple formats:
//! - JSON for CI integration
//! - Human-readable for terminal output
//! - CI summary for GitHub Actions annotations

use crate::{
    DependencyViolation, DocumentationViolation, NamingViolation, OrganizationViolation,
    PatternViolation, QualityViolation, Severity, SolidViolation, TestViolation,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Validation report containing all violations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    /// Timestamp of the validation run
    pub timestamp: String,
    /// Workspace root path
    pub workspace_root: PathBuf,
    /// Summary statistics
    pub summary: ValidationSummary,
    /// Dependency violations
    pub dependency_violations: Vec<DependencyViolation>,
    /// Quality violations
    pub quality_violations: Vec<QualityViolation>,
    /// Pattern violations
    pub pattern_violations: Vec<PatternViolation>,
    /// Test organization violations
    pub test_violations: Vec<TestViolation>,
    /// Documentation violations
    pub documentation_violations: Vec<DocumentationViolation>,
    /// Naming violations
    pub naming_violations: Vec<NamingViolation>,
    /// SOLID principle violations
    pub solid_violations: Vec<SolidViolation>,
    /// Organization violations (file placement, centralization)
    pub organization_violations: Vec<OrganizationViolation>,
}

/// Summary of validation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSummary {
    /// Total number of violations
    pub total_violations: usize,
    /// Number of dependency violations
    pub dependency_count: usize,
    /// Number of quality violations
    pub quality_count: usize,
    /// Number of pattern violations
    pub pattern_count: usize,
    /// Number of test organization violations
    pub test_count: usize,
    /// Number of documentation violations
    pub documentation_count: usize,
    /// Number of naming violations
    pub naming_count: usize,
    /// Number of SOLID principle violations
    pub solid_count: usize,
    /// Number of organization violations
    pub organization_count: usize,
    /// Whether validation passed (no error-level violations)
    pub passed: bool,
}

/// Report generator
pub struct Reporter;

impl Reporter {
    /// Generate JSON report
    pub fn to_json(report: &ValidationReport) -> String {
        serde_json::to_string_pretty(report).unwrap_or_else(|_| "{}".to_string())
    }

    /// Generate human-readable report
    pub fn to_human_readable(report: &ValidationReport) -> String {
        let mut output = String::new();

        output.push_str("=== Architecture Validation Report ===\n\n");
        output.push_str(&format!("Timestamp: {}\n", report.timestamp));
        output.push_str(&format!("Workspace: {}\n\n", report.workspace_root.display()));

        // Summary
        output.push_str("--- Summary ---\n");
        output.push_str(&format!(
            "Total Violations: {}\n",
            report.summary.total_violations
        ));
        output.push_str(&format!(
            "  Dependency:     {}\n",
            report.summary.dependency_count
        ));
        output.push_str(&format!(
            "  Quality:        {}\n",
            report.summary.quality_count
        ));
        output.push_str(&format!(
            "  Patterns:       {}\n",
            report.summary.pattern_count
        ));
        output.push_str(&format!(
            "  Tests:          {}\n",
            report.summary.test_count
        ));
        output.push_str(&format!(
            "  Documentation:  {}\n",
            report.summary.documentation_count
        ));
        output.push_str(&format!(
            "  Naming:         {}\n",
            report.summary.naming_count
        ));
        output.push_str(&format!(
            "  SOLID:          {}\n",
            report.summary.solid_count
        ));
        output.push_str(&format!(
            "  Organization:   {}\n",
            report.summary.organization_count
        ));
        output.push('\n');

        let status = if report.summary.passed {
            "PASSED"
        } else {
            "FAILED"
        };
        output.push_str(&format!("Status: {}\n\n", status));

        // Dependency violations
        if !report.dependency_violations.is_empty() {
            output.push_str("--- Dependency Violations ---\n");
            for v in &report.dependency_violations {
                output.push_str(&format!("  [{:?}] {}\n", v.severity(), v));
            }
            output.push('\n');
        }

        // Quality violations
        if !report.quality_violations.is_empty() {
            output.push_str("--- Quality Violations ---\n");
            for v in &report.quality_violations {
                output.push_str(&format!("  [{:?}] {}\n", v.severity(), v));
            }
            output.push('\n');
        }

        // Pattern violations
        if !report.pattern_violations.is_empty() {
            output.push_str("--- Pattern Violations ---\n");
            for v in &report.pattern_violations {
                output.push_str(&format!("  [{:?}] {}\n", v.severity(), v));
            }
            output.push('\n');
        }

        // Test violations
        if !report.test_violations.is_empty() {
            output.push_str("--- Test Organization Violations ---\n");
            for v in &report.test_violations {
                output.push_str(&format!("  [{:?}] {}\n", v.severity(), v));
            }
            output.push('\n');
        }

        // Documentation violations
        if !report.documentation_violations.is_empty() {
            output.push_str("--- Documentation Violations ---\n");
            for v in &report.documentation_violations {
                output.push_str(&format!("  [{:?}] {}\n", v.severity(), v));
            }
            output.push('\n');
        }

        // Naming violations
        if !report.naming_violations.is_empty() {
            output.push_str("--- Naming Violations ---\n");
            for v in &report.naming_violations {
                output.push_str(&format!("  [{:?}] {}\n", v.severity(), v));
            }
            output.push('\n');
        }

        // SOLID violations
        if !report.solid_violations.is_empty() {
            output.push_str("--- SOLID Violations ---\n");
            for v in &report.solid_violations {
                output.push_str(&format!("  [{:?}] {}\n", v.severity(), v));
            }
            output.push('\n');
        }

        // Organization violations
        if !report.organization_violations.is_empty() {
            output.push_str("--- Organization Violations ---\n");
            for v in &report.organization_violations {
                output.push_str(&format!("  [{:?}] {}\n", v.severity(), v));
            }
            output.push('\n');
        }

        output
    }

    /// Generate CI summary (GitHub Actions format)
    pub fn to_ci_summary(report: &ValidationReport) -> String {
        let mut output = String::new();

        output.push_str("## Architecture Validation\n\n");

        // Status badge
        if report.summary.passed {
            output.push_str("**Status:** :white_check_mark: PASSED\n\n");
        } else {
            output.push_str("**Status:** :x: FAILED\n\n");
        }

        // Summary table
        output.push_str("| Category | Count |\n");
        output.push_str("|----------|-------|\n");
        output.push_str(&format!(
            "| Dependency | {} |\n",
            report.summary.dependency_count
        ));
        output.push_str(&format!(
            "| Quality | {} |\n",
            report.summary.quality_count
        ));
        output.push_str(&format!(
            "| Patterns | {} |\n",
            report.summary.pattern_count
        ));
        output.push_str(&format!(
            "| Tests | {} |\n",
            report.summary.test_count
        ));
        output.push_str(&format!(
            "| Documentation | {} |\n",
            report.summary.documentation_count
        ));
        output.push_str(&format!(
            "| Naming | {} |\n",
            report.summary.naming_count
        ));
        output.push_str(&format!(
            "| SOLID | {} |\n",
            report.summary.solid_count
        ));
        output.push_str(&format!(
            "| Organization | {} |\n",
            report.summary.organization_count
        ));
        output.push_str(&format!(
            "| **Total** | **{}** |\n",
            report.summary.total_violations
        ));
        output.push('\n');

        // Error-level violations as annotations
        let mut errors = Vec::new();

        for v in &report.dependency_violations {
            if v.severity() == Severity::Error {
                errors.push(format!("::error ::{}", v));
            }
        }

        for v in &report.quality_violations {
            if v.severity() == Severity::Error {
                errors.push(format!("::error ::{}", v));
            }
        }

        for v in &report.pattern_violations {
            if v.severity() == Severity::Error {
                errors.push(format!("::error ::{}", v));
            }
        }

        for v in &report.test_violations {
            if v.severity() == Severity::Error {
                errors.push(format!("::error ::{}", v));
            }
        }

        for v in &report.documentation_violations {
            if v.severity() == Severity::Error {
                errors.push(format!("::error ::{}", v));
            }
        }

        for v in &report.naming_violations {
            if v.severity() == Severity::Error {
                errors.push(format!("::error ::{}", v));
            }
        }

        for v in &report.solid_violations {
            if v.severity() == Severity::Error {
                errors.push(format!("::error ::{}", v));
            }
        }

        for v in &report.organization_violations {
            if v.severity() == Severity::Error {
                errors.push(format!("::error ::{}", v));
            }
        }

        if !errors.is_empty() {
            output.push_str("\n### Errors\n\n");
            for e in errors {
                output.push_str(&format!("{}\n", e));
            }
        }

        output
    }

    /// Count error-level violations
    pub fn count_errors(report: &ValidationReport) -> usize {
        let mut count = 0;

        count += report
            .dependency_violations
            .iter()
            .filter(|v| v.severity() == Severity::Error)
            .count();
        count += report
            .quality_violations
            .iter()
            .filter(|v| v.severity() == Severity::Error)
            .count();
        count += report
            .pattern_violations
            .iter()
            .filter(|v| v.severity() == Severity::Error)
            .count();
        count += report
            .test_violations
            .iter()
            .filter(|v| v.severity() == Severity::Error)
            .count();
        count += report
            .documentation_violations
            .iter()
            .filter(|v| v.severity() == Severity::Error)
            .count();
        count += report
            .naming_violations
            .iter()
            .filter(|v| v.severity() == Severity::Error)
            .count();
        count += report
            .solid_violations
            .iter()
            .filter(|v| v.severity() == Severity::Error)
            .count();
        count += report
            .organization_violations
            .iter()
            .filter(|v| v.severity() == Severity::Error)
            .count();

        count
    }

    /// Count warning-level violations
    pub fn count_warnings(report: &ValidationReport) -> usize {
        let mut count = 0;

        count += report
            .dependency_violations
            .iter()
            .filter(|v| v.severity() == Severity::Warning)
            .count();
        count += report
            .quality_violations
            .iter()
            .filter(|v| v.severity() == Severity::Warning)
            .count();
        count += report
            .pattern_violations
            .iter()
            .filter(|v| v.severity() == Severity::Warning)
            .count();
        count += report
            .test_violations
            .iter()
            .filter(|v| v.severity() == Severity::Warning)
            .count();
        count += report
            .documentation_violations
            .iter()
            .filter(|v| v.severity() == Severity::Warning)
            .count();
        count += report
            .naming_violations
            .iter()
            .filter(|v| v.severity() == Severity::Warning)
            .count();
        count += report
            .solid_violations
            .iter()
            .filter(|v| v.severity() == Severity::Warning)
            .count();
        count += report
            .organization_violations
            .iter()
            .filter(|v| v.severity() == Severity::Warning)
            .count();

        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_empty_report() -> ValidationReport {
        ValidationReport {
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            workspace_root: PathBuf::from("/test"),
            summary: ValidationSummary {
                total_violations: 0,
                dependency_count: 0,
                quality_count: 0,
                pattern_count: 0,
                test_count: 0,
                documentation_count: 0,
                naming_count: 0,
                solid_count: 0,
                organization_count: 0,
                passed: true,
            },
            dependency_violations: vec![],
            quality_violations: vec![],
            pattern_violations: vec![],
            test_violations: vec![],
            documentation_violations: vec![],
            naming_violations: vec![],
            solid_violations: vec![],
            organization_violations: vec![],
        }
    }

    #[test]
    fn test_json_output() {
        let report = create_empty_report();
        let json = Reporter::to_json(&report);

        assert!(json.contains("timestamp"));
        assert!(json.contains("summary"));
        assert!(json.contains("passed"));
    }

    #[test]
    fn test_human_readable_output() {
        let report = create_empty_report();
        let output = Reporter::to_human_readable(&report);

        assert!(output.contains("Architecture Validation Report"));
        assert!(output.contains("Summary"));
        assert!(output.contains("PASSED"));
    }

    #[test]
    fn test_ci_summary_output() {
        let report = create_empty_report();
        let output = Reporter::to_ci_summary(&report);

        assert!(output.contains("Architecture Validation"));
        assert!(output.contains(":white_check_mark:"));
        assert!(output.contains("| Category | Count |"));
    }

    #[test]
    fn test_error_counting() {
        let mut report = create_empty_report();
        report.quality_violations.push(QualityViolation::UnwrapInProduction {
            file: PathBuf::from("/test.rs"),
            line: 1,
            context: "test".to_string(),
            severity: Severity::Error,
        });
        report.quality_violations.push(QualityViolation::TodoComment {
            file: PathBuf::from("/test.rs"),
            line: 2,
            content: "TODO".to_string(),
            severity: Severity::Info,
        });

        assert_eq!(Reporter::count_errors(&report), 1);
        assert_eq!(Reporter::count_warnings(&report), 0);
    }
}
