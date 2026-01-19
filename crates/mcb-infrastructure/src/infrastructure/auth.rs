//! Authentication Service Adapter
//!
//! Null implementation of the authentication port for testing.

use async_trait::async_trait;
use dill::{component, interface};
use mcb_application::ports::infrastructure::AuthServiceInterface;
use mcb_domain::error::Result;

/// Null implementation for testing
#[component]
#[interface(dyn AuthServiceInterface)]
pub struct NullAuthService;

impl NullAuthService {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NullAuthService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AuthServiceInterface for NullAuthService {
    async fn validate_token(&self, _token: &str) -> Result<bool> {
        Ok(true)
    }
    async fn generate_token(&self, _subject: &str) -> Result<String> {
        Ok("null-token".to_string())
    }
}
