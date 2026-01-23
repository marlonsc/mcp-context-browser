//! HTTP Client Transport
//!
//! MCP client that connects to a remote MCB server via HTTP.
//! Reads MCP requests from stdin, forwards them to the server,
//! and writes responses to stdout.
//!
//! This enables MCB to run in "client mode" where it acts as a
//! stdio-to-HTTP bridge for Claude Code integration.

use std::io::{self, BufRead, Write};
use std::time::Duration;

use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::types::{McpRequest, McpResponse};

/// HTTP client transport configuration
#[derive(Debug, Clone)]
pub struct HttpClientConfig {
    /// Server URL (e.g., "http://127.0.0.1:8080")
    pub server_url: String,

    /// Session ID for this client connection
    pub session_id: String,

    /// Request timeout
    pub timeout: Duration,
}

/// HTTP client transport
///
/// Bridges stdio (for Claude Code) to HTTP (for MCB server).
/// Each request is forwarded to the server with a session ID header.
pub struct HttpClientTransport {
    config: HttpClientConfig,
    client: reqwest::Client,
}

impl HttpClientTransport {
    /// Create a new HTTP client transport
    ///
    /// # Arguments
    ///
    /// * `server_url` - URL of the MCB server (e.g., "http://127.0.0.1:8080")
    /// * `session_prefix` - Optional prefix for session ID generation
    /// * `timeout` - Request timeout duration
    pub fn new(server_url: String, session_prefix: Option<String>, timeout: Duration) -> Self {
        let session_id = match session_prefix {
            Some(prefix) => format!("{}_{}", prefix, Uuid::new_v4()),
            None => Uuid::new_v4().to_string(),
        };

        let config = HttpClientConfig {
            server_url,
            session_id,
            timeout,
        };

        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .expect("Failed to create HTTP client");

        Self { config, client }
    }

    /// Run the client transport
    ///
    /// Main loop that:
    /// 1. Reads JSON-RPC requests from stdin
    /// 2. Forwards them to the MCB server via HTTP
    /// 3. Writes responses to stdout
    ///
    /// Runs until stdin is closed (EOF).
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!(
            server_url = %self.config.server_url,
            session_id = %self.config.session_id,
            "MCB client transport started"
        );

        let stdin = io::stdin();
        let mut stdout = io::stdout();

        for line in stdin.lock().lines() {
            let line = match line {
                Ok(l) => l,
                Err(e) => {
                    if e.kind() == io::ErrorKind::UnexpectedEof {
                        info!("stdin closed, shutting down");
                        break;
                    }
                    error!(error = %e, "Error reading from stdin");
                    continue;
                }
            };

            // Skip empty lines
            if line.trim().is_empty() {
                continue;
            }

            debug!(request = %line, "Received request from stdin");

            // Parse the request
            let request: McpRequest = match serde_json::from_str(&line) {
                Ok(req) => req,
                Err(e) => {
                    warn!(error = %e, line = %line, "Failed to parse request");
                    let error_response = McpResponse {
                        jsonrpc: "2.0".to_string(),
                        result: None,
                        error: Some(super::types::McpError {
                            code: -32700,
                            message: format!("Parse error: {}", e),
                        }),
                        id: None,
                    };
                    let response_json = serde_json::to_string(&error_response)?;
                    writeln!(stdout, "{}", response_json)?;
                    stdout.flush()?;
                    continue;
                }
            };

            // Forward to server
            let response = match self.send_request(&request).await {
                Ok(resp) => resp,
                Err(e) => {
                    error!(error = %e, "Failed to send request to server");
                    McpResponse {
                        jsonrpc: "2.0".to_string(),
                        result: None,
                        error: Some(super::types::McpError {
                            code: -32603,
                            message: format!("Server communication error: {}", e),
                        }),
                        id: request.id,
                    }
                }
            };

            // Write response to stdout
            let response_json = serde_json::to_string(&response)?;
            debug!(response = %response_json, "Sending response to stdout");
            writeln!(stdout, "{}", response_json)?;
            stdout.flush()?;
        }

        info!("MCB client transport finished");
        Ok(())
    }

    /// Send a request to the MCB server
    async fn send_request(&self, request: &McpRequest) -> Result<McpResponse, reqwest::Error> {
        let url = format!("{}/mcp", self.config.server_url);

        debug!(
            url = %url,
            method = %request.method,
            session_id = %self.config.session_id,
            "Sending request to server"
        );

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("X-Session-Id", &self.config.session_id)
            .json(request)
            .send()
            .await?;

        let status = response.status();
        debug!(status = %status, "Received response from server");

        if !status.is_success() {
            warn!(status = %status, "Server returned non-success status");
        }

        response.json::<McpResponse>().await
    }

    /// Get the session ID for this client
    pub fn session_id(&self) -> &str {
        &self.config.session_id
    }

    /// Get the server URL
    pub fn server_url(&self) -> &str {
        &self.config.server_url
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_id_generation() {
        let client = HttpClientTransport::new(
            "http://localhost:8080".to_string(),
            Some("test".to_string()),
            Duration::from_secs(30),
        );
        assert!(client.session_id().starts_with("test_"));
    }

    #[test]
    fn test_session_id_without_prefix() {
        let client = HttpClientTransport::new(
            "http://localhost:8080".to_string(),
            None,
            Duration::from_secs(30),
        );
        // Should be a valid UUID
        assert!(Uuid::parse_str(client.session_id()).is_ok());
    }
}
