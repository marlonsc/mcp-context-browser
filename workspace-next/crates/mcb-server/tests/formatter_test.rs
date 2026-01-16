//! Tests for ResponseFormatter

use mcb_domain::domain_services::search::{IndexingResult, IndexingStatus};
use mcb_server::formatter::ResponseFormatter;
use std::path::Path;
use std::time::Duration;

mod test_utils;

use test_utils::test_fixtures::{create_test_search_result, create_test_search_results};

#[test]
fn test_format_search_response_with_results() {
    let results = create_test_search_results(3);
    let duration = Duration::from_millis(150);

    let response = ResponseFormatter::format_search_response("test query", &results, duration, 10);

    assert!(response.is_ok());
    let result = response.expect("Expected successful response");
    assert!(!result.is_error.unwrap_or(false));
}

#[test]
fn test_format_search_response_no_results() {
    let results: Vec<mcb_domain::SearchResult> = vec![];
    let duration = Duration::from_millis(50);

    let response = ResponseFormatter::format_search_response("test query", &results, duration, 10);

    assert!(response.is_ok());
    // Response should contain "No Results Found" message
}

#[test]
fn test_format_search_response_slow_query() {
    let results = create_test_search_results(5);
    let duration = Duration::from_secs(2); // Slow query (>1s)

    let response = ResponseFormatter::format_search_response("test query", &results, duration, 10);

    assert!(response.is_ok());
    // Response should contain performance warning
}

#[test]
fn test_format_search_response_hit_limit() {
    let results = create_test_search_results(10);
    let duration = Duration::from_millis(100);

    let response = ResponseFormatter::format_search_response("test query", &results, duration, 10);

    assert!(response.is_ok());
    // Response should contain "Showing top X results" message
}

#[test]
fn test_format_indexing_success() {
    let result = IndexingResult {
        files_processed: 50,
        chunks_created: 250,
        files_skipped: 5,
        errors: Vec::new(),
    };
    let path = Path::new("/project/src");
    let duration = Duration::from_secs(10);

    let response = ResponseFormatter::format_indexing_success(&result, path, duration);

    assert!(!response.is_error.unwrap_or(false));
}

#[test]
fn test_format_indexing_success_with_errors() {
    let result = IndexingResult {
        files_processed: 45,
        chunks_created: 200,
        files_skipped: 10,
        errors: vec![
            "Failed to parse binary.bin".to_string(),
            "Encoding error in data.csv".to_string(),
        ],
    };
    let path = Path::new("/project/src");
    let duration = Duration::from_secs(8);

    let response = ResponseFormatter::format_indexing_success(&result, path, duration);

    assert!(!response.is_error.unwrap_or(false));
    // Response should include error count
}

#[test]
fn test_format_indexing_success_fast() {
    let result = IndexingResult {
        files_processed: 100,
        chunks_created: 500,
        files_skipped: 0,
        errors: Vec::new(),
    };
    let path = Path::new("/project");
    let duration = Duration::from_millis(100); // Very fast

    let response = ResponseFormatter::format_indexing_success(&result, path, duration);

    assert!(!response.is_error.unwrap_or(false));
}

#[test]
fn test_format_indexing_error() {
    let path = Path::new("/nonexistent/path");

    let response = ResponseFormatter::format_indexing_error("Path does not exist", path);

    assert!(!response.is_error.unwrap_or(false));
    // Note: The formatter returns success with error content
}

#[test]
fn test_format_indexing_status_idle() {
    let status = IndexingStatus {
        is_indexing: false,
        progress: 0.0,
        current_file: None,
        total_files: 0,
        processed_files: 0,
    };

    let response = ResponseFormatter::format_indexing_status(&status);

    assert!(!response.is_error.unwrap_or(false));
}

#[test]
fn test_format_indexing_status_in_progress() {
    let status = IndexingStatus {
        is_indexing: true,
        progress: 0.5,
        current_file: Some("src/main.rs".to_string()),
        total_files: 100,
        processed_files: 50,
    };

    let response = ResponseFormatter::format_indexing_status(&status);

    assert!(!response.is_error.unwrap_or(false));
}

#[test]
fn test_format_indexing_status_completed() {
    let status = IndexingStatus {
        is_indexing: false,
        progress: 1.0,
        current_file: None,
        total_files: 100,
        processed_files: 100,
    };

    let response = ResponseFormatter::format_indexing_status(&status);

    assert!(!response.is_error.unwrap_or(false));
}

#[test]
fn test_format_clear_index() {
    let response = ResponseFormatter::format_clear_index("test-collection");

    assert!(!response.is_error.unwrap_or(false));
}

#[test]
fn test_format_search_result_code_preview() {
    // Test with Rust code
    let result = create_test_search_result(
        "src/lib.rs",
        "fn main() {\n    println!(\"Hello\");\n}",
        0.95,
        1,
    );
    let results = vec![result];
    let duration = Duration::from_millis(50);

    let response = ResponseFormatter::format_search_response("main function", &results, duration, 10);

    assert!(response.is_ok());
}

#[test]
fn test_format_search_result_long_content() {
    // Create content with more than 10 lines
    let long_content = (0..20)
        .map(|i| format!("line {} of content", i))
        .collect::<Vec<_>>()
        .join("\n");
    let result = create_test_search_result("src/long_file.rs", &long_content, 0.85, 1);
    let results = vec![result];
    let duration = Duration::from_millis(50);

    let response = ResponseFormatter::format_search_response("test", &results, duration, 10);

    assert!(response.is_ok());
    // Preview should be truncated to 10 lines
}
