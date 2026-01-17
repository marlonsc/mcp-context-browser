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
//!
//! # Multi-Directory Validation
//!
//! The validator can scan multiple source directories (e.g., workspace crates + legacy src/):
//!
//! ```ignore
//! use mcb_validate::{ValidationConfig, ArchitectureValidator};
//!
//! let config = ValidationConfig::new("/workspace")
//!     .with_additional_path("../legacy-src")
//!     .with_exclude_pattern("target/");
//!
//! let mut validator = ArchitectureValidator::with_config(config);
//! let report = validator.validate_all()?;
//! ```

// === New DRY Violation System (Phase 3 Refactoring) ===
pub mod violation_trait;
#[macro_use]
pub mod violation_macro;
pub mod validator_trait;
pub mod generic_reporter;

// === New Validators (using new system) ===
pub mod architecture;

// === Legacy Validators (being migrated to new system) ===
pub mod async_patterns;
pub mod dependency;
pub mod documentation;
pub mod error_boundary;
pub mod implementation;
pub mod kiss;
pub mod naming;
pub mod organization;
pub mod patterns;
pub mod performance;
pub mod pmat;
pub mod quality;
pub mod refactoring;
pub mod reporter;
pub mod shaku;
pub mod solid;
pub mod tests_org;

use std::path::{Path, PathBuf};
use thiserror::Error;

// Re-export new DRY violation system
pub use violation_trait::{Violation, ViolationCategory, ViolationExt};
pub use validator_trait::{LegacyValidatorAdapter, Validator, ValidatorRegistry};
pub use generic_reporter::{GenericReport, GenericReporter, GenericSummary, ViolationEntry};

// Re-export new validators
pub use architecture::{ArchitectureValidator, ArchitectureViolation};

// Re-export legacy validators
pub use dependency::{DependencyValidator, DependencyViolation};
pub use documentation::{DocumentationValidator, DocumentationViolation};
pub use implementation::{ImplementationQualityValidator, ImplementationViolation};
pub use kiss::{KissValidator, KissViolation};
pub use naming::{NamingValidator, NamingViolation};
pub use organization::{OrganizationValidator, OrganizationViolation};
pub use patterns::{PatternValidator, PatternViolation};
pub use quality::{QualityValidator, QualityViolation};
pub use reporter::{Reporter, ValidationReport, ValidationSummary};
pub use shaku::{ShakuValidator, ShakuViolation};

// Re-export ComponentType for strict directory validation
// Used by organization and shaku validators
pub use solid::{SolidValidator, SolidViolation};
pub use refactoring::{RefactoringValidator, RefactoringViolation};
pub use tests_org::{TestValidator, TestViolation};

// New validators for PMAT integration
pub use async_patterns::{AsyncPatternValidator, AsyncViolation};
pub use error_boundary::{ErrorBoundaryValidator, ErrorBoundaryViolation};
pub use performance::{PerformanceValidator, PerformanceViolation};
pub use pmat::{PmatValidator, PmatViolation};

// Re-export ValidationConfig for multi-directory support
// ValidationConfig is defined in this module

/// Result type for validation operations
pub type Result<T> = std::result::Result<T, ValidationError>;

/// Configuration for multi-directory validation
///
/// Allows scanning multiple source directories beyond the standard `crates/` directory.
/// Useful for validating legacy codebases alongside new workspace architecture.
///
/// # Example
///
/// ```ignore
/// use mcb_validate::ValidationConfig;
///
/// let config = ValidationConfig::new("/workspace")
///     .with_additional_path("../src")  // Legacy codebase
///     .with_exclude_pattern("target/")
///     .with_exclude_pattern("tests/fixtures/");
/// ```
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Root directory of the workspace (contains Cargo.toml with [workspace])
    pub workspace_root: PathBuf,
    /// Additional source paths to validate (e.g., legacy src/ directories)
    pub additional_src_paths: Vec<PathBuf>,
    /// Patterns to exclude from validation (e.g., "target/", "tests/")
    pub exclude_patterns: Vec<String>,
}

impl ValidationConfig {
    /// Create a new validation config for the given workspace root
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self {
            workspace_root: workspace_root.into(),
            additional_src_paths: Vec::new(),
            exclude_patterns: Vec::new(),
        }
    }

    /// Add an additional source path to validate
    ///
    /// Paths can be absolute or relative to workspace_root.
    pub fn with_additional_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.additional_src_paths.push(path.into());
        self
    }

    /// Add an exclude pattern (files/directories matching this will be skipped)
    pub fn with_exclude_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.exclude_patterns.push(pattern.into());
        self
    }

    /// Check if a path is from a legacy/additional source (not main crates/)
    pub fn is_legacy_path(&self, path: &Path) -> bool {
        for additional_path in &self.additional_src_paths {
            let full_path = if additional_path.is_absolute() {
                additional_path.clone()
            } else {
                self.workspace_root.join(additional_path)
            };

            // Canonicalize if possible for accurate comparison
            if let (Ok(canonical_full), Ok(canonical_path)) =
                (full_path.canonicalize(), path.canonicalize())
            {
                if canonical_path.starts_with(&canonical_full) {
                    return true;
                }
            }

            // Fallback to string-based check
            if let (Some(full_str), Some(path_str)) = (full_path.to_str(), path.to_str()) {
                if path_str.contains(full_str) || path_str.contains("src/") {
                    return true;
                }
            }
        }
        false
    }

    /// Check if a path should be excluded based on patterns
    pub fn should_exclude(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        self.exclude_patterns
            .iter()
            .any(|pattern| path_str.contains(pattern))
    }

    /// Get all source directories to validate
    ///
    /// Returns crates/ subdirectories plus any additional paths.
    pub fn get_source_dirs(&self) -> Result<Vec<PathBuf>> {
        let mut dirs = Vec::new();

        // Original crates/ scanning
        let crates_dir = self.workspace_root.join("crates");
        if crates_dir.exists() {
            for entry in std::fs::read_dir(&crates_dir)? {
                let entry = entry?;
                let path = entry.path();

                // Skip the validate crate itself (it's a meta-tool)
                if path
                    .file_name()
                    .is_some_and(|n| n == "mcb-validate" || n == "mcb")
                {
                    continue;
                }

                if path.is_dir() && !self.should_exclude(&path) {
                    dirs.push(path);
                }
            }
        }

        // Additional paths from config
        for path in &self.additional_src_paths {
            let full_path = if path.is_absolute() {
                path.clone()
            } else {
                self.workspace_root.join(path)
            };
            if full_path.exists() && !self.should_exclude(&full_path) {
                dirs.push(full_path);
            }
        }

        Ok(dirs)
    }

    /// Get actual source directories to scan for Rust files
    ///
    /// For crate directories (containing `src/` subdirectory), returns `<dir>/src/`.
    /// For flat source directories (like `../src/`), returns the directory itself.
    ///
    /// This handles both:
    /// - Workspace crates: `crates/mcb-domain/` → scans `crates/mcb-domain/src/`
    /// - Legacy source: `../src/` → scans `../src/` directly
    pub fn get_scan_dirs(&self) -> Result<Vec<PathBuf>> {
        let mut scan_dirs = Vec::new();

        for dir in self.get_source_dirs()? {
            let src_subdir = dir.join("src");
            if src_subdir.exists() && src_subdir.is_dir() {
                // Crate-style: has src/ subdirectory
                scan_dirs.push(src_subdir);
            } else if self.is_legacy_path(&dir) {
                // Legacy flat directory: scan directly
                scan_dirs.push(dir);
            } else {
                // Standard crate without src/ directory yet - skip
                continue;
            }
        }

        Ok(scan_dirs)
    }
}

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Error => write!(f, "ERROR"),
            Self::Warning => write!(f, "WARNING"),
            Self::Info => write!(f, "INFO"),
        }
    }
}

/// Component type for strict directory validation
///
/// Used to categorize code components by their architectural role,
/// enabling strict enforcement of where each type should reside.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ComponentType {
    /// Domain port trait (interface definition)
    Port,
    /// Domain entity with identity
    Entity,
    /// Domain value object (immutable)
    ValueObject,
    /// Domain service interface
    DomainService,
    /// Infrastructure adapter implementation
    Adapter,
    /// Repository implementation
    Repository,
    /// Server/transport layer handler
    Handler,
    /// Configuration type
    Config,
    /// Factory for creating components
    Factory,
    /// DI module definition
    DiModule,
}

impl std::fmt::Display for ComponentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Port => write!(f, "Port"),
            Self::Entity => write!(f, "Entity"),
            Self::ValueObject => write!(f, "ValueObject"),
            Self::DomainService => write!(f, "DomainService"),
            Self::Adapter => write!(f, "Adapter"),
            Self::Repository => write!(f, "Repository"),
            Self::Handler => write!(f, "Handler"),
            Self::Config => write!(f, "Config"),
            Self::Factory => write!(f, "Factory"),
            Self::DiModule => write!(f, "DiModule"),
        }
    }
}

/// Main validator that orchestrates all validation checks
pub struct ArchitectureValidator {
    config: ValidationConfig,
    dependency: DependencyValidator,
    quality: QualityValidator,
    patterns: PatternValidator,
    tests: TestValidator,
    documentation: DocumentationValidator,
    naming: NamingValidator,
    solid: SolidValidator,
    organization: OrganizationValidator,
    kiss: KissValidator,
    shaku: ShakuValidator,
    refactoring: RefactoringValidator,
    implementation: ImplementationQualityValidator,
    // New validators for PMAT integration
    performance: PerformanceValidator,
    async_patterns: AsyncPatternValidator,
    error_boundary: ErrorBoundaryValidator,
    pmat: PmatValidator,
}

impl ArchitectureValidator {
    /// Create a new validator for the given workspace root
    ///
    /// This is the simple constructor for validating only the workspace crates.
    /// For multi-directory validation, use `with_config()`.
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        let config = ValidationConfig::new(workspace_root);
        Self::with_config(config)
    }

    /// Create a validator with a custom configuration
    ///
    /// Use this to validate additional source directories beyond crates/.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = ValidationConfig::new("/workspace")
    ///     .with_additional_path("../src");  // Also validate legacy code
    ///
    /// let mut validator = ArchitectureValidator::with_config(config);
    /// ```
    pub fn with_config(config: ValidationConfig) -> Self {
        let root = config.workspace_root.clone();
        Self {
            dependency: DependencyValidator::with_config(config.clone()),
            quality: QualityValidator::with_config(config.clone()),
            patterns: PatternValidator::with_config(config.clone()),
            tests: TestValidator::with_config(config.clone()),
            documentation: DocumentationValidator::with_config(config.clone()),
            naming: NamingValidator::with_config(config.clone()),
            solid: SolidValidator::with_config(config.clone()),
            organization: OrganizationValidator::with_config(config.clone()),
            kiss: KissValidator::with_config(config.clone()),
            shaku: ShakuValidator::with_config(config.clone()),
            refactoring: RefactoringValidator::with_config(config.clone()),
            implementation: ImplementationQualityValidator::with_config(config.clone()),
            // New validators for PMAT integration
            performance: PerformanceValidator::with_config(config.clone()),
            async_patterns: AsyncPatternValidator::with_config(config.clone()),
            error_boundary: ErrorBoundaryValidator::with_config(config.clone()),
            pmat: PmatValidator::with_config(config.clone()),
            config: ValidationConfig {
                workspace_root: root,
                ..config
            },
        }
    }

    /// Get the workspace root path
    pub fn workspace_root(&self) -> &Path {
        &self.config.workspace_root
    }

    /// Get the validation configuration
    pub fn config(&self) -> &ValidationConfig {
        &self.config
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
        let kiss_violations = self.kiss.validate_all()?;
        let shaku_violations = self.shaku.validate_all()?;
        let refactoring_violations = self.refactoring.validate_all()?;
        let implementation_violations = self.implementation.validate_all()?;
        // New validators for PMAT integration
        let performance_violations = self.performance.validate_all()?;
        let async_violations = self.async_patterns.validate_all()?;
        let error_boundary_violations = self.error_boundary.validate_all()?;
        let pmat_violations = self.pmat.validate_all()?;

        let total = dependency_violations.len()
            + quality_violations.len()
            + pattern_violations.len()
            + test_violations.len()
            + doc_violations.len()
            + naming_violations.len()
            + solid_violations.len()
            + organization_violations.len()
            + kiss_violations.len()
            + shaku_violations.len()
            + refactoring_violations.len()
            + implementation_violations.len()
            + performance_violations.len()
            + async_violations.len()
            + error_boundary_violations.len()
            + pmat_violations.len();

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
            kiss_count: kiss_violations.len(),
            shaku_count: shaku_violations.len(),
            refactoring_count: refactoring_violations.len(),
            implementation_count: implementation_violations.len(),
            performance_count: performance_violations.len(),
            async_count: async_violations.len(),
            error_boundary_count: error_boundary_violations.len(),
            pmat_count: pmat_violations.len(),
            passed: total == 0,
        };

        Ok(ValidationReport {
            timestamp: chrono::Utc::now().to_rfc3339(),
            workspace_root: self.config.workspace_root.clone(),
            summary,
            dependency_violations,
            quality_violations,
            pattern_violations,
            test_violations,
            documentation_violations: doc_violations,
            naming_violations,
            solid_violations,
            organization_violations,
            kiss_violations,
            shaku_violations,
            refactoring_violations,
            implementation_violations,
            performance_violations,
            async_violations,
            error_boundary_violations,
            pmat_violations,
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

    /// Run only KISS principle validation
    pub fn validate_kiss(&mut self) -> Result<Vec<KissViolation>> {
        self.kiss.validate_all()
    }

    /// Run only DI/Shaku validation
    pub fn validate_shaku(&mut self) -> Result<Vec<ShakuViolation>> {
        self.shaku.validate_all()
    }

    /// Run only refactoring completeness validation
    pub fn validate_refactoring(&mut self) -> Result<Vec<RefactoringViolation>> {
        self.refactoring.validate_all()
    }

    /// Run only implementation quality validation
    pub fn validate_implementation(&mut self) -> Result<Vec<ImplementationViolation>> {
        self.implementation.validate_all()
    }

    /// Run only performance pattern validation
    pub fn validate_performance(&self) -> Result<Vec<PerformanceViolation>> {
        self.performance.validate_all()
    }

    /// Run only async pattern validation
    pub fn validate_async_patterns(&self) -> Result<Vec<AsyncViolation>> {
        self.async_patterns.validate_all()
    }

    /// Run only error boundary validation
    pub fn validate_error_boundary(&self) -> Result<Vec<ErrorBoundaryViolation>> {
        self.error_boundary.validate_all()
    }

    /// Run only PMAT integration validation
    pub fn validate_pmat(&self) -> Result<Vec<PmatViolation>> {
        self.pmat.validate_all()
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

    #[test]
    fn test_validation_config_creation() {
        let config = ValidationConfig::new("/workspace");
        assert_eq!(config.workspace_root.to_str().unwrap(), "/workspace");
        assert!(config.additional_src_paths.is_empty());
        assert!(config.exclude_patterns.is_empty());
    }

    #[test]
    fn test_validation_config_builder() {
        let config = ValidationConfig::new("/workspace")
            .with_additional_path("../src")
            .with_additional_path("../legacy")
            .with_exclude_pattern("target/")
            .with_exclude_pattern("tests/fixtures/");

        assert_eq!(config.additional_src_paths.len(), 2);
        assert_eq!(config.exclude_patterns.len(), 2);
    }

    #[test]
    fn test_validation_config_should_exclude() {
        let config = ValidationConfig::new("/workspace")
            .with_exclude_pattern("target/")
            .with_exclude_pattern("fixtures/");

        assert!(config.should_exclude(Path::new("/workspace/target/debug")));
        assert!(config.should_exclude(Path::new("/workspace/tests/fixtures/data.json")));
        assert!(!config.should_exclude(Path::new("/workspace/src/lib.rs")));
    }

    #[test]
    fn test_architecture_validator_with_config() {
        let config = ValidationConfig::new("/tmp/test-workspace")
            .with_additional_path("../legacy-src")
            .with_exclude_pattern("target/");

        let validator = ArchitectureValidator::with_config(config);
        let config_ref = validator.config();

        assert_eq!(config_ref.additional_src_paths.len(), 1);
        assert_eq!(config_ref.exclude_patterns.len(), 1);
    }
}
