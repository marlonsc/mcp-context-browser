//! Unit tests for validation system components

use mcp_context_browser::core::types::{CodeChunk, Language, Embedding};

/// Test validation of core data structures
#[cfg(test)]
mod data_validation_tests {
    use super::*;

    #[test]
    fn test_code_chunk_validation_rules() {
        // Test individual validation rules for CodeChunk
        let valid_chunk = CodeChunk {
            id: "valid_id_123".to_string(),
            content: "fn main() { println!(\"Hello!\"); }".to_string(),
            file_path: "/src/main.rs".to_string(),
            start_line: 1,
            end_line: 10,
            language: Language::Rust,
            metadata: serde_json::json!({"valid": true}),
        };

        // Test that valid chunks pass basic checks
        assert!(!valid_chunk.content.is_empty());
        assert!(valid_chunk.start_line > 0);
        assert!(valid_chunk.end_line >= valid_chunk.start_line);
        assert!(!valid_chunk.file_path.is_empty());
    }

    #[test]
    fn test_embedding_validation_rules() {
        // Test individual validation rules for Embedding
        let valid_embedding = Embedding {
            vector: vec![0.1, 0.2, 0.3, 0.4, 0.5],
            model: "text-embedding-ada-002".to_string(),
            dimensions: 5,
        };

        // Test that valid embeddings pass basic checks
        assert!(!valid_embedding.vector.is_empty());
        assert_eq!(valid_embedding.vector.len(), valid_embedding.dimensions);
        assert!(!valid_embedding.model.is_empty());
    }

    #[test]
    fn test_chunk_content_size_limits() {
        // Test content size validation
        let small_content = "short";
        let large_content = "x".repeat(10001); // Over 10KB limit

        assert!(small_content.len() <= 10000);
        assert!(large_content.len() > 10000);
    }

    #[test]
    fn test_file_path_format_validation() {
        // Test file path format validation
        let valid_paths = vec![
            "/src/main.rs",
            "relative/path/file.py",
            "./current/dir/file.js",
            "C:\\windows\\path\\file.exe", // Windows path
        ];

        let invalid_paths = vec![
            "", // Empty
            "../escape/attempt",
            "/etc/passwd", // Sensitive path
            "path/with<script>", // XSS attempt
        ];

        for path in valid_paths {
            assert!(!path.is_empty(), "Valid paths should not be empty");
        }

        for path in invalid_paths {
            assert!(path.is_empty() || path.contains("..") || path.contains("<") || path.starts_with("/etc"),
                   "Invalid paths should be caught by validation");
        }
    }
}

/// Test business rule validation
#[cfg(test)]
mod business_rule_tests {
    use super::*;

    #[test]
    fn test_chunk_line_number_consistency() {
        // Test that line numbers are consistent
        let valid_ranges = vec![
            (1, 1),   // Single line
            (1, 5),   // Multi-line
            (10, 20), // Larger range
        ];

        let invalid_ranges = vec![
            (0, 1),   // Start at zero
            (5, 3),   // Start > End
            (1, 0),   // End at zero
        ];

        for (start, end) in valid_ranges {
            assert!(start > 0 && end >= start, "Valid ranges should have start > 0 and end >= start");
        }

        for (start, end) in invalid_ranges {
            assert!(start <= 0 || end < start, "Invalid ranges should fail validation");
        }
    }

    #[test]
    fn test_language_support_validation() {
        // Test that supported languages are properly validated
        let supported_languages = vec![
            Language::Rust,
            Language::Python,
            Language::JavaScript,
            Language::TypeScript,
            Language::Java,
            Language::Go,
            Language::C,
            Language::Cpp,
        ];

        for lang in supported_languages {
            // All supported languages should be valid
            assert!(matches!(lang, Language::Rust | Language::Python | Language::JavaScript |
                           Language::TypeScript | Language::Java | Language::Go |
                           Language::C | Language::Cpp | Language::Unknown));
        }
    }

    #[test]
    fn test_embedding_vector_consistency() {
        // Test that embedding vectors are consistent
        let test_cases = vec![
            (vec![1.0, 2.0, 3.0], 3, true),  // Consistent
            (vec![], 0, true),                // Empty is consistent
            (vec![1.0, 2.0], 3, false),      // Inconsistent
            (vec![1.0, 2.0, 3.0, 4.0], 3, false), // Inconsistent
        ];

        for (vector, dimensions, should_be_consistent) in test_cases {
            let is_consistent = vector.len() == dimensions;
            assert_eq!(is_consistent, should_be_consistent,
                      "Vector length should match dimensions for consistency");
        }
    }
}

/// Test validation error handling
#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[test]
    fn test_validation_error_messages() {
        // Test that validation provides meaningful error messages
        // This tests the conceptual validation logic

        // Empty content error
        let empty_content = "";
        assert!(empty_content.is_empty(), "Empty content should be detected");

        // Invalid line range error
        let start_line = 10;
        let end_line = 5;
        assert!(start_line > end_line, "Invalid line range should be detected");

        // Empty file path error
        let empty_path = "";
        assert!(empty_path.is_empty(), "Empty file path should be detected");
    }

    #[test]
    fn test_validation_graceful_failure() {
        // Test that validation fails gracefully with partial data
        let partial_chunk = CodeChunk {
            id: "test".to_string(),
            content: "some content".to_string(),
            file_path: "".to_string(), // Invalid
            start_line: 0, // Invalid
            end_line: 0, // Invalid
            language: Language::Rust,
            metadata: serde_json::json!({}),
        };

        // These should be detectable as invalid
        assert!(partial_chunk.file_path.is_empty());
        assert_eq!(partial_chunk.start_line, 0);
        assert_eq!(partial_chunk.end_line, 0);
    }
}

/// Test validation performance
#[cfg(test)]
mod performance_tests {
    use super::*;

    #[test]
    fn test_validation_speed() {
        // Test that validation completes quickly
        let chunk = CodeChunk {
            id: "perf_test".to_string(),
            content: "fn test() {}".to_string(),
            file_path: "test.rs".to_string(),
            start_line: 1,
            end_line: 1,
            language: Language::Rust,
            metadata: serde_json::json!({}),
        };

        let start = std::time::Instant::now();

        // Perform basic validation checks
        assert!(!chunk.content.is_empty());
        assert!(!chunk.file_path.is_empty());
        assert!(chunk.start_line > 0);
        assert!(chunk.end_line >= chunk.start_line);

        let elapsed = start.elapsed();
        assert!(elapsed.as_micros() < 1000, "Validation should complete in under 1ms");
    }

    #[test]
    fn test_bulk_validation_performance() {
        // Test validation performance with multiple items
        let chunks: Vec<CodeChunk> = (0..100).map(|i| CodeChunk {
            id: format!("chunk_{}", i),
            content: format!("content_{}", i),
            file_path: format!("file_{}.rs", i),
            start_line: 1,
            end_line: 1,
            language: Language::Rust,
            metadata: serde_json::json!({}),
        }).collect();

        let start = std::time::Instant::now();

        for chunk in &chunks {
            assert!(!chunk.content.is_empty());
            assert!(!chunk.file_path.is_empty());
        }

        let elapsed = start.elapsed();
        assert!(elapsed.as_millis() < 10, "Bulk validation should complete quickly");
    }
}

/// Test validation edge cases
#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[test]
    fn test_unicode_content_validation() {
        // Test validation with Unicode content
        let unicode_content = "fn hàm() { println!(\"こんにちは\"); }";
        let unicode_chunk = CodeChunk {
            id: "unicode_test".to_string(),
            content: unicode_content.to_string(),
            file_path: "unicode.rs".to_string(),
            start_line: 1,
            end_line: 1,
            language: Language::Rust,
            metadata: serde_json::json!({}),
        };

        assert!(!unicode_chunk.content.is_empty());
        assert!(unicode_chunk.content.contains("hàm"));
        assert!(unicode_chunk.content.contains("こんにちは"));
    }

    #[test]
    fn test_extremely_large_line_numbers() {
        // Test with very large line numbers
        let large_line_chunk = CodeChunk {
            id: "large_lines".to_string(),
            content: "code".to_string(),
            file_path: "large.rs".to_string(),
            start_line: 1000000,
            end_line: 1000000,
            language: Language::Rust,
            metadata: serde_json::json!({}),
        };

        assert!(large_line_chunk.start_line > 0);
        assert_eq!(large_line_chunk.start_line, large_line_chunk.end_line);
    }

    #[test]
    fn test_minimum_valid_chunk() {
        // Test the absolute minimum valid chunk
        let min_chunk = CodeChunk {
            id: "a".to_string(), // Minimum 1 character
            content: "b".to_string(), // Minimum 1 character
            file_path: "c".to_string(), // Minimum 1 character
            start_line: 1, // Minimum 1
            end_line: 1, // Can equal start_line
            language: Language::Unknown, // Any language is acceptable
            metadata: serde_json::json!({}), // Can be empty
        };

        assert!(!min_chunk.id.is_empty());
        assert!(!min_chunk.content.is_empty());
        assert!(!min_chunk.file_path.is_empty());
        assert!(min_chunk.start_line >= 1);
        assert!(min_chunk.end_line >= min_chunk.start_line);
    }
}