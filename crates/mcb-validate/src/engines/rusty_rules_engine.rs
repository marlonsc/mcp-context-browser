//! Rusty Rules Engine Wrapper
//!
//! Wrapper for rusty-rules crate with JSON DSL and composition support.

use async_trait::async_trait;
use glob::Pattern;
use serde_json::Value;
use std::collections::HashMap;

use crate::engines::hybrid_engine::RuleViolation;
use crate::violation_trait::{Severity, ViolationCategory};
use crate::Result;

use super::hybrid_engine::{RuleContext, RuleEngine};

/// Wrapper for rusty-rules engine
pub struct RustyRulesEngineWrapper {
    // In a real implementation, this would hold the actual rusty-rules instance
    rule_definitions: HashMap<String, RustyRule>,
}

/// Rusty rule definition with composition support
#[derive(Debug, Clone)]
pub struct RustyRule {
    pub rule_type: String,
    pub condition: Condition,
    pub action: Action,
}

/// Conditions with composition (all/any/not)
#[derive(Debug, Clone)]
pub enum Condition {
    /// All conditions must be true
    All(Vec<Condition>),
    /// Any condition must be true
    Any(Vec<Condition>),
    /// Negate condition
    Not(Box<Condition>),
    /// Simple condition
    Simple {
        fact_type: String,
        field: String,
        operator: String,
        value: Value,
    },
}

/// Actions to execute when condition matches
#[derive(Debug, Clone)]
pub enum Action {
    Violation { message: String, severity: Severity },
    Custom(String),
}

impl Default for RustyRulesEngineWrapper {
    fn default() -> Self {
        Self::new()
    }
}

impl RustyRulesEngineWrapper {
    pub fn new() -> Self {
        Self {
            rule_definitions: HashMap::new(),
        }
    }

    /// Parse rule definition from JSON
    pub fn parse_rule_definition(&mut self, rule_id: String, definition: &Value) -> Result<()> {
        let rule = self.parse_rule_from_json(definition)?;
        self.rule_definitions.insert(rule_id, rule);
        Ok(())
    }

    fn parse_rule_from_json(&self, definition: &Value) -> Result<RustyRule> {
        // Parse rule type
        let rule_type = definition
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("generic")
            .to_string();

        // Parse condition
        let condition = if let Some(condition_json) = definition.get("condition") {
            self.parse_condition(condition_json)?
        } else {
            Condition::All(vec![]) // Default empty condition
        };

        // Parse action
        let action = if let Some(action_json) = definition.get("action") {
            self.parse_action(action_json)?
        } else {
            Action::Violation {
                message: "Rule violation".to_string(),
                severity: Severity::Warning,
            }
        };

        Ok(RustyRule {
            rule_type,
            condition,
            action,
        })
    }

    #[allow(clippy::only_used_in_recursion)]
    fn parse_condition(&self, condition_json: &Value) -> Result<Condition> {
        if let Some(all_conditions) = condition_json.get("all") {
            if let Some(conditions_array) = all_conditions.as_array() {
                let conditions = conditions_array
                    .iter()
                    .map(|c| self.parse_condition(c))
                    .collect::<Result<Vec<_>>>()?;
                return Ok(Condition::All(conditions));
            }
        }

        if let Some(any_conditions) = condition_json.get("any") {
            if let Some(conditions_array) = any_conditions.as_array() {
                let conditions = conditions_array
                    .iter()
                    .map(|c| self.parse_condition(c))
                    .collect::<Result<Vec<_>>>()?;
                return Ok(Condition::Any(conditions));
            }
        }

        if let Some(not_condition) = condition_json.get("not") {
            let condition = self.parse_condition(not_condition)?;
            return Ok(Condition::Not(Box::new(condition)));
        }

        // Simple condition
        let fact_type = condition_json
            .get("fact_type")
            .and_then(|v| v.as_str())
            .unwrap_or("generic")
            .to_string();

        let field = condition_json
            .get("field")
            .and_then(|v| v.as_str())
            .unwrap_or("value")
            .to_string();

        let operator = condition_json
            .get("operator")
            .and_then(|v| v.as_str())
            .unwrap_or("equals")
            .to_string();

        let value = condition_json.get("value").cloned().unwrap_or(Value::Null);

        Ok(Condition::Simple {
            fact_type,
            field,
            operator,
            value,
        })
    }

    fn parse_action(&self, action_json: &Value) -> Result<Action> {
        if let Some(violation) = action_json.get("violation") {
            let message = violation
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("Rule violation")
                .to_string();

            let severity = violation
                .get("severity")
                .and_then(|v| v.as_str())
                .map(|s| match s {
                    "error" => Severity::Error,
                    "warning" => Severity::Warning,
                    "info" => Severity::Info,
                    _ => Severity::Warning,
                })
                .unwrap_or(Severity::Warning);

            return Ok(Action::Violation { message, severity });
        }

        Ok(Action::Custom("Custom action".to_string()))
    }

    /// Evaluate condition against context (reserved for rule composition)
    #[allow(dead_code)]
    fn evaluate_condition(&self, condition: &Condition, context: &RuleContext) -> bool {
        match condition {
            Condition::All(conditions) => conditions
                .iter()
                .all(|c| self.evaluate_condition(c, context)),
            Condition::Any(conditions) => conditions
                .iter()
                .any(|c| self.evaluate_condition(c, context)),
            Condition::Not(condition) => !self.evaluate_condition(condition, context),
            Condition::Simple {
                fact_type,
                field,
                operator,
                value,
            } => self.evaluate_simple_condition(fact_type, field, operator, value, context),
        }
    }

    #[allow(dead_code)]
    fn evaluate_simple_condition(
        &self,
        fact_type: &str,
        field: &str,
        operator: &str,
        expected_value: &Value,
        context: &RuleContext,
    ) -> bool {
        match fact_type {
            "cargo_dependencies" => {
                self.evaluate_cargo_dependencies(field, operator, expected_value, context)
            }
            "file_pattern" => self.evaluate_file_pattern(field, operator, expected_value, context),
            "file_size" => self.evaluate_file_size(field, operator, expected_value, context),
            "ast_pattern" => self.evaluate_ast_pattern(field, operator, expected_value, context),
            _ => false,
        }
    }

    #[allow(dead_code)]
    fn evaluate_cargo_dependencies(
        &self,
        field: &str,
        operator: &str,
        expected_value: &Value,
        context: &RuleContext,
    ) -> bool {
        match field {
            "not_exists" => {
                if operator == "pattern" {
                    if let Some(pattern) = expected_value.as_str() {
                        return !self.has_forbidden_dependency(pattern, context);
                    }
                }
                false
            }
            _ => false,
        }
    }

    fn has_forbidden_dependency(&self, pattern: &str, context: &RuleContext) -> bool {
        // Check Cargo.toml files for forbidden dependencies
        use glob::Pattern;
        use walkdir::WalkDir;

        let cargo_pattern = Pattern::new("**/Cargo.toml").unwrap();
        let trimmed_pattern = pattern.trim_matches('"');
        let pattern_prefix = trimmed_pattern.trim_end_matches('*');

        for entry in WalkDir::new(&context.workspace_root).into_iter().flatten() {
            let path = entry.path();
            if cargo_pattern.matches_path(path) {
                if let Ok(content) = std::fs::read_to_string(path) {
                    // Try to parse as TOML and check dependencies section
                    if let Ok(toml_value) = content.parse::<toml::Value>() {
                        if let Some(dependencies) = toml_value.get("dependencies") {
                            if let Some(deps_table) = dependencies.as_table() {
                                for dep_name in deps_table.keys() {
                                    if dep_name.starts_with(pattern_prefix) {
                                        return true;
                                    }
                                }
                            }
                        }
                    } else {
                        // Fallback to simple pattern matching
                        for line in content.lines() {
                            let line = line.trim();
                            if line.contains('=') {
                                let dep_name = line.split('=').next().unwrap().trim();
                                if dep_name.starts_with(pattern_prefix) {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }

        false
    }

    #[allow(dead_code)]
    fn evaluate_file_pattern(
        &self,
        field: &str,
        operator: &str,
        expected_value: &Value,
        context: &RuleContext,
    ) -> bool {
        match field {
            "matches" => {
                if operator == "pattern" {
                    if let Some(pattern) = expected_value.as_str() {
                        return Pattern::new(pattern)
                            .map(|p| {
                                context
                                    .file_contents
                                    .keys()
                                    .any(|path| p.matches_path(std::path::Path::new(path)))
                            })
                            .unwrap_or(false);
                    }
                }
                false
            }
            _ => false,
        }
    }

    #[allow(dead_code)]
    fn evaluate_file_size(
        &self,
        field: &str,
        operator: &str,
        expected_value: &Value,
        context: &RuleContext,
    ) -> bool {
        match field {
            "exceeds_limit" => {
                if operator == "extension" {
                    if let Some(extension) = expected_value.as_str() {
                        // Check if any file with the given extension exceeds the configured limit
                        // For simplicity, we'll use a hardcoded limit of 500 for now
                        let max_lines = 500;

                        for (file_path, content) in &context.file_contents {
                            if file_path.ends_with(extension) {
                                let line_count = content.lines().count();
                                if line_count > max_lines {
                                    // Check exclusions
                                    let path_str = file_path.to_string();
                                    if !path_str.contains("/tests/")
                                        && !path_str.contains("/target/")
                                        && !path_str.ends_with("_test.rs") {
                                        return true; // Found a file that exceeds the limit
                                    }
                                }
                            }
                        }
                    }
                }
                false
            }
            _ => false,
        }
    }

    #[allow(dead_code)]
    fn evaluate_ast_pattern(
        &self,
        field: &str,
        operator: &str,
        expected_value: &Value,
        _context: &RuleContext,
    ) -> bool {
        // Simplified AST pattern evaluation
        match field {
            "contains" => {
                if operator == "pattern" {
                    if let Some(pattern) = expected_value.as_str() {
                        // In real implementation, this would analyze AST
                        return pattern == ".unwrap()" || pattern == ".expect(";
                    }
                }
                false
            }
            _ => false,
        }
    }

    /// Execute rule action
    #[allow(dead_code)]
    fn execute_action(
        &self,
        action: &Action,
        rule_id: &str,
        _context: &RuleContext,
    ) -> Vec<RuleViolation> {
        match action {
            Action::Violation { message, severity } => {
                vec![RuleViolation::new(
                    rule_id,
                    ViolationCategory::Architecture, // Could be made configurable
                    *severity,
                    message.clone(),
                )
                .with_context(format!("Rule triggered: {}", rule_id))]
            }
            Action::Custom(action_str) => {
                // Handle custom actions
                vec![RuleViolation::new(
                    rule_id,
                    ViolationCategory::Quality,
                    Severity::Info,
                    format!("Custom action: {}", action_str),
                )
                .with_context("Custom rule action")]
            }
        }
    }
}

#[async_trait]
impl RuleEngine for RustyRulesEngineWrapper {
    async fn execute(
        &self,
        rule_definition: &Value,
        context: &RuleContext,
    ) -> Result<Vec<RuleViolation>> {
        // In a real implementation, this would use the rusty-rules engine
        // For now, we'll simulate the behavior

        let _rule_id = "unknown"; // Would be passed in real implementation

        if let Some(rule_type) = rule_definition.get("type").and_then(|v| v.as_str()) {
            match rule_type {
                "cargo_dependencies" => {
                    self.execute_cargo_dependency_rule(rule_definition, context)
                        .await
                }
                "file_size" => {
                    self.execute_file_size_rule(rule_definition, context)
                        .await
                }
                "ast_pattern" => {
                    self.execute_ast_pattern_rule(rule_definition, context)
                        .await
                }
                _ => Ok(vec![]),
            }
        } else {
            Ok(vec![])
        }
    }
}

impl RustyRulesEngineWrapper {
    async fn execute_cargo_dependency_rule(
        &self,
        rule_definition: &Value,
        context: &RuleContext,
    ) -> Result<Vec<RuleViolation>> {
        let mut violations = Vec::new();

        // Get the condition (default to "not_exists" for backwards compatibility)
        let condition = rule_definition
            .get("condition")
            .and_then(|v| v.as_str())
            .unwrap_or("not_exists");

        if let Some(forbidden_pattern) = rule_definition.get("pattern").and_then(|v| v.as_str()) {
            let has_forbidden = self.has_forbidden_dependency(forbidden_pattern, context);

            match condition {
                "not_exists" => {
                    // Create violation if forbidden dependency EXISTS (should NOT exist)
                    if has_forbidden {
                        violations.push(
                            RuleViolation::new(
                                "CARGO_DEP",
                                ViolationCategory::Architecture,
                                Severity::Error,
                                "Forbidden dependency found",
                            )
                            .with_context(format!("Pattern: {}", forbidden_pattern)),
                        );
                    }
                }
                "exists" => {
                    // Create violation if forbidden dependency does NOT exist (should exist)
                    if !has_forbidden {
                        violations.push(
                            RuleViolation::new(
                                "CARGO_DEP",
                                ViolationCategory::Architecture,
                                Severity::Error,
                                "Required dependency not found",
                            )
                            .with_context(format!("Pattern: {}", forbidden_pattern)),
                        );
                    }
                }
                _ => {
                    // Unknown condition, do nothing
                }
            }
        }

        Ok(violations)
    }

    async fn execute_ast_pattern_rule(
        &self,
        rule_definition: &Value,
        context: &RuleContext,
    ) -> Result<Vec<RuleViolation>> {
        let mut violations = Vec::new();

        if let Some(forbidden) = rule_definition.get("forbidden").and_then(|v| v.as_array()) {
            for pattern_value in forbidden {
                if let Some(pattern) = pattern_value.as_str() {
                    // Simplified check - in real implementation would use AST analysis
                    for (file_path, content) in &context.file_contents {
                        if content.contains(pattern) {
                            violations.push(
                                RuleViolation::new(
                                    "AST_PATTERN",
                                    ViolationCategory::Quality,
                                    Severity::Error,
                                    format!("Found forbidden pattern: {}", pattern),
                                )
                                .with_file(std::path::PathBuf::from(file_path))
                                .with_context(format!("Pattern: {}", pattern)),
                            );
                        }
                    }
                }
            }
        }

        Ok(violations)
    }

    async fn execute_file_size_rule(
        &self,
        rule_definition: &Value,
        context: &RuleContext,
    ) -> Result<Vec<RuleViolation>> {
        let mut violations = Vec::new();

        // Get the condition (default to "exceeds_limit")
        let condition = rule_definition
            .get("condition")
            .and_then(|v| v.as_str())
            .unwrap_or("exceeds_limit");

        // Get the pattern (file extension)
        let pattern = rule_definition
            .get("pattern")
            .and_then(|v| v.as_str())
            .unwrap_or(".rs");

        // Get the message
        let message = rule_definition
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("File exceeds size limit");

        if condition == "exceeds_limit" {
            // Check files that match the pattern
            let max_lines = 500; // Hardcoded for now, could be configurable

            for (file_path, content) in &context.file_contents {
                if file_path.ends_with(pattern) {
                    let line_count = content.lines().count();

                    // Check exclusions
                    let path_str = file_path.to_string();
                    let should_exclude = path_str.contains("/tests/")
                        || path_str.contains("/target/")
                        || path_str.ends_with("_test.rs");

                    if line_count > max_lines && !should_exclude {
                        violations.push(
                            RuleViolation::new(
                                "QUAL006",
                                ViolationCategory::Quality,
                                Severity::Warning,
                                format!("{}: {} lines (max: {})", message, line_count, max_lines),
                            )
                            .with_file(std::path::PathBuf::from(file_path))
                            .with_context(format!("File: {}, Lines: {}", file_path, line_count)),
                        );
                    }
                }
            }
        }

        Ok(violations)
    }
}

impl Clone for RustyRulesEngineWrapper {
    fn clone(&self) -> Self {
        Self {
            rule_definitions: self.rule_definitions.clone(),
        }
    }
}
