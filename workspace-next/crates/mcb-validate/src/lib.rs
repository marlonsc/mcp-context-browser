//! Architecture Validation for MCP Context Browser
//!
//! This crate provides comprehensive validation of workspace crates against:
//! - Clean Architecture principles (dependency direction)
//! - Code quality standards (no unwrap/expect in production)
//! - Professional patterns (DI, async traits, error types)
//! - Test organization (no inline tests)
//! - Documentation completeness
//! - Naming conventions
//! - SOLID principles (SRP, OCP, LSP, ISP, DIP)
//! - Code organization (constants centralization, file placement)

pub mod dependency;
pub mod documentation;
pub mod naming;
pub mod organization;
pub mod patterns;
pub mod quality;
pub mod reporter;
pub mod solid;
pub mod tests_org;

use std::path::{Path, PathBuf};
use thiserror::Error;

pub use dependency::{DependencyValidator, DependencyViolation};
pub use documentation::{DocumentationValidator, DocumentationViolation};
pub use naming::{NamingValidator, NamingViolation};
pub use organization::{OrganizationValidator, OrganizationViolation};
pub use patterns::{PatternValidator, PatternViolation};
pub use quality::{QualityValidator, QualityViolation};
pub use reporter::{Reporter, ValidationReport, ValidationSummary};
pub use solid::{SolidValidator, SolidViolation};
pub use tests_org::{TestValidator, TestViolation};

/// Result type for validation operations
pub type Result<T> = std::result::Result<T, ValidationError>;

/// Validation error types
#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error in {file}: {message}")]
    Parse { file: PathBuf, message: String },

    #[error("TOML parse error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("Configuration error: {0}")]
    Config(String),
}

/// Severity level for violations
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// Main validator that orchestrates all validation checks
pub struct ArchitectureValidator {
    workspace_root: PathBuf,
    dependency: DependencyValidator,
    quality: QualityValidator,
    patterns: PatternValidator,
    tests: TestValidator,
    documentation: DocumentationValidator,
    naming: NamingValidator,
    solid: SolidValidator,
    organization: OrganizationValidator,
}

impl ArchitectureValidator {
    /// Create a new validator for the given workspace root
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        let root = workspace_root.into();
        Self {
            dependency: DependencyValidator::new(root.clone()),
            quality: QualityValidator::new(root.clone()),
            patterns: PatternValidator::new(root.clone()),
            tests: TestValidator::new(root.clone()),
            documentation: DocumentationValidator::new(root.clone()),
            naming: NamingValidator::new(root.clone()),
            solid: SolidValidator::new(root.clone()),
            organization: OrganizationValidator::new(root.clone()),
            workspace_root: root,
        }
    }

    /// Run all validations and return a comprehensive report
    pub fn validate_all(&mut self) -> Result<ValidationReport> {
        let dependency_violations = self.dependency.validate_all()?;
        let quality_violations = self.quality.validate_all()?;
        let pattern_violations = self.patterns.validate_all()?;
        let test_violations = self.tests.validate_all()?;
        let doc_violations = self.documentation.validate_all()?;
        let naming_violations = self.naming.validate_all()?;
        let solid_violations = self.solid.validate_all()?;
        let organization_violations = self.organization.validate_all()?;

        let total = dependency_violations.len()
            + quality_violations.len()
            + pattern_violations.len()
            + test_violations.len()
            + doc_violations.len()
            + naming_violations.len()
            + solid_violations.len()
            + organization_violations.len();

        let summary = ValidationSummary {
            total_violations: total,
            dependency_count: dependency_violations.len(),
            quality_count: quality_violations.len(),
            pattern_count: pattern_violations.len(),
            test_count: test_violations.len(),
            documentation_count: doc_violations.len(),
            naming_count: naming_violations.len(),
            solid_count: solid_violations.len(),
            organization_count: organization_violations.len(),
            passed: total == 0,
        };

        Ok(ValidationReport {
            timestamp: chrono::Utc::now().to_rfc3339(),
            workspace_root: self.workspace_root.clone(),
            summary,
            dependency_violations,
            quality_violations,
            pattern_violations,
            test_violations,
            documentation_violations: doc_violations,
            naming_violations,
            solid_violations,
            organization_violations,
        })
    }

    /// Run only dependency validation
    pub fn validate_dependencies(&mut self) -> Result<Vec<DependencyViolation>> {
        self.dependency.validate_all()
    }

    /// Run only quality validation
    pub fn validate_quality(&mut self) -> Result<Vec<QualityViolation>> {
        self.quality.validate_all()
    }

    /// Run only pattern validation
    pub fn validate_patterns(&mut self) -> Result<Vec<PatternViolation>> {
        self.patterns.validate_all()
    }

    /// Run only test organization validation
    pub fn validate_tests(&mut self) -> Result<Vec<TestViolation>> {
        self.tests.validate_all()
    }

    /// Run only documentation validation
    pub fn validate_documentation(&mut self) -> Result<Vec<DocumentationViolation>> {
        self.documentation.validate_all()
    }

    /// Run only naming validation
    pub fn validate_naming(&mut self) -> Result<Vec<NamingViolation>> {
        self.naming.validate_all()
    }

    /// Run only SOLID principle validation
    pub fn validate_solid(&mut self) -> Result<Vec<SolidViolation>> {
        self.solid.validate_all()
    }

    /// Run only organization validation
    pub fn validate_organization(&mut self) -> Result<Vec<OrganizationViolation>> {
        self.organization.validate_all()
    }
}

/// Get the workspace root from the current directory
pub fn find_workspace_root() -> Option<PathBuf> {
    let current = std::env::current_dir().ok()?;
    find_workspace_root_from(&current)
}

/// Find workspace root starting from a given path
pub fn find_workspace_root_from(start: &Path) -> Option<PathBuf> {
    let mut current = start.to_path_buf();
    loop {
        let cargo_toml = current.join("Cargo.toml");
        if cargo_toml.exists() {
            if let Ok(content) = std::fs::read_to_string(&cargo_toml) {
                if content.contains("[workspace]") {
                    return Some(current);
                }
            }
        }
        if !current.pop() {
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_serialization() {
        let severity = Severity::Error;
        let json = serde_json::to_string(&severity).unwrap();
        assert_eq!(json, "\"Error\"");
    }
}
