//! Handler for the search_code MCP tool
//!
//! This handler is responsible for performing semantic code search.
//! It validates queries, checks permissions, manages caching, and coordinates
//! the search process with proper error handling and timeouts.

use std::sync::Arc;
use std::time::Instant;
use rmcp::model::{CallToolResult, Content};
use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use serde_json;

use crate::core::auth::Permission;
use crate::core::cache::{CacheManager, CacheResult};
use crate::core::limits::ResourceLimits;
use crate::server::args::SearchCodeArgs;
use crate::server::auth::AuthHandler;
use crate::server::formatter::ResponseFormatter;
use crate::services::SearchService;

/// Handler for code search operations
pub struct SearchCodeHandler {
    search_service: Arc<SearchService>,
    auth_handler: Arc<AuthHandler>,
    resource_limits: Arc<ResourceLimits>,
    cache_manager: Arc<CacheManager>,
}

impl SearchCodeHandler {
    /// Create a new search_code handler
    pub fn new(
        search_service: Arc<SearchService>,
        auth_handler: Arc<AuthHandler>,
        resource_limits: Arc<ResourceLimits>,
        cache_manager: Arc<CacheManager>,
    ) -> Self {
        Self {
            search_service,
            auth_handler,
            resource_limits,
            cache_manager,
        }
    }

    /// Handle the search_code tool request
    pub async fn handle(
        &self,
        Parameters(SearchCodeArgs {
            query,
            limit,
            token,
        }): Parameters<SearchCodeArgs>,
    ) -> Result<CallToolResult, McpError> {
        let start_time = Instant::now();

        // Check authentication and permissions
        if let Err(e) = self.auth_handler.check_auth(token.as_ref(), &Permission::SearchCodebase) {
            return Ok(ResponseFormatter::format_auth_error(&e.to_string()));
        }

        // Check resource limits for search operation
        if let Err(e) = self.resource_limits.check_operation_allowed("search").await {
            return Ok(ResponseFormatter::format_resource_limit_error(&e.to_string()));
        }

        // Acquire search permit
        let _permit = match self
            .resource_limits
            .acquire_operation_permit("search")
            .await
        {
            Ok(permit) => permit,
            Err(e) => {
                return Ok(ResponseFormatter::format_resource_limit_error(&e.to_string()));
            }
        };

        // Validate query input
        let query = query.trim();
        if query.is_empty() {
            return Ok(ResponseFormatter::format_query_validation_error(
                "Search query cannot be empty. Please provide a natural language query."
            ));
        }

        if query.len() < 3 {
            return Ok(ResponseFormatter::format_query_validation_error(
                "Search query too short. Please use at least 3 characters for meaningful results."
            ));
        }

        // Validate limit
        let limit = limit.clamp(1, 50); // Reasonable bounds for performance
        let collection = "default";

        // Check cache for search results
        let cache_key = format!("{}:{}:{}", collection, query, limit);
        let cached_result: CacheResult<serde_json::Value> =
            self.cache_manager.get("search_results", &cache_key).await;

        if let CacheResult::Hit(cached_data) = cached_result {
            if let Ok(search_results) =
                serde_json::from_value::<Vec<crate::core::types::SearchResult>>(cached_data)
            {
                tracing::info!(
                    "✅ Search cache hit for query: '{}' (limit: {})",
                    query,
                    limit
                );
                return ResponseFormatter::format_search_response(
                    query,
                    &search_results,
                    start_time.elapsed(),
                    true,
                );
            }
        }

        tracing::info!(
            "Performing semantic search for query: '{}' (limit: {})",
            query,
            limit
        );

        // Add timeout for search operations
        let search_future = self.search_service.search(collection, query, limit);
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(30), // 30 second timeout
            search_future,
        )
        .await;

        let duration = start_time.elapsed();

        match result {
            Ok(Ok(results)) => {
                // Cache search results
                let _ = self
                    .cache_manager
                    .set(
                        "search_results",
                        &cache_key,
                        serde_json::to_value(&results).unwrap_or_default(),
                    )
                    .await;

                // Use the simplified response formatting that was moved to the search method
                Self::format_search_response_with_cache(query, &results, duration)
            }
            Ok(Err(e)) => {
                Ok(ResponseFormatter::format_search_error(&e.to_string(), query))
            }
            Err(_) => {
                Ok(ResponseFormatter::format_search_timeout(query))
            }
        }
    }

    /// Format search response (extracted from the original implementation)
    fn format_search_response_with_cache(
        query: &str,
        results: &[crate::core::types::SearchResult],
        duration: std::time::Duration,
    ) -> Result<CallToolResult, McpError> {
        let mut message = "🔍 **Semantic Code Search Results**\n\n".to_string();
        message.push_str(&format!("**Query:** \"{}\" \n", query));
        message.push_str(&format!(
            "**Search completed in:** {:.2}s\n",
            duration.as_secs_f64()
        ));
        message.push_str(&format!("**Results found:** {}\n\n", results.len()));

        if results.is_empty() {
            message.push_str("❌ **No Results Found**\n\n");
            message.push_str("**Possible Reasons:**\n");
            message.push_str("• Codebase not indexed yet (run `index_codebase` first)\n");
            message.push_str("• Query terms not present in the codebase\n");
            message.push_str("• Try different keywords or more general terms\n\n");
            message.push_str("**💡 Search Tips:**\n");
            message.push_str("• Use natural language: \"find error handling\", \"authentication logic\"\n");
            message.push_str("• Be specific: \"HTTP request middleware\" > \"middleware\"\n");
            message.push_str("• Include technologies: \"React component state management\"\n");
            message.push_str("• Try synonyms: \"validate\" instead of \"check\"\n");
        } else {
            message.push_str("📊 **Search Results:**\n\n");

            for (i, result) in results.iter().enumerate() {
                message.push_str(&format!(
                    "**{}.** 📁 `{}` (line {})\n",
                    i + 1,
                    result.file_path,
                    result.line_number
                ));

                // Add context lines around the match for better understanding
                let lines: Vec<&str> = result.content.lines().collect();
                let preview_lines = if lines.len() > 10 {
                    lines
                        .iter()
                        .take(10)
                        .cloned()
                        .collect::<Vec<_>>()
                        .join("\n")
                } else {
                    result.content.clone()
                };

                // Detect language for syntax highlighting
                let file_ext = std::path::Path::new(&result.file_path)
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("");

                let lang_hint = match file_ext {
                    "rs" => "rust",
                    "py" => "python",
                    "js" => "javascript",
                    "ts" => "typescript",
                    "go" => "go",
                    "java" => "java",
                    "cpp" | "cc" | "cxx" => "cpp",
                    "c" => "c",
                    "cs" => "csharp",
                    _ => "",
                };

                if lang_hint.is_empty() {
                    message.push_str(&format!("```\n{}\n```\n", preview_lines));
                } else {
                    message.push_str(&format!("``` {}\n{}\n```\n", lang_hint, preview_lines));
                }

                message.push_str(&format!("🎯 **Relevance Score:** {:.3}\n\n", result.score));
            }

            // Add pagination hint if we hit the limit
            if results.len() == 10 {
                message.push_str(&format!(
                    "💡 **Showing top {} results.** For more results, try:\n",
                    10
                ));
                message.push_str("• More specific search terms\n");
                message.push_str("• Different query formulations\n");
                message.push_str("• Breaking complex queries into simpler ones\n");
            }

            // Performance insights
            if duration.as_millis() > 1000 {
                message.push_str(&format!(
                    "\n⚠️ **Performance Note:** Search took {:.2}s. \
                    Consider using more specific queries for faster results.\n",
                    duration.as_secs_f64()
                ));
            }
        }

        tracing::info!(
            "Search completed: found {} results in {:?}",
            results.len(),
            duration
        );
        Ok(CallToolResult::success(vec![Content::text(message)]))
    }
}