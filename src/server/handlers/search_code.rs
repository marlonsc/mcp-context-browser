//! Handler for the search_code MCP tool
//!
//! This handler is responsible for performing semantic code search.
//! It validates queries, checks permissions, manages caching, and coordinates
//! the search process with proper error handling and timeouts.

use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use rmcp::ErrorData as McpError;
use serde_json;
use std::sync::Arc;

use crate::domain::ports::SearchServiceInterface;
use crate::domain::validation::{StringValidator, StringValidatorTrait, ValidationError};
use crate::infrastructure::auth::Permission;
use crate::infrastructure::cache::SharedCacheProvider;
use crate::infrastructure::constants::{
    HTTP_REQUEST_TIMEOUT, SEARCH_QUERY_MAX_LENGTH, SEARCH_QUERY_MIN_LENGTH,
    SEARCH_RESULT_CACHE_TTL, SEARCH_RESULT_LIMIT_MAX, SEARCH_RESULT_LIMIT_MIN,
};
use crate::infrastructure::limits::ResourceLimits;
use crate::infrastructure::service_helpers::TimedOperation;
use crate::server::args::SearchCodeArgs;
use crate::server::auth::AuthHandler;
use crate::server::formatter::ResponseFormatter;

/// Handler for code search operations
pub struct SearchCodeHandler {
    search_service: Arc<dyn SearchServiceInterface>,
    auth_handler: Arc<AuthHandler>,
    resource_limits: Arc<ResourceLimits>,
    cache_provider: Option<SharedCacheProvider>,
}

impl SearchCodeHandler {
    /// Create a new search_code handler
    pub fn new(
        search_service: Arc<dyn SearchServiceInterface>,
        auth_handler: Arc<AuthHandler>,
        resource_limits: Arc<ResourceLimits>,
        cache_provider: Option<SharedCacheProvider>,
    ) -> Self {
        Self {
            search_service,
            auth_handler,
            resource_limits,
            cache_provider,
        }
    }

    /// Validate search query using the validation framework
    fn validate_search_query(&self, query: &str) -> Result<String, CallToolResult> {
        let trimmed = query.trim();

        // Create validator for search queries
        let validator = StringValidator::not_empty()
            .combine_with(StringValidator::min_length(SEARCH_QUERY_MIN_LENGTH))
            .combine_with(StringValidator::max_length(SEARCH_QUERY_MAX_LENGTH));

        match validator.validate(trimmed) {
            Ok(validated) => Ok(validated),
            Err(ValidationError::Required { .. }) => {
                Err(ResponseFormatter::format_query_validation_error(
                    "Search query cannot be empty. Please provide a natural language query.",
                ))
            }
            Err(ValidationError::TooShort { .. }) => {
                Err(ResponseFormatter::format_query_validation_error(
                    &format!("Search query too short. Please use at least {} characters for meaningful results.", SEARCH_QUERY_MIN_LENGTH),
                ))
            }
            Err(ValidationError::TooLong { .. }) => {
                Err(ResponseFormatter::format_query_validation_error(
                    &format!("Search query too long. Please limit to {} characters.", SEARCH_QUERY_MAX_LENGTH),
                ))
            }
            _ => Err(ResponseFormatter::format_query_validation_error(
                "Invalid search query format.",
            )),
        }
    }

    /// Handle the search_code tool request
    pub async fn handle(
        &self,
        Parameters(SearchCodeArgs {
            query,
            limit,
            token,
            filters: _,
            ..
        }): Parameters<SearchCodeArgs>,
    ) -> Result<CallToolResult, McpError> {
        let timer = TimedOperation::start();

        // Check authentication and permissions
        if let Err(e) = self
            .auth_handler
            .check_auth(token.as_ref(), &Permission::SearchCodebase)
        {
            return Ok(ResponseFormatter::format_auth_error(&e.to_string()));
        }

        // Check resource limits for search operation
        if let Err(e) = self.resource_limits.check_operation_allowed("search").await {
            return Ok(ResponseFormatter::format_resource_limit_error(
                &e.to_string(),
            ));
        }

        // Acquire search permit
        let _permit = match self
            .resource_limits
            .acquire_operation_permit("search")
            .await
        {
            Ok(permit) => permit,
            Err(e) => {
                return Ok(ResponseFormatter::format_resource_limit_error(
                    &e.to_string(),
                ));
            }
        };

        // Validate query input using validation framework
        let query = match self.validate_search_query(&query) {
            Ok(validated) => validated,
            Err(error_response) => return Ok(error_response),
        };

        // Validate limit
        let limit = limit.clamp(SEARCH_RESULT_LIMIT_MIN, SEARCH_RESULT_LIMIT_MAX);
        let collection = "default";

        // Check cache for search results (if cache is enabled)
        let cache_key = format!("{}:{}:{}", collection, query, limit);
        if let Some(ref cache) = self.cache_provider {
            if let Ok(Some(cached_bytes)) = cache.get("search_results", &cache_key).await {
                if let Ok(search_results) =
                    serde_json::from_slice::<Vec<crate::domain::types::SearchResult>>(&cached_bytes)
                {
                    tracing::info!(
                        "✅ Search cache hit for query: '{}' (limit: {})",
                        query,
                        limit
                    );
                    return ResponseFormatter::format_search_response(
                        &query,
                        &search_results,
                        timer.elapsed(),
                        true,
                    );
                }
            }
        }

        tracing::info!(
            "Performing semantic search for query: '{}' (limit: {})",
            query,
            limit
        );

        // Add timeout for search operations
        let search_future = self.search_service.search(collection, &query, limit);
        let result = tokio::time::timeout(HTTP_REQUEST_TIMEOUT, search_future).await;

        let duration = timer.elapsed();

        match result {
            Ok(Ok(results)) => {
                // Cache search results (if cache is enabled)
                if let Some(ref cache) = self.cache_provider {
                    if let Ok(serialized) = serde_json::to_vec(&results) {
                        let _ = cache
                            .set(
                                "search_results",
                                &cache_key,
                                serialized,
                                SEARCH_RESULT_CACHE_TTL,
                            )
                            .await;
                    }
                }

                // Use the response formatter
                ResponseFormatter::format_search_response(&query, &results, duration, false)
            }
            Ok(Err(e)) => Ok(ResponseFormatter::format_search_error(
                &e.to_string(),
                &query,
            )),
            Err(_) => Ok(ResponseFormatter::format_search_timeout(&query)),
        }
    }
}
