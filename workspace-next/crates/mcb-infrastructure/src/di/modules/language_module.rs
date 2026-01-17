//! Language Module - Provides language processing services
//!
//! This module provides language chunking implementations.
//! Always provides UniversalLanguageChunkingProvider.

use shaku::module;

// Import language providers
use mcb_providers::language::UniversalLanguageChunkingProvider;

// Import traits
use crate::di::modules::traits::LanguageModule;

/// Language module providing language processing implementations
///
/// ## Services Provided
/// - LanguageChunkingProvider: For code chunking operations
///
/// ## Implementation
/// - UniversalLanguageChunkingProvider: Supports all programming languages
///
/// ## Notes
/// This module always provides the universal chunker as it's stateless
/// and supports all languages that the system can handle.
module! {
    pub LanguageModuleImpl: LanguageModule {
        components = [
            // Universal language chunker (always available)
            UniversalLanguageChunkingProvider
        ],
        providers = []
    }
}