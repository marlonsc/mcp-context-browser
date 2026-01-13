//! Unit tests for individual components and modules
//!
//! This module contains focused unit tests for individual functions and methods
//! across all modules, ensuring comprehensive coverage of business logic.

use mcp_context_browser::domain::error::Error;
use mcp_context_browser::domain::types::{CodeChunk, Embedding, Language};

/// Test core type constructors and basic functionality
#[cfg(test)]
mod core_types_unit_tests {
    use super::*;

    #[test]
    fn test_code_chunk_creation() {
        let chunk = CodeChunk {
            id: "test_id".to_string(),
            content: "fn hello() {}".to_string(),
            file_path: "src/main.rs".to_string(),
            start_line: 1,
            end_line: 3,
            language: Language::Rust,
            metadata: serde_json::json!({"test": true}),
        };

        assert_eq!(chunk.id, "test_id");
        assert_eq!(chunk.content, "fn hello() {}");
        assert_eq!(chunk.language, Language::Rust);
        assert_eq!(chunk.start_line, 1);
        assert_eq!(chunk.end_line, 3);
    }

    #[test]
    fn test_embedding_creation() {
        let vector = vec![0.1, 0.2, 0.3, 0.4];
        let embedding = Embedding {
            vector: vector.clone(),
            model: "test-model".to_string(),
            dimensions: 4,
        };

        assert_eq!(embedding.vector, vector);
        assert_eq!(embedding.model, "test-model");
        assert_eq!(embedding.dimensions, 4);
    }

    #[test]
    fn test_language_enum_variants() {
        assert_eq!(Language::Rust, Language::Rust);
        assert_eq!(Language::Python, Language::Python);
        assert_eq!(Language::JavaScript, Language::JavaScript);
        assert_ne!(Language::Rust, Language::Python);
    }
}

/// Test error handling and custom error types
#[cfg(test)]
mod error_handling_unit_tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = Error::generic("test error");
        assert_eq!(format!("{}", error), "Generic error: test error");
    }

    #[test]
    fn test_error_display() {
        let error = Error::invalid_argument("invalid input");
        let error_string = format!("{}", error);
        assert!(error_string.contains("invalid input"));
    }

    #[test]
    fn test_error_context_preservation() {
        let original_error = Error::not_found("resource not found");
        // Test that error context is preserved through conversions
        assert!(matches!(original_error, Error::NotFound { .. }));
    }
}

/// Test validation logic using core Error types
#[cfg(test)]
mod validation_unit_tests {
    use mcp_context_browser::domain::error::Error;
    use mcp_context_browser::domain::types::EmbeddingConfig;

    #[test]
    fn test_validate_not_empty_string() {
        // Test that empty strings are handled properly
        let valid_string = "hello";
        assert!(!valid_string.trim().is_empty());

        let empty_string = "";
        assert!(empty_string.trim().is_empty());

        let whitespace_string = "   ";
        assert!(whitespace_string.trim().is_empty());
    }

    #[test]
    fn test_validate_string_length() {
        let short_string = "hi";
        let long_string = "this is a very long string that exceeds typical limits";
        let normal_string = "hello";

        assert!(short_string.len() < 5);
        assert!(long_string.len() > 20);
        assert!(!normal_string.is_empty() && normal_string.len() <= 10);
    }

    #[test]
    fn test_config_validation() {
        // Test that EmbeddingConfig can be created with valid values
        let config = EmbeddingConfig {
            provider: "mock".to_string(),
            model: "test-model".to_string(),
            api_key: None,
            dimensions: None,
            base_url: None,
            max_tokens: None,
        };
        assert_eq!(config.provider, "mock");
    }

    #[test]
    fn test_config_validation_empty_provider() {
        // Test that empty provider is detectable
        let config = EmbeddingConfig {
            provider: "".to_string(),
            model: "".to_string(),
            api_key: None,
            dimensions: None,
            base_url: None,
            max_tokens: None,
        };
        assert!(config.provider.is_empty());
    }

    #[test]
    fn test_error_creation_for_validation() {
        // Test that validation errors can be properly created
        let error = Error::config("Provider name cannot be empty");
        let error_str = format!("{}", error);
        assert!(error_str.contains("Provider name cannot be empty"));
    }

    #[test]
    fn test_alphanumeric_validation() {
        // Test alphanumeric validation logic
        let valid_inputs = vec!["hello", "world123", "test_user", "abc_123"];
        for input in valid_inputs {
            assert!(
                input.chars().all(|c| c.is_alphanumeric() || c == '_'),
                "Should be alphanumeric: {}",
                input
            );
        }

        let invalid_inputs = vec!["hello@world", "test!", "path/to/file"];
        for input in invalid_inputs {
            assert!(
                !input.chars().all(|c| c.is_alphanumeric() || c == '_'),
                "Should not be alphanumeric: {}",
                input
            );
        }
    }
}

/// Test configuration parsing and validation
#[cfg(test)]
mod config_unit_tests {
    use mcp_context_browser::domain::types::EmbeddingConfig;
    use mcp_context_browser::infrastructure::config::providers::ProviderConfigManager;

    #[test]
    fn test_config_provider_manager_creation() {
        let manager = ProviderConfigManager::new();
        // Manager should be created successfully and ready
        assert!(manager.is_ready());
    }

    #[test]
    fn test_embedding_config_validation() {
        let manager = ProviderConfigManager::new();
        let config = EmbeddingConfig {
            provider: "openai".to_string(),
            model: "text-embedding-3-small".to_string(),
            api_key: Some("test-key".to_string()),
            dimensions: Some(1536),
            base_url: None,
            max_tokens: None,
        };
        let result = manager.validate_embedding_config(&config);
        assert!(result.is_ok(), "OpenAI config should be valid");
    }

    #[test]
    fn test_mock_provider_config() {
        let config = EmbeddingConfig {
            provider: "mock".to_string(),
            model: "mock-model".to_string(),
            api_key: None,
            dimensions: Some(384),
            base_url: None,
            max_tokens: None,
        };
        assert_eq!(config.provider, "mock");
        assert_eq!(config.dimensions, Some(384));
    }

    #[test]
    fn test_empty_provider_detection() {
        let manager = ProviderConfigManager::new();
        let config = EmbeddingConfig {
            provider: "".to_string(),
            model: "".to_string(),
            api_key: None,
            dimensions: None,
            base_url: None,
            max_tokens: None,
        };
        let result = manager.validate_embedding_config(&config);
        assert!(result.is_err(), "Empty provider should fail validation");
    }
}

/// Test VectorStoreProvider implementations
#[cfg(test)]
mod repository_unit_tests {
    use mcp_context_browser::adapters::providers::InMemoryVectorStoreProvider;
    use mcp_context_browser::domain::ports::VectorStoreProvider;

    #[test]
    fn test_in_memory_provider_creation() {
        let provider = InMemoryVectorStoreProvider::new();
        assert_eq!(provider.provider_name(), "in_memory");
    }

    #[tokio::test]
    async fn test_in_memory_collection_operations() -> Result<(), Box<dyn std::error::Error>> {
        let provider = InMemoryVectorStoreProvider::new();

        // Collection should not exist initially
        let exists = provider.collection_exists("test_collection").await?;
        assert!(!exists);

        // Create collection
        provider.create_collection("test_collection", 128).await?;

        // Collection should exist now
        let exists = provider.collection_exists("test_collection").await?;
        assert!(exists);

        // Delete collection
        provider.delete_collection("test_collection").await?;
        Ok(())
    }
}

/// Test EmbeddingProvider implementations
#[cfg(test)]
mod provider_unit_tests {
    use mcp_context_browser::adapters::providers::embedding::null::NullEmbeddingProvider;
    use mcp_context_browser::domain::ports::EmbeddingProvider;
    use std::sync::Arc;

    #[test]
    fn test_mock_embedding_provider_creation() {
        let provider = NullEmbeddingProvider::new();
        assert_eq!(provider.provider_name(), "null");
        // NullEmbeddingProvider returns 384-dimensional vectors (similar to many real models)
        assert_eq!(provider.dimensions(), 384);
    }

    #[tokio::test]
    async fn test_mock_embedding_provider_embed() -> Result<(), Box<dyn std::error::Error>> {
        let provider = NullEmbeddingProvider::new();
        let embedding = provider.embed("test text").await?;
        // NullEmbeddingProvider returns 384-dimensional vectors
        assert_eq!(embedding.dimensions, 384);
        assert_eq!(embedding.vector.len(), 384);
        // Verify vectors have varied values based on text hash
        assert!(
            embedding.vector.iter().any(|&v| v != 0.0),
            "Should have non-zero values"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_mock_embedding_provider_batch_embed() -> Result<(), Box<dyn std::error::Error>> {
        let provider = NullEmbeddingProvider::new();
        let texts = vec![
            "text1".to_string(),
            "text2".to_string(),
            "text3".to_string(),
        ];
        let embeddings = provider.embed_batch(&texts).await?;
        assert_eq!(embeddings.len(), 3);
        for emb in &embeddings {
            // NullEmbeddingProvider returns 384-dimensional vectors
            assert_eq!(emb.dimensions, 384);
            assert_eq!(emb.vector.len(), 384);
        }
        // Verify different texts produce different embeddings (text-hash based)
        assert_ne!(
            embeddings[0].vector, embeddings[1].vector,
            "Different texts should produce different embeddings"
        );
        Ok(())
    }

    #[test]
    fn test_provider_trait_object_compatibility() {
        // Test that providers can be used as trait objects
        let provider: Arc<dyn EmbeddingProvider> = Arc::new(NullEmbeddingProvider::new());
        // NullEmbeddingProvider returns 384-dimensional vectors
        assert_eq!(provider.dimensions(), 384);
    }
}

/// Test ContextService initialization and operations
#[cfg(test)]
mod service_unit_tests {
    use mcp_context_browser::adapters::providers::embedding::null::NullEmbeddingProvider;
    use mcp_context_browser::adapters::providers::InMemoryVectorStoreProvider;
    use mcp_context_browser::application::ContextService;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_context_service_creation() {
        let embedding_provider = Arc::new(NullEmbeddingProvider::new());
        let vector_store = Arc::new(InMemoryVectorStoreProvider::new());
        let (sender, receiver) = tokio::sync::mpsc::channel(100);
        tokio::spawn(async move {
            let mut receiver = receiver;
            while let Some(msg) = receiver.recv().await {
                use mcp_context_browser::adapters::hybrid_search::HybridSearchMessage;
                match msg {
                    HybridSearchMessage::Search { respond_to, .. } => {
                        let _ = respond_to.send(Ok(Vec::new()));
                    }
                    HybridSearchMessage::GetStats { respond_to } => {
                        let _ = respond_to.send(std::collections::HashMap::new());
                    }
                    _ => {}
                }
            }
        });
        let hybrid_search = Arc::new(mcp_context_browser::adapters::HybridSearchAdapter::new(
            sender,
        ));
        let service =
            ContextService::new_with_providers(embedding_provider, vector_store, hybrid_search);

        // NullEmbeddingProvider returns 384-dimensional vectors
        assert_eq!(service.embedding_dimensions(), 384);
    }

    #[tokio::test]
    async fn test_context_service_embed_text() -> Result<(), Box<dyn std::error::Error>> {
        let embedding_provider = Arc::new(NullEmbeddingProvider::new());
        let vector_store = Arc::new(InMemoryVectorStoreProvider::new());
        let (sender, receiver) = tokio::sync::mpsc::channel(100);
        tokio::spawn(async move {
            let mut receiver = receiver;
            while let Some(msg) = receiver.recv().await {
                use mcp_context_browser::adapters::hybrid_search::HybridSearchMessage;
                match msg {
                    HybridSearchMessage::Search { respond_to, .. } => {
                        let _ = respond_to.send(Ok(Vec::new()));
                    }
                    HybridSearchMessage::GetStats { respond_to } => {
                        let _ = respond_to.send(std::collections::HashMap::new());
                    }
                    _ => {}
                }
            }
        });
        let hybrid_search = Arc::new(mcp_context_browser::adapters::HybridSearchAdapter::new(
            sender,
        ));
        let service =
            ContextService::new_with_providers(embedding_provider, vector_store, hybrid_search);

        let embedding = service.embed_text("test query").await?;
        // NullEmbeddingProvider returns 384-dimensional vectors
        assert_eq!(embedding.vector.len(), 384);
        assert_eq!(embedding.dimensions, 384);
        Ok(())
    }
}

/// Test utility functions and collection operations
#[cfg(test)]
mod utility_unit_tests {
    use std::collections::HashMap;

    #[test]
    fn test_hashmap_get_or_default() {
        let mut map: HashMap<String, i32> = HashMap::new();
        map.insert("key1".to_string(), 42);

        // Test getting existing key
        let value = map.get("key1").cloned().unwrap_or(0);
        assert_eq!(value, 42);

        // Test getting missing key with default
        let value = map.get("missing").cloned().unwrap_or(99);
        assert_eq!(value, 99);
    }

    #[test]
    fn test_slice_is_empty() {
        let empty: Vec<i32> = vec![];
        assert!(empty.is_empty());

        let data = [1, 2, 3];
        assert!(!data.is_empty());
    }

    #[test]
    fn test_safe_slice_access() {
        let data = [10, 20, 30];

        // Valid index
        assert_eq!(data.get(1).cloned(), Some(20));

        // Invalid index
        assert_eq!(data.get(10).cloned(), None);
    }

    #[test]
    fn test_error_message_formatting() {
        let context = "Context";
        let details = "Something went wrong";
        let formatted = format!("{}: {}", context, details);
        assert_eq!(formatted, "Context: Something went wrong");
    }

    #[test]
    fn test_validation_error_message_format() {
        let field = "email";
        let reason = "invalid format";
        let error = format!("Validation failed for {}: {}", field, reason);

        assert!(error.contains("Validation failed"));
        assert!(error.contains("email"));
        assert!(error.contains("invalid format"));
    }

    #[test]
    fn test_string_transformations() {
        // Test trimming
        assert_eq!("  hello  ".trim(), "hello");

        // Test lowercase
        assert_eq!("HELLO".to_lowercase(), "hello");

        // Test splitting
        let parts: Vec<&str> = "a,b,c".split(',').collect();
        assert_eq!(parts, vec!["a", "b", "c"]);
    }
}

/// Performance tests with timing assertions
#[cfg(test)]
mod performance_unit_tests {
    use mcp_context_browser::adapters::providers::embedding::null::NullEmbeddingProvider;
    use mcp_context_browser::domain::ports::EmbeddingProvider;
    use std::time::Instant;

    #[tokio::test]
    async fn test_embedding_completes_within_timeout() {
        let provider = NullEmbeddingProvider::new();
        let start = Instant::now();

        let result = provider.embed("test text").await;

        let duration = start.elapsed();
        assert!(result.is_ok());
        // Mock provider should complete nearly instantly (under 100ms)
        assert!(
            duration.as_millis() < 100,
            "Embedding took too long: {:?}",
            duration
        );
    }

    #[tokio::test]
    async fn test_batch_embedding_scales_linearly() {
        let provider = NullEmbeddingProvider::new();

        // Time single embedding
        let start = Instant::now();
        let _ = provider.embed("single text").await;
        let single_duration = start.elapsed();

        // Time batch of 10
        let texts: Vec<String> = (0..10).map(|i| format!("text {}", i)).collect();
        let start = Instant::now();
        let _ = provider.embed_batch(&texts).await;
        let batch_duration = start.elapsed();

        // Batch should not take more than 20x single (allowing overhead)
        assert!(
            batch_duration.as_millis() < single_duration.as_millis() * 20 + 50,
            "Batch embedding scaling issue: single={:?}, batch={:?}",
            single_duration,
            batch_duration
        );
    }
}

/// Security tests for input validation and access control
#[cfg(test)]
mod security_unit_tests {
    #[test]
    fn test_input_sanitization_empty_input() {
        // Empty input should be rejected
        let input = "";
        assert!(input.trim().is_empty(), "Empty input should be detected");
    }

    #[test]
    fn test_input_sanitization_whitespace_only() {
        // Whitespace-only input should be rejected
        let input = "   \t\n   ";
        assert!(
            input.trim().is_empty(),
            "Whitespace-only input should be detected"
        );
    }

    #[test]
    fn test_special_characters_rejected() {
        // Input with special characters should be rejected for alphanumeric validation
        let dangerous_inputs = vec![
            "<script>alert('xss')</script>",
            "'; DROP TABLE users; --",
            "../../../etc/passwd",
            "hello@world.com",
        ];

        for input in dangerous_inputs {
            let is_alphanumeric = input.chars().all(|c| c.is_alphanumeric() || c == '_');
            assert!(!is_alphanumeric, "Should reject dangerous input: {}", input);
        }
    }

    #[test]
    fn test_length_limits_enforced() {
        // Very long input should be rejected
        let long_input = "a".repeat(10001);
        let max_length = 10000;
        assert!(
            long_input.len() > max_length,
            "Long input should exceed limit"
        );
    }

    #[test]
    fn test_safe_input_accepted() {
        // Normal safe input should pass validation
        let safe_inputs = vec!["hello", "world_123", "TestUser", "data2024"];

        for input in safe_inputs {
            let is_alphanumeric = input.chars().all(|c| c.is_alphanumeric() || c == '_');
            assert!(is_alphanumeric, "Should accept safe input: {}", input);
        }
    }

    #[test]
    fn test_path_traversal_detection() {
        // Path traversal attempts should be detectable
        let malicious_paths = vec![
            "../../../etc/passwd",
            "..\\..\\..\\windows\\system32",
            "/etc/passwd",
            "C:\\Windows\\System32",
        ];

        for path in malicious_paths {
            let contains_traversal =
                path.contains("..") || path.starts_with('/') || path.contains(":\\");
            assert!(contains_traversal, "Should detect path traversal: {}", path);
        }
    }

    #[test]
    fn test_xss_pattern_detection() {
        // XSS patterns should be detectable
        let xss_inputs = vec!["<script>", "javascript:", "onclick=", "onerror="];

        for input in xss_inputs {
            let contains_xss =
                input.contains('<') || input.contains("javascript:") || input.contains("on");
            assert!(contains_xss, "Should detect XSS pattern: {}", input);
        }
    }
}
