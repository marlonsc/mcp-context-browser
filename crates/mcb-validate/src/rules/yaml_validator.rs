//! YAML Rule Validator
//!
//! Validates YAML rules against JSON Schema using jsonschema crate.

use jsonschema::JSONSchema;
use serde_json::Value;
use std::path::Path;

use crate::Result;

/// Validator for YAML-based rules using JSON Schema
pub struct YamlRuleValidator {
    schema: JSONSchema,
}

impl YamlRuleValidator {
    /// Create a new validator with the schema
    pub fn new() -> Result<Self> {
        let schema_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("crates/mcb-validate/rules/schema.json");

        let schema_content = std::fs::read_to_string(&schema_path)
            .map_err(|e| crate::ValidationError::Io(e))?;

        let schema_value: Value = serde_json::from_str(&schema_content)
            .map_err(|e| crate::ValidationError::Parse {
                file: schema_path,
                message: format!("Schema parse error: {}", e),
            })?;

        let schema = JSONSchema::compile(&schema_value)
            .map_err(|e| crate::ValidationError::Config(format!("Schema compilation error: {:?}", e)))?;

        Ok(Self { schema })
    }

    /// Validate a rule against the schema
    pub fn validate_rule(&self, rule: &Value) -> Result<()> {
        let validation_result = self.schema.validate(rule);

        let errors: Vec<jsonschema::ValidationError> = validation_result.err()
            .map(|iter| iter.collect())
            .unwrap_or_default();

        if !errors.is_empty() {
            let error_messages: Vec<String> = errors
                .iter()
                .map(|e| format!("{}: {}", e.instance_path, e.to_string()))
                .collect();

            return Err(crate::ValidationError::Config(format!(
                "Rule validation failed:\n{}",
                error_messages.join("\n")
            )));
        }

        Ok(())
    }

    /// Validate multiple rules
    pub fn validate_rules(&self, rules: &[Value]) -> Result<()> {
        for (index, rule) in rules.iter().enumerate() {
            if let Err(e) = self.validate_rule(rule) {
                return Err(crate::ValidationError::Config(format!(
                    "Rule at index {} validation failed: {}",
                    index, e
                )));
            }
        }
        Ok(())
    }

    /// Validate YAML against schema
    pub fn validate_yaml(&self, yaml_value: &serde_yaml::Value) -> Result<()> {
        let json_value = serde_json::to_value(yaml_value)
            .map_err(|e| crate::ValidationError::Parse {
                file: "yaml_rule".into(),
                message: format!("JSON conversion error: {}", e),
            })?;

        let errors: Vec<jsonschema::ValidationError> = self.schema.validate(&json_value).err()
            .map(|iter| iter.collect())
            .unwrap_or_default();
        if errors.is_empty() {
            Ok(())
        } else {
            Err(crate::ValidationError::Config(format!("Schema validation failed: {:?}", errors)))
        }
    }

    /// Create validator from custom schema
    pub fn from_schema(schema: &Value) -> Result<Self> {
        let compiled_schema = JSONSchema::compile(schema)
            .map_err(|e| crate::ValidationError::Config(format!("Schema compilation error: {:?}", e)))?;

        Ok(Self {
            schema: compiled_schema,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_loading() {
        let validator = YamlRuleValidator::new();
        assert!(validator.is_ok());
    }

    #[test]
    fn test_valid_rule_validation() {
        let validator = YamlRuleValidator::new().unwrap();

        let valid_rule = serde_json::json!({
            "schema": "rule/v1",
            "id": "TEST001",
            "name": "Test Rule",
            "category": "architecture",
            "severity": "error",
            "description": "This is a test rule with enough description to pass validation requirements",
            "rationale": "This rule exists for testing purposes and has enough rationale text",
            "engine": "rust-rule-engine",
            "config": {
                "crate_name": "test-crate"
            },
            "rule": {
                "type": "cargo_dependencies"
            }
        });

        assert!(validator.validate_rule(&valid_rule).is_ok());
    }

    #[test]
    fn test_invalid_rule_validation() {
        let validator = YamlRuleValidator::new().unwrap();

        // Missing required field
        let invalid_rule = serde_json::json!({
            "name": "Test Rule",
            "category": "architecture"
        });

        assert!(validator.validate_rule(&invalid_rule).is_err());
    }

    #[test]
    fn test_invalid_category() {
        let validator = YamlRuleValidator::new().unwrap();

        let invalid_rule = serde_json::json!({
            "schema": "rule/v1",
            "id": "TEST001",
            "name": "Test Rule",
            "category": "invalid_category",
            "severity": "error",
            "description": "This is a test rule description",
            "rationale": "This is the rationale for the rule",
            "engine": "rust-rule-engine",
            "rule": {}
        });

        assert!(validator.validate_rule(&invalid_rule).is_err());
    }

    #[test]
    fn test_invalid_severity() {
        let validator = YamlRuleValidator::new().unwrap();

        let invalid_rule = serde_json::json!({
            "schema": "rule/v1",
            "id": "TEST001",
            "name": "Test Rule",
            "category": "architecture",
            "severity": "invalid_severity",
            "description": "This is a test rule description",
            "rationale": "This is the rationale for the rule",
            "engine": "rust-rule-engine",
            "rule": {}
        });

        assert!(validator.validate_rule(&invalid_rule).is_err());
    }

    #[test]
    fn test_invalid_engine() {
        let validator = YamlRuleValidator::new().unwrap();

        let invalid_rule = serde_json::json!({
            "schema": "rule/v1",
            "id": "TEST001",
            "name": "Test Rule",
            "category": "architecture",
            "severity": "error",
            "description": "This is a test rule description",
            "rationale": "This is the rationale for the rule",
            "engine": "invalid_engine",
            "rule": {}
        });

        assert!(validator.validate_rule(&invalid_rule).is_err());
    }
}