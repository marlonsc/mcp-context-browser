//! Language Module - Provides language processing services
//!
//! This module provides language chunking implementations.
//! Always provides UniversalLanguageChunkingProvider.

use shaku::module;

// Import language providers
use mcb_providers::language::UniversalLanguageChunkingProvider;

// Import traits
use crate::di::modules::traits::LanguageModule;

module! {
    pub LanguageModuleImpl: LanguageModule {
        components = [
            // Universal language chunker (always available)
            UniversalLanguageChunkingProvider
        ],
        providers = []
    }
}
