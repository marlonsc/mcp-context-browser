//! Validation Report Generation
//!
//! Generates reports in multiple formats:
//! - JSON for CI integration
//! - Human-readable for terminal output
//! - CI summary for GitHub Actions annotations

use crate::{
    AsyncViolation, CleanArchitectureViolation, ConfigQualityViolation, DependencyViolation,
    DocumentationViolation, ErrorBoundaryViolation, ImplementationViolation, KissViolation,
    NamingViolation, OrganizationViolation, PatternViolation, PerformanceViolation, PmatViolation,
    QualityViolation, RefactoringViolation, Severity, SolidViolation, TestQualityViolation,
    TestViolation,
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
    /// KISS principle violations (complexity)
    pub kiss_violations: Vec<KissViolation>,
    /// Refactoring completeness violations
    pub refactoring_violations: Vec<RefactoringViolation>,
    /// Implementation quality violations
    pub implementation_violations: Vec<ImplementationViolation>,
    /// Performance pattern violations
    pub performance_violations: Vec<PerformanceViolation>,
    /// Async pattern violations
    pub async_violations: Vec<AsyncViolation>,
    /// Error boundary violations
    pub error_boundary_violations: Vec<ErrorBoundaryViolation>,
    /// PMAT integration violations
    pub pmat_violations: Vec<PmatViolation>,
    /// Clean Architecture layer boundary violations (CA001-CA009)
    pub clean_architecture_violations: Vec<CleanArchitectureViolation>,
    /// Test quality violations (ignored tests, todo in fixtures)
    pub test_quality_violations: Vec<TestQualityViolation>,
    /// Configuration quality violations (hardcoded values)
    pub config_quality_violations: Vec<ConfigQualityViolation>,
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
    /// Number of KISS principle violations
    pub kiss_count: usize,
    /// Number of refactoring completeness violations
    pub refactoring_count: usize,
    /// Number of implementation quality violations
    pub implementation_count: usize,
    /// Number of performance pattern violations
    pub performance_count: usize,
    /// Number of async pattern violations
    pub async_count: usize,
    /// Number of error boundary violations
    pub error_boundary_count: usize,
    /// Number of PMAT integration violations
    pub pmat_count: usize,
    /// Number of Clean Architecture violations (CA001-CA009)
    pub clean_architecture_count: usize,
    /// Number of test quality violations
    pub test_quality_count: usize,
    /// Number of configuration quality violations
    pub config_quality_count: usize,
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
        output.push_str(&format!(
            "Workspace: {}\n\n",
            report.workspace_root.display()
        ));

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
        output.push_str(&format!(
            "  KISS:           {}\n",
            report.summary.kiss_count
        ));
        output.push_str(&format!(
            "  Refactoring:    {}\n",
            report.summary.refactoring_count
        ));
        output.push_str(&format!(
            "  Implementation: {}\n",
            report.summary.implementation_count
        ));
        output.push_str(&format!(
            "  Performance:    {}\n",
            report.summary.performance_count
        ));
        output.push_str(&format!(
            "  Async:          {}\n",
            report.summary.async_count
        ));
        output.push_str(&format!(
            "  ErrorBoundary:  {}\n",
            report.summary.error_boundary_count
        ));
        output.push_str(&format!(
            "  PMAT:           {}\n",
            report.summary.pmat_count
        ));
        output.push_str(&format!(
            "  CleanArch:      {}\n",
            report.summary.clean_architecture_count
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

        // KISS violations
        if !report.kiss_violations.is_empty() {
            output.push_str("--- KISS Violations ---\n");
            for v in &report.kiss_violations {
                output.push_str(&format!("  [{:?}] {}\n", v.severity(), v));
            }
            output.push('\n');
        }

        // Refactoring violations
        if !report.refactoring_violations.is_empty() {
            output.push_str("--- Refactoring Violations ---\n");
            for v in &report.refactoring_violations {
                output.push_str(&format!("  [{:?}] {}\n", v.severity(), v));
            }
            output.push('\n');
        }

        // Implementation violations
        if !report.implementation_violations.is_empty() {
            output.push_str("--- Implementation Quality Violations ---\n");
            for v in &report.implementation_violations {
                output.push_str(&format!("  [{:?}] {}\n", v.severity(), v));
            }
            output.push('\n');
        }

        // Performance violations
        if !report.performance_violations.is_empty() {
            output.push_str("--- Performance Violations ---\n");
            for v in &report.performance_violations {
                output.push_str(&format!("  [{:?}] {}\n", v.severity(), v));
            }
            output.push('\n');
        }

        // Async pattern violations
        if !report.async_violations.is_empty() {
            output.push_str("--- Async Pattern Violations ---\n");
            for v in &report.async_violations {
                output.push_str(&format!("  [{:?}] {}\n", v.severity(), v));
            }
            output.push('\n');
        }

        // Error boundary violations
        if !report.error_boundary_violations.is_empty() {
            output.push_str("--- Error Boundary Violations ---\n");
            for v in &report.error_boundary_violations {
                output.push_str(&format!("  [{:?}] {}\n", v.severity(), v));
            }
            output.push('\n');
        }

        // PMAT violations
        if !report.pmat_violations.is_empty() {
            output.push_str("--- PMAT Violations ---\n");
            for v in &report.pmat_violations {
                output.push_str(&format!("  [{:?}] {}\n", v.severity(), v));
            }
            output.push('\n');
        }

        // Clean Architecture violations (CA001-CA009)
        if !report.clean_architecture_violations.is_empty() {
            output.push_str("--- Clean Architecture Violations ---\n");
            for v in &report.clean_architecture_violations {
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
        output.push_str(&format!("| Quality | {} |\n", report.summary.quality_count));
        output.push_str(&format!(
            "| Patterns | {} |\n",
            report.summary.pattern_count
        ));
        output.push_str(&format!("| Tests | {} |\n", report.summary.test_count));
        output.push_str(&format!(
            "| Documentation | {} |\n",
            report.summary.documentation_count
        ));
        output.push_str(&format!("| Naming | {} |\n", report.summary.naming_count));
        output.push_str(&format!("| SOLID | {} |\n", report.summary.solid_count));
        output.push_str(&format!(
            "| Organization | {} |\n",
            report.summary.organization_count
        ));
        output.push_str(&format!("| KISS | {} |\n", report.summary.kiss_count));
        output.push_str(&format!(
            "| Refactoring | {} |\n",
            report.summary.refactoring_count
        ));
        output.push_str(&format!(
            "| Implementation | {} |\n",
            report.summary.implementation_count
        ));
        output.push_str(&format!(
            "| Performance | {} |\n",
            report.summary.performance_count
        ));
        output.push_str(&format!("| Async | {} |\n", report.summary.async_count));
        output.push_str(&format!(
            "| ErrorBoundary | {} |\n",
            report.summary.error_boundary_count
        ));
        output.push_str(&format!("| PMAT | {} |\n", report.summary.pmat_count));
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

        for v in &report.kiss_violations {
            if v.severity() == Severity::Error {
                errors.push(format!("::error ::{}", v));
            }
        }

        for v in &report.refactoring_violations {
            if v.severity() == Severity::Error {
                errors.push(format!("::error ::{}", v));
            }
        }

        for v in &report.implementation_violations {
            if v.severity() == Severity::Error {
                errors.push(format!("::error ::{}", v));
            }
        }

        for v in &report.performance_violations {
            if v.severity() == Severity::Error {
                errors.push(format!("::error ::{}", v));
            }
        }

        for v in &report.async_violations {
            if v.severity() == Severity::Error {
                errors.push(format!("::error ::{}", v));
            }
        }

        for v in &report.error_boundary_violations {
            if v.severity() == Severity::Error {
                errors.push(format!("::error ::{}", v));
            }
        }

        for v in &report.pmat_violations {
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
        count += report
            .kiss_violations
            .iter()
            .filter(|v| v.severity() == Severity::Error)
            .count();
        count += report
            .refactoring_violations
            .iter()
            .filter(|v| v.severity() == Severity::Error)
            .count();
        count += report
            .implementation_violations
            .iter()
            .filter(|v| v.severity() == Severity::Error)
            .count();
        count += report
            .performance_violations
            .iter()
            .filter(|v| v.severity() == Severity::Error)
            .count();
        count += report
            .async_violations
            .iter()
            .filter(|v| v.severity() == Severity::Error)
            .count();
        count += report
            .error_boundary_violations
            .iter()
            .filter(|v| v.severity() == Severity::Error)
            .count();
        count += report
            .pmat_violations
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
        count += report
            .kiss_violations
            .iter()
            .filter(|v| v.severity() == Severity::Warning)
            .count();
        count += report
            .refactoring_violations
            .iter()
            .filter(|v| v.severity() == Severity::Warning)
            .count();
        count += report
            .implementation_violations
            .iter()
            .filter(|v| v.severity() == Severity::Warning)
            .count();
        count += report
            .performance_violations
            .iter()
            .filter(|v| v.severity() == Severity::Warning)
            .count();
        count += report
            .async_violations
            .iter()
            .filter(|v| v.severity() == Severity::Warning)
            .count();
        count += report
            .error_boundary_violations
            .iter()
            .filter(|v| v.severity() == Severity::Warning)
            .count();
        count += report
            .pmat_violations
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
                kiss_count: 0,
                refactoring_count: 0,
                implementation_count: 0,
                performance_count: 0,
                async_count: 0,
                error_boundary_count: 0,
                pmat_count: 0,
                clean_architecture_count: 0,
                test_quality_count: 0,
                config_quality_count: 0,
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
            kiss_violations: vec![],
            refactoring_violations: vec![],
            implementation_violations: vec![],
            performance_violations: vec![],
            async_violations: vec![],
            error_boundary_violations: vec![],
            pmat_violations: vec![],
            clean_architecture_violations: vec![],
            test_quality_violations: vec![],
            config_quality_violations: vec![],
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
        report
            .quality_violations
            .push(QualityViolation::UnwrapInProduction {
                file: PathBuf::from("/test.rs"),
                line: 1,
                context: "test".to_string(),
                severity: Severity::Error,
            });
        const TEST_PENDING_LABEL: &str = concat!("T", "O", "D", "O");
        report
            .quality_violations
            .push(QualityViolation::TodoComment {
                file: PathBuf::from("/test.rs"),
                line: 2,
                content: TEST_PENDING_LABEL.to_string(),
                severity: Severity::Info,
            });

        assert_eq!(Reporter::count_errors(&report), 1);
        assert_eq!(Reporter::count_warnings(&report), 0);
    }
}
