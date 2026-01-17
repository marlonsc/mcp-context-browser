//! Server-specific constants
//!
//! Contains constants specific to the MCP server implementation,
//! including JSON-RPC error codes and protocol-related values.

// ============================================================================
// JSON-RPC ERROR CODES (Standard)
// ============================================================================

/// JSON-RPC Method not found error code
pub const JSONRPC_METHOD_NOT_FOUND: i32 = -32601;

/// JSON-RPC Parse error code
pub const JSONRPC_PARSE_ERROR: i32 = -32700;

/// JSON-RPC Invalid request error code
pub const JSONRPC_INVALID_REQUEST: i32 = -32600;

/// JSON-RPC Invalid params error code
pub const JSONRPC_INVALID_PARAMS: i32 = -32602;

/// JSON-RPC Internal error code
pub const JSONRPC_INTERNAL_ERROR: i32 = -32603;
