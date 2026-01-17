//! Cache Module - Provides caching services
//!
//! This module provides cache provider implementations.
//! Uses null provider as default for testing.

use shaku::module;

// Import cache providers
use mcb_providers::cache::NullCacheProvider;

// Import traits
use crate::di::modules::traits::CacheModule;

/// Cache module providing cache provider implementations
///
/// ## Services Provided
/// - CacheProvider: For caching operations
///
/// ## Default Implementation
/// - NullCacheProvider: No-op cache for testing
///
/// ## Production Override
/// Can be overridden with Redis, Moka, or other cache providers
module! {
    pub CacheModuleImpl: CacheModule {
        components = [
            // Default null cache provider (testing fallback)
            NullCacheProvider
        ],
        providers = []
    }
}