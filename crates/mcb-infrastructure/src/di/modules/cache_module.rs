//! Cache Module - Provides caching services
//!
//! This module provides cache provider implementations.
//! Uses null provider as default for testing.

use shaku::module;

// Import cache providers
use mcb_providers::cache::NullCacheProvider;

// Import traits
use crate::di::modules::traits::CacheModule;

module! {
    pub CacheModuleImpl: CacheModule {
        components = [
            // Default null cache provider (testing fallback)
            NullCacheProvider
        ],
        providers = []
    }
}
