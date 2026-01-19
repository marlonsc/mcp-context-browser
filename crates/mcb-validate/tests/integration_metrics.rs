//! Integration tests for Phase 4: Complexity Metrics
//!
//! Tests the full flow: Rust file → MetricsAnalyzer → violations

#[cfg(test)]
mod integration_metrics_tests {
    use mcb_validate::metrics::{MetricThresholds, MetricType, MetricsAnalyzer};
    use mcb_validate::violation_trait::{Severity, Violation};
    use std::path::Path;
    use tempfile::TempDir;

    /// Test analyzing a simple function that should pass all thresholds
    #[test]
    fn test_simple_function_passes() {
        let analyzer = MetricsAnalyzer::new();

        let content = r#"
fn add(a: i32, b: i32) -> i32 {
    a + b
}
"#;

        let violations = analyzer
            .analyze_rust_content(content, Path::new("simple.rs"))
            .unwrap();

        assert!(
            violations.is_empty(),
            "Simple function should pass all thresholds"
        );
    }

    /// Test detecting high cognitive complexity
    #[test]
    fn test_detects_high_cognitive_complexity() {
        let thresholds = MetricThresholds::new().with_threshold(
            MetricType::CognitiveComplexity,
            5,
            Severity::Warning,
        );

        let analyzer = MetricsAnalyzer::with_thresholds(thresholds);

        let content = r#"
fn complex(x: i32) -> i32 {
    if x > 0 {
        if x > 10 {
            if x > 100 {
                for i in 0..x {
                    if i % 2 == 0 {
                        return i;
                    }
                }
            }
        }
    }
    x
}
"#;

        let violations = analyzer
            .analyze_rust_content(content, Path::new("complex.rs"))
            .unwrap();

        assert!(
            !violations.is_empty(),
            "Should detect cognitive complexity violation"
        );

        let complexity_violations: Vec<_> = violations
            .iter()
            .filter(|v| v.metric_type == MetricType::CognitiveComplexity)
            .collect();

        assert!(!complexity_violations.is_empty());
        assert_eq!(complexity_violations[0].item_name, "complex");
    }

    /// Test detecting deep nesting
    #[test]
    fn test_detects_deep_nesting() {
        let thresholds =
            MetricThresholds::new().with_threshold(MetricType::NestingDepth, 2, Severity::Error);

        let analyzer = MetricsAnalyzer::with_thresholds(thresholds);

        let content = r#"
fn deeply_nested(x: i32) {
    if x > 0 {
        while x > 10 {
            for i in 0..x {
                if i > 5 {
                    println!("deep!");
                }
            }
        }
    }
}
"#;

        let violations = analyzer
            .analyze_rust_content(content, Path::new("nested.rs"))
            .unwrap();

        let nesting_violations: Vec<_> = violations
            .iter()
            .filter(|v| v.metric_type == MetricType::NestingDepth)
            .collect();

        assert!(
            !nesting_violations.is_empty(),
            "Should detect nesting depth violation"
        );
        assert_eq!(nesting_violations[0].severity, Severity::Error);
    }

    /// Test detecting long functions
    #[test]
    fn test_detects_long_function() {
        let thresholds = MetricThresholds::new().with_threshold(
            MetricType::FunctionLength,
            5,
            Severity::Warning,
        );

        let analyzer = MetricsAnalyzer::with_thresholds(thresholds);

        let content = r#"
fn long_function() {
    let a = 1;
    let b = 2;
    let c = 3;
    let d = 4;
    let e = 5;
    let f = 6;
    let g = 7;
}
"#;

        let violations = analyzer
            .analyze_rust_content(content, Path::new("long.rs"))
            .unwrap();

        let length_violations: Vec<_> = violations
            .iter()
            .filter(|v| v.metric_type == MetricType::FunctionLength)
            .collect();

        assert!(
            !length_violations.is_empty(),
            "Should detect function length violation"
        );
    }

    /// Test analyzing impl block methods
    #[test]
    fn test_analyzes_impl_methods() {
        let thresholds = MetricThresholds::new().with_threshold(
            MetricType::CognitiveComplexity,
            3,
            Severity::Warning,
        );

        let analyzer = MetricsAnalyzer::with_thresholds(thresholds);

        let content = r#"
struct Calculator;

impl Calculator {
    fn simple(&self) -> i32 {
        1 + 1
    }

    fn complex(&self, x: i32) -> i32 {
        if x > 0 {
            if x > 10 {
                if x > 100 {
                    return x * 2;
                }
            }
        }
        x
    }
}
"#;

        let violations = analyzer
            .analyze_rust_content(content, Path::new("impl.rs"))
            .unwrap();

        // Should detect complex but not simple
        let names: Vec<_> = violations.iter().map(|v| &v.item_name).collect();
        assert!(names.contains(&&"complex".to_string()));
        assert!(!names.contains(&&"simple".to_string()));
    }

    /// Test analyzing a real file
    #[test]
    fn test_analyze_real_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");

        std::fs::write(
            &file_path,
            r#"
fn test_function(x: i32) -> i32 {
    match x {
        0 => 0,
        1 => 1,
        _ => x * 2,
    }
}
"#,
        )
        .unwrap();

        let analyzer = MetricsAnalyzer::new();
        let _violations = analyzer.analyze_rust_file(&file_path).unwrap();

        // Should complete without error, violations depend on thresholds
        assert!(true); // Just verify it runs without panicking
    }

    /// Test multiple files analysis
    #[test]
    fn test_analyze_multiple_files() {
        let temp_dir = TempDir::new().unwrap();

        let file1 = temp_dir.path().join("file1.rs");
        let file2 = temp_dir.path().join("file2.rs");

        std::fs::write(&file1, "fn simple() { let x = 1; }").unwrap();
        std::fs::write(
            &file2,
            r#"
fn complex(x: i32) {
    if x > 0 {
        if x > 10 {
            if x > 100 {
                println!("nested");
            }
        }
    }
}
"#,
        )
        .unwrap();

        let thresholds = MetricThresholds::new().with_threshold(
            MetricType::CognitiveComplexity,
            3,
            Severity::Warning,
        );

        let analyzer = MetricsAnalyzer::with_thresholds(thresholds);
        let violations = analyzer.analyze_files(&[file1, file2]).unwrap();

        // Should find complexity in file2 but not file1
        assert!(!violations.is_empty());
        assert!(violations.iter().any(|v| v.file.ends_with("file2.rs")));
    }

    /// Test thresholds from YAML config
    #[test]
    fn test_thresholds_from_yaml_config() {
        let yaml = serde_json::json!({
            "cognitive_complexity": {
                "max": 10,
                "severity": "error"
            },
            "function_length": {
                "max": 30,
                "severity": "warning"
            },
            "nesting_depth": {
                "max": 3
            }
        });

        let thresholds = MetricThresholds::from_yaml(&yaml);

        let cc = thresholds.get(MetricType::CognitiveComplexity).unwrap();
        assert_eq!(cc.max_value, 10);
        assert_eq!(cc.severity, Severity::Error);

        let fl = thresholds.get(MetricType::FunctionLength).unwrap();
        assert_eq!(fl.max_value, 30);
        assert_eq!(fl.severity, Severity::Warning);

        let nd = thresholds.get(MetricType::NestingDepth).unwrap();
        assert_eq!(nd.max_value, 3);
        assert_eq!(nd.severity, Severity::Warning); // Default
    }

    /// Test violation message format
    #[test]
    fn test_violation_message_format() {
        // Use threshold of 0 so any if/for/while triggers a violation
        let thresholds = MetricThresholds::new().with_threshold(
            MetricType::CognitiveComplexity,
            0,
            Severity::Warning,
        );

        let analyzer = MetricsAnalyzer::with_thresholds(thresholds);

        let content = r#"
fn with_if(x: i32) {
    if x > 0 {
        println!("positive");
    }
}
"#;

        let violations = analyzer
            .analyze_rust_content(content, Path::new("msg.rs"))
            .unwrap();

        assert!(
            !violations.is_empty(),
            "Should have violations with threshold=0"
        );
        let v = &violations[0];

        // Check violation fields
        assert_eq!(v.item_name, "with_if");
        assert_eq!(v.metric_type, MetricType::CognitiveComplexity);
        assert!(v.actual_value >= 1); // Should be at least 1 (the if statement)
        assert_eq!(v.threshold, 0);

        // Check message() method
        let msg = v.message();
        assert!(msg.contains("with_if"));
        assert!(msg.contains("cognitive complexity"));
    }

    /// Test suggestion text
    #[test]
    fn test_suggestion_text() {
        let thresholds =
            MetricThresholds::new().with_threshold(MetricType::NestingDepth, 1, Severity::Warning);

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
            .analyze_rust_content(content, Path::new("suggestion.rs"))
            .unwrap();

        assert!(!violations.is_empty());
        let v = &violations[0];

        // Check suggestion is provided
        let suggestion = v.suggestion();
        assert!(suggestion.is_some());
        let suggestion_text = suggestion.unwrap();
        assert!(!suggestion_text.is_empty());
        // Should mention early returns or extracting logic
        assert!(
            suggestion_text.contains("early returns")
                || suggestion_text.contains("extract")
                || suggestion_text.contains("guard")
        );
    }
}
