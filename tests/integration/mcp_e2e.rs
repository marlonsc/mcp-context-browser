//! End-to-end tests for MCP server with Ollama
//!
//! These tests simulate real MCP client interactions using stdio transport.
//! The rmcp stdio transport uses newline-delimited JSON (NDJSON) format.

use mcp_context_browser::server::McpServer;
use rmcp::ServerHandler;
use tempfile::tempdir;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;

/// Test utilities for MCP end-to-end tests
mod test_utils {
    use super::*;

    pub async fn create_test_server() -> Result<McpServer, Box<dyn std::error::Error>> {
        // Create server with defaults (should use Ollama if available, fallback to mock)
        let server = McpServer::new(None).await?;
        Ok(server)
    }

    pub async fn run_stdio_integration_test() -> Result<(), Box<dyn std::error::Error>> {
        // This is a more realistic integration test that uses the actual MCP stdio protocol
        // We'll skip this test if the binary doesn't exist
        let binary_path = std::env::current_exe()?
            .parent()
            .ok_or("Failed to get parent dir")?
            .join("../../../target/debug/mcp-context-browser");

        if !binary_path.exists() {
            println!("⚠️  Binary not found, skipping stdio integration test");
            return Ok(());
        }

        // Create test codebase
        let _temp_dir = create_test_codebase().await?;

        // Start the MCP server process
        let mut child = Command::new(&binary_path)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        let stdin = child.stdin.take().ok_or("Failed to get stdin")?;
        let stdout = child.stdout.take().ok_or("Failed to get stdout")?;

        let mut stdin_writer = tokio::io::BufWriter::new(stdin);
        let mut stdout_reader = BufReader::new(stdout).lines();

        // Initialize MCP protocol - using newline-delimited JSON (NDJSON) format
        let init_message = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test-client","version":"1.0.0"}}}"#;

        // Send initialize request as NDJSON (JSON followed by newline)
        AsyncWriteExt::write_all(&mut stdin_writer, format!("{}\n", init_message).as_bytes())
            .await?;
        AsyncWriteExt::flush(&mut stdin_writer).await?;

        // Read response (NDJSON)
        let response = tokio::time::timeout(
            tokio::time::Duration::from_secs(15),
            stdout_reader.next_line(),
        )
        .await??
        .ok_or("Empty response")?;

        assert!(response.contains("jsonrpc"));
        assert!(response.contains("result") || response.contains("error"));

        // Send initialized notification (required by MCP protocol before other requests)
        let initialized_message = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
        AsyncWriteExt::write_all(
            &mut stdin_writer,
            format!("{}\n", initialized_message).as_bytes(),
        )
        .await?;
        AsyncWriteExt::flush(&mut stdin_writer).await?;

        // Test tools request
        let tools_message = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#;
        AsyncWriteExt::write_all(&mut stdin_writer, format!("{}\n", tools_message).as_bytes())
            .await?;
        AsyncWriteExt::flush(&mut stdin_writer).await?;

        let tools_response = tokio::time::timeout(
            tokio::time::Duration::from_secs(15),
            stdout_reader.next_line(),
        )
        .await??
        .ok_or("Empty tools response")?;

        assert!(tools_response.contains("index_codebase"));
        assert!(tools_response.contains("search_code"));

        // Clean up
        let _ = child.kill().await;

        Ok(())
    }

    pub async fn create_test_codebase() -> Result<tempfile::TempDir, Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;

        // Create a simple Rust project structure
        let src_dir = temp_dir.path().join("src");
        std::fs::create_dir(&src_dir)?;

        // Create main.rs
        std::fs::write(
            src_dir.join("main.rs"),
            r#"// Simple Rust application
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
"#,
        )?;

        // Create lib.rs
        std::fs::write(
            src_dir.join("lib.rs"),
            r#"//! Test library for MCP indexing

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
"#,
        )?;

        // Create Cargo.toml
        std::fs::write(
            temp_dir.path().join("Cargo.toml"),
            r#"[package]
name = "mcp-test-app"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "1.0"
tokio = "1.0"
"#,
        )?;

        Ok(temp_dir)
    }
}

#[cfg(test)]
mod mcp_server_tests {
    use super::*;

    #[tokio::test]
    async fn test_mcp_server_initialization() -> Result<(), Box<dyn std::error::Error>> {
        let server = test_utils::create_test_server()
            .await?;

        // Test that server info is available
        let server_info = server.get_info();
        assert_eq!(server_info.server_info.name, "MCP Context Browser");
        assert!(server_info.capabilities.tools.is_some());

        // Test that server info contains tools capability
        let server_info = server.get_info();
        assert!(
            server_info.capabilities.tools.is_some(),
            "Server should support tools"
        );

        println!("✅ MCP server initialization test passed");
        Ok(())
    }

    #[tokio::test]
    async fn test_mcp_stdio_integration() -> Result<(), Box<dyn std::error::Error>> {
        // Test the full MCP protocol integration via stdio transport
        test_utils::run_stdio_integration_test()
            .await?;

        println!("✅ MCP stdio integration test passed");
        Ok(())
    }
}

#[cfg(test)]
mod stdio_transport_tests {
    use super::*;
    use std::process::Stdio;
    use tokio::time::{timeout, Duration};

    #[tokio::test]
    async fn test_stdio_transport_basic() -> Result<(), Box<dyn std::error::Error>> {
        // This test requires the binary to be built
        // We'll skip if binary doesn't exist
        let binary_path = std::env::current_exe()?
            .parent()
            .ok_or("Failed to get parent dir")?
            .join("../../../target/debug/mcp-context-browser");

        if !binary_path.exists() {
            println!("⚠️  Binary not found, skipping stdio transport test");
            return Ok(());
        }

        // Start the MCP server process
        let mut child = Command::new(&binary_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        // Get handles to stdin/stdout
        let stdin = child.stdin.take().ok_or("Failed to get stdin")?;
        let stdout = child.stdout.take().ok_or("Failed to get stdout")?;

        // Initialize MCP protocol - using newline-delimited JSON (NDJSON) format
        let init_message = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test-client","version":"1.0.0"}}}"#;

        let mut stdin_writer = tokio::io::BufWriter::new(stdin);
        let mut stdout_reader = BufReader::new(stdout).lines();

        // Send initialize request as NDJSON (JSON followed by newline)
        stdin_writer
            .write_all(format!("{}\n", init_message).as_bytes())
            .await?;
        stdin_writer.flush().await?;

        // Read response with timeout (NDJSON)
        let response = timeout(Duration::from_secs(15), stdout_reader.next_line())
            .await??
            .ok_or("Empty response")?;

        // Should get a valid JSON-RPC response
        println!("DEBUG: Stdio test response: {}", response);
        assert!(
            response.contains("jsonrpc"),
            "Response should contain jsonrpc: {}",
            response
        );
        assert!(
            response.contains("result") || response.contains("error"),
            "Response should contain result or error: {}",
            response
        );

        // Clean up
        let _ = child.kill().await;

        println!("✅ MCP stdio transport test passed");
        Ok(())
    }
}
