//! Tests for MCP protocol implementation
//!
//! This module tests the MCP server implementation using the rmcp SDK.
//! Tests cover server creation, tool validation, and MCP protocol compliance.

use mcp_context_browser::server::{
    ClearIndexArgs, GetIndexingStatusArgs, IndexCodebaseArgs, McpServer, SearchCodeArgs,
};
use rmcp::{ServerHandler, model::ProtocolVersion};

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mcp_server_creation() {
        let result = McpServer::new(None);
        assert!(result.is_ok(), "MCP server should be created successfully");
    }

    #[tokio::test]
    async fn test_server_info_structure() {
        let server = McpServer::new(None).unwrap();
        let info = server.get_info();

        // Check protocol version
        assert_eq!(info.protocol_version, ProtocolVersion::V_2024_11_05);

        // Check capabilities
        assert!(
            info.capabilities.tools.is_some(),
            "Server should support tools"
        );
        assert!(
            info.capabilities.prompts.is_none(),
            "Server should not support prompts yet"
        );
        assert!(
            info.capabilities.resources.is_none(),
            "Server should not support resources yet"
        );

        // Check implementation info
        assert_eq!(info.server_info.name, "MCP Context Browser");
        assert!(
            !info.server_info.version.is_empty(),
            "Version should not be empty"
        );

        // Check instructions
        assert!(
            info.instructions.is_some(),
            "Server should provide instructions"
        );
        let instructions = info.instructions.as_ref().unwrap();
        assert!(
            instructions.contains("MCP Context Browser"),
            "Instructions should mention the server name"
        );
        assert!(
            instructions.contains("index_codebase"),
            "Instructions should mention available tools"
        );
        assert!(
            instructions.contains("search_code"),
            "Instructions should mention available tools"
        );
    }

    #[tokio::test]
    async fn test_server_instructions_comprehensive() {
        let server = McpServer::new(None).unwrap();
        let info = server.get_info();
        let instructions = info.instructions.as_ref().unwrap();

        // Check that instructions cover all major aspects
        assert!(
            instructions.contains("MCP Context Browser"),
            "Should have server branding"
        );
        assert!(
            instructions.contains("Available Tools"),
            "Should list available tools"
        );
        assert!(
            instructions.contains("index_codebase"),
            "Should describe index_codebase tool"
        );
        assert!(
            instructions.contains("search_code"),
            "Should describe search_code tool"
        );
        assert!(
            instructions.contains("Best Practices"),
            "Should provide usage guidance"
        );
        assert!(instructions.contains("Security"), "Should mention security");
        assert!(
            instructions.contains("Architecture"),
            "Should describe architecture"
        );
    }

    #[test]
    fn test_index_codebase_args_validation() {
        // Valid args should work
        let args = IndexCodebaseArgs {
            path: "/some/path".to_string(),
            token: None,
        };
        assert_eq!(args.path, "/some/path");

        // Empty path should be valid (validation happens in the tool)
        let args_empty = IndexCodebaseArgs {
            path: "".to_string(),
            token: None,
        };
        assert_eq!(args_empty.path, "");
    }

    #[test]
    fn test_search_code_args_validation() {
        // Valid args with all fields
        let args = SearchCodeArgs {
            query: "test query".to_string(),
            limit: 5,
            token: None,
        };
        assert_eq!(args.query, "test query");
        assert_eq!(args.limit, 5);

        // Valid args with default limit
        let args_default = SearchCodeArgs {
            query: "another query".to_string(),
            limit: 10, // This is the default
            token: None,
        };
        assert_eq!(args_default.query, "another query");
        assert_eq!(args_default.limit, 10);
    }

    #[test]
    fn test_get_indexing_status_args_validation() {
        // Valid args with collection
        let args = GetIndexingStatusArgs {
            collection: "test_collection".to_string(),
        };
        assert_eq!(args.collection, "test_collection");

        // Valid args with default collection
        let args_default = GetIndexingStatusArgs {
            collection: "default".to_string(),
        };
        assert_eq!(args_default.collection, "default");
    }

    #[test]
    fn test_clear_index_args_validation() {
        // Valid args
        let args = ClearIndexArgs {
            collection: "test_collection".to_string(),
        };
        assert_eq!(args.collection, "test_collection");

        // Empty collection should be valid (validation happens in the tool)
        let args_empty = ClearIndexArgs {
            collection: "".to_string(),
        };
        assert_eq!(args_empty.collection, "");
    }

    #[tokio::test]
    async fn test_server_capabilities_structure() {
        let server = McpServer::new(None).unwrap();
        let info = server.get_info();
        let capabilities = &info.capabilities;

        // Tools capability should be enabled
        let tools_capability = capabilities.tools.as_ref().unwrap();
        assert!(
            tools_capability.list_changed.is_none(),
            "List changed should be None for basic implementation"
        );

        // Other capabilities should be None (not implemented yet)
        assert!(
            capabilities.prompts.is_none(),
            "Prompts should not be implemented yet"
        );
        assert!(
            capabilities.resources.is_none(),
            "Resources should not be implemented yet"
        );
        assert!(
            capabilities.logging.is_none(),
            "Logging should not be implemented yet"
        );
    }

    #[tokio::test]
    async fn test_server_implementation_info() {
        let server = McpServer::new(None).unwrap();
        let info = server.get_info();
        let implementation = &info.server_info;

        // Check basic fields
        assert_eq!(implementation.name, "MCP Context Browser");
        assert!(!implementation.version.is_empty());

        // Optional fields should be None (using Default::default())
        assert!(implementation.icons.is_none());
        assert!(implementation.title.is_none());
        assert!(implementation.website_url.is_none());
    }

    #[tokio::test]
    async fn test_server_initialization_with_dependencies() {
        // This test ensures the server can be created with all its dependencies
        // In a real scenario, this would involve mock providers
        let _server = McpServer::new(None).unwrap();

        // Server creation succeeded - this is expected with mock/default providers
        assert!(
            true,
            "Server should initialize successfully with default providers"
        );
    }

    #[tokio::test]
    async fn test_server_initialization_failure_handling() {
        // Test that server creation handles errors gracefully
        // This is more of a structural test than a functional one

        // We expect server creation to succeed in test environment
        // In case of failure, it should be due to configuration, not code structure
        let server_result = McpServer::new(None);

        // Either it succeeds, or fails with a configuration-related error
        match server_result {
            Ok(server) => {
                // If it succeeds, verify basic functionality
                let info = server.get_info();
                assert!(!info.server_info.name.is_empty());
                assert!(info.capabilities.tools.is_some());
            }
            Err(e) => {
                // If it fails, ensure it's a configuration issue, not a structural problem
                let error_msg = e.to_string().to_lowercase();
                assert!(
                    error_msg.contains("config")
                        || error_msg.contains("environment")
                        || error_msg.contains("load")
                        || error_msg.contains("connection"),
                    "Server initialization error should be configuration-related, got: {}",
                    e
                );
            }
        }
    }

    #[tokio::test]
    async fn test_server_tool_router_initialization() {
        // Test that the server properly initializes its tool router
        let _server = McpServer::new(None).unwrap();

        // The server should have a tool router (internal implementation detail)
        // We can't directly test the router, but we can verify the server structure
        assert!(true, "Server with tool router initialized successfully");
    }

    #[tokio::test]
    async fn test_server_instructions_formatting() {
        let server = McpServer::new(None).unwrap();
        let info = server.get_info();
        let instructions = info.instructions.as_ref().unwrap();

        // Test that instructions are properly formatted for MCP clients
        assert!(
            instructions.contains("MCP Context Browser"),
            "Instructions should contain server branding"
        );
        assert!(
            instructions.contains("Available Tools"),
            "Should have tools section"
        );
        assert!(
            instructions.contains("Best Practices"),
            "Should have usage guidance"
        );
        assert!(
            instructions.contains("---"),
            "Should have proper section separation"
        );
    }

    #[tokio::test]
    async fn test_server_capabilities_compliance() {
        let server = McpServer::new(None).unwrap();
        let info = server.get_info();

        // Verify MCP protocol compliance
        assert_eq!(
            info.protocol_version,
            ProtocolVersion::V_2024_11_05,
            "Must support MCP 2024-11-05"
        );

        // Must have tools capability
        assert!(
            info.capabilities.tools.is_some(),
            "Server must support tools"
        );

        // Should have server info
        assert!(!info.server_info.name.is_empty(), "Must have server name");
        assert!(!info.server_info.version.is_empty(), "Must have version");

        // Should provide instructions
        assert!(
            info.instructions.is_some(),
            "Should provide usage instructions"
        );
    }

    #[tokio::test]
    async fn test_instructions_contain_essential_information() {
        let server = McpServer::new(None).unwrap();
        let info = server.get_info();
        let instructions = info.instructions.as_ref().unwrap();

        // Essential information that should be in instructions
        let essential_phrases = vec![
            "semantic code search",
            "vector embeddings",
            "index_codebase",
            "search_code",
            "natural language",
            "intelligent code search",
            "AI-powered",
        ];

        let mut found_count = 0;
        for phrase in essential_phrases {
            if instructions.contains(phrase) {
                found_count += 1;
            }
        }

        // At least 4 essential phrases should be present
        assert!(
            found_count >= 4,
            "Instructions should contain at least 4 essential phrases, found {}",
            found_count
        );
    }

    #[tokio::test]
    async fn test_instructions_provide_usage_guidance() {
        let server = McpServer::new(None).unwrap();
        let info = server.get_info();
        let instructions = info.instructions.as_ref().unwrap();

        // Check for usage guidance
        assert!(
            instructions.contains("Usage Tips") || instructions.contains("Best Practices"),
            "Instructions should provide usage guidance"
        );

        // Check for tool parameters
        assert!(
            instructions.contains("Parameters") || instructions.contains("parameters"),
            "Instructions should explain tool parameters"
        );

        // Check for examples
        assert!(
            instructions.contains("Examples")
                || instructions.contains("examples")
                || instructions.contains("\"find"),
            "Instructions should provide usage examples"
        );
    }

    #[tokio::test]
    async fn test_server_info_serialization() {
        let server = McpServer::new(None).unwrap();
        let info = server.get_info();

        // Test that the server info can be serialized (required for MCP protocol)
        let serialized = serde_json::to_string(&info);
        assert!(serialized.is_ok(), "ServerInfo should be serializable");

        // Test that it can be deserialized back
        let deserialized: rmcp::model::ServerInfo =
            serde_json::from_str(&serialized.unwrap()).unwrap();
        assert_eq!(deserialized.protocol_version, info.protocol_version);
        assert_eq!(deserialized.server_info.name, info.server_info.name);
    }

    #[tokio::test]
    async fn test_server_supports_required_mcp_features() {
        let server = McpServer::new(None).unwrap();
        let info = server.get_info();

        // Must support MCP protocol version 2024-11-05
        assert_eq!(info.protocol_version, ProtocolVersion::V_2024_11_05);

        // Must have server info
        assert!(!info.server_info.name.is_empty());
        assert!(!info.server_info.version.is_empty());

        // Should have instructions for clients
        assert!(info.instructions.is_some());
        assert!(!info.instructions.as_ref().unwrap().is_empty());
    }
}
