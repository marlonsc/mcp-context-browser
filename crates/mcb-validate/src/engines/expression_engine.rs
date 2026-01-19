//! Expression Engine Wrapper
//!
//! Wrapper for evalexpr crate providing simple boolean expression evaluation.
//! Use this engine for rules that don't require complex GRL syntax (when/then).

use async_trait::async_trait;
use evalexpr::{ContextWithMutableVariables, HashMapContext, Value as EvalValue};
use serde_json::Value;
use std::collections::HashMap;

use crate::Result;
use crate::engines::hybrid_engine::{RuleContext, RuleEngine, RuleViolation};
use crate::violation_trait::{Severity, ViolationCategory};

/// Wrapper for evalexpr engine
///
/// Evaluates simple boolean expressions like:
/// - `file_count > 500`
/// - `dependency_exists("serde")`
/// - `not contains_pattern(".unwrap()")`
pub struct ExpressionEngine {
    /// Cached contexts for repeated evaluations
    cached_contexts: HashMap<String, HashMapContext>,
}

impl Default for ExpressionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ExpressionEngine {
    pub fn new() -> Self {
        Self {
            cached_contexts: HashMap::new(),
        }
    }

    /// Build context from rule context for expression evaluation
    fn build_eval_context(&self, rule_context: &RuleContext) -> HashMapContext {
        let mut ctx = HashMapContext::new();

        // Add file count
        let _ = ctx.set_value(
            "file_count".to_string(),
            EvalValue::Int(rule_context.file_contents.len() as i64),
        );

        // Add workspace root path length
        let _ = ctx.set_value(
            "workspace_path_len".to_string(),
            EvalValue::Int(rule_context.workspace_root.to_string_lossy().len() as i64),
        );

        // Check for common patterns in files
        let has_unwrap = rule_context
            .file_contents
            .values()
            .any(|content| content.contains(".unwrap()"));
        let _ = ctx.set_value("has_unwrap".to_string(), EvalValue::Boolean(has_unwrap));

        let has_expect = rule_context
            .file_contents
            .values()
            .any(|content| content.contains(".expect("));
        let _ = ctx.set_value("has_expect".to_string(), EvalValue::Boolean(has_expect));

        // Check for async patterns
        let has_async = rule_context
            .file_contents
            .values()
            .any(|content| content.contains("async fn"));
        let _ = ctx.set_value("has_async".to_string(), EvalValue::Boolean(has_async));

        // Check for test patterns (supports both absolute and relative paths)
        let has_tests = rule_context.file_contents.keys().any(|path| {
            path.contains("/tests/")
                || path.starts_with("tests/")
                || path.contains("_test.rs")
                || path.contains("test_")
        });
        let _ = ctx.set_value("has_tests".to_string(), EvalValue::Boolean(has_tests));

        ctx
    }

    /// Evaluate a simple expression
    pub fn evaluate_expression(&self, expression: &str, context: &RuleContext) -> Result<bool> {
        let eval_ctx = self.build_eval_context(context);

        match evalexpr::eval_boolean_with_context(expression, &eval_ctx) {
            Ok(result) => Ok(result),
            Err(e) => Err(crate::ValidationError::Config(format!(
                "Expression evaluation error: {}",
                e
            ))),
        }
    }

    /// Evaluate with custom variables
    pub fn evaluate_with_variables(
        &self,
        expression: &str,
        variables: &HashMap<String, serde_json::Value>,
    ) -> Result<bool> {
        let mut ctx = HashMapContext::new();

        for (key, value) in variables {
            let eval_value = self.json_to_eval_value(value);
            let _ = ctx.set_value(key.clone(), eval_value);
        }

        match evalexpr::eval_boolean_with_context(expression, &ctx) {
            Ok(result) => Ok(result),
            Err(e) => Err(crate::ValidationError::Config(format!(
                "Expression evaluation error: {}",
                e
            ))),
        }
    }

    /// Convert JSON value to evalexpr value
    #[allow(clippy::only_used_in_recursion)]
    fn json_to_eval_value(&self, value: &serde_json::Value) -> EvalValue {
        match value {
            serde_json::Value::Null => EvalValue::Empty,
            serde_json::Value::Bool(b) => EvalValue::Boolean(*b),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    EvalValue::Int(i)
                } else if let Some(f) = n.as_f64() {
                    EvalValue::Float(f)
                } else {
                    EvalValue::Empty
                }
            }
            serde_json::Value::String(s) => EvalValue::String(s.clone()),
            serde_json::Value::Array(arr) => {
                let tuple: Vec<EvalValue> =
                    arr.iter().map(|v| self.json_to_eval_value(v)).collect();
                EvalValue::Tuple(tuple)
            }
            serde_json::Value::Object(_) => {
                // Objects not directly supported, convert to string
                EvalValue::String(value.to_string())
            }
        }
    }

    /// Execute expression-based rule and generate violations
    pub async fn execute_expression_rule(
        &self,
        rule_id: &str,
        expression: &str,
        context: &RuleContext,
        message: &str,
        severity: Severity,
        category: ViolationCategory,
    ) -> Result<Vec<RuleViolation>> {
        let mut violations = Vec::new();

        match self.evaluate_expression(expression, context) {
            Ok(true) => {
                // Expression matched - generate violation
                violations.push(
                    RuleViolation::new(rule_id, category, severity, message)
                        .with_context(format!("Expression: {}", expression)),
                );
            }
            Ok(false) => {
                // Expression did not match - no violation
            }
            Err(e) => {
                // Expression evaluation failed - report as warning
                violations.push(
                    RuleViolation::new(
                        rule_id,
                        ViolationCategory::Configuration,
                        Severity::Warning,
                        format!("Expression evaluation failed: {}", e),
                    )
                    .with_context(format!("Expression: {}", expression)),
                );
            }
        }

        Ok(violations)
    }
}

#[async_trait]
impl RuleEngine for ExpressionEngine {
    async fn execute(
        &self,
        rule_definition: &Value,
        context: &RuleContext,
    ) -> Result<Vec<RuleViolation>> {
        // Extract expression from rule definition
        let expression = rule_definition
            .get("expression")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::ValidationError::Config(
                    "Missing 'expression' field in rule definition".into(),
                )
            })?;

        let rule_id = rule_definition
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("EXPR_RULE");

        let message = rule_definition
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("Expression rule violation");

        let severity = rule_definition
            .get("severity")
            .and_then(|v| v.as_str())
            .map(|s| match s {
                "error" => Severity::Error,
                "warning" => Severity::Warning,
                _ => Severity::Info,
            })
            .unwrap_or(Severity::Warning);

        let category = rule_definition
            .get("category")
            .and_then(|v| v.as_str())
            .map(|c| match c {
                "architecture" => ViolationCategory::Architecture,
                "quality" => ViolationCategory::Quality,
                "performance" => ViolationCategory::Performance,
                _ => ViolationCategory::Quality,
            })
            .unwrap_or(ViolationCategory::Quality);

        self.execute_expression_rule(rule_id, expression, context, message, severity, category)
            .await
    }
}

impl Clone for ExpressionEngine {
    fn clone(&self) -> Self {
        Self {
            cached_contexts: self.cached_contexts.clone(),
        }
    }
}
