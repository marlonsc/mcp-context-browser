// Clippy allows for complex patterns in validation code
// These allows are needed for mcb-validate which has complex parsing and validation logic
#![allow(clippy::all)]
#![allow(clippy::pedantic)]
#![allow(clippy::restriction)]

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
pub mod generic_reporter;
pub mod validator_trait;

// === Configuration System (Phase 5) ===
pub mod config;
pub mod scan;

// === Rule Registry (Phase 3) ===
pub mod engines;
pub mod rules;

// === Pattern Registry (YAML-driven patterns) ===
pub mod pattern_registry;

// === Rule Filtering System (Phase 6) ===
pub mod filters;

// === Linter Integration (Phase 1 - Pure Rust Pipeline) ===
pub mod linters;

// === AST Analysis (Phase 2 - Pure Rust Pipeline) ===
pub mod ast;

// === Metrics Analysis (Phase 4 - Complexity Metrics) ===
pub mod metrics;

// === Duplication Detection (Phase 5 - Clone Detection) ===
pub mod duplication;

// === New Validators (using new system) ===
pub mod clean_architecture;
pub mod config_quality;
pub mod layer_flow;
pub mod port_adapter;
pub mod test_quality;
pub mod visibility;

// === Legacy Validators (being migrated to new system) ===
pub mod async_patterns;
pub mod dependency;
pub mod documentation;
pub mod error_boundary;
pub mod implementation;
pub mod kiss;
pub mod naming;
pub mod organization;
pub mod pattern_validator;
pub mod performance;
pub mod pmat;
pub mod quality;
pub mod refactoring;
pub mod reporter;
pub mod solid;
pub mod tests_org;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

// Re-export new DRY violation system
pub use generic_reporter::{GenericReport, GenericReporter, GenericSummary, ViolationEntry};
pub use validator_trait::{LegacyValidatorAdapter, Validator, ValidatorRegistry};
pub use violation_trait::{Violation, ViolationCategory, ViolationExt};

// Re-export configuration system
pub use config::{
    ArchitectureRulesConfig, FileConfig, GeneralConfig, OrganizationRulesConfig,
    QualityRulesConfig, RulesConfig, SolidRulesConfig, ValidatorsConfig,
};

// Re-export rule registry and YAML system
pub use engines::{HybridRuleEngine, RuleEngineType};
pub use rules::templates::TemplateEngine;
pub use rules::yaml_loader::{
    AstSelector, MetricThresholdConfig, MetricsConfig, RuleFix, ValidatedRule, YamlRuleLoader,
};
pub use rules::yaml_validator::YamlRuleValidator;
pub use rules::{Rule, RuleRegistry};

// Re-export linter integration
pub use linters::{
    ClippyLinter, LintViolation, LinterEngine, LinterType, RuffLinter, YamlRuleExecutor,
};

// Re-export AST module types
pub use ast::{
    AstDecoder, AstEngine, AstNode, AstParseResult, AstParser, AstQuery, AstQueryBuilder,
    AstQueryPatterns, AstViolation, Position, QueryCondition, Span, UnwrapDetection,
    UnwrapDetector,
};

// Re-export Metrics module types (Phase 4)
pub use metrics::{
    MetricThreshold, MetricThresholds, MetricType, MetricViolation, MetricsAnalyzer,
};

// Re-export new validators
pub use clean_architecture::{CleanArchitectureValidator, CleanArchitectureViolation};
pub use config_quality::{ConfigQualityValidator, ConfigQualityViolation};
pub use layer_flow::{LayerFlowValidator, LayerFlowViolation};
pub use port_adapter::{PortAdapterValidator, PortAdapterViolation};
pub use test_quality::{TestQualityValidator, TestQualityViolation};
pub use visibility::{VisibilityValidator, VisibilityViolation};

// Re-export legacy validators
pub use dependency::{DependencyValidator, DependencyViolation};
pub use documentation::{DocumentationValidator, DocumentationViolation};
pub use implementation::{ImplementationQualityValidator, ImplementationViolation};
pub use kiss::{KissValidator, KissViolation};
pub use naming::{NamingValidator, NamingViolation};
pub use organization::{OrganizationValidator, OrganizationViolation};
pub use pattern_validator::{PatternValidator, PatternViolation};
pub use quality::{QualityValidator, QualityViolation};
pub use reporter::{Reporter, ValidationReport, ValidationSummary};

// Re-export ComponentType for strict directory validation
pub use refactoring::{RefactoringValidator, RefactoringViolation};
pub use solid::{SolidValidator, SolidViolation};
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

                // mcb-validate validates itself - no special treatment
                // mcb (facade crate) is minimal re-exports, skip for now
                if path.file_name().is_some_and(|n| n == "mcb") {
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

    #[error("YAML parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Invalid regex pattern: {0}")]
    InvalidRegex(String),
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
    refactoring: RefactoringValidator,
    implementation: ImplementationQualityValidator,
    // New validators for PMAT integration
    performance: PerformanceValidator,
    async_patterns: AsyncPatternValidator,
    error_boundary: ErrorBoundaryValidator,
    pmat: PmatValidator,
    // New quality validators (v0.1.2)
    test_quality: TestQualityValidator,
    config_quality: ConfigQualityValidator,
    // Clean Architecture validator (CA001-CA009)
    clean_architecture: CleanArchitectureValidator,
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
            refactoring: RefactoringValidator::with_config(config.clone()),
            implementation: ImplementationQualityValidator::with_config(config.clone()),
            // New validators for PMAT integration
            performance: PerformanceValidator::with_config(config.clone()),
            async_patterns: AsyncPatternValidator::with_config(config.clone()),
            error_boundary: ErrorBoundaryValidator::with_config(config.clone()),
            pmat: PmatValidator::with_config(config.clone()),
            // New quality validators (v0.1.2)
            test_quality: TestQualityValidator::with_config(config.clone()),
            config_quality: ConfigQualityValidator::with_config(config.clone()),
            // Clean Architecture validator (CA001-CA009)
            clean_architecture: CleanArchitectureValidator::with_config(config.clone()),
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
        let refactoring_violations = self.refactoring.validate_all()?;
        let implementation_violations = self.implementation.validate_all()?;
        // New validators for PMAT integration
        let performance_violations = self.performance.validate_all()?;
        let async_violations = self.async_patterns.validate_all()?;
        let error_boundary_violations = self.error_boundary.validate_all()?;
        let pmat_violations = self.pmat.validate_all()?;
        // New quality validators (v0.1.2)
        let test_quality_violations = self.test_quality.validate()?;
        let config_quality_violations = self.config_quality.validate()?;
        // Clean Architecture validator (CA001-CA009)
        let clean_architecture_violations = self.clean_architecture.validate_all()?;

        let total = dependency_violations.len()
            + quality_violations.len()
            + pattern_violations.len()
            + test_violations.len()
            + doc_violations.len()
            + naming_violations.len()
            + solid_violations.len()
            + organization_violations.len()
            + kiss_violations.len()
            + refactoring_violations.len()
            + implementation_violations.len()
            + performance_violations.len()
            + async_violations.len()
            + error_boundary_violations.len()
            + pmat_violations.len()
            + test_quality_violations.len()
            + config_quality_violations.len()
            + clean_architecture_violations.len();

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
            refactoring_count: refactoring_violations.len(),
            implementation_count: implementation_violations.len(),
            performance_count: performance_violations.len(),
            async_count: async_violations.len(),
            error_boundary_count: error_boundary_violations.len(),
            pmat_count: pmat_violations.len(),
            clean_architecture_count: clean_architecture_violations.len(),
            test_quality_count: test_quality_violations.len(),
            config_quality_count: config_quality_violations.len(),
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
            refactoring_violations,
            implementation_violations,
            performance_violations,
            async_violations,
            error_boundary_violations,
            pmat_violations,
            clean_architecture_violations,
            test_quality_violations,
            config_quality_violations,
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

    // ========== YAML-Based Validation (Phase 9) ==========

    /// Create a YAML-based validator
    pub fn yaml_validator(&self) -> Result<YamlRuleValidator> {
        YamlRuleValidator::new()
    }

    /// Load and validate all YAML rules
    pub async fn load_yaml_rules(&self) -> Result<Vec<crate::rules::yaml_loader::ValidatedRule>> {
        let rules_dir = self.config.workspace_root.join("crates/mcb-validate/rules");

        let mut loader = YamlRuleLoader::new(rules_dir)?;
        loader.load_all_rules().await
    }

    /// Validate using YAML rules with hybrid engines
    pub async fn validate_with_yaml_rules(&self) -> Result<GenericReport> {
        use crate::violation_trait::ViolationCategory;

        let rules = self.load_yaml_rules().await?;
        let engine = HybridRuleEngine::new();

        let mut violations: Vec<Box<dyn Violation>> = Vec::new();

        // Scan for files to validate
        let file_contents = self.scan_files_for_validation()?;

        for rule in rules.into_iter().filter(|r| r.enabled) {
            let context = engines::hybrid_engine::RuleContext {
                workspace_root: self.config.workspace_root.clone(),
                config: self.config.clone(),
                ast_data: HashMap::new(),   // Would be populated by scanner
                cargo_data: HashMap::new(), // Would be populated by scanner
                file_contents: file_contents.clone(),
            };

            // Determine severity
            let severity = match rule.severity.to_lowercase().as_str() {
                "error" => crate::violation_trait::Severity::Error,
                "warning" => crate::violation_trait::Severity::Warning,
                _ => crate::violation_trait::Severity::Info,
            };

            // Determine category
            let category = match rule.category.to_lowercase().as_str() {
                "architecture" | "clean-architecture" => ViolationCategory::Architecture,
                "quality" => ViolationCategory::Quality,
                "performance" => ViolationCategory::Performance,
                "organization" => ViolationCategory::Organization,
                "solid" => ViolationCategory::Solid,
                "di" | "dependency-injection" => ViolationCategory::DependencyInjection,
                "migration" => ViolationCategory::Configuration, // Use Configuration for migration rules
                _ => ViolationCategory::Quality,
            };

            // Check if this is a lint-based rule
            if !rule.lint_select.is_empty() {
                // Use linter execution
                let result = engine
                    .execute_lint_rule(
                        &rule.id,
                        &rule.lint_select,
                        &context,
                        rule.message.as_deref(),
                        severity,
                        category,
                    )
                    .await?;

                violations.extend(
                    result
                        .violations
                        .into_iter()
                        .map(|v| Box::new(v) as Box<dyn Violation>),
                );
            } else {
                // Use rule engine execution
                let engine_type = match rule.engine.as_str() {
                    "rust-rule-engine" => RuleEngineType::RustRuleEngine,
                    "rusty-rules" => RuleEngineType::RustyRules,
                    _ => RuleEngineType::RustyRules, // Default
                };

                let result = engine
                    .execute_rule(&rule.id, engine_type, &rule.rule_definition, &context)
                    .await?;

                violations.extend(
                    result
                        .violations
                        .into_iter()
                        .map(|v| Box::new(v) as Box<dyn Violation>),
                );
            }
        }

        Ok(GenericReporter::create_report(
            &violations,
            self.config.workspace_root.clone(),
        ))
    }

    /// Scan files for validation context
    fn scan_files_for_validation(&self) -> Result<HashMap<String, String>> {
        let mut file_contents = HashMap::new();

        // Scan all source directories
        if let Ok(scan_dirs) = self.config.get_scan_dirs() {
            for dir in scan_dirs {
                self.scan_directory(&dir, &mut file_contents)?;
            }
        }

        Ok(file_contents)
    }

    /// Recursively scan a directory for source files
    fn scan_directory(
        &self,
        dir: &Path,
        file_contents: &mut HashMap<String, String>,
    ) -> Result<()> {
        if !dir.exists() || !dir.is_dir() {
            return Ok(());
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if self.config.should_exclude(&path) {
                continue;
            }

            if path.is_dir() {
                self.scan_directory(&path, file_contents)?;
            } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                // Include common source file extensions
                let is_source = matches!(
                    ext,
                    "rs" | "py" | "js" | "ts" | "go" | "java" | "c" | "cpp" | "h" | "hpp"
                );
                if is_source {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        file_contents.insert(path.to_string_lossy().to_string(), content);
                    }
                }
            }
        }

        Ok(())
    }

    // ========== Registry-Based Validation (Phase 7) ==========

    /// Create a ValidatorRegistry with the standard new validators
    ///
    /// This registry contains validators that implement the new `Validator` trait:
    /// - CleanArchitectureValidator
    /// - LayerFlowValidator
    /// - PortAdapterValidator
    /// - VisibilityValidator
    pub fn new_validator_registry(&self) -> ValidatorRegistry {
        ValidatorRegistry::standard_for(&self.config.workspace_root)
    }

    /// Validate using the new registry-based system
    ///
    /// This runs only the validators that have been migrated to the new
    /// trait-based architecture. Use `validate_all()` for comprehensive
    /// validation including legacy validators.
    ///
    /// # Returns
    ///
    /// A `GenericReport` containing all violations from registry validators.
    pub fn validate_with_registry(&self) -> Result<GenericReport> {
        let registry = self.new_validator_registry();
        let violations = registry
            .validate_all(&self.config)
            .map_err(|e| ValidationError::Config(e.to_string()))?;

        Ok(GenericReporter::create_report(
            &violations,
            self.config.workspace_root.clone(),
        ))
    }

    /// Validate specific validators by name using the registry
    ///
    /// # Arguments
    ///
    /// * `names` - Names of validators to run (e.g., &["clean_architecture", "layer_flow"])
    ///
    /// # Available validators
    ///
    /// - "clean_architecture" - Clean Architecture compliance
    /// - "layer_flow" - Layer dependency rules
    /// - "port_adapter" - Port/adapter patterns
    /// - "visibility" - Visibility modifiers
    pub fn validate_named(&self, names: &[&str]) -> Result<GenericReport> {
        let registry = self.new_validator_registry();
        let violations = registry
            .validate_named(&self.config, names)
            .map_err(|e| ValidationError::Config(e.to_string()))?;

        Ok(GenericReporter::create_report(
            &violations,
            self.config.workspace_root.clone(),
        ))
    }

    /// Run both legacy and new validators, returning a combined report
    ///
    /// This method provides the most comprehensive validation by running:
    /// 1. All legacy validators (via `validate_all()`)
    /// 2. All new registry validators (via `validate_with_registry()`)
    ///
    /// # Note
    ///
    /// As validators are migrated to the new system, they will be removed
    /// from the legacy path and added to the registry path to avoid
    /// duplicate violation reports.
    pub fn validate_comprehensive(&mut self) -> Result<(ValidationReport, GenericReport)> {
        let legacy_report = self.validate_all()?;
        let registry_report = self.validate_with_registry()?;
        Ok((legacy_report, registry_report))
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
