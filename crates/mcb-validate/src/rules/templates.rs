//! Template Engine for YAML Rules
//!
//! Provides template inheritance and variable substitution for DRY rule definitions.

use std::collections::HashMap;
use std::path::Path;
use serde_yaml;
use walkdir::WalkDir;

use crate::Result;

/// Template engine for YAML rules with inheritance and substitution
pub struct TemplateEngine {
    templates: HashMap<String, serde_yaml::Value>,
}

impl TemplateEngine {
    /// Create a new template engine
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
        }
    }

    /// Load all templates from the templates directory
    pub async fn load_templates(&mut self, rules_dir: &Path) -> Result<()> {
        let templates_dir = rules_dir.join("templates");

        if !templates_dir.exists() {
            return Ok(()); // No templates directory, that's fine
        }

        for entry in WalkDir::new(&templates_dir) {
            let entry = entry.map_err(|e| crate::ValidationError::Io(e.into()))?;
            let path = entry.path();

            if path.extension().and_then(|ext| ext.to_str()) == Some("yml") {
                let template_name = path.file_stem()
                    .and_then(|name| name.to_str())
                    .ok_or_else(|| crate::ValidationError::Config(
                        format!("Invalid template filename: {:?}", path)
                    ))?;

                let content = tokio::fs::read_to_string(path).await
                    .map_err(|e| crate::ValidationError::Io(e))?;

                let template: serde_yaml::Value = serde_yaml::from_str(&content)
                    .map_err(|e| crate::ValidationError::Parse {
                        file: path.to_path_buf(),
                        message: format!("Template parse error: {}", e),
                    })?;

                // Verify this is actually a template
                if template.get("_base").and_then(|v| v.as_bool()).unwrap_or(false) {
                    self.templates.insert(template_name.to_string(), template);
                }
            }
        }

        Ok(())
    }

    /// Apply a template to a rule definition
    pub fn apply_template(&self, template_name: &str, rule: &serde_yaml::Value) -> Result<serde_yaml::Value> {
        let template = self.templates.get(template_name)
            .ok_or_else(|| crate::ValidationError::Config(
                format!("Template '{}' not found", template_name)
            ))?;

        // Start with the template as base
        let mut result = template.clone();

        // Override with rule-specific values
        self.merge_yaml_values(&mut result, rule);

        // Process variable substitutions
        self.substitute_variables(&mut result, rule)?;

        // Remove template metadata
        if let Some(obj) = result.as_mapping_mut() {
            obj.remove("_base");
            obj.remove("name"); // Template name, not rule name
        }

        Ok(result)
    }

    /// Extend a rule with another rule (inheritance)
    pub fn extend_rule(&self, _extends_name: &str, rule: &serde_yaml::Value) -> Result<serde_yaml::Value> {
        // For now, just return the rule as-is
        // In a full implementation, this would look up the base rule
        // and merge it with the extending rule
        Ok(rule.clone())
    }

    /// Merge two YAML values (rule overrides template)
    fn merge_yaml_values(&self, base: &mut serde_yaml::Value, override_value: &serde_yaml::Value) {
        if let (serde_yaml::Value::Mapping(base_map), serde_yaml::Value::Mapping(override_map)) =
            (base, override_value) {

            for (key, override_val) in override_map {
                base_map.insert(key.clone(), override_val.clone());
            }
        }
    }

    /// Substitute variables in the form {{variable_name}}
    fn substitute_variables(&self, value: &mut serde_yaml::Value, variables: &serde_yaml::Value) -> Result<()> {
        match value {
            serde_yaml::Value::String(s) => {
                *s = self.substitute_string(s, variables)?;
            }
            serde_yaml::Value::Mapping(map) => {
                for val in map.values_mut() {
                    self.substitute_variables(val, variables)?;
                }
            }
            serde_yaml::Value::Sequence(seq) => {
                for item in seq {
                    self.substitute_variables(item, variables)?;
                }
            }
            _ => {} // Other types don't need substitution
        }
        Ok(())
    }

    /// Substitute variables in a string
    fn substitute_string(&self, input: &str, variables: &serde_yaml::Value) -> Result<String> {
        let mut result = input.to_string();

        // Find all {{variable}} patterns
        let var_pattern = regex::Regex::new(r"\{\{(\w+)\}\}")
            .map_err(|e| crate::ValidationError::Config(format!("Regex error: {}", e)))?;

        for capture in var_pattern.captures_iter(input) {
            if let Some(var_name) = capture.get(1) {
                let var_value = self.get_variable_value(variables, var_name.as_str())?;
                result = result.replace(&format!("{{{{{}}}}}", var_name.as_str()), &var_value);
            }
        }

        Ok(result)
    }

    /// Get variable value from the variables YAML
    fn get_variable_value(&self, variables: &serde_yaml::Value, var_name: &str) -> Result<String> {
        if let Some(value) = variables.get(var_name) {
            match value {
                serde_yaml::Value::String(s) => Ok(s.clone()),
                serde_yaml::Value::Number(n) => Ok(n.to_string()),
                serde_yaml::Value::Bool(b) => Ok(b.to_string()),
                serde_yaml::Value::Sequence(seq) => {
                    // For arrays, join with commas (for patterns)
                    let strings: Vec<String> = seq
                        .iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| s.to_string())
                        .collect();
                    Ok(strings.join(","))
                }
                _ => Ok(format!("{:?}", value)),
            }
        } else {
            Err(crate::ValidationError::Config(format!("Variable '{}' not found", var_name)))
        }
    }

    /// Get available templates
    pub fn get_templates(&self) -> &HashMap<String, serde_yaml::Value> {
        &self.templates
    }

    /// Check if a template exists
    pub fn has_template(&self, name: &str) -> bool {
        self.templates.contains_key(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_load_templates() {
        let temp_dir = TempDir::new().unwrap();
        let rules_dir = temp_dir.path().join("rules");
        let templates_dir = rules_dir.join("templates");
        std::fs::create_dir_all(&templates_dir).unwrap();

        // Create a template
        let template_content = r#"
_base: true
name: "test_template"
category: "architecture"
severity: "error"
config:
  crate_name: "{{crate_name}}"
"#;

        std::fs::write(templates_dir.join("test-template.yml"), template_content).unwrap();

        let mut engine = TemplateEngine::new();
        engine.load_templates(&rules_dir).await.unwrap();

        assert!(engine.has_template("test-template"));
    }

    #[test]
    fn test_apply_template() {
        let mut engine = TemplateEngine::new();

        // Add a template
        let template: serde_yaml::Value = serde_yaml::from_str(r#"
_base: true
name: "test_template"
category: "architecture"
severity: "error"
config:
  crate_name: "{{crate_name}}"
"#).unwrap();

        engine.templates.insert("test_template".to_string(), template);

        // Create a rule that uses the template
        let rule: serde_yaml::Value = serde_yaml::from_str(r#"
_template: "test_template"
id: "TEST001"
config:
  crate_name: "my-crate"
"#).unwrap();

        let result = engine.apply_template("test_template", &rule).unwrap();

        // Check that template was applied and variables substituted
        assert_eq!(result.get("category").unwrap().as_str().unwrap(), "architecture");
        assert_eq!(result.get("severity").unwrap().as_str().unwrap(), "error");
        assert_eq!(result.get("id").unwrap().as_str().unwrap(), "TEST001");

        let config = result.get("config").unwrap();
        assert_eq!(config.get("crate_name").unwrap().as_str().unwrap(), "my-crate");
    }

    #[test]
    fn test_variable_substitution() {
        let engine = TemplateEngine::new();

        let variables: serde_yaml::Value = serde_yaml::from_str(r#"
crate_name: "test-crate"
forbidden_prefixes:
  - "mcb-"
  - "forbidden-"
"#).unwrap();

        let mut test_string = "{{crate_name}} should not use {{forbidden_prefixes}}".to_string();
        let result = engine.substitute_string(&test_string, &variables).unwrap();

        assert_eq!(result, "test-crate should not use mcb-,forbidden-");
    }
}