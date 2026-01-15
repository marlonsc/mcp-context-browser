//! Search Code Tool Handler
//!
//! Handles the search_code MCP tool call using the domain search service.

use crate::handlers::SearchCodeArgs;
use crate::McpServer;
use mcb_domain::SearchResult;
use rmcp::model::CallToolRequest;
use tracing::{info, instrument};

/// Handle the search_code tool call
#[instrument(skip(server), fields(query = %request.arguments["query"]))]
pub async fn handle_search_code(
    server: &McpServer,
    request: CallToolRequest,
) -> Result<rmcp::model::CallToolResponse, rmcp::Error> {
    let args: SearchCodeArgs = serde_json::from_value(request.arguments)
        .map_err(|e| rmcp::Error::invalid_params(format!("Invalid arguments: {}", e)))?;

    info!("Searching code with query: {}", args.query);

    // Use the search service from the domain layer
    let search_results = server
        .search_service()
        .search(&args.query, args.limit, args.file_path.as_deref(), args.language.as_deref())
        .await
        .map_err(|e| {
            rmcp::Error::internal_error(format!("Search failed: {}", e))
        })?;

    // Format results for MCP response
    let content = format_search_results(&search_results);

    Ok(rmcp::model::CallToolResponse {
        content: vec![rmcp::model::Content::text(content)],
        is_error: None,
    })
}

/// Format search results into a human-readable string
fn format_search_results(results: &[SearchResult]) -> String {
    if results.is_empty() {
        return "No results found.".to_string();
    }

    let mut output = format!("Found {} results:\n\n", results.len());

    for (i, result) in results.iter().enumerate() {
        output.push_str(&format!(
            "{}. **{}** ({})\n",
            i + 1,
            result.file_path,
            result.language
        ));

        if let Some(ref snippet) = result.snippet {
            output.push_str(&format!("   ```{}\n   {}\n   ```\n", result.language, snippet));
        }

        output.push_str(&format!(
            "   Score: {:.3}, Line: {}\n\n",
            result.score, result.start_line
        ));
    }

    output
}