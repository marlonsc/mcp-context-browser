//! Response formatting utilities for MCP server
//!
//! This module contains utilities for formatting tool responses in a consistent,
//! user-friendly way. It handles the presentation of search results, indexing status,
//! and error messages.

use rmcp::model::{CallToolResult, Content};
use rmcp::ErrorData as McpError;
use std::path::Path;

/// Response formatter for MCP server tools
pub struct ResponseFormatter;

impl ResponseFormatter {
    /// Format search response for display
    pub fn format_search_response(
        query: &str,
        results: &[crate::domain::types::SearchResult],
        duration: std::time::Duration,
        from_cache: bool,
    ) -> Result<CallToolResult, McpError> {
        let mut message = "🔍 **Semantic Code Search Results**\n\n".to_string();
        message.push_str(&format!("**Query:** \"{}\" \n", query));
        message.push_str(&format!(
            "**Search completed in:** {:.2}s",
            duration.as_secs_f64()
        ));
        if from_cache {
            message.push_str(" (from cache)");
        }
        message.push_str(&format!("\n**Results found:** {}\n\n", results.len()));

        if results.is_empty() {
            Self::format_empty_search_response(&mut message);
        } else {
            Self::format_search_results(&mut message, results, duration);
        }

        tracing::info!(
            "Search completed: found {} results in {:?}",
            results.len(),
            duration
        );
        Ok(CallToolResult::success(vec![Content::text(message)]))
    }

    /// Format response when no search results are found
    fn format_empty_search_response(message: &mut String) {
        message.push_str("❌ **No Results Found**\n\n");
        message.push_str("**Possible Reasons:**\n");
        message.push_str("• Codebase not indexed yet (run `index_codebase` first)\n");
        message.push_str("• Query terms not present in the codebase\n");
        message.push_str("• Try different keywords or more general terms\n\n");
        message.push_str("**💡 Search Tips:**\n");
        message.push_str(
            "• Use natural language: \"find error handling\", \"authentication logic\"\n",
        );
        message.push_str("• Be specific: \"HTTP request middleware\" > \"middleware\"\n");
        message.push_str("• Include technologies: \"React component state management\"\n");
        message.push_str("• Try synonyms: \"validate\" instead of \"check\"\n");
    }

    /// Format search results with code snippets and metadata
    fn format_search_results(
        message: &mut String,
        results: &[crate::domain::types::SearchResult],
        duration: std::time::Duration,
    ) {
        message.push_str("📊 **Search Results:**\n\n");

        for (i, result) in results.iter().enumerate() {
            message.push_str(&format!(
                "**{}.** 📁 `{}` (line {})\n",
                i + 1,
                result.file_path,
                result.line_number
            ));

            Self::format_code_preview(message, result);
            message.push_str(&format!("🎯 **Relevance Score:** {:.3}\n\n", result.score));
        }

        // Add pagination hint if we hit the limit
        if results.len() == 10 {
            // Assuming default limit
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

    /// Format code preview with syntax highlighting
    fn format_code_preview(message: &mut String, result: &crate::domain::types::SearchResult) {
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
        let file_ext = Path::new(&result.file_path)
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
    }

    /// Format indexing completion response
    pub fn format_indexing_success(
        chunk_count: usize,
        path: &std::path::Path,
        duration: std::time::Duration,
    ) -> CallToolResult {
        let message = format!(
            "✅ **Indexing Completed Successfully**\n\n\
             📊 **Statistics**:\n\
             • Files processed: {} chunks\n\
             • Source directory: `{}`\n\
             • Processing time: {:.2}s\n\
             • Performance: {:.0} chunks/sec\n\n\
             🎯 **Next Steps**:\n\
             • Use `search_code` tool for semantic queries\n\
             • Try queries like \"find authentication functions\" or \"show error handling\"\n\
             • Results are ranked by semantic relevance",
            chunk_count,
            path.display(),
            duration.as_secs_f64(),
            chunk_count as f64 / duration.as_secs_f64()
        );
        tracing::info!(
            "Indexing completed successfully: {} chunks in {:?}",
            chunk_count,
            duration
        );
        CallToolResult::success(vec![Content::text(message)])
    }

    /// Format indexing error response
    pub fn format_indexing_error(error: &str, path: &std::path::Path) -> CallToolResult {
        let message = format!(
            "❌ **Indexing Failed**\n\n\
             **Error Details**: {}\n\n\
             **Troubleshooting**:\n\
             • Verify the directory contains readable source files\n\
             • Check file permissions and access rights\n\
             • Ensure supported file types (.rs, .py, .js, .ts, etc.)\n\
             • Try indexing a smaller directory first\n\n\
             **Supported Languages**: Rust, Python, JavaScript, TypeScript, Go, Java, C++, C#",
            error
        );
        tracing::error!("Indexing failed for path {}: {}", path.display(), error);
        CallToolResult::success(vec![Content::text(message)])
    }

    /// Format indexing timeout response
    pub fn format_indexing_timeout(path: &std::path::Path) -> CallToolResult {
        let message = "⏰ **Indexing Timed Out**\n\n\
            The indexing operation exceeded the 5-minute timeout limit.\n\n\
            **Possible Causes**:\n\
            • Very large codebase (>10,000 files)\n\
            • Slow I/O operations\n\
            • Network issues with embedding provider\n\
            • Resource constraints\n\n\
            **Recommendations**:\n\
            • Try indexing smaller subdirectories\n\
            • Check system resources (CPU, memory, disk I/O)\n\
            • Verify embedding provider connectivity\n\
            • Consider using a more powerful machine for large codebases"
            .to_string();

        tracing::warn!("Indexing timed out for path: {}", path.display());
        CallToolResult::success(vec![Content::text(message)])
    }

    /// Format search error response
    pub fn format_search_error(error: &str, query: &str) -> CallToolResult {
        let message = format!(
            "❌ **Search Failed**\n\n\
             **Error Details**: {}\n\n\
             **Troubleshooting Steps:**\n\
             1. **Index Check**: Ensure codebase is indexed using `index_codebase`\n\
             2. **Service Status**: Verify system is running with `get_indexing_status`\n\
             3. **Query Format**: Try simpler, more specific queries\n\
             4. **Resource Check**: Ensure sufficient system resources (CPU, memory)\n\n\
             **Common Issues:**\n\
             • Database connection problems\n\
             • Embedding service unavailable\n\
             • Corrupted index data\n\
             • Resource exhaustion",
            error
        );
        tracing::error!("Search failed for query '{}': {}", query, error);
        CallToolResult::success(vec![Content::text(message)])
    }

    /// Format search timeout response
    pub fn format_search_timeout(query: &str) -> CallToolResult {
        let message = "⏰ **Search Timed Out**\n\n\
            The search operation exceeded the 30-second timeout limit.\n\n\
            **Possible Causes:**\n\
            • Very large codebase with many matches\n\
            • Slow vector similarity computation\n\
            • Database performance issues\n\
            • High system load\n\n\
            **Optimization Suggestions:**\n\
            • Use more specific search terms\n\
            • Reduce result limit\n\
            • Try searching during off-peak hours\n\
            • Consider database performance tuning"
            .to_string();

        tracing::warn!("Search timed out for query: '{}'", query);
        CallToolResult::success(vec![Content::text(message)])
    }

    /// Format authentication/authorization error
    pub fn format_auth_error(error: &str) -> CallToolResult {
        CallToolResult::success(vec![Content::text(format!(
            "❌ Authentication/Authorization Error: {}",
            error
        ))])
    }

    /// Format resource limit error
    pub fn format_resource_limit_error(error: &str) -> CallToolResult {
        CallToolResult::success(vec![Content::text(format!(
            "❌ Resource Limit Error: {}",
            error
        ))])
    }

    /// Format validation error for path
    pub fn format_path_validation_error(error: &str) -> CallToolResult {
        CallToolResult::success(vec![Content::text(format!("❌ Error: {}", error))])
    }

    /// Format validation error for query
    pub fn format_query_validation_error(error: &str) -> CallToolResult {
        CallToolResult::success(vec![Content::text(format!("❌ Error: {}", error))])
    }
}
