//! Tests for MCP protocol implementation
//!
//! This module tests the MCP protocol handlers and message processing.

use mcp_context_browser::server::{McpToolHandlers, CallToolResponse, CallToolResultContent, Tool};
use serde_json::json;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_tools_returns_expected_tools() {
        let tools = McpToolHandlers::get_tools();

        assert_eq!(tools.len(), 2);

        // Check index_codebase tool
        let index_tool = &tools[0];
        assert_eq!(index_tool.name, "index_codebase");
        assert_eq!(index_tool.description, "Index a codebase directory for semantic search");
        assert!(index_tool.input_schema.is_object());

        let schema = index_tool.input_schema.as_object().unwrap();
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"].is_object());
        assert!(schema["required"].is_array());

        let required = schema["required"].as_array().unwrap();
        assert_eq!(required.len(), 1);
        assert_eq!(required[0], "path");

        // Check search_code tool
        let search_tool = &tools[1];
        assert_eq!(search_tool.name, "search_code");
        assert_eq!(search_tool.description, "Search for code using natural language queries");
        assert!(search_tool.input_schema.is_object());

        let search_schema = search_tool.input_schema.as_object().unwrap();
        assert_eq!(search_schema["type"], "object");
        assert!(search_schema["properties"].is_object());
        assert!(search_schema["required"].is_array());

        let search_required = search_schema["required"].as_array().unwrap();
        assert_eq!(search_required.len(), 1);
        assert_eq!(search_required[0], "query");
    }

    #[test]
    fn test_mcp_tool_handlers_creation() {
        let result = McpToolHandlers::new();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_index_codebase_success() {
        let handlers = McpToolHandlers::new().unwrap();

        let temp_dir = tempfile::tempdir().unwrap();
        let args = json!({
            "path": temp_dir.path().to_str().unwrap()
        });

        let result = handlers.handle_tool_call("index_codebase", args).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(!response.is_error);

        let CallToolResultContent::Text { text } = &response.content[0];
        assert!(text.contains("Successfully indexed"));
        assert!(text.contains("code chunks"));
    }

    #[tokio::test]
    async fn test_handle_index_codebase_missing_path() {
        let handlers = McpToolHandlers::new().unwrap();

        let args = json!({}); // Missing path

        let result = handlers.handle_tool_call("index_codebase", args).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Missing path argument"));
    }

    #[tokio::test]
    async fn test_handle_search_code_success() {
        let handlers = McpToolHandlers::new().unwrap();

        let args = json!({
            "query": "test search",
            "limit": 5
        });

        let result = handlers.handle_tool_call("search_code", args).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(!response.is_error);

        let CallToolResultContent::Text { text } = &response.content[0];
        assert!(text.contains("Search Results for"));
        assert!(text.contains("\"test search\""));
    }

    #[tokio::test]
    async fn test_handle_search_code_missing_query() {
        let handlers = McpToolHandlers::new().unwrap();

        let args = json!({}); // Missing query

        let result = handlers.handle_tool_call("search_code", args).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Missing query argument"));
    }

    #[tokio::test]
    async fn test_handle_search_code_with_limit() {
        let handlers = McpToolHandlers::new().unwrap();

        let args = json!({
            "query": "test query",
            "limit": 3
        });

        let result = handlers.handle_tool_call("search_code", args).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(!response.is_error);
    }

    #[tokio::test]
    async fn test_handle_unknown_tool() {
        let handlers = McpToolHandlers::new().unwrap();

        let args = json!({});
        let result = handlers.handle_tool_call("unknown_tool", args).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Unknown tool"));
    }

    #[tokio::test]
    async fn test_handle_search_code_no_results() {
        let handlers = McpToolHandlers::new().unwrap();

        let args = json!({
            "query": "nonexistent query that should return no results",
            "limit": 10
        });

        let result = handlers.handle_tool_call("search_code", args).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(!response.is_error);

        let CallToolResultContent::Text { text } = &response.content[0];
        assert!(text.contains("No relevant results found"));
    }

    #[tokio::test]
    async fn test_handle_index_codebase_nonexistent_path() {
        let handlers = McpToolHandlers::new().unwrap();

        let args = json!({
            "path": "/non/existent/path/that/does/not/exist"
        });

        let result = handlers.handle_tool_call("index_codebase", args).await;

        assert!(result.is_ok()); // MVP implementation doesn't fail on nonexistent paths
        let response = result.unwrap();
        assert!(!response.is_error);
    }

    #[tokio::test]
    async fn test_handle_search_code_default_limit() {
        let handlers = McpToolHandlers::new().unwrap();

        let args = json!({
            "query": "test query"
            // No limit specified, should default to 10
        });

        let result = handlers.handle_tool_call("search_code", args).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(!response.is_error);
    }

    #[tokio::test]
    async fn test_handle_search_code_zero_limit() {
        let handlers = McpToolHandlers::new().unwrap();

        let args = json!({
            "query": "test query",
            "limit": 0
        });

        let result = handlers.handle_tool_call("search_code", args).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(!response.is_error);
    }

    #[test]
    fn test_call_tool_response_text_content() {
        let response = CallToolResponse {
            content: vec![CallToolResultContent::Text {
                text: "Test message".to_string(),
            }],
            is_error: false,
        };

        assert_eq!(response.content.len(), 1);
        assert!(!response.is_error);

        let CallToolResultContent::Text { text } = &response.content[0];
        assert_eq!(text, "Test message");
    }

    #[test]
    fn test_call_tool_response_error_content() {
        let response = CallToolResponse {
            content: vec![CallToolResultContent::Text {
                text: "Error occurred".to_string(),
            }],
            is_error: true,
        };

        assert_eq!(response.content.len(), 1);
        assert!(response.is_error);
    }

    #[test]
    fn test_tool_definition_serialization() {
        let tool = Tool {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "param": {"type": "string"}
                }
            }),
        };

        let serialized = serde_json::to_string(&tool).unwrap();
        let deserialized: Tool = serde_json::from_str(&serialized).unwrap();

        assert_eq!(tool.name, deserialized.name);
        assert_eq!(tool.description, deserialized.description);
        assert_eq!(tool.input_schema, deserialized.input_schema);
    }
}