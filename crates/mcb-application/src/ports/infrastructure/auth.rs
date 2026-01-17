//! Authentication Service Port
//!
//! Defines the contract for authentication services.

use async_trait::async_trait;
use mcb_domain::error::Result;
use shaku::Interface;

/// Authentication service interface
#[async_trait]
pub trait AuthServiceInterface: Interface + Send + Sync {
    /// Validate a JWT token
    async fn validate_token(&self, token: &str) -> Result<bool>;

    /// Generate a new JWT token
    async fn generate_token(&self, subject: &str) -> Result<String>;
}
