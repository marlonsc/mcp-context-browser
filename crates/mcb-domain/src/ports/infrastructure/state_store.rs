//! State Store Port
//!
//! Defines the contract for simple key-value state persistence.

use crate::error::Result;
use async_trait::async_trait;

/// State store interface for key-value persistence
#[async_trait]
pub trait StateStoreProvider: Send + Sync {
    /// Save data to a key
    async fn save(&self, key: &str, data: &[u8]) -> Result<()>;

    /// Load data from a key
    async fn load(&self, key: &str) -> Result<Option<Vec<u8>>>;

    /// Delete data for a key
    async fn delete(&self, key: &str) -> Result<()>;
}
