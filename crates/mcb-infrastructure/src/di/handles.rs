//! Provider Handles - Runtime-swappable provider wrappers
//!
//! These components wrap providers in RwLock to allow runtime reconfiguration
//! via admin API without restarting the application.
//!
//! ## Pattern
//!
//! ```text
//! linkme registry → Resolver → Handle (RwLock) → Domain Services
//!                      ↑
//!              AdminService.switch_provider()
//! ```

// dill macros removed - they conflict with the need for manual constructors
// that accept initial provider instances. Use add_value pattern in bootstrap.rs instead.
use mcb_application::ports::providers::{
    CacheProvider, EmbeddingProvider, LanguageChunkingProvider, VectorStoreProvider,
};
use std::sync::{Arc, RwLock};

// ============================================================================
// Embedding Provider Handle
// ============================================================================

/// Handle for runtime-swappable embedding provider
///
/// Wraps the current embedding provider in a RwLock, allowing admin API
/// to switch providers without restarting the application.
///
/// Note: dill `#[component]` removed - requires manual constructor with initial provider.
/// Use `add_value` pattern in bootstrap.rs instead.
pub struct EmbeddingProviderHandle {
    inner: RwLock<Arc<dyn EmbeddingProvider>>,
}

impl EmbeddingProviderHandle {
    /// Create a new handle with an initial provider
    pub fn new(provider: Arc<dyn EmbeddingProvider>) -> Self {
        Self {
            inner: RwLock::new(provider),
        }
    }

    /// Get the current provider
    pub fn get(&self) -> Arc<dyn EmbeddingProvider> {
        self.inner
            .read()
            .expect("EmbeddingProviderHandle lock poisoned") // mcb-validate-ignore: lock_poisoning_recovery
            .clone()
    }

    /// Set a new provider (used by admin service)
    pub fn set(&self, new_provider: Arc<dyn EmbeddingProvider>) {
        *self
            .inner
            .write()
            .expect("EmbeddingProviderHandle lock poisoned") = new_provider; // mcb-validate-ignore: lock_poisoning_recovery
    }

    /// Get provider name for diagnostics
    pub fn provider_name(&self) -> String {
        self.get().provider_name().to_string()
    }
}

impl std::fmt::Debug for EmbeddingProviderHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EmbeddingProviderHandle")
            .field("provider", &self.provider_name())
            .finish()
    }
}

// ============================================================================
// Vector Store Provider Handle
// ============================================================================

/// Handle for runtime-swappable vector store provider
///
/// Wraps the current vector store provider in a RwLock, allowing admin API
/// to switch providers without restarting the application.
///
/// Note: dill `#[component]` removed - requires manual constructor with initial provider.
/// Use `add_value` pattern in bootstrap.rs instead.
pub struct VectorStoreProviderHandle {
    inner: RwLock<Arc<dyn VectorStoreProvider>>,
}

impl VectorStoreProviderHandle {
    /// Create a new handle with an initial provider
    pub fn new(provider: Arc<dyn VectorStoreProvider>) -> Self {
        Self {
            inner: RwLock::new(provider),
        }
    }

    /// Get the current provider
    pub fn get(&self) -> Arc<dyn VectorStoreProvider> {
        self.inner
            .read()
            .expect("VectorStoreProviderHandle lock poisoned") // mcb-validate-ignore: lock_poisoning_recovery
            .clone()
    }

    /// Set a new provider (used by admin service)
    pub fn set(&self, new_provider: Arc<dyn VectorStoreProvider>) {
        *self
            .inner
            .write()
            .expect("VectorStoreProviderHandle lock poisoned") = new_provider; // mcb-validate-ignore: lock_poisoning_recovery
    }
}

impl std::fmt::Debug for VectorStoreProviderHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VectorStoreProviderHandle").finish()
    }
}

// ============================================================================
// Cache Provider Handle
// ============================================================================

/// Handle for runtime-swappable cache provider
///
/// Wraps the current cache provider in a RwLock, allowing admin API
/// to switch providers without restarting the application.
///
/// Note: dill `#[component]` removed - requires manual constructor with initial provider.
/// Use `add_value` pattern in bootstrap.rs instead.
pub struct CacheProviderHandle {
    inner: RwLock<Arc<dyn CacheProvider>>,
}

impl CacheProviderHandle {
    /// Create a new handle with an initial provider
    pub fn new(provider: Arc<dyn CacheProvider>) -> Self {
        Self {
            inner: RwLock::new(provider),
        }
    }

    /// Get the current provider
    pub fn get(&self) -> Arc<dyn CacheProvider> {
        self.inner
            .read()
            .expect("CacheProviderHandle lock poisoned") // mcb-validate-ignore: lock_poisoning_recovery
            .clone()
    }

    /// Set a new provider (used by admin service)
    pub fn set(&self, new_provider: Arc<dyn CacheProvider>) {
        *self
            .inner
            .write()
            .expect("CacheProviderHandle lock poisoned") = new_provider; // mcb-validate-ignore: lock_poisoning_recovery
    }

    /// Get provider name for diagnostics
    pub fn provider_name(&self) -> String {
        self.get().provider_name().to_string()
    }
}

impl std::fmt::Debug for CacheProviderHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CacheProviderHandle")
            .field("provider", &self.provider_name())
            .finish()
    }
}

// ============================================================================
// Language Chunking Provider Handle
// ============================================================================

/// Handle for runtime-swappable language chunking provider
///
/// Wraps the current language chunking provider in a RwLock, allowing admin API
/// to switch providers without restarting the application.
///
/// Note: dill `#[component]` removed - requires manual constructor with initial provider.
/// Use `add_value` pattern in bootstrap.rs instead.
pub struct LanguageProviderHandle {
    inner: RwLock<Arc<dyn LanguageChunkingProvider>>,
}

impl LanguageProviderHandle {
    /// Create a new handle with an initial provider
    pub fn new(provider: Arc<dyn LanguageChunkingProvider>) -> Self {
        Self {
            inner: RwLock::new(provider),
        }
    }

    /// Get the current provider
    pub fn get(&self) -> Arc<dyn LanguageChunkingProvider> {
        self.inner
            .read()
            .expect("LanguageProviderHandle lock poisoned") // mcb-validate-ignore: lock_poisoning_recovery
            .clone()
    }

    /// Set a new provider (used by admin service)
    pub fn set(&self, new_provider: Arc<dyn LanguageChunkingProvider>) {
        *self
            .inner
            .write()
            .expect("LanguageProviderHandle lock poisoned") = new_provider; // mcb-validate-ignore: lock_poisoning_recovery
    }
}

impl std::fmt::Debug for LanguageProviderHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LanguageProviderHandle").finish()
    }
}
