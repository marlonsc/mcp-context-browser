//! YAML Rule Loader
//!
//! Automatically loads and validates YAML-based rules with template support.

use serde::{Deserialize, Serialize};
use serde_yaml;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use super::templates::TemplateEngine;
use super::yaml_validator::YamlRuleValidator;
use crate::Result;

/// Loaded and validated YAML rule
#[derive(Debug, Clone)]
pub struct ValidatedRule {
    pub id: String,
    pub name: String,
    pub category: String,
    pub severity: String,
    pub enabled: bool,
    pub description: String,
    pub rationale: String,
    pub engine: String,
    pub config: serde_json::Value,
    pub rule_definition: serde_json::Value,
    pub fixes: Vec<RuleFix>,
    /// Linter codes to execute (e.g., ["F401"] for Ruff, ["clippy::unwrap_used"] for Clippy)
    pub lint_select: Vec<String>,
    /// Custom message for violations
    pub message: Option<String>,
    /// AST selectors for multi-language pattern matching (Phase 2)
    pub selectors: Vec<AstSelector>,
    /// Tree-sitter query string for complex AST matching (Phase 2)
    pub ast_query: Option<String>,
    /// Metrics configuration for schema v3 rules (Phase 4)
    pub metrics: Option<MetricsConfig>,
}

/// Metrics configuration for rule/v3 rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Cognitive complexity threshold
    pub cognitive_complexity: Option<MetricThresholdConfig>,
    /// Cyclomatic complexity threshold
    pub cyclomatic_complexity: Option<MetricThresholdConfig>,
    /// Function length threshold
    pub function_length: Option<MetricThresholdConfig>,
    /// Nesting depth threshold
    pub nesting_depth: Option<MetricThresholdConfig>,
}

/// Configuration for a single metric threshold
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricThresholdConfig {
    /// Maximum allowed value
    pub max: u32,
    /// Severity level when threshold is exceeded
    pub severity: Option<String>,
    /// Languages this threshold applies to
    pub languages: Option<Vec<String>>,
}

/// AST selector for language-specific pattern matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstSelector {
    /// Programming language (e.g., "rust", "python")
    pub language: String,
    /// AST node type to match (e.g., "call_expression", "function_definition")
    pub node_type: String,
    /// Tree-sitter query pattern (optional, for complex matching)
    pub pattern: Option<String>,
}

/// Suggested fix for a rule violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleFix {
    pub fix_type: String,
    pub pattern: Option<String>,
    pub message: String,
}

/// YAML rule loader with automatic discovery
pub struct YamlRuleLoader {
    validator: YamlRuleValidator,
    template_engine: TemplateEngine,
    rules_dir: PathBuf,
}

impl YamlRuleLoader {
    /// Create a new YAML rule loader
    pub fn new(rules_dir: PathBuf) -> Result<Self> {
        Ok(Self {
            validator: YamlRuleValidator::new()?,
            template_engine: TemplateEngine::new(),
            rules_dir,
        })
    }

    /// Load all rules from the rules directory
    pub async fn load_all_rules(&mut self) -> Result<Vec<ValidatedRule>> {
        let mut rules = Vec::new();

        // Load templates first
        self.template_engine.load_templates(&self.rules_dir).await?;

        // Load rule files
        for entry in WalkDir::new(&self.rules_dir) {
            let entry = entry.map_err(|e| crate::ValidationError::Io(std::io::Error::other(e)))?;
            let path = entry.path();

            if self.is_rule_file(path) {
                let loaded_rules = self.load_rule_file(path).await?;
                rules.extend(loaded_rules);
            }
        }

        Ok(rules)
    }

    /// Load rules from a specific file
    pub async fn load_rule_file(&self, path: &Path) -> Result<Vec<ValidatedRule>> {
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(crate::ValidationError::Io)?;

        let yaml_value: serde_yaml::Value =
            serde_yaml::from_str(&content).map_err(|e| crate::ValidationError::Parse {
                file: path.to_path_buf(),
                message: format!("YAML parse error: {}", e),
            })?;

        // Check if this is a template
        if yaml_value
            .get("_base")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            // This is a template, skip it
            return Ok(vec![]);
        }

        // Apply template if specified
        let processed_yaml =
            if let Some(template_name) = yaml_value.get("_template").and_then(|v| v.as_str()) {
                self.template_engine
                    .apply_template(template_name, &yaml_value)?
            } else {
                yaml_value
            };

        // Handle extends
        let processed_yaml =
            if let Some(extends_name) = processed_yaml.get("_extends").and_then(|v| v.as_str()) {
                self.template_engine
                    .extend_rule(extends_name, &processed_yaml)?
            } else {
                processed_yaml
            };

        // Convert to JSON for validation
        let json_value: serde_json::Value =
            serde_json::to_value(processed_yaml).map_err(|e| crate::ValidationError::Parse {
                file: path.to_path_buf(),
                message: format!("YAML to JSON conversion error: {}", e),
            })?;

        // Validate against schema
        self.validator.validate_rule(&json_value)?;

        // Convert to validated rule
        let validated_rule = self.yaml_to_validated_rule(json_value)?;

        Ok(vec![validated_rule])
    }

    /// Check if a file is a rule file
    fn is_rule_file(&self, path: &Path) -> bool {
        path.extension().and_then(|ext| ext.to_str()) == Some("yml")
            && !path.to_string_lossy().contains("/templates/")
    }

    /// Convert YAML/JSON value to ValidatedRule
    fn yaml_to_validated_rule(&self, value: serde_json::Value) -> Result<ValidatedRule> {
        let obj = value
            .as_object()
            .ok_or_else(|| crate::ValidationError::Config("Rule must be an object".to_string()))?;

        let id = obj
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::ValidationError::Config("Rule must have an 'id' field".to_string())
            })?
            .to_string();

        let name = obj
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unnamed Rule")
            .to_string();

        let category = obj
            .get("category")
            .and_then(|v| v.as_str())
            .unwrap_or("quality")
            .to_string();

        let severity = obj
            .get("severity")
            .and_then(|v| v.as_str())
            .unwrap_or("warning")
            .to_string();

        let enabled = obj.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true);

        let description = obj
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("No description provided")
            .to_string();

        let rationale = obj
            .get("rationale")
            .and_then(|v| v.as_str())
            .unwrap_or("No rationale provided")
            .to_string();

        let engine = obj
            .get("engine")
            .and_then(|v| v.as_str())
            .unwrap_or("rusty-rules")
            .to_string();

        let config = obj
            .get("config")
            .cloned()
            .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

        let rule_definition = obj
            .get("rule")
            .cloned()
            .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

        let fixes = obj
            .get("fixes")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|fix| {
                        if let Some(fix_obj) = fix.as_object() {
                            Some(RuleFix {
                                fix_type: fix_obj.get("type")?.as_str()?.to_string(),
                                pattern: fix_obj
                                    .get("pattern")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string()),
                                message: fix_obj.get("message")?.as_str()?.to_string(),
                            })
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Extract lint_select codes (for Ruff/Clippy integration)
        let lint_select = obj
            .get("lint_select")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|code| code.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        // Extract custom message
        let message = obj
            .get("message")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Extract AST selectors (Phase 2)
        let selectors = obj
            .get("selectors")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|sel| {
                        if let Some(sel_obj) = sel.as_object() {
                            Some(AstSelector {
                                language: sel_obj.get("language")?.as_str()?.to_string(),
                                node_type: sel_obj.get("node_type")?.as_str()?.to_string(),
                                pattern: sel_obj
                                    .get("pattern")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string()),
                            })
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Extract ast_query (Phase 2)
        let ast_query = obj
            .get("ast_query")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Extract metrics configuration (Phase 4 - rule/v3)
        let metrics = obj.get("metrics").and_then(|v| {
            serde_json::from_value::<MetricsConfig>(v.clone()).ok()
        });

        Ok(ValidatedRule {
            id,
            name,
            category,
            severity,
            enabled,
            description,
            rationale,
            engine,
            config,
            rule_definition,
            fixes,
            lint_select,
            message,
            selectors,
            ast_query,
            metrics,
        })
    }

    /// Get rule file path for a rule ID
    pub fn get_rule_path(&self, rule_id: &str) -> Option<PathBuf> {
        // This would need a more sophisticated mapping
        // For now, just search in the rules directory
        for entry in WalkDir::new(&self.rules_dir).into_iter().flatten() {
            let path = entry.path();
            if self.is_rule_file(path) {
                if let Ok(content) = std::fs::read_to_string(path) {
                    if content.contains(&format!("id: {}", rule_id)) {
                        return Some(path.to_path_buf());
                    }
                }
            }
        }
        None
    }
}
