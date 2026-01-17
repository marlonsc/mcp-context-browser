//! HTTP Response Utilities
//!
//! Helper functions for processing HTTP responses from API providers.
//! These are shared utilities, not ports.

use mcb_domain::error::{Error, Result};
use reqwest::Response;

/// Format error message for embedding provider
fn embedding_error(provider: &str, context: &str, details: &str) -> Error {
    Error::embedding(format!("{provider} {context}: {details}"))
}

/// Utilities for processing HTTP responses
///
/// Provides common response handling patterns used by embedding providers.
pub struct HttpResponseUtils;

impl HttpResponseUtils {
    /// Check response status and parse JSON
    ///
    /// # Arguments
    /// * `response` - The HTTP response to check
    /// * `provider_name` - Name of the provider for error messages
    ///
    /// # Returns
    /// Parsed JSON value on success, or an appropriate error
    pub async fn check_and_parse(
        response: Response,
        provider_name: &str,
    ) -> Result<serde_json::Value> {
        let status = response.status();

        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            let code = status.as_u16();

            return Err(match code {
                401 => embedding_error(provider_name, "authentication failed", &error_text),
                429 => embedding_error(provider_name, "rate limit exceeded", &error_text),
                500..=599 => embedding_error(
                    provider_name,
                    &format!("server error ({code})"),
                    &error_text,
                ),
                _ => embedding_error(
                    provider_name,
                    &format!("request failed ({code})"),
                    &error_text,
                ),
            });
        }

        response
            .json()
            .await
            .map_err(|e| embedding_error(provider_name, "response parse failed", &e.to_string()))
    }
}
