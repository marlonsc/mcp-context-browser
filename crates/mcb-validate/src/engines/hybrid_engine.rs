//! Hybrid Rule Engine
//!
//! Orchestrates multiple rule engines for maximum flexibility:
//! - rust-rule-engine: Complex rules with RETE algorithm
//! - rusty-rules: Composable rules with JSON DSL
//! - validator/garde: Field validations

use std::collections::HashMap;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::violation_trait::{Violation, ViolationCategory, Severity};
use crate::ValidationConfig;
use crate::Result;

use super::rust_rule_engine::RustRuleEngineWrapper;
use super::rusty_rules_engine::RustyRulesEngineWrapper;
use super::validator_engine::ValidatorEngine;

/// Types of rule engines supported
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuleEngineType {
    RustRuleEngine,
    RustyRules,
}

/// Concrete violation structure for rule engines
#[derive(Debug, Clone)]
pub struct RuleViolation {
    pub id: String,
    pub category: ViolationCategory,
    pub severity: Severity,
    pub message: String,
    pub file: Option<std::path::PathBuf>,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub context: Option<String>,
}

impl Violation for RuleViolation {
    fn id(&self) -> &str {
        &self.id
    }

    fn category(&self) -> ViolationCategory {
        self.category
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn file(&self) -> Option<&std::path::PathBuf> {
        self.file.as_ref()
    }

    fn line(&self) -> Option<usize> {
        self.line
    }

    fn message(&self) -> String {
        self.message.clone()
    }
}

impl std::fmt::Display for RuleViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.id, self.message)
    }
}

impl RuleViolation {
    pub fn new(id: impl Into<String>, category: ViolationCategory, severity: Severity, message: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            category,
            severity,
            message: message.into(),
            file: None,
            line: None,
            column: None,
            context: None,
        }
    }

    pub fn with_file(mut self, file: std::path::PathBuf) -> Self {
        self.file = Some(file);
        self
    }

    pub fn with_location(mut self, line: usize, column: usize) -> Self {
        self.line = Some(line);
        self.column = Some(column);
        self
    }

    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }
}

/// Result of rule execution
#[derive(Debug, Clone)]
pub struct RuleResult {
    pub violations: Vec<RuleViolation>,
    pub execution_time_ms: u64,
}

/// Context passed to rule engines during execution
#[derive(Debug, Clone)]
pub struct RuleContext {
    pub workspace_root: std::path::PathBuf,
    pub config: ValidationConfig,
    pub ast_data: HashMap<String, serde_json::Value>,
    pub cargo_data: HashMap<String, serde_json::Value>,
    pub file_contents: HashMap<String, String>,
}

/// Hybrid engine that coordinates multiple rule engines
pub struct HybridRuleEngine {
    rust_rule_engine: RustRuleEngineWrapper,
    rusty_rules_engine: RustyRulesEngineWrapper,
    validator_engine: ValidatorEngine,
    cache: HashMap<String, Vec<u8>>, // Compiled rule cache
}

impl HybridRuleEngine {
    /// Create a new hybrid rule engine
    pub fn new() -> Self {
        Self {
            rust_rule_engine: RustRuleEngineWrapper::new(),
            rusty_rules_engine: RustyRulesEngineWrapper::new(),
            validator_engine: ValidatorEngine::new(),
            cache: HashMap::new(),
        }
    }

    /// Execute a rule using the appropriate engine
    pub async fn execute_rule(
        &self,
        _rule_id: &str,
        engine_type: RuleEngineType,
        rule_definition: &serde_json::Value,
        context: &RuleContext,
    ) -> Result<RuleResult> {
        let start_time = std::time::Instant::now();

        let violations = match engine_type {
            RuleEngineType::RustRuleEngine => {
                self.rust_rule_engine.execute(rule_definition, context).await?
            }
            RuleEngineType::RustyRules => {
                self.rusty_rules_engine.execute(rule_definition, context).await?
            }
        };

        let execution_time = start_time.elapsed().as_millis() as u64;

        Ok(RuleResult {
            violations,
            execution_time_ms: execution_time,
        })
    }

    /// Execute multiple rules in parallel
    pub async fn execute_rules_batch(
        &self,
        rules: Vec<(String, RuleEngineType, serde_json::Value)>,
        context: &RuleContext,
    ) -> Result<Vec<(String, RuleResult)>> {
        let mut handles = Vec::new();

        for (rule_id, engine_type, rule_def) in rules {
            let engine = self.clone();
            let ctx = context.clone();
            let rule_id_clone = rule_id.clone();

            let handle = tokio::spawn(async move {
                let result = engine
                    .execute_rule(&rule_id_clone, engine_type, &rule_def, &ctx)
                    .await;
                (rule_id_clone, result)
            });

            handles.push(handle);
        }

        let mut results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok((rule_id, Ok(result))) => results.push((rule_id, result)),
                Ok((rule_id, Err(e))) => {
                    eprintln!("Warning: Rule '{}' execution error: {}", rule_id, e);
                    // Continue with other tasks
                }
                Err(e) => {
                    eprintln!("Warning: Task join error: {}", e);
                    // Continue with other tasks
                }
            }
        }

        Ok(results)
    }

    /// Validate rule definition using validator/garde
    pub fn validate_rule_definition(
        &self,
        rule_definition: &serde_json::Value,
    ) -> Result<()> {
        self.validator_engine.validate_rule_definition(rule_definition)
    }

    /// Get cached compiled rule
    pub fn get_cached_rule(&self, rule_id: &str) -> Option<&Vec<u8>> {
        self.cache.get(rule_id)
    }

    /// Cache compiled rule
    pub fn cache_compiled_rule(&mut self, rule_id: String, compiled: Vec<u8>) {
        self.cache.insert(rule_id, compiled);
    }

    /// Clear rule cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

impl Default for HybridRuleEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for HybridRuleEngine {
    fn clone(&self) -> Self {
        Self {
            rust_rule_engine: self.rust_rule_engine.clone(),
            rusty_rules_engine: self.rusty_rules_engine.clone(),
            validator_engine: self.validator_engine.clone(),
            cache: self.cache.clone(),
        }
    }
}

/// Trait for rule engines
#[async_trait]
pub trait RuleEngine: Send + Sync {
    async fn execute(
        &self,
        rule_definition: &serde_json::Value,
        context: &RuleContext,
    ) -> Result<Vec<RuleViolation>>;
}