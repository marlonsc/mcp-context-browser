//! Tests for architecture rules like CA001

#[cfg(test)]
mod architecture_rules_tests {
    use mcb_validate::rules::yaml_loader::YamlRuleLoader;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_ca001_rule_loading() {
        // Test if the CA001 rule can be loaded from the YAML files
        let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf();
        let rules_dir = workspace_root.join("crates/mcb-validate/rules");

        println!("Rules directory: {rules_dir:?}");
        println!("Rules directory exists: {}", rules_dir.exists());

        if rules_dir.exists() {
            let mut loader = YamlRuleLoader::new(rules_dir).unwrap();
            let rules = loader.load_all_rules().await.unwrap();

            println!("Loaded {} rules", rules.len());

            // Look for CA001 rule
            let ca001_rule = rules.iter().find(|r| r.id == "CA001");

            if let Some(rule) = ca001_rule {
                println!("Found CA001 rule: {:?}", rule.name);
                println!("Rule engine: {:?}", rule.engine);
                println!("Rule definition: {:?}", rule.rule_definition);

                // Check if it's a RETE rule
                assert_eq!(
                    rule.engine, "rust-rule-engine",
                    "CA001 should use rust-rule-engine"
                );

                // Check if it has the expected properties
                assert!(
                    rule.name.contains("Domain"),
                    "CA001 should be about domain layer"
                );
            } else {
                println!("CA001 rule not found!");
                println!(
                    "Available rules: {:?}",
                    rules.iter().map(|r| &r.id).collect::<Vec<_>>()
                );
                panic!("CA001 rule should be loaded");
            }
        } else {
            panic!("Rules directory does not exist: {rules_dir:?}");
        }
    }
}
