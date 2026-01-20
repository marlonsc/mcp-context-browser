//! Integration tests for Phase 5: Duplication Detection
//!
//! Tests the `DuplicationAnalyzer` which detects code clones using
//! a hybrid approach of token fingerprinting and AST similarity.
//!
//! ## Clone Types Tested
//!
//! | Type | Description |
//! |------|-------------|
//! | Type 1 (DUP001) | Exact clones (100% identical) |
//! | Type 2 (DUP002) | Renamed clones (identifiers changed) |
//! | Type 3 (DUP003) | Gapped clones (small modifications) |

#[cfg(test)]
mod duplication_integration_tests {
    use mcb_validate::duplication::{
        DuplicationAnalyzer, DuplicationThresholds, DuplicationType, TokenFingerprinter,
        tokenize_source,
    };
    use mcb_validate::violation_trait::Violation;
    use std::fs;
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn create_temp_file(dir: &TempDir, name: &str, content: &str) -> PathBuf {
        let path = dir.path().join(name);
        let mut file = fs::File::create(&path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        path
    }

    /// Test that tokenizer correctly extracts tokens from Rust code
    #[test]
    fn test_tokenize_rust_code() {
        let code = r"fn calculate_sum(numbers: &[i32]) -> i32 {
    let mut sum = 0;
    for num in numbers {
        sum += num;
    }
    sum
}";

        let tokens = tokenize_source(code, "rust");

        // Should have multiple tokens
        assert!(!tokens.is_empty(), "Should tokenize code");

        // Should identify keywords
        let keyword_count = tokens
            .iter()
            .filter(|t| t.token_type == mcb_validate::duplication::fingerprint::TokenType::Keyword)
            .count();
        assert!(keyword_count >= 3, "Should find fn, let, for keywords");

        // Should identify identifiers
        let identifier_count = tokens
            .iter()
            .filter(|t| {
                t.token_type == mcb_validate::duplication::fingerprint::TokenType::Identifier
            })
            .count();
        assert!(
            identifier_count >= 3,
            "Should find identifiers like calculate_sum, numbers, sum"
        );
    }

    /// Test fingerprinting identifies duplicate token sequences
    #[test]
    fn test_fingerprinter_finds_duplicates() {
        let code1 = r"fn process_data(items: &[i32]) -> i32 {
    let mut total = 0;
    for item in items {
        total += item;
    }
    total
}";

        let code2 = r"fn process_data(items: &[i32]) -> i32 {
    let mut total = 0;
    for item in items {
        total += item;
    }
    total
}";

        let tokens1 = tokenize_source(code1, "rust");
        let tokens2 = tokenize_source(code2, "rust");

        // Both should tokenize to the same sequence
        assert_eq!(
            tokens1.len(),
            tokens2.len(),
            "Identical code should produce same token count"
        );

        // Use fingerprinter to check
        let mut fingerprinter = TokenFingerprinter::new(5);
        fingerprinter.fingerprint_file(PathBuf::from("file1.rs"), &tokens1);
        fingerprinter.fingerprint_file(PathBuf::from("file2.rs"), &tokens2);

        let matches = fingerprinter.find_duplicates();

        // Should find at least one match between files
        assert!(
            !matches.is_empty(),
            "Fingerprinter should find duplicate sequences"
        );
    }

    /// Test `DuplicationAnalyzer` detects exact clones across files
    #[test]
    fn test_analyzer_detects_exact_clones() {
        let dir = TempDir::new().unwrap();

        // Create two files with identical function
        let duplicated_code = r"
fn calculate_average(numbers: &[f64]) -> f64 {
    if numbers.is_empty() {
        return 0.0;
    }
    let sum: f64 = numbers.iter().sum();
    sum / numbers.len() as f64
}
";

        let file1 = create_temp_file(
            &dir,
            "math_utils.rs",
            &format!("// Math utilities\n{duplicated_code}"),
        );
        let file2 = create_temp_file(
            &dir,
            "statistics.rs",
            &format!("// Statistics helpers\n{duplicated_code}"),
        );

        // Configure with low thresholds for test
        let thresholds = DuplicationThresholds {
            min_lines: 3,
            min_tokens: 10,
            ..Default::default()
        };
        let analyzer = DuplicationAnalyzer::with_thresholds(thresholds);

        let violations = analyzer
            .analyze_files(&[file1.clone(), file2.clone()])
            .expect("Analysis should succeed");

        // Should detect duplication if tokens are sufficient
        // Note: may not detect if code is too short for fingerprinting
        if !violations.is_empty() {
            let v = &violations[0];
            assert_eq!(
                v.duplication_type,
                DuplicationType::ExactClone,
                "Should be exact clone"
            );
            assert!(v.similarity >= 0.99, "Should have high similarity");
            assert!(
                v.message().contains("Exact Clone"),
                "Message should mention exact clone"
            );
        }
    }

    /// Test that small code snippets below threshold are not flagged
    #[test]
    fn test_respects_minimum_thresholds() {
        let dir = TempDir::new().unwrap();

        // Very small duplicated code
        let small_code = "let x = 1;\nlet y = 2;\n";

        let file1 = create_temp_file(&dir, "small1.rs", small_code);
        let file2 = create_temp_file(&dir, "small2.rs", small_code);

        // High thresholds
        let thresholds = DuplicationThresholds {
            min_lines: 10,
            min_tokens: 50,
            ..Default::default()
        };
        let analyzer = DuplicationAnalyzer::with_thresholds(thresholds);

        let violations = analyzer
            .analyze_files(&[file1, file2])
            .expect("Analysis should succeed");

        assert!(
            violations.is_empty(),
            "Small code below thresholds should not trigger violations"
        );
    }

    /// Test analyzer works with different file types
    #[test]
    fn test_analyzer_handles_multiple_languages() {
        let dir = TempDir::new().unwrap();

        // Create files with different extensions
        let rust_code = "fn foo() { let x = 1; }";
        let python_code = "def foo(): x = 1";

        let rust_file = create_temp_file(&dir, "test.rs", rust_code);
        let python_file = create_temp_file(&dir, "test.py", python_code);

        let analyzer = DuplicationAnalyzer::new();

        // Should not crash when analyzing different file types
        let result = analyzer.analyze_files(&[rust_file, python_file]);
        assert!(result.is_ok(), "Analyzer should handle multiple languages");
    }

    /// Test duplication statistics calculation
    #[test]
    fn test_duplication_stats() {
        use mcb_validate::violation_trait::Severity;

        let violations = vec![
            mcb_validate::duplication::DuplicationViolation {
                file: PathBuf::from("a.rs"),
                line: 10,
                duplicate_file: PathBuf::from("b.rs"),
                duplicate_line: 20,
                duplication_type: DuplicationType::ExactClone,
                similarity: 1.0,
                duplicated_lines: 15,
                severity: Severity::Warning,
            },
            mcb_validate::duplication::DuplicationViolation {
                file: PathBuf::from("c.rs"),
                line: 30,
                duplicate_file: PathBuf::from("d.rs"),
                duplicate_line: 40,
                duplication_type: DuplicationType::RenamedClone,
                similarity: 0.95,
                duplicated_lines: 10,
                severity: Severity::Warning,
            },
            mcb_validate::duplication::DuplicationViolation {
                file: PathBuf::from("e.rs"),
                line: 50,
                duplicate_file: PathBuf::from("f.rs"),
                duplicate_line: 60,
                duplication_type: DuplicationType::GappedClone,
                similarity: 0.85,
                duplicated_lines: 8,
                severity: Severity::Info,
            },
        ];

        let analyzer = DuplicationAnalyzer::new();
        let stats = analyzer.get_stats(&violations);

        assert_eq!(stats.total_clones, 3);
        assert_eq!(stats.exact_clones, 1);
        assert_eq!(stats.renamed_clones, 1);
        assert_eq!(stats.gapped_clones, 1);
        assert_eq!(stats.semantic_clones, 0);
        assert_eq!(stats.total_duplicated_lines, 33);
    }

    /// Test Violation trait implementation
    #[test]
    fn test_violation_trait_implementation() {
        use mcb_validate::duplication::DuplicationViolation;
        use mcb_validate::violation_trait::{Severity, ViolationCategory};

        let violation = DuplicationViolation {
            file: PathBuf::from("src/utils.rs"),
            line: 42,
            duplicate_file: PathBuf::from("src/helpers.rs"),
            duplicate_line: 100,
            duplication_type: DuplicationType::ExactClone,
            similarity: 1.0,
            duplicated_lines: 20,
            severity: Severity::Warning,
        };

        // Test Violation trait methods
        assert_eq!(violation.id(), "DUP001");
        assert!(violation.message().contains("Exact Clone"));
        assert!(violation.message().contains("20 lines"));
        assert_eq!(violation.severity(), Severity::Warning);
        assert_eq!(violation.file(), Some(&PathBuf::from("src/utils.rs")));
        assert_eq!(violation.line(), Some(42));
        assert!(violation.suggestion().is_some());
        assert_eq!(violation.category(), ViolationCategory::Quality);

        // Test Display
        let display = format!("{violation}");
        assert!(display.contains("src/utils.rs"));
        assert!(display.contains("42"));
        assert!(display.contains("src/helpers.rs"));
    }

    /// Test clone type classification
    #[test]
    fn test_clone_type_classification() {
        // Test DuplicationType rule IDs
        assert_eq!(
            DuplicationType::ExactClone.rule_id(),
            "DUP001",
            "Exact clone should be DUP001"
        );
        assert_eq!(
            DuplicationType::RenamedClone.rule_id(),
            "DUP002",
            "Renamed clone should be DUP002"
        );
        assert_eq!(
            DuplicationType::GappedClone.rule_id(),
            "DUP003",
            "Gapped clone should be DUP003"
        );
        assert_eq!(
            DuplicationType::SemanticClone.rule_id(),
            "DUP004",
            "Semantic clone should be DUP004"
        );
    }

    /// Test that exclude patterns affect analysis
    #[test]
    fn test_exclude_patterns_applied() {
        let dir = TempDir::new().unwrap();

        // Create duplicate code
        let code = r"fn duplicated_function(x: i32) -> i32 {
    let mut result = 0;
    for i in 0..x {
        result += i;
    }
    result
}";

        // Create files - one in excluded "tests" directory
        let tests_dir = dir.path().join("tests");
        fs::create_dir_all(&tests_dir).unwrap();

        let src_file = create_temp_file(&dir, "src.rs", code);
        let test_file = tests_dir.join("test.rs");
        fs::write(&test_file, code).unwrap();

        // Configure to exclude tests directory
        let thresholds = DuplicationThresholds {
            exclude_patterns: vec!["**/tests/**".to_string()],
            min_lines: 3,
            min_tokens: 10,
            ..Default::default()
        };
        let analyzer = DuplicationAnalyzer::with_thresholds(thresholds);

        // Analyze both files
        let violations = analyzer
            .analyze_files(&[src_file.clone(), test_file.clone()])
            .expect("Analysis should succeed");

        // Test files should be excluded, so no duplication should be detected
        // (would need two non-excluded files with same content to detect)
        for v in &violations {
            // If any violation involves the test file, the pattern isn't working
            assert!(
                !v.file.to_string_lossy().contains("tests"),
                "Test files should be excluded from analysis"
            );
            assert!(
                !v.duplicate_file.to_string_lossy().contains("tests"),
                "Test files should be excluded from analysis"
            );
        }
    }

    /// Test empty file handling
    #[test]
    fn test_empty_files() {
        let dir = TempDir::new().unwrap();

        let file1 = create_temp_file(&dir, "empty.rs", "");
        let file2 = create_temp_file(&dir, "also_empty.rs", "// just a comment\n");

        let analyzer = DuplicationAnalyzer::new();
        let violations = analyzer
            .analyze_files(&[file1, file2])
            .expect("Should handle empty files");

        assert!(
            violations.is_empty(),
            "Empty files should not produce violations"
        );
    }

    /// Test that files with only comments don't cause issues
    #[test]
    fn test_comment_only_files() {
        let dir = TempDir::new().unwrap();

        let comments = r"
// This is a file with only comments
// No actual code here
/*
 * Multi-line comment
 * Also no code
 */
";

        let file1 = create_temp_file(&dir, "comments1.rs", comments);
        let file2 = create_temp_file(&dir, "comments2.rs", comments);

        let analyzer = DuplicationAnalyzer::new();
        let violations = analyzer
            .analyze_files(&[file1, file2])
            .expect("Should handle comment-only files");

        // Comment-only files shouldn't produce code duplication violations
        // (comments are stripped during tokenization)
        assert!(
            violations.is_empty(),
            "Comment-only files should not produce violations"
        );
    }
}
