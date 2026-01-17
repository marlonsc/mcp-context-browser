//! Authentication Service Interface

use async_trait::async_trait;
use shaku::Interface;
use mcb_domain::error::Result;

/// Authentication service interface
#[async_trait]
pub trait AuthServiceInterface: Interface + Send + Sync {
    /// Validate a JWT token
    async fn validate_token(&self, token: &str) -> Result<bool>;

    /// Generate a new JWT token
    async fn generate_token(&self, subject: &str) -> Result<String>;
}

/// Null implementation for testing
#[derive(shaku::Component)]
#[shaku(interface = AuthServiceInterface)]
pub struct NullAuthService;

impl NullAuthService {
    pub fn new() -> Self { Self }
}

impl Default for NullAuthService {
    fn default() -> Self { Self::new() }
}

#[async_trait]
impl AuthServiceInterface for NullAuthService {
    async fn validate_token(&self, _token: &str) -> Result<bool> { Ok(true) }
    async fn generate_token(&self, _subject: &str) -> Result<String> { Ok("null-token".to_string()) }
}
