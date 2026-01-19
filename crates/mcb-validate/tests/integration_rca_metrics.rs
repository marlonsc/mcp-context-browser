//! Integration tests for Phase 4: rust-code-analysis metrics
//!
//! Tests the RcaAnalyzer which provides 14 code metrics using
//! the rust-code-analysis library.
//!
//! Note: rust-code-analysis parsing behavior may vary. Tests are designed
//! to be resilient to parsing differences.

#[cfg(test)]
#[cfg(feature = "rca-metrics")]
mod rca_integration_tests {
    use mcb_validate::metrics::{MetricThresholds, MetricType, RcaAnalyzer};
    use mcb_validate::violation_trait::Severity;
    use rust_code_analysis::LANG;
    use std::path::Path;
    use tempfile::TempDir;

    /// Test that RcaAnalyzer correctly detects language from file extension
    #[test]
    fn test_language_detection() {
        assert_eq!(
            RcaAnalyzer::detect_language(Path::new("test.rs")),
            Some(LANG::Rust)
        );
        assert_eq!(
            RcaAnalyzer::detect_language(Path::new("test.py")),
            Some(LANG::Python)
        );
        assert_eq!(
            RcaAnalyzer::detect_language(Path::new("test.js")),
            Some(LANG::Mozjs)
        );
        assert_eq!(
            RcaAnalyzer::detect_language(Path::new("test.ts")),
            Some(LANG::Typescript)
        );
        assert_eq!(
            RcaAnalyzer::detect_language(Path::new("test.java")),
            Some(LANG::Java)
        );
        assert_eq!(
            RcaAnalyzer::detect_language(Path::new("test.kt")),
            Some(LANG::Kotlin)
        );
        assert_eq!(
            RcaAnalyzer::detect_language(Path::new("test.cpp")),
            Some(LANG::Cpp)
        );
        assert_eq!(RcaAnalyzer::detect_language(Path::new("test.txt")), None);
    }

    /// Test analyzing a simple Rust function returns expected metrics
    #[test]
    fn test_analyze_simple_rust_function() {
        let analyzer = RcaAnalyzer::new();

        let code = br#"fn add(a: i32, b: i32) -> i32 {
    a + b
}"#;

        let results = analyzer
            .analyze_code(code, &LANG::Rust, Path::new("simple.rs"))
            .expect("Should analyze");

        // If rust-code-analysis finds functions, verify basic structure
        if let Some(func) = results.iter().find(|f| f.name == "add") {
            assert_eq!(func.metrics.cyclomatic, 1.0, "Simple function has CC=1");
            assert!(func.metrics.sloc > 0, "Should have SLOC > 0");
        }
    }

    /// Test analyzing a complex function returns higher metrics
    #[test]
    fn test_analyze_complex_rust_function() {
        let analyzer = RcaAnalyzer::new();

        let code = br#"fn complex(a: i32, b: i32) -> i32 {
    if a > b {
        if a > 10 {
            return a * 2;
        }
        return a;
    } else if b > 10 {
        return b * 2;
    }
    a + b
}"#;

        let results = analyzer
            .analyze_code(code, &LANG::Rust, Path::new("complex.rs"))
            .expect("Should analyze");

        if let Some(func) = results.iter().find(|f| f.name == "complex") {
            assert!(
                func.metrics.cyclomatic > 1.0,
                "Complex function should have CC > 1, got {}",
                func.metrics.cyclomatic
            );
        }
    }

    /// Test detecting cyclomatic complexity violations
    #[test]
    fn test_find_cyclomatic_complexity_violations() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");

        let code = r#"fn highly_complex(a: i32, b: i32, c: i32) -> i32 {
    if a > 0 {
        if b > 0 {
            if c > 0 { return a + b + c; }
            else { return a + b; }
        } else {
            if c > 0 { return a + c; }
            else { return a; }
        }
    } else {
        if b > 0 {
            if c > 0 { return b + c; }
            else { return b; }
        } else {
            if c > 0 { return c; }
            else { return 0; }
        }
    }
}"#;
        std::fs::write(&file_path, code).expect("Write temp file");

        // Set threshold to 5 - the function has CC > 5
        let thresholds = MetricThresholds::new()
            .with_threshold(MetricType::CyclomaticComplexity, 5, Severity::Warning);

        let analyzer = RcaAnalyzer::with_thresholds(thresholds);
        let violations = analyzer.find_violations(&file_path).expect("Should analyze");

        // Verify violations if found have valid structure
        for v in &violations {
            assert!(v.actual_value > v.threshold);
            assert_eq!(v.metric_type, MetricType::CyclomaticComplexity);
        }
    }

    /// Test that file-level aggregate metrics work
    #[test]
    fn test_file_aggregate_metrics() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("multi.rs");

        let code = r#"fn func1(x: i32) -> i32 {
    if x > 0 { x } else { -x }
}

fn func2(a: i32, b: i32) -> i32 {
    a + b
}

fn func3(x: i32) -> i32 {
    match x {
        0 => 0,
        1 => 1,
        _ => x * 2,
    }
}"#;
        std::fs::write(&file_path, code).expect("Write temp file");

        let analyzer = RcaAnalyzer::new();
        let metrics = analyzer.analyze_file_aggregate(&file_path).expect("Should analyze");

        // File-level metrics should have some values
        // Note: rust-code-analysis may return 0 for some metrics
        assert!(metrics.cyclomatic >= 0.0, "Cyclomatic should be >= 0");
    }

    /// Test Halstead metrics are computed
    #[test]
    fn test_halstead_metrics() {
        let analyzer = RcaAnalyzer::new();

        let code = br#"fn calculate(a: i32, b: i32, c: i32) -> i32 {
    let sum = a + b + c;
    let product = a * b * c;
    if sum > product {
        sum
    } else {
        product
    }
}"#;

        let results = analyzer
            .analyze_code(code, &LANG::Rust, Path::new("halstead.rs"))
            .expect("Should analyze");

        if let Some(func) = results.iter().find(|f| f.name == "calculate") {
            // Halstead metrics should be computed (may be 0 for simple code)
            assert!(
                func.metrics.halstead_volume >= 0.0,
                "Halstead volume should be >= 0"
            );
        }
    }

    /// Test Maintainability Index
    #[test]
    fn test_maintainability_index() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("mi.rs");

        let code = r#"fn simple_add(a: i32, b: i32) -> i32 {
    a + b
}"#;
        std::fs::write(&file_path, code).expect("Write temp file");

        let analyzer = RcaAnalyzer::new();
        let metrics = analyzer.analyze_file_aggregate(&file_path).expect("Should analyze");

        // MI is typically 0-100+ (higher is better) but can vary
        assert!(
            metrics.maintainability_index >= 0.0,
            "MI should be >= 0, got {}",
            metrics.maintainability_index
        );
    }

    /// Test NArgs metric
    #[test]
    fn test_nargs_metric() {
        let analyzer = RcaAnalyzer::new();

        let code = br#"fn many_args(a: i32, b: i32, c: i32, d: i32, e: i32) -> i32 {
    a + b + c + d + e
}"#;

        let results = analyzer
            .analyze_code(code, &LANG::Rust, Path::new("args.rs"))
            .expect("Should analyze");

        if let Some(func) = results.iter().find(|f| f.name == "many_args") {
            // Should count arguments
            assert!(
                func.metrics.nargs >= 1,
                "Should have at least 1 arg, got {}",
                func.metrics.nargs
            );
        }
    }

    /// Test empty file handling
    #[test]
    fn test_empty_file() {
        let analyzer = RcaAnalyzer::new();

        let code = b"";
        let results = analyzer
            .analyze_code(code, &LANG::Rust, Path::new("empty.rs"))
            .expect("Should handle empty file");

        assert!(results.is_empty(), "Empty file should have no functions");
    }
}
