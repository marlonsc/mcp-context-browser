//! RETE Engine Wrapper
//!
//! Wrapper for rust-rule-engine crate implementing RETE-UL algorithm.
//! Use this engine for complex GRL rules with when/then syntax.
//!
//! Uses `cargo_metadata` for reliable Cargo.toml/workspace parsing.
//! Fails fast if cargo_metadata is unavailable.

use async_trait::async_trait;
use cargo_metadata::MetadataCommand;
use rust_rule_engine::{Facts, GRLParser, KnowledgeBase, RustRuleEngine, Value as RreValue};
use serde_json::Value;

use crate::Result;
use crate::engines::hybrid_engine::{RuleContext, RuleEngine, RuleViolation};
use crate::violation_trait::{Severity, ViolationCategory};

/// Prefix for internal workspace dependencies (mcb-*)
const INTERNAL_DEP_PREFIX: &str = "mcb-";

/// RETE Engine wrapper for rust-rule-engine library
pub struct ReteEngine {
    /// The knowledge base containing rules
    kb: KnowledgeBase,
}

impl Default for ReteEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ReteEngine {
    /// Create a new RETE engine instance
    pub fn new() -> Self {
        Self {
            kb: KnowledgeBase::new("mcb-validate"),
        }
    }

    /// Load GRL rules into the knowledge base
    pub fn load_grl(&mut self, grl_code: &str) -> Result<()> {
        let rules = GRLParser::parse_rules(grl_code)
            .map_err(|e| crate::ValidationError::Config(format!("Failed to parse GRL: {}", e)))?;

        for rule in rules {
            self.kb.add_rule(rule).map_err(|e| {
                crate::ValidationError::Config(format!("Failed to add rule: {}", e))
            })?;
        }

        Ok(())
    }

    /// Build facts from rule context using cargo_metadata
    ///
    /// IMPORTANT: All facts MUST use "Facts." prefix to match GRL syntax.
    /// GRL conditions like `Facts.has_internal_dependencies == true` require
    /// facts to be set with `facts.set("Facts.has_internal_dependencies", ...)`.
    ///
    /// Requires cargo_metadata to succeed. Fails fast if unavailable.
    fn build_facts(&self, context: &RuleContext) -> Result<Facts> {
        let facts = Facts::new();

        // Use cargo_metadata for reliable workspace/package parsing
        let manifest_path = context.workspace_root.join("Cargo.toml");

        // Try to get metadata from cargo
        let metadata_result = MetadataCommand::new()
            .manifest_path(&manifest_path)
            .no_deps() // We only need local workspace packages
            .exec();

        match metadata_result {
            Ok(metadata) if !metadata.packages.is_empty() => {
                // Get root package name (or first workspace member)
                // cargo_metadata 0.23 returns PackageName, convert to String
                // Note: The guard `!metadata.packages.is_empty()` ensures at least one package exists
                let root_name = metadata
                    .root_package()
                    .map(|p| p.name.to_string())
                    .or_else(|| metadata.packages.first().map(|p| p.name.to_string()))
                    .unwrap_or_else(|| "unknown".to_string());

                facts.set("Facts.crate_name", RreValue::String(root_name));

                // Collect all dependencies from all packages
                let mut internal_deps_count = 0;

                for package in &metadata.packages {
                    for dep in &package.dependencies {
                        // Create fact: Facts.crate_{package}_depends_on_{dep} = true
                        // cargo_metadata 0.23 uses PackageName, convert to String for comparison
                        let pkg_name = package.name.to_string();
                        let dep_name = dep.name.to_string();
                        let key = format!("Facts.crate_{}_depends_on_{}", pkg_name, dep_name);
                        facts.set(&key, RreValue::Boolean(true));

                        // Count internal dependencies (mcb-*)
                        if dep_name.starts_with(INTERNAL_DEP_PREFIX) {
                            internal_deps_count += 1;
                        }
                    }
                }

                facts.set(
                    "Facts.internal_dependencies_count",
                    RreValue::Number(f64::from(internal_deps_count)),
                );

                facts.set(
                    "Facts.has_internal_dependencies",
                    RreValue::Boolean(internal_deps_count > 0),
                );
            }
            Ok(_metadata) => {
                // Empty metadata - fail fast
                return Err(crate::ValidationError::Config(
                    "cargo_metadata returned empty packages".into(),
                ));
            }
            Err(e) => {
                return Err(crate::ValidationError::Config(format!(
                    "cargo_metadata failed: {}",
                    e
                )));
            }
        }

        // Add file facts
        for path in context.file_contents.keys() {
            let key = format!("Facts.file_{}_exists", path.replace(['/', '.'], "_"));
            facts.set(&key, RreValue::Boolean(true));
        }

        Ok(facts)
    }

    /// Execute GRL rules against context and return violations
    pub async fn execute_grl(
        &mut self,
        grl_code: &str,
        context: &RuleContext,
    ) -> Result<Vec<RuleViolation>> {
        // Load rules into knowledge base
        self.load_grl(grl_code)?;

        // Build facts from context
        let facts = self.build_facts(context)?;

        // Initialize violation markers in facts (rules will set these when triggered)
        // Use Facts. prefix for GRL compatibility
        facts.set("Facts.violation_triggered", RreValue::Boolean(false));
        facts.set("Facts.violation_message", RreValue::String(String::new()));
        facts.set("Facts.violation_rule_name", RreValue::String(String::new()));

        // Create engine and execute
        let mut engine = RustRuleEngine::new(self.kb.clone());
        let result = engine
            .execute(&facts)
            .map_err(|e| crate::ValidationError::Config(format!("RETE execution failed: {}", e)))?;

        // Convert results to violations
        let mut violations = Vec::new();

        // If any rules fired, check if they set violation markers
        if result.rules_fired > 0 {
            // Check if violation was triggered by rule action (use Facts. prefix)
            if let Some(RreValue::Boolean(true)) = facts.get("Facts.violation_triggered") {
                let message = facts
                    .get("Facts.violation_message")
                    .and_then(|v| {
                        if let RreValue::String(s) = v {
                            Some(s.clone())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| "Rule violation detected".to_string());

                let rule_name = facts
                    .get("Facts.violation_rule_name")
                    .and_then(|v| {
                        if let RreValue::String(s) = v {
                            Some(s.clone())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| "GRL_RULE".to_string());

                violations.push(
                    RuleViolation::new(
                        &rule_name,
                        ViolationCategory::Architecture,
                        Severity::Error,
                        message,
                    )
                    .with_context(format!(
                        "GRL Rule Engine: {} rules fired in {} cycles",
                        result.rules_fired, result.cycle_count
                    )),
                );
            }
        }

        Ok(violations)
    }
}

#[async_trait]
impl RuleEngine for ReteEngine {
    async fn execute(
        &self,
        rule_definition: &Value,
        context: &RuleContext,
    ) -> Result<Vec<RuleViolation>> {
        // Extract GRL code from rule definition
        let grl_code = rule_definition
            .get("rule")
            .or_else(|| rule_definition.get("grl"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::ValidationError::Config(
                    "Missing 'rule' or 'grl' field in rule definition".into(),
                )
            })?;

        // Create mutable engine for execution
        let mut engine = Self::new();
        engine.execute_grl(grl_code, context).await
    }
}

impl Clone for ReteEngine {
    fn clone(&self) -> Self {
        Self {
            kb: KnowledgeBase::new("mcb-validate"),
        }
    }
}
