//! Validator Trait and Registry
//!
//! Provides a unified interface for all validators and a registry
//! for managing and running validators.

use crate::violation_trait::Violation;
use crate::ValidationConfig;
use anyhow::Result;

/// All validators implement this trait
///
/// This enables a plugin-like architecture where validators can be
/// registered and run uniformly.
pub trait Validator: Send + Sync {
    /// Unique name of this validator
    fn name(&self) -> &'static str;

    /// Run validation and return violations
    fn validate(&self, config: &ValidationConfig) -> Result<Vec<Box<dyn Violation>>>;

    /// Whether this validator is enabled by default
    fn enabled_by_default(&self) -> bool {
        true
    }

    /// Description of what this validator checks
    fn description(&self) -> &'static str {
        ""
    }
}

/// Registry of validators
///
/// Manages a collection of validators and provides methods to run
/// all or selected validators.
pub struct ValidatorRegistry {
    validators: Vec<Box<dyn Validator>>,
}

impl Default for ValidatorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidatorRegistry {
    /// Create an empty registry
    pub fn new() -> Self {
        Self {
            validators: Vec::new(),
        }
    }

    /// Register a validator
    pub fn register(&mut self, validator: Box<dyn Validator>) {
        self.validators.push(validator);
    }

    /// Register a validator (builder pattern)
    pub fn with_validator(mut self, validator: Box<dyn Validator>) -> Self {
        self.register(validator);
        self
    }

    /// Get all registered validators
    pub fn validators(&self) -> &[Box<dyn Validator>] {
        &self.validators
    }

    /// Run all enabled validators
    pub fn validate_all(&self, config: &ValidationConfig) -> Result<Vec<Box<dyn Violation>>> {
        let mut all_violations = Vec::new();

        for validator in &self.validators {
            if validator.enabled_by_default() {
                match validator.validate(config) {
                    Ok(violations) => all_violations.extend(violations),
                    Err(e) => {
                        eprintln!(
                            "Warning: Validator '{}' failed, skipping: {}",
                            validator.name(),
                            e
                        );
                    }
                }
            }
        }

        Ok(all_violations)
    }

    /// Run specific validators by name
    pub fn validate_named(
        &self,
        config: &ValidationConfig,
        names: &[&str],
    ) -> Result<Vec<Box<dyn Violation>>> {
        let mut all_violations = Vec::new();

        for validator in &self.validators {
            if names.contains(&validator.name()) {
                match validator.validate(config) {
                    Ok(violations) => all_violations.extend(violations),
                    Err(e) => {
                        eprintln!(
                            "Warning: Validator '{}' failed, skipping: {}",
                            validator.name(),
                            e
                        );
                    }
                }
            }
        }

        Ok(all_violations)
    }

    /// Create a registry with standard validators
    ///
    /// This registers all built-in validators with default configuration.
    /// Validators include:
    /// - Architecture: clean_architecture, layer_flow, port_adapter, visibility
    /// - Dependencies: dependency, shaku
    /// - Quality: quality, solid, naming, patterns, documentation, tests_org
    /// - Performance: performance, async_patterns, kiss, pmat
    /// - Organization: organization, implementation, refactoring, error_boundary
    pub fn standard() -> Self {
        Self::standard_for(".")
    }

    /// Create a registry with standard validators for a specific workspace
    pub fn standard_for(workspace_root: impl Into<std::path::PathBuf>) -> Self {
        // Architecture validators
        use crate::clean_architecture::CleanArchitectureValidator;
        use crate::layer_flow::LayerFlowValidator;
        use crate::port_adapter::PortAdapterValidator;
        use crate::visibility::VisibilityValidator;

        // Dependency validators
        use crate::dependency::DependencyValidator;
        use crate::shaku::ShakuValidator;

        // Quality validators
        use crate::quality::QualityValidator;
        use crate::solid::SolidValidator;
        use crate::naming::NamingValidator;
        use crate::patterns::PatternValidator;
        use crate::documentation::DocumentationValidator;
        use crate::tests_org::TestValidator;

        // Performance validators
        use crate::performance::PerformanceValidator;
        use crate::async_patterns::AsyncPatternValidator;
        use crate::kiss::KissValidator;
        use crate::pmat::PmatValidator;

        // Organization validators
        use crate::organization::OrganizationValidator;
        use crate::implementation::ImplementationQualityValidator;
        use crate::refactoring::RefactoringValidator;
        use crate::error_boundary::ErrorBoundaryValidator;

        let root = workspace_root.into();

        Self::new()
            // Architecture
            .with_validator(Box::new(CleanArchitectureValidator::new(&root)))
            .with_validator(Box::new(LayerFlowValidator::new()))
            .with_validator(Box::new(PortAdapterValidator::new()))
            .with_validator(Box::new(VisibilityValidator::new()))
            // Dependencies
            .with_validator(Box::new(DependencyValidator::new(&root)))
            .with_validator(Box::new(ShakuValidator::new(&root)))
            // Quality
            .with_validator(Box::new(QualityValidator::new(&root)))
            .with_validator(Box::new(SolidValidator::new(&root)))
            .with_validator(Box::new(NamingValidator::new(&root)))
            .with_validator(Box::new(PatternValidator::new(&root)))
            .with_validator(Box::new(DocumentationValidator::new(&root)))
            .with_validator(Box::new(TestValidator::new(&root)))
            // Performance
            .with_validator(Box::new(PerformanceValidator::new(&root)))
            .with_validator(Box::new(AsyncPatternValidator::new(&root)))
            .with_validator(Box::new(KissValidator::new(&root)))
            .with_validator(Box::new(PmatValidator::new(&root)))
            // Organization
            .with_validator(Box::new(OrganizationValidator::new(&root)))
            .with_validator(Box::new(ImplementationQualityValidator::new(&root)))
            .with_validator(Box::new(RefactoringValidator::new(&root)))
            .with_validator(Box::new(ErrorBoundaryValidator::new(&root)))
    }
}

/// Helper struct for wrapping existing validators
///
/// This allows existing validators to be used with the new registry
/// during the migration period.
pub struct LegacyValidatorAdapter<F>
where
    F: Fn(&ValidationConfig) -> Result<Vec<Box<dyn Violation>>> + Send + Sync,
{
    name: &'static str,
    description: &'static str,
    validate_fn: F,
}

impl<F> LegacyValidatorAdapter<F>
where
    F: Fn(&ValidationConfig) -> Result<Vec<Box<dyn Violation>>> + Send + Sync,
{
    /// Create a new adapter
    pub fn new(name: &'static str, description: &'static str, validate_fn: F) -> Self {
        Self {
            name,
            description,
            validate_fn,
        }
    }
}

impl<F> Validator for LegacyValidatorAdapter<F>
where
    F: Fn(&ValidationConfig) -> Result<Vec<Box<dyn Violation>>> + Send + Sync,
{
    fn name(&self) -> &'static str {
        self.name
    }

    fn description(&self) -> &'static str {
        self.description
    }

    fn validate(&self, config: &ValidationConfig) -> Result<Vec<Box<dyn Violation>>> {
        (self.validate_fn)(config)
    }
}
