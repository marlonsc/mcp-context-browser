//! End-to-end tests for MCP server with Ollama
//!
//! These tests simulate real MCP client interactions using stdio transport.

use mcp_context_browser::server::McpServer;
use rmcp::model::{CallToolRequestParam, ReadResourceRequestParam};
use rmcp::service::RequestContext;
use rmcp::transport::stdio;
use rmcp::RoleServer;
use std::sync::Arc;
use tempfile::tempdir;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};

/// Test utilities for MCP end-to-end tests
mod test_utils {
    use super::*;

    pub async fn create_test_server() -> Result<McpServer, Box<dyn std::error::Error>> {
        // Create server with defaults (should use Ollama if available, fallback to mock)
        let server = McpServer::new(None)?;
        Ok(server)
    }

    pub async fn create_test_codebase() -> Result<tempfile::TempDir, Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;

        // Create a simple Rust project structure
        let src_dir = temp_dir.path().join("src");
        std::fs::create_dir(&src_dir)?;

        // Create main.rs
        std::fs::write(src_dir.join("main.rs"), r#"// Simple Rust application
fn main() {
    println!("Hello from MCP test!");
    let app = Application::new();
    app.run();
}

struct Application {
    config: Config,
}

impl Application {
    fn new() -> Self {
        Self {
            config: Config::default(),
        }
    }

    fn run(&self) {
        println!("Running application with config: {:?}", self.config);
        self.process_data();
        self.handle_errors();
    }

    fn process_data(&self) {
        let data = vec![1, 2, 3, 4, 5];
        for item in data {
            println!("Processing item: {}", item);
        }
    }

    fn handle_errors(&self) {
        match self.validate_config() {
            Ok(_) => println!("Configuration is valid"),
            Err(e) => eprintln!("Configuration error: {}", e),
        }
    }

    fn validate_config(&self) -> Result<(), String> {
        if self.config.port == 0 {
            return Err("Invalid port".to_string());
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
struct Config {
    host: String,
    port: u16,
    debug: bool,
}
"#)?;

        // Create lib.rs
        std::fs::write(src_dir.join("lib.rs"), r#"//! Test library for MCP indexing

pub mod utils;

/// Utility functions for data processing
pub mod data {
    pub fn process_items(items: &[i32]) -> Vec<i32> {
        items.iter().map(|x| x * 2).collect()
    }

    pub fn filter_even(items: &[i32]) -> Vec<i32> {
        items.iter().filter(|x| *x % 2 == 0).cloned().collect()
    }
}

/// Error handling utilities
pub mod error {
    use std::fmt;

    #[derive(Debug)]
    pub struct AppError {
        message: String,
    }

    impl AppError {
        pub fn new(message: &str) -> Self {
            Self {
                message: message.to_string(),
            }
        }
    }

    impl fmt::Display for AppError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", self.message)
        }
    }

    impl std::error::Error for AppError {}
}
"#)?;

        // Create Cargo.toml
        std::fs::write(temp_dir.path().join("Cargo.toml"), r#"[package]
name = "mcp-test-app"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "1.0"
tokio = "1.0"
"#)?;

        Ok(temp_dir)
    }
}

#[cfg(test)]
mod mcp_server_tests {
    use super::*;

    #[tokio::test]
    async fn test_mcp_server_initialization() {
        let server = test_utils::create_test_server().await
            .expect("Failed to create MCP server");

        // Test that server info is available
        let server_info = server.get_info();
        assert_eq!(server_info.server_info.name, "MCP Context Browser");
        assert!(server_info.capabilities.tools.is_some());

        // Test that tools are available
        let tools_result = server.list_tools(None, RequestContext::new(RoleServer)).await
            .expect("Failed to list tools");

        let tool_names: Vec<&str> = tools_result.tools.iter()
            .map(|t| t.name.as_ref())
            .collect();

        assert!(tool_names.contains(&"index_codebase"));
        assert!(tool_names.contains(&"search_code"));
        assert!(tool_names.contains(&"get_indexing_status"));

        println!("✅ MCP server initialization test passed");
    }

    #[tokio::test]
    async fn test_mcp_index_and_search_workflow() {
        let server = test_utils::create_test_server().await
            .expect("Failed to create MCP server");

        // Create test codebase
        let temp_dir = test_utils::create_test_codebase().await
            .expect("Failed to create test codebase");

        // Test indexing
        let index_args = serde_json::json!({
            "path": temp_dir.path().to_string(),
            "token": "test-token"
        });

        let index_request = CallToolRequestParam {
            name: "index_codebase".into(),
            arguments: Some(index_args.as_object().unwrap().clone()),
            meta: Default::default(),
        };

        let index_result = server.call_tool(index_request, RequestContext::new(RoleServer)).await
            .expect("Indexing failed");

        // Should get success response
        assert!(index_result.is_success());

        // Test search
        let search_args = serde_json::json!({
            "query": "process data",
            "limit": 5,
            "token": "test-token"
        });

        let search_request = CallToolRequestParam {
            name: "search_code".into(),
            arguments: Some(search_args.as_object().unwrap().clone()),
            meta: Default::default(),
        };

        let search_result = server.call_tool(search_request, RequestContext::new(RoleServer)).await
            .expect("Search failed");

        // Should get success response
        assert!(search_result.is_success());

        println!("✅ MCP index and search workflow test passed");
    }

    #[tokio::test]
    async fn test_mcp_status_reporting() {
        let server = test_utils::create_test_server().await
            .expect("Failed to create MCP server");

        // Test status reporting
        let status_args = serde_json::json!({
            "collection": "default"
        });

        let status_request = CallToolRequestParam {
            name: "get_indexing_status".into(),
            arguments: Some(status_args.as_object().unwrap().clone()),
            meta: Default::default(),
        };

        let status_result = server.call_tool(status_request, RequestContext::new(RoleServer)).await
            .expect("Status check failed");

        // Should get success response
        assert!(status_result.is_success());

        println!("✅ MCP status reporting test passed");
    }

    #[tokio::test]
    async fn test_mcp_error_handling() {
        let server = test_utils::create_test_server().await
            .expect("Failed to create MCP server");

        // Test with invalid path
        let index_args = serde_json::json!({
            "path": "/nonexistent/path/that/does/not/exist",
            "token": "test-token"
        });

        let index_request = CallToolRequestParam {
            name: "index_codebase".into(),
            arguments: Some(index_args.as_object().unwrap().clone()),
            meta: Default::default(),
        };

        let index_result = server.call_tool(index_request, RequestContext::new(RoleServer)).await
            .expect("Should handle error gracefully");

        // Should get success response (error is wrapped in content)
        assert!(index_result.is_success());

        // Test with empty query
        let search_args = serde_json::json!({
            "query": "",
            "limit": 5,
            "token": "test-token"
        });

        let search_request = CallToolRequestParam {
            name: "search_code".into(),
            arguments: Some(search_args.as_object().unwrap().clone()),
            meta: Default::default(),
        };

        let search_result = server.call_tool(search_request, RequestContext::new(RoleServer)).await
            .expect("Should handle empty query gracefully");

        assert!(search_result.is_success());

        println!("✅ MCP error handling test passed");
    }
}

#[cfg(test)]
mod stdio_transport_tests {
    use super::*;
    use std::process::Stdio;
    use tokio::time::{timeout, Duration};

    #[tokio::test]
    async fn test_stdio_transport_basic() {
        // This test requires the binary to be built
        // We'll skip if binary doesn't exist
        let binary_path = std::env::current_exe()
            .expect("Failed to get current exe path")
            .parent()
            .expect("Failed to get parent dir")
            .join("../../../target/debug/mcp-context-browser");

        if !binary_path.exists() {
            println!("⚠️  Binary not found, skipping stdio transport test");
            return;
        }

        // Start the MCP server process
        let mut child = Command::new(&binary_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to start MCP server");

        // Get handles to stdin/stdout
        let stdin = child.stdin.take().expect("Failed to get stdin");
        let stdout = child.stdout.take().expect("Failed to get stdout");

        // Initialize MCP protocol
        let init_message = r#"{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "test-client", "version": "1.0.0"}}}"#;

        let mut stdin_writer = tokio::io::BufWriter::new(stdin);
        let mut stdout_reader = tokio::io::BufReader::new(stdout).lines();

        // Send initialize request
        stdin_writer.write_all(format!("Content-Length: {}\r\n\r\n{}", init_message.len(), init_message).as_bytes()).await
            .expect("Failed to send initialize request");
        stdin_writer.flush().await.expect("Failed to flush");

        // Read response with timeout
        let response = timeout(Duration::from_secs(5), stdout_reader.next_line()).await
            .expect("Initialize response timeout")
            .expect("Failed to read initialize response")
            .expect("Empty response");

        // Should get a valid JSON-RPC response
        assert!(response.contains("jsonrpc"));
        assert!(response.contains("result") || response.contains("error"));

        // Clean up
        let _ = child.kill().await;

        println!("✅ MCP stdio transport test passed");
    }
}