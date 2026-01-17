//! Test fixtures for mcb-server tests
//!
//! Provides factory functions for creating test data and temporary directories.

use mcb_application::domain_services::search::{IndexingResult, IndexingStatus};
use mcb_domain::value_objects::SearchResult;
use std::path::PathBuf;
use tempfile::TempDir;

/// Create a test search result
pub fn create_test_search_result(
    file_path: &str,
    content: &str,
    score: f64,
    start_line: u32,
) -> SearchResult {
    SearchResult {
        id: format!("chunk_{}", start_line),
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
                &format!("src/module_{}.rs", i),
                &format!("fn test_function_{}() {{ /* implementation */ }}", i),
                0.9 - (i as f64 * 0.05),
                (i * 10 + 1) as u32,
            )
        })
        .collect()
}

/// Create a test indexing result
pub fn create_test_indexing_result(
    files_processed: usize,
    chunks_created: usize,
    files_skipped: usize,
) -> IndexingResult {
    IndexingResult {
        files_processed,
        chunks_created,
        files_skipped,
        errors: Vec::new(),
    }
}

/// Create a test indexing result with errors
pub fn create_test_indexing_result_with_errors(
    files_processed: usize,
    chunks_created: usize,
    errors: Vec<String>,
) -> IndexingResult {
    IndexingResult {
        files_processed,
        chunks_created,
        files_skipped: errors.len(),
        errors,
    }
}

/// Create a test indexing status (idle)
pub fn create_idle_status() -> IndexingStatus {
    IndexingStatus {
        is_indexing: false,
        progress: 0.0,
        current_file: None,
        total_files: 0,
        processed_files: 0,
    }
}

/// Create a test indexing status (in progress)
pub fn create_in_progress_status(progress: f64, current_file: &str) -> IndexingStatus {
    IndexingStatus {
        is_indexing: true,
        progress,
        current_file: Some(current_file.to_string()),
        total_files: 100,
        processed_files: (progress * 100.0) as usize,
    }
}

/// Create a temporary codebase directory with sample files
pub fn create_temp_codebase() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let codebase_path = temp_dir.path().join("sample_codebase");
    std::fs::create_dir_all(&codebase_path).expect("Failed to create codebase directory");

    // Create sample Rust file
    let rust_file = codebase_path.join("lib.rs");
    std::fs::write(
        &rust_file,
        r#"//! Sample library for testing

/// Add two numbers
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// Subtract two numbers
pub fn subtract(a: i32, b: i32) -> i32 {
    a - b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(1, 2), 3);
    }
}
"#,
    )
    .expect("Failed to write Rust file");

    // Create sample Python file
    let python_file = codebase_path.join("main.py");
    std::fs::write(
        &python_file,
        r#"#!/usr/bin/env python3
"""Sample Python module for testing."""

def greet(name: str) -> str:
    """Return a greeting message."""
    return f"Hello, {name}!"

if __name__ == "__main__":
    print(greet("World"))
"#,
    )
    .expect("Failed to write Python file");

    (temp_dir, codebase_path)
}

/// Create an empty temporary directory
pub fn create_empty_temp_dir() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let path = temp_dir.path().to_path_buf();
    (temp_dir, path)
}
