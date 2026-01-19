//! RETE Engine Wrapper
//!
//! Wrapper for rust-rule-engine crate implementing RETE-UL algorithm.
//! Use this engine for complex GRL rules with when/then syntax.

use async_trait::async_trait;
use rust_rule_engine::{Facts, GRLParser, KnowledgeBase, RustRuleEngine, Value as RreValue};
use serde_json::Value;
use std::path::Path;
use walkdir::WalkDir;

use crate::Result;
use crate::engines::hybrid_engine::{RuleContext, RuleEngine, RuleViolation};
use crate::violation_trait::{Severity, ViolationCategory};

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

    /// Build facts from rule context
    ///
    /// IMPORTANT: All facts MUST use "Facts." prefix to match GRL syntax.
    /// GRL conditions like `Facts.has_internal_dependencies == true` require
    /// facts to be set with `facts.set("Facts.has_internal_dependencies", ...)`.
    fn build_facts(&self, context: &RuleContext) -> Result<Facts> {
        let facts = Facts::new();

        // Add crate information (with Facts. prefix for GRL compatibility)
        let crate_name = self.extract_crate_name_from_context(context);
        facts.set("Facts.crate_name", RreValue::String(crate_name));

        // Add dependencies as facts
        let deps = self.collect_dependencies(&context.workspace_root)?;
        for (crate_nm, dep_name, _version) in &deps {
            // Create fact: Facts.crate_{crate_name}_depends_on_{dep_name} = true
            let key = format!("Facts.crate_{}_depends_on_{}", crate_nm, dep_name);
            facts.set(&key, RreValue::Boolean(true));
        }

        // Add file facts
        for path in context.file_contents.keys() {
            let key = format!("Facts.file_{}_exists", path.replace(['/', '.'], "_"));
            facts.set(&key, RreValue::Boolean(true));
        }

        // Add a list of internal dependencies (mcb-*)
        let internal_deps: Vec<String> = deps
            .iter()
            .filter(|(_, dep, _)| dep.starts_with("mcb-"))
            .map(|(crate_nm, dep, _)| format!("{}:{}", crate_nm, dep))
            .collect();

        facts.set(
            "Facts.internal_dependencies_count",
            RreValue::Number(internal_deps.len() as f64),
        );

        // Add convenience flags (main fact for CA001)
        let has_internal_deps = !internal_deps.is_empty();
        facts.set(
            "Facts.has_internal_dependencies",
            RreValue::Boolean(has_internal_deps),
        );

        Ok(facts)
    }

    /// Extract crate name from context
    fn extract_crate_name_from_context(&self, context: &RuleContext) -> String {
        // Try to find Cargo.toml in workspace root
        let cargo_path = context.workspace_root.join("Cargo.toml");
        if let Ok(content) = std::fs::read_to_string(&cargo_path) {
            return self.extract_crate_name(&content);
        }

        // Fallback: use directory name
        context
            .workspace_root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string()
    }

    /// Extract crate name from Cargo.toml content
    fn extract_crate_name(&self, content: &str) -> String {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("name = ") || trimmed.starts_with("name=") {
                return trimmed
                    .split('=')
                    .nth(1)
                    .map(|s| s.trim().trim_matches('"').to_string())
                    .unwrap_or_default();
            }
        }
        String::new()
    }

    /// Collect dependencies from all Cargo.toml files
    fn collect_dependencies(&self, root: &Path) -> Result<Vec<(String, String, String)>> {
        let mut deps = Vec::new();

        for entry in WalkDir::new(root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().file_name().is_some_and(|n| n == "Cargo.toml"))
        {
            let path = entry.path();
            // Skip target directory
            if path.to_string_lossy().contains("/target/") {
                continue;
            }

            if let Ok(content) = std::fs::read_to_string(path) {
                let crate_name = self.extract_crate_name(&content);
                for (dep_name, version) in self.extract_dependencies(&content) {
                    deps.push((crate_name.clone(), dep_name, version));
                }
            }
        }

        Ok(deps)
    }

    /// Extract dependencies from Cargo.toml content
    fn extract_dependencies(&self, content: &str) -> Vec<(String, String)> {
        let mut deps = Vec::new();
        let mut in_deps_section = false;

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("[dependencies]")
                || trimmed.starts_with("[dev-dependencies]")
                || trimmed.starts_with("[build-dependencies]")
            {
                in_deps_section = true;
                continue;
            }

            if trimmed.starts_with('[') {
                in_deps_section = false;
                continue;
            }

            if in_deps_section && !trimmed.is_empty() && !trimmed.starts_with('#') {
                if let Some(eq_pos) = trimmed.find('=') {
                    let dep_name = trimmed[..eq_pos].trim().to_string();
                    let value_part = trimmed[eq_pos + 1..].trim();

                    let version = if value_part.starts_with('"') {
                        value_part.trim_matches('"').to_string()
                    } else if value_part.contains("version") {
                        value_part
                            .split("version")
                            .nth(1)
                            .and_then(|s| s.split('"').nth(1))
                            .unwrap_or("*")
                            .to_string()
                    } else if value_part.contains("workspace") {
                        "workspace".to_string()
                    } else {
                        "*".to_string()
                    };

                    deps.push((dep_name, version));
                }
            }
        }

        deps
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
