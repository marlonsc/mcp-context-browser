//! Validator Engine
//!
//! Uses validator and garde crates for field-level validations
//! and rule definition validation.

use serde_json::Value;
use validator::{Validate, ValidationErrors};

use crate::Result;

/// Engine for field validations using validator and garde
#[derive(Clone)]
pub struct ValidatorEngine;

impl Default for ValidatorEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidatorEngine {
    pub fn new() -> Self {
        Self
    }

    /// Validate rule definition structure
    pub fn validate_rule_definition(&self, rule_definition: &Value) -> Result<()> {
        // Convert JSON to a validatable structure
        let rule_config: RuleConfigValidation = serde_json::from_value(rule_definition.clone())
            .map_err(|e| crate::ValidationError::Parse {
                file: "rule_definition".into(),
                message: format!("Invalid rule structure: {}", e),
            })?;

        // Use validator for basic validations
        validator::Validate::validate(&rule_config)
            .map_err(|e| crate::ValidationError::Config(format!("Validation error: {:?}", e)))?;

        // Validate category if present
        if let Some(ref category) = rule_config.category {
            validate_category(category).map_err(|e| {
                crate::ValidationError::Config(format!("Invalid category: {:?}", e))
            })?;
        }

        // Validate engine if present
        if let Some(ref engine) = rule_config.engine {
            validate_engine(engine)
                .map_err(|e| crate::ValidationError::Config(format!("Invalid engine: {:?}", e)))?;
        }

        // Validate severity if present
        if let Some(ref severity) = rule_config.severity {
            validate_severity(severity).map_err(|e| {
                crate::ValidationError::Config(format!("Invalid severity: {:?}", e))
            })?;
        }

        Ok(())
    }
}

/// Structure for validating rule configurations
#[derive(Debug, Clone, Validate, serde::Deserialize)]
pub struct RuleConfigValidation {
    /// Rule ID validation
    #[validate(length(min = 4, max = 10))]
    pub id: Option<String>,

    /// Name validation
    #[validate(length(min = 3, max = 100))]
    pub name: Option<String>,

    /// Category validation
    pub category: Option<String>,

    /// Severity validation
    pub severity: Option<String>,

    /// Description validation
    #[validate(length(min = 10, max = 500))]
    pub description: Option<String>,

    /// Rationale validation
    #[validate(length(min = 10, max = 500))]
    pub rationale: Option<String>,

    /// Engine validation
    pub engine: Option<String>,

    /// Config validation
    pub config: Option<RuleConfigFields>,
}

/// Configuration fields validation
#[derive(Debug, Clone, serde::Deserialize)]
pub struct RuleConfigFields {
    /// Crate name validation
    pub crate_name: Option<String>,

    /// Forbidden prefixes validation
    pub forbidden_prefixes: Option<Vec<String>>,

    /// File patterns validation
    pub file_patterns: Option<Vec<String>>,

    /// Exclude patterns validation
    pub exclude_patterns: Option<Vec<String>>,

    /// Forbidden patterns validation
    pub forbidden_patterns: Option<Vec<String>>,
}

/// Validator functions for custom validations
fn validate_category(category: &str) -> std::result::Result<(), ValidationErrors> {
    let valid_categories = [
        "architecture",
        "quality",
        "solid",
        "dependency_injection",
        "configuration",
        "web_framework",
        "migration",
        "performance",
    ];

    if valid_categories.contains(&category) {
        Ok(())
    } else {
        let mut errors = ValidationErrors::new();
        errors.add(
            "category",
            validator::ValidationError::new("invalid_category"),
        );
        Err(errors)
    }
}

fn validate_severity(severity: &str) -> std::result::Result<(), ValidationErrors> {
    let valid_severities = ["error", "warning", "info"];

    if valid_severities.contains(&severity) {
        Ok(())
    } else {
        let mut errors = ValidationErrors::new();
        errors.add(
            "severity",
            validator::ValidationError::new("invalid_severity"),
        );
        Err(errors)
    }
}

fn validate_engine(engine: &str) -> std::result::Result<(), ValidationErrors> {
    let valid_engines = ["rust-rule-engine", "rusty-rules"];

    if valid_engines.contains(&engine) {
        Ok(())
    } else {
        let mut errors = ValidationErrors::new();
        errors.add("engine", validator::ValidationError::new("invalid_engine"));
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_rule_config() {
        let engine = ValidatorEngine::new();

        let valid_rule = serde_json::json!({
            "id": "TEST001",
            "name": "Test Rule",
            "category": "architecture",
            "severity": "error",
            "description": "This is a test rule with enough description",
            "rationale": "This rule exists for testing purposes and has enough rationale",
            "engine": "rust-rule-engine",
            "config": {
                "crate_name": "test-crate",
                "forbidden_prefixes": ["test"]
            }
        });

        assert!(engine.validate_rule_definition(&valid_rule).is_ok());
    }

    #[test]
    fn test_invalid_category() {
        let engine = ValidatorEngine::new();

        let invalid_rule = serde_json::json!({
            "id": "TEST001",
            "name": "Test Rule",
            "category": "invalid_category",
            "severity": "error",
            "description": "This is a test rule",
            "rationale": "This rule exists for testing",
            "engine": "rust-rule-engine"
        });

        assert!(engine.validate_rule_definition(&invalid_rule).is_err());
    }

    #[test]
    fn test_invalid_engine() {
        let engine = ValidatorEngine::new();

        let invalid_rule = serde_json::json!({
            "id": "TEST001",
            "name": "Test Rule",
            "category": "architecture",
            "severity": "error",
            "description": "This is a test rule",
            "rationale": "This rule exists for testing",
            "engine": "invalid_engine"
        });

        assert!(engine.validate_rule_definition(&invalid_rule).is_err());
    }
}
