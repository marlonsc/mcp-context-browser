//! Rule Registry System
//!
//! Provides declarative rule definitions and registry management.

pub mod registry; // Legacy registry (kept for compatibility)
pub mod templates;
pub mod yaml_loader;
pub mod yaml_validator;

// Re-export legacy for compatibility
pub use registry::{clean_architecture_rules, layer_boundary_rules, Rule, RuleRegistry};

// Re-export YAML system
pub use templates::TemplateEngine;
pub use yaml_loader::{
    AstSelector, MetricThresholdConfig, MetricsConfig, RuleFix, ValidatedRule, YamlRuleLoader,
};
pub use yaml_validator::YamlRuleValidator;
