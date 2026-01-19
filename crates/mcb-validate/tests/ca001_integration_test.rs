//! Integration test for CA001 rule

#[cfg(test)]
mod ca001_integration_tests {
    use mcb_validate::ValidationConfig;
    #[allow(unused_imports)]
    use mcb_validate::Violation; // Used for type context in violation iteration
    use mcb_validate::engines::RuleContext;
    use mcb_validate::engines::rete_engine::ReteEngine;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_ca001_detects_mcb_domain_violations() {
        let mut engine = ReteEngine::new();

        // Create a temporary directory structure that mimics the real workspace
        let temp_dir = tempfile::TempDir::new().unwrap();
        let crates_dir = temp_dir.path().join("crates");
        std::fs::create_dir(&crates_dir).unwrap();

        // Create mcb-domain with forbidden dependency
        let domain_dir = crates_dir.join("mcb-domain");
        std::fs::create_dir(&domain_dir).unwrap();

        let domain_cargo = r#"
[package]
name = "mcb-domain"
version = "0.1.0"

[dependencies]
serde = "1.0"
thiserror = "1.0"
# This should trigger CA001 violation
mcb-infrastructure = "0.1.0"
"#;

        std::fs::write(domain_dir.join("Cargo.toml"), domain_cargo).unwrap();

        // Create context pointing to the temp directory
        let context = RuleContext {
            workspace_root: temp_dir.path().to_path_buf(),
            config: ValidationConfig::new(temp_dir.path()),
            ast_data: HashMap::new(),
            cargo_data: HashMap::new(),
            file_contents: HashMap::new(),
        };

        // Load CA001 GRL rule
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

        // Should detect the violation
        assert!(
            !violations.is_empty(),
            "CA001 should detect forbidden dependency in mcb-domain"
        );
        assert!(
            violations[0].message.contains("Domain layer cannot depend"),
            "Should have correct violation message"
        );
    }

    #[tokio::test]
    async fn test_ca001_allows_clean_mcb_domain() {
        let mut engine = ReteEngine::new();

        // Create a temporary directory structure
        let temp_dir = tempfile::TempDir::new().unwrap();
        let crates_dir = temp_dir.path().join("crates");
        std::fs::create_dir(&crates_dir).unwrap();

        // Create clean mcb-domain without forbidden dependencies
        let domain_dir = crates_dir.join("mcb-domain");
        std::fs::create_dir(&domain_dir).unwrap();

        let domain_cargo = r#"
[package]
name = "mcb-domain"
version = "0.1.0"

[dependencies]
serde = "1.0"
thiserror = "1.0"
uuid = "1.0"
chrono = "0.4"
"#;

        std::fs::write(domain_dir.join("Cargo.toml"), domain_cargo).unwrap();

        // Create context
        let context = RuleContext {
            workspace_root: temp_dir.path().to_path_buf(),
            config: ValidationConfig::new(temp_dir.path()),
            ast_data: HashMap::new(),
            cargo_data: HashMap::new(),
            file_contents: HashMap::new(),
        };

        // Load CA001 GRL rule
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

        // Should NOT detect violations for clean dependencies
        assert!(
            violations.is_empty(),
            "CA001 should NOT detect violations for clean mcb-domain"
        );
    }
}
