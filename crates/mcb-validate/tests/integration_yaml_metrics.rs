//! Integration tests for YAML Metrics Rules (Phase 4)
//!
//! Tests the full pipeline: YAML rule → MetricsConfig → MetricsAnalyzer → violations

#[cfg(test)]
mod yaml_metrics_tests {
    use mcb_validate::metrics::{MetricThresholds, MetricType, MetricsAnalyzer};
    use mcb_validate::rules::yaml_loader::{MetricThresholdConfig, MetricsConfig, YamlRuleLoader};
    use mcb_validate::violation_trait::Violation;
    use std::path::Path;
    use tempfile::TempDir;

    /// Test that MetricsConfig can be converted to MetricThresholds
    #[test]
    fn test_metrics_config_to_thresholds() {
        let config = MetricsConfig {
            cognitive_complexity: Some(MetricThresholdConfig {
                max: 10,
                severity: Some("error".to_string()),
                languages: Some(vec!["rust".to_string()]),
            }),
            cyclomatic_complexity: None,
            function_length: Some(MetricThresholdConfig {
                max: 30,
                severity: Some("warning".to_string()),
                languages: None,
            }),
            nesting_depth: Some(MetricThresholdConfig {
                max: 3,
                severity: None, // Defaults to warning
                languages: None,
            }),
        };

        let thresholds = MetricThresholds::from_metrics_config(&config);

        let cc = thresholds.get(MetricType::CognitiveComplexity).unwrap();
        assert_eq!(cc.max_value, 10);
        assert_eq!(cc.severity, mcb_validate::violation_trait::Severity::Error);

        let fl = thresholds.get(MetricType::FunctionLength).unwrap();
        assert_eq!(fl.max_value, 30);
        assert_eq!(
            fl.severity,
            mcb_validate::violation_trait::Severity::Warning
        );

        let nd = thresholds.get(MetricType::NestingDepth).unwrap();
        assert_eq!(nd.max_value, 3);
        assert_eq!(
            nd.severity,
            mcb_validate::violation_trait::Severity::Warning
        );
    }

    /// Test analyzing code with thresholds from MetricsConfig
    #[test]
    fn test_analyze_with_metrics_config() {
        let config = MetricsConfig {
            cognitive_complexity: Some(MetricThresholdConfig {
                max: 3,
                severity: Some("error".to_string()),
                languages: None,
            }),
            cyclomatic_complexity: None,
            function_length: None,
            nesting_depth: None,
        };

        let thresholds = MetricThresholds::from_metrics_config(&config);
        let analyzer = MetricsAnalyzer::with_thresholds(thresholds);

        let content = r#"
fn complex(x: i32) -> i32 {
    if x > 0 {
        if x > 10 {
            if x > 100 {
                return x * 2;
            }
        }
    }
    x
}
"#;

        let violations = analyzer
            .analyze_rust_content(content, Path::new("test.rs"))
            .unwrap();

        assert!(!violations.is_empty(), "Should detect complexity violation");
        let v = &violations[0];
        assert_eq!(v.item_name, "complex");
        assert_eq!(v.metric_type, MetricType::CognitiveComplexity);
        assert_eq!(v.severity, mcb_validate::violation_trait::Severity::Error);
    }

    /// Test loading a metrics rule from YAML file
    #[tokio::test]
    async fn test_load_yaml_metrics_rule() {
        let temp_dir = TempDir::new().unwrap();
        let rules_dir = temp_dir.path().join("rules");
        std::fs::create_dir_all(&rules_dir).unwrap();

        // Create a metrics rule YAML
        let rule_yaml = r#"
schema: "rule/v3"
id: "METRIC001"
name: "Cognitive Complexity Limit"
category: "metrics"
severity: "warning"
enabled: true
description: "Enforces a maximum cognitive complexity limit for functions"
rationale: "High cognitive complexity makes code harder to understand"

metrics:
  cognitive_complexity:
    max: 5
    severity: warning
    languages: ["rust"]
  nesting_depth:
    max: 3
    severity: error
"#;

        std::fs::write(rules_dir.join("METRIC001.yml"), rule_yaml).unwrap();

        let mut loader = YamlRuleLoader::new(rules_dir).unwrap();
        let rules = loader.load_all_rules().await.unwrap();

        assert_eq!(rules.len(), 1);
        let rule = &rules[0];
        assert_eq!(rule.id, "METRIC001");
        assert_eq!(rule.category, "metrics");

        // Verify metrics were parsed correctly
        assert!(rule.metrics.is_some(), "Metrics should be present");
        let metrics = rule.metrics.as_ref().unwrap();

        assert!(metrics.cognitive_complexity.is_some());
        let cc = metrics.cognitive_complexity.as_ref().unwrap();
        assert_eq!(cc.max, 5);

        assert!(metrics.nesting_depth.is_some());
        let nd = metrics.nesting_depth.as_ref().unwrap();
        assert_eq!(nd.max, 3);
    }

    /// Test full pipeline: YAML rule → MetricThresholds → MetricsAnalyzer → violations
    #[tokio::test]
    async fn test_full_yaml_metrics_pipeline() {
        let temp_dir = TempDir::new().unwrap();
        let rules_dir = temp_dir.path().join("rules");
        std::fs::create_dir_all(&rules_dir).unwrap();

        // Create a metrics rule YAML
        let rule_yaml = r#"
schema: "rule/v3"
id: "METRIC001"
name: "Cognitive Complexity Limit"
category: "metrics"
severity: "warning"
enabled: true
description: "Enforces complexity limits"
rationale: "Keep code simple"

metrics:
  cognitive_complexity:
    max: 2
    severity: error
"#;

        std::fs::write(rules_dir.join("METRIC001.yml"), rule_yaml).unwrap();

        // Load the rule
        let mut loader = YamlRuleLoader::new(rules_dir).unwrap();
        let rules = loader.load_all_rules().await.unwrap();
        let rule = &rules[0];

        // Convert to thresholds
        let metrics_config = rule.metrics.as_ref().unwrap();
        let thresholds = MetricThresholds::from_metrics_config(metrics_config);

        // Analyze code
        let analyzer = MetricsAnalyzer::with_thresholds(thresholds);

        let content = r#"
fn nested(x: i32) {
    if x > 0 {
        if x > 10 {
            println!("nested");
        }
    }
}
"#;

        let violations = analyzer
            .analyze_rust_content(content, Path::new("test.rs"))
            .unwrap();

        // Should detect cognitive complexity > 2
        assert!(!violations.is_empty(), "Should detect violation");
        let v = &violations[0];
        assert_eq!(v.id(), "METRIC001");
        assert!(v.actual_value > 2, "Actual value should exceed threshold");
    }

    /// Test that rules without metrics field work correctly
    #[tokio::test]
    async fn test_rule_without_metrics() {
        let temp_dir = TempDir::new().unwrap();
        let rules_dir = temp_dir.path().join("rules");
        std::fs::create_dir_all(&rules_dir).unwrap();

        // Create a non-metrics rule (rule/v2)
        let rule_yaml = r#"
schema: "rule/v2"
id: "QUAL001"
name: "Test Rule"
category: "quality"
severity: "warning"
enabled: true
description: "A test rule without metrics"
rationale: "For testing"

lint_select: ["F401"]
"#;

        std::fs::write(rules_dir.join("QUAL001.yml"), rule_yaml).unwrap();

        let mut loader = YamlRuleLoader::new(rules_dir).unwrap();
        let rules = loader.load_all_rules().await.unwrap();

        assert_eq!(rules.len(), 1);
        let rule = &rules[0];
        assert!(
            rule.metrics.is_none(),
            "Non-metrics rule should have no metrics"
        );
    }
}
