//! Distributed Lock Provider Port
//!
//! Defines the contract for distributed lock coordination services.

use crate::error::Result;
use async_trait::async_trait;

/// Lock guard token returned when a lock is acquired
#[derive(Debug, Clone)]
pub struct LockGuard {
    /// Lock key
    pub key: String,
    /// Unique token for this lock acquisition
    pub token: String,
}

/// Distributed lock provider interface
#[async_trait]
pub trait LockProvider: Send + Sync {
    /// Acquire a distributed lock
    async fn acquire_lock(&self, key: &str) -> Result<LockGuard>;

    /// Release a distributed lock
    async fn release_lock(&self, guard: LockGuard) -> Result<()>;
}
