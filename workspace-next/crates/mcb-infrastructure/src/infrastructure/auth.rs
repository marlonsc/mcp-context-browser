//! Authentication Service Adapter
//!
//! Null implementation of the authentication port for testing.

use async_trait::async_trait;
use mcb_domain::error::Result;
use mcb_domain::ports::infrastructure::AuthServiceInterface;

/// Null implementation for testing
#[derive(shaku::Component)]
#[shaku(interface = AuthServiceInterface)]
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
