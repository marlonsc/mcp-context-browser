//! Snapshot Provider Interface

use async_trait::async_trait;
use shaku::Interface;
use mcb_domain::error::Result;

/// Snapshot provider for state persistence
#[async_trait]
pub trait SnapshotProvider: Interface + Send + Sync {
    /// Save a snapshot
    async fn save(&self, key: &str, data: &[u8]) -> Result<()>;

    /// Load a snapshot
    async fn load(&self, key: &str) -> Result<Option<Vec<u8>>>;

    /// Delete a snapshot
    async fn delete(&self, key: &str) -> Result<()>;
}

/// Null implementation for testing
#[derive(shaku::Component)]
#[shaku(interface = SnapshotProvider)]
pub struct NullSnapshotProvider;

impl NullSnapshotProvider {
    pub fn new() -> Self { Self }
}

impl Default for NullSnapshotProvider {
    fn default() -> Self { Self::new() }
}

#[async_trait]
impl SnapshotProvider for NullSnapshotProvider {
    async fn save(&self, _key: &str, _data: &[u8]) -> Result<()> { Ok(()) }
    async fn load(&self, _key: &str) -> Result<Option<Vec<u8>>> { Ok(None) }
    async fn delete(&self, _key: &str) -> Result<()> { Ok(()) }
}
