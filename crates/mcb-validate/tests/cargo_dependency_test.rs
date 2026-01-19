//! Tests for cargo dependency detection

#[cfg(test)]
mod cargo_dependency_tests {
    use mcb_validate::ValidationConfig;
    use mcb_validate::engines::RuleContext;
    use mcb_validate::engines::rusty_rules_engine::RustyRulesEngineWrapper;
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn create_test_context() -> RuleContext {
        RuleContext {
            workspace_root: PathBuf::from("/test/workspace"),
            config: mcb_validate::ValidationConfig::new("/test/workspace"),
            ast_data: HashMap::new(),
            cargo_data: HashMap::new(),
            file_contents: HashMap::new(),
        }
    }

    #[test]
    fn test_cargo_dependency_detection() {
        let engine = RustyRulesEngineWrapper::new();
        let context = create_test_context();

        // Test the has_forbidden_dependency function
        // Since it's private, we'll test through the evaluate_condition method
        // by creating a test rule

        use mcb_validate::engines::hybrid_engine::RuleEngine;
        use serde_json::json;

        let rule_definition = json!({
            "type": "cargo_dependencies",
            "condition": "not_exists",
            "pattern": "mcb-*",
            "target": "mcb-domain"
        });

        // This should work since mcb-domain doesn't depend on mcb-* crates
        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(engine.execute(&rule_definition, &context));
        assert!(
            result.is_ok(),
            "Cargo dependency rule execution should succeed"
        );

        let violations = result.unwrap();
        // Should not find violations since mcb-domain doesn't depend on mcb-* crates
        assert_eq!(
            violations.len(),
            0,
            "Should not find violations for clean dependencies"
        );
    }

    #[test]
    fn test_cargo_dependency_detection_with_violation() {
        // Create a temporary directory with a Cargo.toml that has forbidden dependencies
        let temp_dir = tempfile::TempDir::new().unwrap();
        let cargo_path = temp_dir.path().join("Cargo.toml");

        // Create a Cargo.toml with forbidden mcb-* dependencies
        let cargo_content = r#"
[package]
name = "test-crate"
version = "0.1.0"

[dependencies]
serde = "1.0"
mcb-infrastructure = "0.1.0"  # This should be detected as forbidden
mcb-domain = "0.1.0"          # This should also be detected
"#;

        std::fs::write(&cargo_path, cargo_content).unwrap();

        let mut context = create_test_context();
        context.workspace_root = temp_dir.path().to_path_buf();
        context.config = ValidationConfig::new(temp_dir.path());

        let engine = RustyRulesEngineWrapper::new();

        use mcb_validate::engines::hybrid_engine::RuleEngine;
        use serde_json::json;

        let rule_definition = json!({
            "type": "cargo_dependencies",
            "condition": "not_exists",
            "pattern": "mcb-*",
            "target": "test-crate"
        });

        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(engine.execute(&rule_definition, &context));

        assert!(
            result.is_ok(),
            "Cargo dependency rule execution should succeed"
        );

        let violations = result.unwrap();

        // Should find violations since test-crate depends on mcb-* crates
        assert!(
            !violations.is_empty(),
            "Should find violations for forbidden dependencies"
        );
    }
}
