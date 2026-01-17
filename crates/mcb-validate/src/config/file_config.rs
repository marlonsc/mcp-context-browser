//! File-based Configuration
//!
//! Loads validation configuration from `.mcb-validate.toml` files,
//! providing project-specific rule customization.
//!
//! # Example Configuration
//!
//! ```toml
//! [general]
//! workspace_root = "."
//! exclude_patterns = ["target/", "tests/fixtures/"]
//!
//! [rules.architecture]
//! enabled = true
//! severity = "Error"
//!
//! [rules.quality]
//! enabled = true
//! max_file_lines = 500
//! allow_unwrap_in_tests = true
//!
//! [validators]
//! dependency = true
//! organization = true
//! quality = true
//! ```

use crate::Severity;
use serde::Deserialize;
use std::path::{Path, PathBuf};

/// Root configuration loaded from `.mcb-validate.toml`
#[derive(Debug, Clone, Default, Deserialize)]
pub struct FileConfig {
    /// General settings
    #[serde(default)]
    pub general: GeneralConfig,

    /// Rule-specific configuration
    #[serde(default)]
    pub rules: RulesConfig,

    /// Validator enable/disable flags
    #[serde(default)]
    pub validators: ValidatorsConfig,
}

impl FileConfig {
    /// Load configuration from a file
    pub fn load(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }

    /// Load configuration from workspace root, or return defaults
    ///
    /// Looks for `.mcb-validate.toml` in the workspace root directory.
    pub fn load_or_default(workspace_root: impl Into<PathBuf>) -> Self {
        let root = workspace_root.into();
        let config_path = root.join(".mcb-validate.toml");

        if config_path.exists() {
            match Self::load(&config_path) {
                Ok(mut config) => {
                    // Override workspace_root with the actual path
                    config.general.workspace_root = Some(root);
                    config
                }
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to load {}: {}. Using defaults.",
                        config_path.display(),
                        e
                    );
                    Self::default_for(root)
                }
            }
        } else {
            Self::default_for(root)
        }
    }

    /// Create default configuration for a workspace
    pub fn default_for(workspace_root: impl Into<PathBuf>) -> Self {
        Self {
            general: GeneralConfig {
                workspace_root: Some(workspace_root.into()),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// Get the workspace root path
    pub fn workspace_root(&self) -> PathBuf {
        self.general
            .workspace_root
            .clone()
            .unwrap_or_else(|| PathBuf::from("."))
    }

    /// Check if a validator is enabled
    pub fn is_validator_enabled(&self, name: &str) -> bool {
        match name {
            "dependency" => self.validators.dependency,
            "organization" => self.validators.organization,
            "quality" => self.validators.quality,
            "solid" => self.validators.solid,
            "shaku" => self.validators.shaku,
            "architecture" => self.validators.architecture,
            "refactoring" => self.validators.refactoring,
            "naming" => self.validators.naming,
            "documentation" => self.validators.documentation,
            "patterns" => self.validators.patterns,
            "kiss" => self.validators.kiss,
            "tests" => self.validators.tests,
            "async_patterns" => self.validators.async_patterns,
            "error_boundary" => self.validators.error_boundary,
            "performance" => self.validators.performance,
            "implementation" => self.validators.implementation,
            "pmat" => self.validators.pmat,
            "clean_architecture" => self.validators.clean_architecture,
            _ => true, // Unknown validators enabled by default
        }
    }
}

/// General configuration settings
#[derive(Debug, Clone, Deserialize, Default)]
pub struct GeneralConfig {
    /// Workspace root path (auto-detected if not set)
    pub workspace_root: Option<PathBuf>,

    /// Patterns to exclude from validation
    #[serde(default)]
    pub exclude_patterns: Vec<String>,

    /// Additional source paths to validate (beyond crates/)
    #[serde(default)]
    pub additional_src_paths: Vec<PathBuf>,

    /// Output format (human, json, ci)
    #[serde(default = "default_output_format")]
    pub output_format: String,
}

fn default_output_format() -> String {
    "human".to_string()
}

/// Rule-specific configuration
#[derive(Debug, Clone, Deserialize, Default)]
pub struct RulesConfig {
    /// Architecture validation rules
    #[serde(default)]
    pub architecture: ArchitectureRulesConfig,

    /// Code quality rules
    #[serde(default)]
    pub quality: QualityRulesConfig,

    /// Organization rules
    #[serde(default)]
    pub organization: OrganizationRulesConfig,

    /// SOLID principle rules
    #[serde(default)]
    pub solid: SolidRulesConfig,

    /// Shaku/DI rules
    #[serde(default)]
    pub shaku: ShakuRulesConfig,
}

/// Architecture validation rules configuration
#[derive(Debug, Clone, Deserialize)]
pub struct ArchitectureRulesConfig {
    /// Whether architecture validation is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Default severity for architecture violations
    #[serde(default = "default_error_severity")]
    pub severity: Severity,

    /// Layer boundary rules
    #[serde(default)]
    pub layer_boundaries: LayerBoundariesConfig,
}

impl Default for ArchitectureRulesConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            severity: Severity::Error,
            layer_boundaries: LayerBoundariesConfig::default(),
        }
    }
}

/// Layer boundary configuration
#[derive(Debug, Clone, Deserialize)]
pub struct LayerBoundariesConfig {
    /// Allowed internal dependencies for domain layer
    #[serde(default)]
    pub domain_internal_deps: Vec<String>,

    /// Allowed internal dependencies for application layer
    #[serde(default = "default_app_deps")]
    pub application_internal_deps: Vec<String>,

    /// Allowed internal dependencies for providers layer
    #[serde(default = "default_providers_deps")]
    pub providers_internal_deps: Vec<String>,

    /// Allowed internal dependencies for infrastructure layer
    #[serde(default = "default_infra_deps")]
    pub infrastructure_internal_deps: Vec<String>,

    /// Allowed internal dependencies for server layer
    #[serde(default = "default_server_deps")]
    pub server_internal_deps: Vec<String>,
}

fn default_app_deps() -> Vec<String> {
    vec!["mcb-domain".to_string()]
}

fn default_providers_deps() -> Vec<String> {
    vec!["mcb-domain".to_string(), "mcb-application".to_string()]
}

fn default_infra_deps() -> Vec<String> {
    vec![
        "mcb-domain".to_string(),
        "mcb-application".to_string(),
        "mcb-providers".to_string(),
    ]
}

fn default_server_deps() -> Vec<String> {
    vec![
        "mcb-domain".to_string(),
        "mcb-application".to_string(),
        "mcb-infrastructure".to_string(),
    ]
}

impl Default for LayerBoundariesConfig {
    fn default() -> Self {
        Self {
            domain_internal_deps: vec![],
            application_internal_deps: default_app_deps(),
            providers_internal_deps: default_providers_deps(),
            infrastructure_internal_deps: default_infra_deps(),
            server_internal_deps: default_server_deps(),
        }
    }
}

/// Code quality rules configuration
#[derive(Debug, Clone, Deserialize)]
pub struct QualityRulesConfig {
    /// Whether quality validation is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Maximum lines per file
    #[serde(default = "default_max_file_lines")]
    pub max_file_lines: usize,

    /// Maximum lines per function
    #[serde(default = "default_max_function_lines")]
    pub max_function_lines: usize,

    /// Allow unwrap in test code
    #[serde(default = "default_true")]
    pub allow_unwrap_in_tests: bool,

    /// Allow expect with message (vs raw unwrap)
    #[serde(default = "default_true")]
    pub allow_expect_with_message: bool,

    /// Files/patterns exempt from unwrap/expect checks
    #[serde(default)]
    pub exempt_patterns: Vec<String>,
}

fn default_max_file_lines() -> usize {
    500
}

fn default_max_function_lines() -> usize {
    50
}

impl Default for QualityRulesConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_file_lines: 500,
            max_function_lines: 50,
            allow_unwrap_in_tests: true,
            allow_expect_with_message: true,
            exempt_patterns: vec![],
        }
    }
}

/// Organization rules configuration
#[derive(Debug, Clone, Deserialize)]
pub struct OrganizationRulesConfig {
    /// Whether organization validation is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Magic numbers allowed (e.g., common sizes)
    #[serde(default = "default_magic_allowlist")]
    pub magic_number_allowlist: Vec<i64>,

    /// Strict directory structure enforcement
    #[serde(default = "default_true")]
    pub strict_directory_structure: bool,
}

fn default_magic_allowlist() -> Vec<i64> {
    vec![
        0, 1, 2, 10, 100, 1000, // Common constants
        16384, 32768, 65536, // Buffer sizes
        86400, 3600, 60, // Time constants
    ]
}

impl Default for OrganizationRulesConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            magic_number_allowlist: default_magic_allowlist(),
            strict_directory_structure: true,
        }
    }
}

/// SOLID principles rules configuration
#[derive(Debug, Clone, Deserialize)]
pub struct SolidRulesConfig {
    /// Whether SOLID validation is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Maximum methods per trait (ISP)
    #[serde(default = "default_max_trait_methods")]
    pub max_trait_methods: usize,

    /// Maximum methods per impl block (SRP)
    #[serde(default = "default_max_impl_methods")]
    pub max_impl_methods: usize,

    /// Maximum match arms before suggesting polymorphism
    #[serde(default = "default_max_match_arms")]
    pub max_match_arms: usize,

    /// Maximum parameters per function
    #[serde(default = "default_max_params")]
    pub max_function_params: usize,
}

fn default_max_trait_methods() -> usize {
    10
}

fn default_max_impl_methods() -> usize {
    15
}

fn default_max_match_arms() -> usize {
    10
}

fn default_max_params() -> usize {
    5
}

impl Default for SolidRulesConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_trait_methods: 10,
            max_impl_methods: 15,
            max_match_arms: 10,
            max_function_params: 5,
        }
    }
}

/// Shaku/DI rules configuration
#[derive(Debug, Clone, Deserialize)]
pub struct ShakuRulesConfig {
    /// Whether Shaku validation is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Require Component derive for port implementations
    #[serde(default = "default_true")]
    pub require_component_derive: bool,

    /// Require interface annotation
    #[serde(default = "default_true")]
    pub require_interface_annotation: bool,

    /// Allow direct service construction in tests
    #[serde(default = "default_true")]
    pub allow_direct_construction_in_tests: bool,
}

impl Default for ShakuRulesConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            require_component_derive: true,
            require_interface_annotation: true,
            allow_direct_construction_in_tests: true,
        }
    }
}

/// Validator enable/disable flags
#[derive(Debug, Clone, Deserialize)]
pub struct ValidatorsConfig {
    #[serde(default = "default_true")]
    pub dependency: bool,
    #[serde(default = "default_true")]
    pub organization: bool,
    #[serde(default = "default_true")]
    pub quality: bool,
    #[serde(default = "default_true")]
    pub solid: bool,
    #[serde(default = "default_true")]
    pub shaku: bool,
    #[serde(default = "default_true")]
    pub architecture: bool,
    #[serde(default = "default_true")]
    pub refactoring: bool,
    #[serde(default = "default_true")]
    pub naming: bool,
    #[serde(default = "default_true")]
    pub documentation: bool,
    #[serde(default = "default_true")]
    pub patterns: bool,
    #[serde(default = "default_true")]
    pub kiss: bool,
    #[serde(default = "default_true")]
    pub tests: bool,
    #[serde(default = "default_true")]
    pub async_patterns: bool,
    #[serde(default = "default_true")]
    pub error_boundary: bool,
    #[serde(default = "default_true")]
    pub performance: bool,
    #[serde(default = "default_true")]
    pub implementation: bool,
    #[serde(default = "default_true")]
    pub pmat: bool,
    #[serde(default = "default_true")]
    pub clean_architecture: bool,
}

impl Default for ValidatorsConfig {
    fn default() -> Self {
        Self {
            dependency: true,
            organization: true,
            quality: true,
            solid: true,
            shaku: true,
            architecture: true,
            refactoring: true,
            naming: true,
            documentation: true,
            patterns: true,
            kiss: true,
            tests: true,
            async_patterns: true,
            error_boundary: true,
            performance: true,
            implementation: true,
            pmat: true,
            clean_architecture: true,
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_error_severity() -> Severity {
    Severity::Error
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = FileConfig::default();
        assert!(config.validators.quality);
        assert!(config.validators.architecture);
        assert_eq!(config.rules.quality.max_file_lines, 500);
    }

    #[test]
    fn test_is_validator_enabled() {
        let config = FileConfig::default();
        assert!(config.is_validator_enabled("quality"));
        assert!(config.is_validator_enabled("architecture"));
        assert!(config.is_validator_enabled("unknown_validator"));
    }

    #[test]
    fn test_load_from_toml() {
        let toml_content = r#"
            [general]
            exclude_patterns = ["target/"]

            [rules.quality]
            max_file_lines = 300
            allow_unwrap_in_tests = false

            [validators]
            documentation = false
        "#;

        let config: FileConfig = toml::from_str(toml_content).unwrap();
        assert_eq!(config.general.exclude_patterns, vec!["target/"]);
        assert_eq!(config.rules.quality.max_file_lines, 300);
        assert!(!config.rules.quality.allow_unwrap_in_tests);
        assert!(!config.validators.documentation);
        assert!(config.validators.quality); // Default true
    }

    #[test]
    fn test_layer_boundaries_defaults() {
        let config = LayerBoundariesConfig::default();
        assert!(config.domain_internal_deps.is_empty());
        assert_eq!(config.application_internal_deps, vec!["mcb-domain"]);
        assert!(config
            .server_internal_deps
            .contains(&"mcb-infrastructure".to_string()));
        assert!(!config
            .server_internal_deps
            .contains(&"mcb-providers".to_string()));
    }

    #[test]
    fn test_magic_number_allowlist() {
        let config = OrganizationRulesConfig::default();
        assert!(config.magic_number_allowlist.contains(&0));
        assert!(config.magic_number_allowlist.contains(&86400));
        assert!(config.magic_number_allowlist.contains(&65536));
    }
}
