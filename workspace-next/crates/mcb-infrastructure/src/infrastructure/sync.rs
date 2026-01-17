//! Sync Provider Interface

use async_trait::async_trait;
use shaku::Interface;
use mcb_domain::error::Result;

/// Sync provider for distributed coordination
#[async_trait]
pub trait SyncProvider: Interface + Send + Sync {
    /// Acquire a distributed lock
    async fn acquire_lock(&self, key: &str) -> Result<LockGuard>;

    /// Release a distributed lock
    async fn release_lock(&self, guard: LockGuard) -> Result<()>;
}

/// Lock guard token
#[derive(Debug, Clone)]
pub struct LockGuard {
    pub key: String,
    pub token: String,
}

/// Null implementation for testing
#[derive(shaku::Component)]
#[shaku(interface = SyncProvider)]
pub struct NullSyncProvider;

impl NullSyncProvider {
    pub fn new() -> Self { Self }
}

impl Default for NullSyncProvider {
    fn default() -> Self { Self::new() }
}

#[async_trait]
impl SyncProvider for NullSyncProvider {
    async fn acquire_lock(&self, key: &str) -> Result<LockGuard> {
        Ok(LockGuard {
            key: key.to_string(),
            token: "null-token".to_string(),
        })
    }
    async fn release_lock(&self, _guard: LockGuard) -> Result<()> { Ok(()) }
}
