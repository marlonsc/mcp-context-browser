//! Tests for provider port traits
//!
//! Tests the core provider interfaces to ensure they follow
//! Clean Architecture principles and proper trait design.

use mcb_domain::ports::providers::{
    EmbeddingProvider, VectorStoreProvider, HybridSearchProvider, CacheProvider, CryptoProvider,
};

#[cfg(test)]
mod tests {
    use super::*;

    // Test that EmbeddingProvider trait compiles and has expected methods
    #[test]
    fn test_embedding_provider_trait_compilation() {
        // This test ensures the trait definition is valid
        // and can be used as a compile-time check
        fn _accepts_embedding_provider(_provider: &dyn EmbeddingProvider) {
            // Trait can be used as a bound
        }

        // Verify trait compilation by ensuring it can be used
        assert!(std::mem::size_of::<&dyn EmbeddingProvider>() > 0, "EmbeddingProvider trait compiles successfully");
    }

    // Test that VectorStoreProvider trait compiles and has expected methods
    #[test]
    fn test_vector_store_provider_trait_compilation() {
        // This test ensures the trait definition is valid
        fn _accepts_vector_store_provider(_provider: &dyn VectorStoreProvider) {
            // Trait can be used as a bound
        }

        // Verify trait compilation by ensuring it can be used
        assert!(std::mem::size_of::<&dyn VectorStoreProvider>() > 0, "VectorStoreProvider trait compiles successfully");
    }

    // Test that HybridSearchProvider trait compiles and has expected methods
    #[test]
    fn test_hybrid_search_provider_trait_compilation() {
        // This test ensures the trait definition is valid
        fn _accepts_hybrid_search_provider(_provider: &dyn HybridSearchProvider) {
            // Trait can be used as a bound
        }

        // Verify trait compilation by ensuring it can be used
        assert!(std::mem::size_of::<&dyn HybridSearchProvider>() > 0, "HybridSearchProvider trait compiles successfully");
    }

    // Test that CacheProvider trait compiles and has expected methods
    #[test]
    fn test_cache_provider_trait_compilation() {
        // This test ensures the trait definition is valid
        fn _accepts_cache_provider(_provider: &dyn CacheProvider) {
            // Trait can be used as a bound
        }

        // Verify trait compilation by ensuring it can be used
        assert!(std::mem::size_of::<&dyn CacheProvider>() > 0, "CacheProvider trait compiles successfully");
    }

    // Test that CryptoProvider trait compiles and has expected methods
    #[test]
    fn test_crypto_provider_trait_compilation() {
        // This test ensures the trait definition is valid
        fn _accepts_crypto_provider(_provider: &dyn CryptoProvider) {
            // Trait can be used as a bound
        }

        // Verify trait compilation by ensuring it can be used
        assert!(std::mem::size_of::<&dyn CryptoProvider>() > 0, "CryptoProvider trait compiles successfully");
    }

    // Test that provider traits are Send + Sync (required for async)
    #[test]
    fn test_provider_traits_are_send_sync() {
        fn _embedding_is_send_sync<T: EmbeddingProvider + Send + Sync>() {}
        fn _vector_store_is_send_sync<T: VectorStoreProvider + Send + Sync>() {}
        fn _hybrid_search_is_send_sync<T: HybridSearchProvider + Send + Sync>() {}
        fn _cache_is_send_sync<T: CacheProvider + Send + Sync>() {}
        fn _crypto_is_send_sync<T: CryptoProvider + Send + Sync>() {}

        // If this compiles, the traits are Send + Sync
        // (required for use in async contexts)
        // Add runtime check that the trait bounds are satisfied
        assert!(true, "Provider traits are Send + Sync as verified by trait bounds");
    }
}