//! MCP Error Handling Tests
//!
//! Comprehensive tests that validate BOTH the `is_error` flag AND the content
//! of error/success messages for MCP compliance.
//!
//! Phase 2 of v0.1.2: These tests verify that errors return `is_error: Some(true)`
//! and contain proper troubleshooting information.
#![allow(clippy::collapsible_if)]

use mcb_application::domain_services::search::{IndexingResult, IndexingStatus};
use mcb_server::formatter::ResponseFormatter;
use std::path::Path;
use std::time::Duration;

use crate::test_utils::test_fixtures::{create_test_search_result, create_test_search_results};

// =============================================================================
// ERROR RESPONSE TESTS - Must have is_error: Some(true) AND proper content
// =============================================================================

#[test]
fn test_format_indexing_error_has_is_error_true() {
    let path = Path::new("/nonexistent/path");
    let response = ResponseFormatter::format_indexing_error("Path does not exist", path);

    // CRITICAL: Error responses MUST have is_error: Some(true) per MCP spec
    assert!(
        response.is_error.unwrap_or(false),
        "Error response MUST have is_error: true for MCP compliance"
    );
}

#[test]
fn test_format_indexing_error_contains_error_details() {
    let path = Path::new("/nonexistent/path");
    let error_message = "Directory not found";
    let response = ResponseFormatter::format_indexing_error(error_message, path);

    // Extract text content
    let text = extract_text_content(&response.content);

    // Verify error message is included
    assert!(
        text.contains(error_message),
        "Error response MUST contain the error message. Got: {}",
        text
    );
}

#[test]
fn test_format_indexing_error_contains_troubleshooting() {
    let path = Path::new("/some/path");
    let response = ResponseFormatter::format_indexing_error("Storage quota exceeded", path);

    let text = extract_text_content(&response.content);

    // Verify troubleshooting section is present
    assert!(
        text.contains("Troubleshooting"),
        "Error response MUST contain troubleshooting section. Got: {}",
        text
    );

    // Verify specific troubleshooting tips
    assert!(
        text.contains("Verify the directory"),
        "Error response MUST contain directory verification tip"
    );
    assert!(
        text.contains("file permissions"),
        "Error response MUST contain permissions tip"
    );
    assert!(
        text.contains("supported file types"),
        "Error response MUST contain supported types tip"
    );
}

#[test]
fn test_format_indexing_error_contains_supported_languages() {
    let path = Path::new("/some/path");
    let response = ResponseFormatter::format_indexing_error("Parse error", path);

    let text = extract_text_content(&response.content);

    // Verify supported languages are listed
    assert!(
        text.contains("Supported Languages"),
        "Error response MUST list supported languages"
    );
    assert!(
        text.contains("Rust"),
        "Error response MUST mention Rust as supported"
    );
    assert!(
        text.contains("Python"),
        "Error response MUST mention Python as supported"
    );
    assert!(
        text.contains("JavaScript"),
        "Error response MUST mention JavaScript as supported"
    );
}

#[test]
fn test_format_indexing_error_contains_failed_indicator() {
    let path = Path::new("/some/path");
    let response = ResponseFormatter::format_indexing_error("Any error", path);

    let text = extract_text_content(&response.content);

    // Verify clear failure indicator
    assert!(
        text.contains("Failed") || text.contains("❌"),
        "Error response MUST clearly indicate failure. Got: {}",
        text
    );
}

// =============================================================================
// SUCCESS RESPONSE TESTS - Must have is_error: false/None AND proper content
// =============================================================================

#[test]
fn test_format_indexing_success_has_is_error_false() {
    let result = IndexingResult {
        files_processed: 50,
        chunks_created: 250,
        files_skipped: 5,
        errors: Vec::new(),
    };
    let path = Path::new("/project/src");
    let duration = Duration::from_secs(10);

    let response = ResponseFormatter::format_indexing_success(&result, path, duration);

    // Success responses should NOT have is_error: true
    assert!(
        !response.is_error.unwrap_or(false),
        "Success response MUST NOT have is_error: true"
    );
}

#[test]
fn test_format_indexing_success_contains_statistics() {
    let result = IndexingResult {
        files_processed: 42,
        chunks_created: 156,
        files_skipped: 3,
        errors: Vec::new(),
    };
    let path = Path::new("/my/project");
    let duration = Duration::from_secs(5);

    let response = ResponseFormatter::format_indexing_success(&result, path, duration);
    let text = extract_text_content(&response.content);

    // Verify statistics are present with correct values
    assert!(
        text.contains("42"),
        "Success response MUST contain files_processed count (42). Got: {}",
        text
    );
    assert!(
        text.contains("156"),
        "Success response MUST contain chunks_created count (156). Got: {}",
        text
    );
    assert!(
        text.contains("3"),
        "Success response MUST contain files_skipped count (3). Got: {}",
        text
    );
}

#[test]
fn test_format_indexing_success_contains_path() {
    let result = IndexingResult {
        files_processed: 10,
        chunks_created: 50,
        files_skipped: 0,
        errors: Vec::new(),
    };
    let path = Path::new("/test/project/path");
    let duration = Duration::from_millis(500);

    let response = ResponseFormatter::format_indexing_success(&result, path, duration);
    let text = extract_text_content(&response.content);

    // Verify path is included
    assert!(
        text.contains("/test/project/path"),
        "Success response MUST contain the indexed path. Got: {}",
        text
    );
}

#[test]
fn test_format_indexing_success_contains_timing() {
    let result = IndexingResult {
        files_processed: 100,
        chunks_created: 500,
        files_skipped: 0,
        errors: Vec::new(),
    };
    let path = Path::new("/project");
    let duration = Duration::from_secs(8);

    let response = ResponseFormatter::format_indexing_success(&result, path, duration);
    let text = extract_text_content(&response.content);

    // Verify timing is included (should show "8" somewhere in seconds)
    assert!(
        text.contains("8") && (text.contains("sec") || text.contains("s")),
        "Success response MUST contain processing time. Got: {}",
        text
    );
}

#[test]
fn test_format_indexing_success_contains_next_steps() {
    let result = IndexingResult {
        files_processed: 10,
        chunks_created: 50,
        files_skipped: 0,
        errors: Vec::new(),
    };
    let path = Path::new("/project");
    let duration = Duration::from_secs(1);

    let response = ResponseFormatter::format_indexing_success(&result, path, duration);
    let text = extract_text_content(&response.content);

    // Verify next steps guidance is provided
    assert!(
        text.contains("search_code"),
        "Success response MUST mention search_code for next steps. Got: {}",
        text
    );
}

#[test]
fn test_format_indexing_success_with_errors_lists_them() {
    let result = IndexingResult {
        files_processed: 45,
        chunks_created: 200,
        files_skipped: 10,
        errors: vec![
            "Failed to parse binary.bin".to_string(),
            "Encoding error in data.csv".to_string(),
            "Unsupported format: image.png".to_string(),
        ],
    };
    let path = Path::new("/project");
    let duration = Duration::from_secs(5);

    let response = ResponseFormatter::format_indexing_success(&result, path, duration);
    let text = extract_text_content(&response.content);

    // Still success (not is_error: true) but should list errors
    assert!(
        !response.is_error.unwrap_or(false),
        "Indexing with non-fatal errors is still success"
    );

    // Verify errors are listed
    assert!(
        text.contains("binary.bin"),
        "Response MUST list individual errors. Got: {}",
        text
    );
    assert!(
        text.contains("data.csv"),
        "Response MUST list all errors. Got: {}",
        text
    );
}

// =============================================================================
// SEARCH RESPONSE TESTS
// =============================================================================

#[test]
fn test_format_search_response_success_has_is_error_false() {
    let results = create_test_search_results(3);
    let duration = Duration::from_millis(150);

    let response = ResponseFormatter::format_search_response("test query", &results, duration, 10)
        .expect("Should format successfully");

    assert!(
        !response.is_error.unwrap_or(false),
        "Search success MUST NOT have is_error: true"
    );
}

#[test]
fn test_format_search_response_contains_query() {
    let results = create_test_search_results(2);
    let duration = Duration::from_millis(100);
    let query = "find authentication functions";

    let response = ResponseFormatter::format_search_response(query, &results, duration, 10)
        .expect("Should format successfully");
    let text = extract_text_content(&response.content);

    assert!(
        text.contains(query),
        "Search response MUST contain the query. Got: {}",
        text
    );
}

#[test]
fn test_format_search_response_contains_results_count() {
    let results = create_test_search_results(5);
    let duration = Duration::from_millis(100);

    let response = ResponseFormatter::format_search_response("test", &results, duration, 10)
        .expect("Should format successfully");
    let text = extract_text_content(&response.content);

    assert!(
        text.contains("5"),
        "Search response MUST contain results count (5). Got: {}",
        text
    );
}

#[test]
fn test_format_search_response_contains_file_paths() {
    let results = vec![
        create_test_search_result("src/auth/login.rs", "fn login() {}", 0.95, 1),
        create_test_search_result("src/user/profile.rs", "fn get_profile() {}", 0.90, 10),
    ];
    let duration = Duration::from_millis(50);

    let response = ResponseFormatter::format_search_response("test", &results, duration, 10)
        .expect("Should format successfully");
    let text = extract_text_content(&response.content);

    assert!(
        text.contains("src/auth/login.rs"),
        "Search response MUST contain file paths. Got: {}",
        text
    );
    assert!(
        text.contains("src/user/profile.rs"),
        "Search response MUST contain all file paths. Got: {}",
        text
    );
}

#[test]
fn test_format_search_response_contains_relevance_scores() {
    let results = vec![create_test_search_result(
        "src/main.rs",
        "fn main() {}",
        0.875,
        1,
    )];
    let duration = Duration::from_millis(50);

    let response = ResponseFormatter::format_search_response("test", &results, duration, 10)
        .expect("Should format successfully");
    let text = extract_text_content(&response.content);

    // Score 0.875 should appear somewhere
    assert!(
        text.contains("0.875") || text.contains("0.87") || text.contains("87"),
        "Search response MUST contain relevance scores. Got: {}",
        text
    );
}

#[test]
fn test_format_search_response_empty_contains_tips() {
    let results: Vec<mcb_domain::SearchResult> = vec![];
    let duration = Duration::from_millis(50);

    let response = ResponseFormatter::format_search_response("nonexistent", &results, duration, 10)
        .expect("Should format successfully");
    let text = extract_text_content(&response.content);

    // Empty results should have helpful tips
    assert!(
        text.contains("No Results") || text.contains("No results"),
        "Empty search MUST indicate no results. Got: {}",
        text
    );
    assert!(
        text.contains("index") || text.contains("Index"),
        "Empty search MUST suggest indexing. Got: {}",
        text
    );
}

#[test]
fn test_format_search_response_slow_query_has_warning() {
    let results = create_test_search_results(3);
    let duration = Duration::from_secs(2); // Slow query

    let response = ResponseFormatter::format_search_response("test", &results, duration, 10)
        .expect("Should format successfully");
    let text = extract_text_content(&response.content);

    // Slow queries should have performance note
    assert!(
        text.contains("Performance") || text.contains("⚠️"),
        "Slow query MUST have performance warning. Got: {}",
        text
    );
}

// =============================================================================
// INDEXING STATUS TESTS
// =============================================================================

#[test]
fn test_format_indexing_status_idle_has_is_error_false() {
    let status = IndexingStatus {
        is_indexing: false,
        progress: 0.0,
        current_file: None,
        total_files: 0,
        processed_files: 0,
    };

    let response = ResponseFormatter::format_indexing_status(&status);

    assert!(
        !response.is_error.unwrap_or(false),
        "Status response MUST NOT have is_error: true"
    );
}

#[test]
fn test_format_indexing_status_idle_contains_idle_indicator() {
    let status = IndexingStatus {
        is_indexing: false,
        progress: 0.0,
        current_file: None,
        total_files: 0,
        processed_files: 0,
    };

    let response = ResponseFormatter::format_indexing_status(&status);
    let text = extract_text_content(&response.content);

    assert!(
        text.contains("Idle") || text.contains("idle") || text.contains("not running"),
        "Idle status MUST indicate idle state. Got: {}",
        text
    );
}

#[test]
fn test_format_indexing_status_in_progress_contains_progress() {
    let status = IndexingStatus {
        is_indexing: true,
        progress: 0.65,
        current_file: Some("src/main.rs".to_string()),
        total_files: 100,
        processed_files: 65,
    };

    let response = ResponseFormatter::format_indexing_status(&status);
    let text = extract_text_content(&response.content);

    // Should show progress percentage (65%)
    assert!(
        text.contains("65") && text.contains("%"),
        "In-progress status MUST show progress percentage. Got: {}",
        text
    );
}

#[test]
fn test_format_indexing_status_in_progress_contains_current_file() {
    let status = IndexingStatus {
        is_indexing: true,
        progress: 0.5,
        current_file: Some("src/lib.rs".to_string()),
        total_files: 50,
        processed_files: 25,
    };

    let response = ResponseFormatter::format_indexing_status(&status);
    let text = extract_text_content(&response.content);

    assert!(
        text.contains("src/lib.rs"),
        "In-progress status MUST show current file. Got: {}",
        text
    );
}

#[test]
fn test_format_indexing_status_in_progress_contains_file_counts() {
    let status = IndexingStatus {
        is_indexing: true,
        progress: 0.3,
        current_file: Some("test.rs".to_string()),
        total_files: 200,
        processed_files: 60,
    };

    let response = ResponseFormatter::format_indexing_status(&status);
    let text = extract_text_content(&response.content);

    // Should show processed/total
    assert!(
        text.contains("60") && text.contains("200"),
        "In-progress status MUST show file counts. Got: {}",
        text
    );
}

// =============================================================================
// CLEAR INDEX TESTS
// =============================================================================

#[test]
fn test_format_clear_index_has_is_error_false() {
    let response = ResponseFormatter::format_clear_index("test-collection");

    assert!(
        !response.is_error.unwrap_or(false),
        "Clear index success MUST NOT have is_error: true"
    );
}

#[test]
fn test_format_clear_index_contains_collection_name() {
    let response = ResponseFormatter::format_clear_index("my-project-index");
    let text = extract_text_content(&response.content);

    assert!(
        text.contains("my-project-index"),
        "Clear index MUST contain collection name. Got: {}",
        text
    );
}

#[test]
fn test_format_clear_index_contains_success_indicator() {
    let response = ResponseFormatter::format_clear_index("any-collection");
    let text = extract_text_content(&response.content);

    assert!(
        text.contains("Cleared") || text.contains("cleared") || text.contains("✅"),
        "Clear index MUST indicate success. Got: {}",
        text
    );
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Extract text content from CallToolResult content vector
fn extract_text_content(content: &[rmcp::model::Content]) -> String {
    content
        .iter()
        .filter_map(|c| {
            // Content can be serialized to JSON and we can extract text from there
            if let Ok(json) = serde_json::to_value(c) {
                if let Some(text) = json.get("text") {
                    return text.as_str().map(|s| s.to_string());
                }
            }
            None
        })
        .collect::<Vec<_>>()
        .join("\n")
}

// =============================================================================
// INTEGRATION TESTS - Full handler flow
// =============================================================================

mod handler_error_tests {
    use super::*;
    use mcb_server::args::IndexCodebaseArgs;
    use mcb_server::handlers::IndexCodebaseHandler;
    use rmcp::handler::server::wrapper::Parameters;
    use std::sync::Arc;

    use crate::test_utils::mock_services::MockIndexingService;

    #[tokio::test]
    async fn test_handler_service_error_has_is_error_true() {
        let mock_service = MockIndexingService::new().with_failure("Storage quota exceeded");
        let handler = IndexCodebaseHandler::new(Arc::new(mock_service));

        // Create a temp directory for valid path
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let args = IndexCodebaseArgs {
            path: temp_dir.path().to_string_lossy().to_string(),
            collection: Some("test".to_string()),
            extensions: None,
            ignore_patterns: None,
            max_file_size: None,
            follow_symlinks: None,
            token: None,
        };

        let result = handler.handle(Parameters(args)).await;

        // Service errors should be wrapped in CallToolResult with is_error: true
        match result {
            Ok(response) => {
                assert!(
                    response.is_error.unwrap_or(false),
                    "Service error should result in is_error: true. Got response with is_error: {:?}",
                    response.is_error
                );
            }
            Err(_) => {
                // MCP protocol error is also acceptable for service failures
            }
        }
    }

    #[tokio::test]
    async fn test_handler_service_error_contains_error_message() {
        let error_msg = "Database connection failed";
        let mock_service = MockIndexingService::new().with_failure(error_msg);
        let handler = IndexCodebaseHandler::new(Arc::new(mock_service));

        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let args = IndexCodebaseArgs {
            path: temp_dir.path().to_string_lossy().to_string(),
            collection: Some("test".to_string()),
            extensions: None,
            ignore_patterns: None,
            max_file_size: None,
            follow_symlinks: None,
            token: None,
        };

        let result = handler.handle(Parameters(args)).await;

        match result {
            Ok(response) => {
                let text = extract_text_content(&response.content);
                assert!(
                    text.contains(error_msg)
                        || text.contains("Database")
                        || text.contains("failed"),
                    "Error response should contain error details. Got: {}",
                    text
                );
            }
            Err(err) => {
                // Check the MCP error message
                let err_str = format!("{:?}", err);
                assert!(
                    err_str.contains("Database") || err_str.contains("failed"),
                    "MCP error should contain error details. Got: {}",
                    err_str
                );
            }
        }
    }
}
