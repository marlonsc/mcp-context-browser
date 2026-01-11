//! HTTP Transport for MCP
//!
//! Implements Streamable HTTP transport per MCP specification:
//! - POST /mcp: Client-to-server messages
//! - GET /mcp: Server-to-client SSE stream
//! - DELETE /mcp: Session termination

use axum::{
    extract::State,
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Json, Router,
};
use serde::Serialize;
use std::sync::Arc;
use tracing::{debug, info};

use super::config::TransportConfig;
use super::session::{SessionManager, SessionState};
use super::versioning::{headers, CompatibilityResult, VersionChecker};
use crate::infrastructure::connection_tracker::ConnectionTracker;

/// HTTP transport state shared across handlers
#[derive(Clone)]
pub struct HttpTransportState {
    pub session_manager: Arc<SessionManager>,
    pub version_checker: Arc<VersionChecker>,
    pub connection_tracker: Arc<ConnectionTracker>,
    pub config: TransportConfig,
}

/// Create the MCP HTTP transport router
pub fn create_mcp_router(state: HttpTransportState) -> Router {
    Router::new()
        // MCP message endpoint
        .route("/mcp", post(handle_mcp_post))
        .route("/mcp", get(handle_mcp_get))
        .route("/mcp", delete(handle_mcp_delete))
        // Version and health endpoints
        .route("/mcp/version", get(handle_version_info))
        .route("/mcp/health", get(handle_transport_health))
        .with_state(state)
}

/// Handle POST requests (client-to-server messages)
async fn handle_mcp_post(
    State(state): State<HttpTransportState>,
    headers: HeaderMap,
    Json(request): Json<serde_json::Value>,
) -> Result<Response, McpError> {
    // Track the request
    let _guard = state
        .connection_tracker
        .request_start()
        .ok_or(McpError::ServerDraining)?;

    // Check version compatibility
    check_version_compatibility(&state, &headers)?;

    // Get or create session
    let session_id = headers
        .get("Mcp-Session-Id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // Handle initialization (no session ID yet)
    if is_initialize_request(&request) {
        return handle_initialize(&state, request).await;
    }

    // Validate session for non-init requests
    let session_id = session_id.ok_or(McpError::MissingSessionId)?;

    let session = state
        .session_manager
        .get_session(&session_id)
        .ok_or(McpError::SessionNotFound)?;

    if session.state == SessionState::Terminated {
        return Err(McpError::SessionTerminated);
    }

    // Update session activity
    state.session_manager.touch_session(&session_id);

    // Process the request (placeholder - integrate with actual MCP server)
    let response = process_mcp_request(&request).await?;

    // Buffer response for resumption
    let mut resp = Json(&response).into_response();
    if let Some(event_id) = state
        .session_manager
        .buffer_message(&session_id, response.clone())
    {
        if let Ok(header_value) = HeaderValue::from_str(&event_id) {
            resp.headers_mut().insert("Mcp-Event-Id", header_value);
        }
    }

    // Add version headers
    add_version_headers(&state, resp.headers_mut());

    Ok(resp)
}

/// Handle initialization request
async fn handle_initialize(
    state: &HttpTransportState,
    request: serde_json::Value,
) -> Result<Response, McpError> {
    // Create new session
    let session = state
        .session_manager
        .create_session()
        .map_err(|e| McpError::SessionError(e.to_string()))?;

    // Extract and store client info
    if let Some(client_info) = request.get("params").and_then(|p| p.get("clientInfo")) {
        state
            .session_manager
            .set_client_info(&session.id, client_info.clone());
    }

    // Activate the session
    state
        .session_manager
        .activate_session(&session.id)
        .map_err(|e| McpError::SessionError(e.to_string()))?;

    // Process initialization (placeholder)
    let response = process_mcp_request(&request).await?;

    // Build response with session ID
    let mut resp = Json(&response).into_response();
    if let Ok(header_value) = HeaderValue::from_str(&session.id) {
        resp.headers_mut().insert("Mcp-Session-Id", header_value);
    }

    // Add version headers
    add_version_headers(state, resp.headers_mut());

    info!("Created new session: {}", session.id);
    Ok(resp)
}

/// Handle GET requests (SSE stream for server-to-client)
async fn handle_mcp_get(
    State(state): State<HttpTransportState>,
    headers: HeaderMap,
) -> Result<Response, McpError> {
    let session_id = headers
        .get("Mcp-Session-Id")
        .and_then(|v| v.to_str().ok())
        .ok_or(McpError::MissingSessionId)?;

    let _session = state
        .session_manager
        .get_session(session_id)
        .ok_or(McpError::SessionNotFound)?;

    // Check for resumption request
    let _last_event_id = headers.get("Last-Event-ID").and_then(|v| v.to_str().ok());

    // For now, return a placeholder response
    // Full SSE implementation would create a stream here
    Ok(Json(serde_json::json!({
        "status": "sse_not_implemented",
        "message": "SSE streaming is not yet implemented"
    }))
    .into_response())
}

/// Handle DELETE requests (session termination)
async fn handle_mcp_delete(
    State(state): State<HttpTransportState>,
    headers: HeaderMap,
) -> StatusCode {
    if let Some(session_id) = headers.get("Mcp-Session-Id").and_then(|v| v.to_str().ok()) {
        if state.session_manager.terminate_session(session_id) {
            info!("Terminated session: {}", session_id);
            StatusCode::OK
        } else {
            StatusCode::NOT_FOUND
        }
    } else {
        StatusCode::BAD_REQUEST
    }
}

/// Handle version info request
async fn handle_version_info(State(state): State<HttpTransportState>) -> Json<VersionInfoResponse> {
    Json(VersionInfoResponse {
        server: "mcp-context-browser".to_string(),
        version: state.version_checker.version_string(),
        protocol: state.version_checker.get_version_info(),
    })
}

/// Handle transport health check
async fn handle_transport_health(
    State(state): State<HttpTransportState>,
) -> Json<TransportHealthResponse> {
    Json(TransportHealthResponse {
        status: "healthy".to_string(),
        transport: "streamable-http".to_string(),
        active_sessions: state.session_manager.active_session_count(),
        total_sessions: state.session_manager.total_session_count(),
        active_requests: state.connection_tracker.active_count(),
        draining: state.connection_tracker.is_draining(),
    })
}

// Helper functions

fn is_initialize_request(request: &serde_json::Value) -> bool {
    request
        .get("method")
        .and_then(|m| m.as_str())
        .map(|m| m == "initialize")
        .unwrap_or(false)
}

fn check_version_compatibility(
    state: &HttpTransportState,
    headers: &HeaderMap,
) -> Result<CompatibilityResult, McpError> {
    if let Some(expected) = headers
        .get(headers::EXPECTED_SERVER_VERSION)
        .and_then(|v| v.to_str().ok())
    {
        let result = state.version_checker.check_compatibility(expected);
        if !result.allows_proceed() {
            return Err(McpError::VersionIncompatible(match &result {
                CompatibilityResult::Incompatible { message } => message.clone(),
                _ => "Version incompatible".to_string(),
            }));
        }
        Ok(result)
    } else {
        Ok(CompatibilityResult::Compatible)
    }
}

fn add_version_headers(state: &HttpTransportState, headers: &mut axum::http::HeaderMap) {
    if let Ok(v) = HeaderValue::from_str(&state.version_checker.version_string()) {
        headers.insert(headers::SERVER_VERSION, v);
    }
}

async fn process_mcp_request(request: &serde_json::Value) -> Result<serde_json::Value, McpError> {
    // Placeholder - this should integrate with the actual MCP server
    debug!("Processing MCP request: {:?}", request.get("method"));

    Ok(serde_json::json!({
        "jsonrpc": "2.0",
        "id": request.get("id"),
        "result": {
            "status": "processed",
            "note": "HTTP transport placeholder response"
        }
    }))
}

// Response types

#[derive(Serialize)]
struct VersionInfoResponse {
    server: String,
    version: String,
    protocol: super::versioning::VersionInfo,
}

#[derive(Serialize)]
struct TransportHealthResponse {
    status: String,
    transport: String,
    active_sessions: usize,
    total_sessions: usize,
    active_requests: usize,
    draining: bool,
}

// Error handling

#[derive(Debug)]
pub enum McpError {
    MissingSessionId,
    SessionNotFound,
    SessionTerminated,
    SessionError(String),
    ServerDraining,
    VersionIncompatible(String),
    ProcessingError(String),
}

impl IntoResponse for McpError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            McpError::MissingSessionId => {
                (StatusCode::BAD_REQUEST, "Missing Mcp-Session-Id header")
            }
            McpError::SessionNotFound => (StatusCode::NOT_FOUND, "Session not found or expired"),
            McpError::SessionTerminated => (StatusCode::GONE, "Session has been terminated"),
            McpError::SessionError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Session error"),
            McpError::ServerDraining => {
                (StatusCode::SERVICE_UNAVAILABLE, "Server is shutting down")
            }
            McpError::VersionIncompatible(_) => (StatusCode::BAD_REQUEST, "Version incompatible"),
            McpError::ProcessingError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Processing error"),
        };

        let body = serde_json::json!({
            "error": {
                "code": status.as_u16(),
                "message": message,
            }
        });

        (status, Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::connection_tracker::ConnectionTrackerConfig;

    fn create_test_state() -> HttpTransportState {
        HttpTransportState {
            session_manager: Arc::new(SessionManager::with_defaults()),
            version_checker: Arc::new(VersionChecker::with_defaults()),
            connection_tracker: Arc::new(
                ConnectionTracker::new(ConnectionTrackerConfig::default()),
            ),
            config: TransportConfig::default(),
        }
    }

    #[test]
    fn test_is_initialize_request() {
        let init = serde_json::json!({"method": "initialize"});
        assert!(is_initialize_request(&init));

        let other = serde_json::json!({"method": "tools/list"});
        assert!(!is_initialize_request(&other));
    }

    #[tokio::test]
    async fn test_version_info_handler() {
        let state = create_test_state();
        let response = handle_version_info(State(state)).await;

        assert_eq!(response.0.server, "mcp-context-browser");
        assert!(!response.0.version.is_empty());
    }

    #[tokio::test]
    async fn test_health_handler() {
        let state = create_test_state();
        let response = handle_transport_health(State(state)).await;

        assert_eq!(response.0.status, "healthy");
        assert_eq!(response.0.transport, "streamable-http");
        assert!(!response.0.draining);
    }
}
