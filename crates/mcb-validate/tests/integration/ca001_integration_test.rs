//! Integration test for CA001 rule
//!
//! These tests verify the GRL rule logic for detecting forbidden dependencies.
//! Uses the actual workspace so `cargo_metadata` works correctly.

#[cfg(test)]
mod ca001_integration_tests {
    use mcb_validate::ValidationConfig;
    #[allow(unused_imports)]
    use mcb_validate::Violation; // Used for type context in violation iteration
    use mcb_validate::engines::RuleContext;
    use mcb_validate::engines::rete_engine::ReteEngine;
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};

    /// Get the workspace root for tests (the actual project root)
    fn get_workspace_root() -> PathBuf {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(manifest_dir)
            .parent() // crates/
            .and_then(|p| p.parent()) // workspace root
            .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
    }

    #[tokio::test]
    async fn test_ca001_detects_mcb_domain_violations() {
        // This test verifies the GRL rule fires when has_internal_dependencies is true.
        // The actual workspace HAS internal dependencies (mcb-* crates), so the rule fires.
        let mut engine = ReteEngine::new();

        let workspace_root = get_workspace_root();

        // Create context using the real workspace
        let context = RuleContext {
            workspace_root: workspace_root.clone(),
            config: ValidationConfig::new(&workspace_root),
            ast_data: HashMap::new(),
            cargo_data: HashMap::new(),
            file_contents: HashMap::new(),
        };

        // Load CA001 GRL rule - fires when internal dependencies exist
        let grl = r#"
rule "DomainIndependence" salience 10 {
    when
        Facts.has_internal_dependencies == true
    then
        Facts.violation_triggered = true;
        Facts.violation_message = "Domain layer cannot depend on internal mcb-* crates";
        Facts.violation_rule_name = "CA001";
}
"#;

        let result = engine.execute_grl(grl, &context).await;

        assert!(result.is_ok(), "CA001 rule execution should succeed");

        let violations = result.unwrap();
        println!("CA001 violations found: {}", violations.len());
        for violation in &violations {
            println!("Violation: {}", violation.message);
        }

        // Our actual workspace HAS internal mcb-* dependencies, so this should trigger
        assert!(
            !violations.is_empty(),
            "CA001 should detect internal dependencies in actual workspace"
        );
        assert!(
            violations[0].message.contains("Domain layer cannot depend"),
            "Should have correct violation message"
        );
    }

    #[tokio::test]
    async fn test_ca001_allows_clean_dependencies() {
        // This test verifies the GRL rule does NOT fire when the condition is false.
        // We test the rule logic by using a rule that checks for FALSE (no internal deps).
        let mut engine = ReteEngine::new();

        let workspace_root = get_workspace_root();

        // Create context using the real workspace
        let context = RuleContext {
            workspace_root: workspace_root.clone(),
            config: ValidationConfig::new(&workspace_root),
            ast_data: HashMap::new(),
            cargo_data: HashMap::new(),
            file_contents: HashMap::new(),
        };

        // Load a rule that fires only when NO internal dependencies exist
        // Since our workspace HAS internal deps, this should NOT fire
        let grl = r#"
rule "NoInternalDepsCheck" salience 10 {
    when
        Facts.has_internal_dependencies == false
    then
        Facts.violation_triggered = true;
        Facts.violation_message = "No internal dependencies found";
        Facts.violation_rule_name = "NO_INTERNAL_DEPS";
}
"#;

        let result = engine.execute_grl(grl, &context).await;

        assert!(result.is_ok(), "Rule execution should succeed");

        let violations = result.unwrap();
        println!("Violations found: {}", violations.len());

        // Our workspace HAS internal dependencies, so this rule should NOT fire
        assert!(
            violations.is_empty(),
            "Rule checking for no-internal-deps should NOT fire on a workspace with internal deps"
        );
    }
}
