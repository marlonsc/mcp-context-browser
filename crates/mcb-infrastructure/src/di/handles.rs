//! Provider Handles - Runtime-swappable provider wrappers
//!
//! Type aliases for the generic `Handle<T>` for backward compatibility.
//! All handle types use the same underlying generic implementation.
//!
//! ## Pattern
//!
//! ```text
//! linkme registry → Resolver → Handle<T> (RwLock) → Domain Services
//!                      ↑
//!              AdminService.switch_provider()
//! ```

use super::handle::Handle;
use mcb_domain::ports::providers::{
    CacheProvider, EmbeddingProvider, LanguageChunkingProvider, VectorStoreProvider,
};

// ============================================================================
// Type Aliases (Backward Compatibility)
// ============================================================================

/// Handle for runtime-swappable embedding provider
///
/// Wraps the current embedding provider in a RwLock, allowing admin API
/// to switch providers without restarting the application.
pub type EmbeddingProviderHandle = Handle<dyn EmbeddingProvider>;

/// Handle for runtime-swappable vector store provider
///
/// Wraps the current vector store provider in a RwLock, allowing admin API
/// to switch providers without restarting the application.
pub type VectorStoreProviderHandle = Handle<dyn VectorStoreProvider>;

/// Handle for runtime-swappable cache provider
///
/// Wraps the current cache provider in a RwLock, allowing admin API
/// to switch providers without restarting the application.
pub type CacheProviderHandle = Handle<dyn CacheProvider>;

/// Handle for runtime-swappable language chunking provider
///
/// Wraps the current language chunking provider in a RwLock, allowing admin API
/// to switch providers without restarting the application.
pub type LanguageProviderHandle = Handle<dyn LanguageChunkingProvider>;

// ============================================================================
// Extension Trait for Provider-Specific Methods
// ============================================================================

/// Extension methods for EmbeddingProviderHandle
///
/// # Example
///
/// ```
/// use mcb_infrastructure::di::{EmbeddingProviderHandle, EmbeddingHandleExt};
///
/// fn log_provider(handle: &EmbeddingProviderHandle) {
///     println!("Using: {}", handle.provider_name());
/// }
/// ```
pub trait EmbeddingHandleExt {
    /// Get provider name for diagnostics
    fn provider_name(&self) -> String;
}

impl EmbeddingHandleExt for EmbeddingProviderHandle {
    fn provider_name(&self) -> String {
        self.get().provider_name().to_string()
    }
}

/// Extension methods for CacheProviderHandle
///
/// # Example
///
/// ```
/// use mcb_infrastructure::di::{CacheProviderHandle, CacheHandleExt};
///
/// fn log_cache(handle: &CacheProviderHandle) {
///     println!("Cache provider: {}", handle.provider_name());
/// }
/// ```
pub trait CacheHandleExt {
    /// Get provider name for diagnostics
    fn provider_name(&self) -> String;
}

impl CacheHandleExt for CacheProviderHandle {
    fn provider_name(&self) -> String {
        self.get().provider_name().to_string()
    }
}
