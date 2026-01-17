//! State Store Port
//!
//! Defines the contract for simple key-value state persistence.

use async_trait::async_trait;
use mcb_domain::error::Result;
use shaku::Interface;

/// State store interface for key-value persistence
#[async_trait]
pub trait StateStoreProvider: Interface + Send + Sync {
    /// Save data to a key
    async fn save(&self, key: &str, data: &[u8]) -> Result<()>;

    /// Load data from a key
    async fn load(&self, key: &str) -> Result<Option<Vec<u8>>>;

    /// Delete data for a key
    async fn delete(&self, key: &str) -> Result<()>;
}
