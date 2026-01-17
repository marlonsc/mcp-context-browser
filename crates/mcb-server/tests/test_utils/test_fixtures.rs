//! Test fixtures for mcb-server tests
//!
//! Provides factory functions for creating test data and temporary directories.

#![allow(dead_code)]

use mcb_domain::SearchResult;

/// Create a single test search result
pub fn create_test_search_result(
    file_path: &str,
    content: &str,
    score: f64,
    start_line: u32,
) -> SearchResult {
    SearchResult {
        id: format!("test-result-{}", start_line),
        file_path: file_path.to_string(),
        start_line,
        content: content.to_string(),
        score,
        language: "rust".to_string(),
    }
}

/// Create multiple test search results
pub fn create_test_search_results(count: usize) -> Vec<SearchResult> {
    (0..count)
        .map(|i| {
            create_test_search_result(
                &format!("src/file_{}.rs", i),
                &format!("fn test_function_{}() {{\n    // test code\n}}", i),
                0.95 - (i as f64 * 0.05),
                (i as u32) * 10 + 1,
            )
        })
        .collect()
}
