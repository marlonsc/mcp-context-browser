//! Stdio Transport Integration Tests
//!
//! End-to-end tests for the stdio transport mode used by Claude Code.
//! These tests spawn the actual mcb binary and communicate via stdin/stdout.
//!
//! Critical for preventing regressions in MCP protocol communication:
//! - Log pollution (ANSI codes in stdout)
//! - JSON-RPC message framing
//! - Protocol handshake
//!
//! Run with: `cargo test -p mcb-server --test integration stdio_transport`

use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

/// Get the path to the mcb binary.
///
/// Uses CARGO_BIN_EXE_mcb which is set by cargo test when
/// the binary is built as part of the test run.
fn get_mcb_path() -> PathBuf {
    // cargo test sets this environment variable when the binary is part of the workspace
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_mcb") {
        return PathBuf::from(path);
    }

    // Fallback: look in target directory relative to manifest
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let debug_path = PathBuf::from(manifest_dir).join("../../target/debug/mcb");
    if debug_path.exists() {
        return debug_path;
    }

    let release_path = PathBuf::from(manifest_dir).join("../../target/release/mcb");
    if release_path.exists() {
        return release_path;
    }

    panic!(
        "mcb binary not found. Run `cargo build -p mcb-server` first.\n\
         Checked:\n\
         - CARGO_BIN_EXE_mcb env var\n\
         - {}/../../target/debug/mcb\n\
         - {}/../../target/release/mcb",
        manifest_dir, manifest_dir
    );
}

/// Helper to spawn mcb binary with stdio transport
fn spawn_mcb_stdio() -> std::process::Child {
    let mcb_path = get_mcb_path();

    Command::new(&mcb_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| panic!("Failed to spawn mcb at {:?}: {}", mcb_path, e))
}

/// Send a JSON-RPC request and read the response
fn send_request_get_response(
    stdin: &mut std::process::ChildStdin,
    stdout: &mut BufReader<std::process::ChildStdout>,
    request: &serde_json::Value,
) -> serde_json::Value {
    // Send request with newline delimiter
    let request_str = serde_json::to_string(request).unwrap();
    writeln!(stdin, "{}", request_str).expect("Failed to write request");
    stdin.flush().expect("Failed to flush stdin");

    // Read response line
    let mut response_line = String::new();
    stdout
        .read_line(&mut response_line)
        .expect("Failed to read response");

    serde_json::from_str(&response_line).expect("Failed to parse JSON response")
}

/// Create the MCP initialize request required to start a session
fn create_initialize_request(id: i64) -> serde_json::Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-03-26",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        },
        "id": id
    })
}

/// Send the initialized notification (required after initialize response)
fn send_initialized_notification(stdin: &mut std::process::ChildStdin) {
    let notification = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    });
    let notification_str = serde_json::to_string(&notification).unwrap();
    writeln!(stdin, "{}", notification_str).expect("Failed to write notification");
    stdin.flush().expect("Failed to flush stdin");
}

/// Initialize the MCP session (required before any other requests)
fn initialize_mcp_session(
    stdin: &mut std::process::ChildStdin,
    stdout: &mut BufReader<std::process::ChildStdout>,
) -> serde_json::Value {
    let init_request = create_initialize_request(0);
    let response = send_request_get_response(stdin, stdout, &init_request);

    // Send initialized notification
    send_initialized_notification(stdin);

    response
}

// =============================================================================
// STDOUT PURITY TESTS - Prevent regression of commit ffbe441
// =============================================================================

/// Test that stdout contains no ANSI escape codes (log pollution)
///
/// This prevents regression of the fix in commit ffbe441 where ANSI color codes
/// from logging were polluting the JSON-RPC stream on stdout.
#[test]
fn test_stdio_no_ansi_codes_in_output() {
    let mut child = spawn_mcb_stdio();

    let mut stdin = child.stdin.take().expect("Failed to get stdin");
    let stdout = child.stdout.take().expect("Failed to get stdout");
    let mut stdout_reader = BufReader::new(stdout);

    // Send initialize request (required by MCP protocol)
    let request = create_initialize_request(1);

    let request_str = serde_json::to_string(&request).unwrap();
    writeln!(stdin, "{}", request_str).expect("Failed to write request");
    stdin.flush().expect("Failed to flush stdin");

    // Read response
    let mut response_line = String::new();
    stdout_reader
        .read_line(&mut response_line)
        .expect("Failed to read response");

    // CRITICAL: Check for ANSI escape codes
    // \x1b[ is the start of ANSI escape sequences
    assert!(
        !response_line.contains("\x1b["),
        "ANSI escape codes found in stdout! This breaks JSON-RPC protocol.\nResponse: {:?}",
        response_line
    );

    // Also check for common ANSI codes
    assert!(
        !response_line.contains("\x1b"),
        "Escape character found in stdout! Response: {:?}",
        response_line
    );

    // Kill the process and wait to avoid zombies
    drop(stdin);
    let _ = child.kill();
    let _ = child.wait();
}

/// Test that response is valid JSON (not corrupted by logs)
#[test]
fn test_stdio_response_is_valid_json() {
    let mut child = spawn_mcb_stdio();

    let mut stdin = child.stdin.take().expect("Failed to get stdin");
    let stdout = child.stdout.take().expect("Failed to get stdout");
    let mut stdout_reader = BufReader::new(stdout);

    // Initialize session first (required by MCP protocol)
    let _ = initialize_mcp_session(&mut stdin, &mut stdout_reader);

    // Send tools/list request
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "tools/list",
        "id": 1
    });

    let request_str = serde_json::to_string(&request).unwrap();
    writeln!(stdin, "{}", request_str).expect("Failed to write request");
    stdin.flush().expect("Failed to flush stdin");

    // Read response
    let mut response_line = String::new();
    stdout_reader
        .read_line(&mut response_line)
        .expect("Failed to read response");

    // Verify it's valid JSON
    let response: Result<serde_json::Value, _> = serde_json::from_str(&response_line);
    assert!(
        response.is_ok(),
        "Response is not valid JSON! Raw output: {:?}\nParse error: {:?}",
        response_line,
        response.err()
    );

    // Verify it has JSON-RPC structure
    let response = response.unwrap();
    assert_eq!(
        response.get("jsonrpc").and_then(|v| v.as_str()),
        Some("2.0"),
        "Response missing jsonrpc field"
    );

    drop(stdin);
    let _ = child.kill();
    let _ = child.wait();
}

// =============================================================================
// END-TO-END ROUNDTRIP TESTS
// =============================================================================

/// Test complete tools/list roundtrip via stdio
#[test]
fn test_stdio_roundtrip_tools_list() {
    let mut child = spawn_mcb_stdio();

    let mut stdin = child.stdin.take().expect("Failed to get stdin");
    let stdout = child.stdout.take().expect("Failed to get stdout");
    let mut stdout_reader = BufReader::new(stdout);

    // Initialize session first (required by MCP protocol)
    let _ = initialize_mcp_session(&mut stdin, &mut stdout_reader);

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "tools/list",
        "id": 42
    });

    let response = send_request_get_response(&mut stdin, &mut stdout_reader, &request);

    // Verify response structure
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 42);
    assert!(
        response["error"].is_null(),
        "Unexpected error: {:?}",
        response["error"]
    );

    // Verify tools are returned
    let result = &response["result"];
    assert!(result["tools"].is_array(), "tools should be an array");

    let tools = result["tools"].as_array().unwrap();
    assert!(!tools.is_empty(), "Should have at least one tool");

    // Verify expected tools exist
    let tool_names: Vec<&str> = tools
        .iter()
        .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
        .collect();

    assert!(
        tool_names.contains(&"index_codebase"),
        "Missing index_codebase tool"
    );
    assert!(
        tool_names.contains(&"search_code"),
        "Missing search_code tool"
    );
    assert!(
        tool_names.contains(&"get_indexing_status"),
        "Missing get_indexing_status tool"
    );
    assert!(
        tool_names.contains(&"clear_index"),
        "Missing clear_index tool"
    );

    drop(stdin);
    let _ = child.kill();
    let _ = child.wait();
}

/// Test initialize request via stdio
#[test]
fn test_stdio_roundtrip_initialize() {
    let mut child = spawn_mcb_stdio();

    let mut stdin = child.stdin.take().expect("Failed to get stdin");
    let stdout = child.stdout.take().expect("Failed to get stdout");
    let mut stdout_reader = BufReader::new(stdout);

    let request = create_initialize_request(1);

    let response = send_request_get_response(&mut stdin, &mut stdout_reader, &request);

    // Verify response structure
    assert_eq!(response["jsonrpc"], "2.0");
    assert!(
        response["error"].is_null(),
        "Unexpected error: {:?}",
        response["error"]
    );

    let result = &response["result"];

    // Verify protocol version is a proper string (not Debug format)
    let version = &result["protocolVersion"];
    assert!(version.is_string(), "protocolVersion should be a string");
    assert!(
        !version.as_str().unwrap().contains("ProtocolVersion"),
        "protocolVersion has Debug format leak"
    );

    // Verify serverInfo
    assert!(result["serverInfo"].is_object(), "Should have serverInfo");
    assert!(
        result["serverInfo"]["name"].is_string(),
        "Should have server name"
    );

    drop(stdin);
    let _ = child.kill();
    let _ = child.wait();
}

/// Test error response via stdio (unknown method)
#[test]
fn test_stdio_error_response_format() {
    let mut child = spawn_mcb_stdio();

    let mut stdin = child.stdin.take().expect("Failed to get stdin");
    let stdout = child.stdout.take().expect("Failed to get stdout");
    let mut stdout_reader = BufReader::new(stdout);

    // Initialize session first (required by MCP protocol)
    let _ = initialize_mcp_session(&mut stdin, &mut stdout_reader);

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "nonexistent/method",
        "id": 99
    });

    let response = send_request_get_response(&mut stdin, &mut stdout_reader, &request);

    // Verify error response structure
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 99);
    assert!(response["result"].is_null(), "Should not have result");
    assert!(response["error"].is_object(), "Should have error object");

    let error = &response["error"];
    assert!(error["code"].is_i64(), "Error should have numeric code");
    assert!(error["message"].is_string(), "Error should have message");

    drop(stdin);
    let _ = child.kill();
    let _ = child.wait();
}

// =============================================================================
// LOGGING TO STDERR TEST
// =============================================================================

/// Test that logs go to stderr, not stdout
#[test]
fn test_stdio_logs_go_to_stderr() {
    let mut child = spawn_mcb_stdio();

    let mut stdin = child.stdin.take().expect("Failed to get stdin");
    let stdout = child.stdout.take().expect("Failed to get stdout");
    let stderr = child.stderr.take().expect("Failed to get stderr");

    let mut stdout_reader = BufReader::new(stdout);
    let stderr_reader = BufReader::new(stderr);

    // Initialize session first (required by MCP protocol)
    let _ = initialize_mcp_session(&mut stdin, &mut stdout_reader);

    // Send request
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "tools/list",
        "id": 1
    });

    let request_str = serde_json::to_string(&request).unwrap();
    writeln!(stdin, "{}", request_str).expect("Failed to write request");
    stdin.flush().expect("Failed to flush stdin");

    // Read stdout response
    let mut response_line = String::new();
    stdout_reader
        .read_line(&mut response_line)
        .expect("Failed to read response");

    // Stdout should be pure JSON
    let response: serde_json::Value =
        serde_json::from_str(&response_line).expect("Stdout should be valid JSON");
    assert_eq!(response["jsonrpc"], "2.0");

    // Give some time for stderr to accumulate logs
    std::thread::sleep(Duration::from_millis(100));

    // Check if stderr has content (logs)
    // Note: We can't guarantee logs are present, but if they are, they should be on stderr
    let stderr_lines: Vec<_> = stderr_reader.lines().take(10).collect();

    // If there are any stderr lines with log-like content, that's expected behavior
    // The key assertion is that stdout ONLY has JSON
    for line in stderr_lines.into_iter().flatten() {
        // Stderr lines should NOT be valid JSON-RPC responses
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&line);
        if let Ok(json) = parsed {
            assert!(
                json.get("jsonrpc").is_none(),
                "JSON-RPC message found in stderr - should be on stdout!"
            );
        }
    }

    drop(stdin);
    let _ = child.kill();
    let _ = child.wait();
}
