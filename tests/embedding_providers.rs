//! Unit tests for embedding providers

use mcp_context_browser::core::error::Error;
use mcp_context_browser::providers::EmbeddingProvider;
use mcp_context_browser::providers::embedding::{
    GeminiEmbeddingProvider, OllamaEmbeddingProvider, OpenAIEmbeddingProvider,
    VoyageAIEmbeddingProvider,
};

// Note: MockHttpServer removed as it was unused and causing warnings

#[cfg(test)]
mod openai_tests {
    use super::*;
    use mcp_context_browser::providers::embedding::OpenAIEmbeddingProvider;
    use mockito::Server;
    use serde_json::json;

    #[test]
    fn test_openai_provider_creation() {
        let provider = OpenAIEmbeddingProvider::new(
            "test-key".to_string(),
            Some("https://api.openai.com/v1".to_string()),
            "text-embedding-3-small".to_string(),
        )
        .expect("Failed to create OpenAI provider");

        assert_eq!(provider.provider_name(), "openai");
        assert_eq!(provider.model(), "text-embedding-3-small");
        assert_eq!(provider.dimensions(), 1536);
        assert_eq!(provider.max_tokens(), 8192);
    }

    #[test]
    fn test_openai_dimensions_by_model() {
        let models_and_dims = vec![
            ("text-embedding-3-small", 1536),
            ("text-embedding-3-large", 3072),
            ("text-embedding-ada-002", 1536),
            ("unknown-model", 1536), // fallback
        ];

        for (model, expected_dims) in models_and_dims {
            let provider =
                OpenAIEmbeddingProvider::new("test-key".to_string(), None, model.to_string())
                    .expect("Failed to create OpenAI provider");
            assert_eq!(
                provider.dimensions(),
                expected_dims,
                "Model {} should have {} dimensions",
                model,
                expected_dims
            );
        }
    }

    #[test]
    fn test_openai_base_url() {
        let provider_no_url = OpenAIEmbeddingProvider::new(
            "test-key".to_string(),
            None,
            "text-embedding-3-small".to_string(),
        )
        .expect("Failed to create OpenAI provider");
        assert_eq!(provider_no_url.base_url(), "https://api.openai.com/v1");

        let provider_with_url = OpenAIEmbeddingProvider::new(
            "test-key".to_string(),
            Some("https://custom.openai.com/v1".to_string()),
            "text-embedding-3-small".to_string(),
        )
        .expect("Failed to create OpenAI provider");
        assert_eq!(provider_with_url.base_url(), "https://custom.openai.com/v1");
    }

    #[test]
    fn test_openai_embed_with_mock_server() {
        let mut server = Server::new();
        let embedding_vec = vec![0.0_f32; 1536];
        let response_body = json!({
            "data": [
                {
                    "embedding": embedding_vec
                }
            ]
        })
        .to_string();

        let _mock = server
            .mock("POST", "/embeddings")
            .match_header("authorization", "Bearer test-key")
            .match_header("content-type", "application/json")
            .with_status(200)
            .with_body(response_body)
            .create();

        let provider = OpenAIEmbeddingProvider::new(
            "test-key".to_string(),
            Some(server.url()),
            "text-embedding-3-small".to_string(),
        )
        .expect("Failed to create OpenAI provider");

        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(provider.embed("Hello, world!"))
            .unwrap();

        assert_eq!(result.model, "text-embedding-3-small");
        assert_eq!(result.dimensions, 1536);
        assert_eq!(result.vector.len(), 1536);
    }
}

#[cfg(test)]
mod ollama_tests {
    use super::*;
    use mcp_context_browser::providers::embedding::OllamaEmbeddingProvider;

    #[test]
    fn test_ollama_provider_creation() {
        let provider = OllamaEmbeddingProvider::new(
            "http://localhost:11434".to_string(),
            "nomic-embed-text".to_string(),
        )
        .expect("Failed to create Ollama provider");

        assert_eq!(provider.provider_name(), "ollama");
        assert_eq!(provider.model(), "nomic-embed-text");
        assert_eq!(provider.dimensions(), 768);
        assert_eq!(provider.max_tokens(), 8192);
    }

    #[test]
    fn test_ollama_dimensions_by_model() {
        let models_and_dims = vec![
            ("nomic-embed-text", 768),
            ("all-minilm", 384),
            ("mxbai-embed-large", 1024),
            ("snowflake-arctic-embed", 768),
            ("unknown-model", 768), // fallback
        ];

        for (model, expected_dims) in models_and_dims {
            let provider = OllamaEmbeddingProvider::new(
                "http://localhost:11434".to_string(),
                model.to_string(),
            )
            .expect("Failed to create Ollama provider");
            assert_eq!(
                provider.dimensions(),
                expected_dims,
                "Model {} should have {} dimensions",
                model,
                expected_dims
            );
        }
    }

    #[test]
    fn test_ollama_max_tokens_by_model() {
        let models_and_tokens = vec![
            ("nomic-embed-text", 8192),
            ("all-minilm", 512),
            ("mxbai-embed-large", 512),
            ("snowflake-arctic-embed", 512),
            ("unknown-model", 8192), // fallback
        ];

        for (model, expected_tokens) in models_and_tokens {
            let provider = OllamaEmbeddingProvider::new(
                "http://localhost:11434".to_string(),
                model.to_string(),
            )
            .unwrap();
            assert_eq!(
                provider.max_tokens(),
                expected_tokens,
                "Model {} should support {} max tokens",
                model,
                expected_tokens
            );
        }
    }
}

#[cfg(test)]
mod voyageai_tests {
    use super::*;
    use mcp_context_browser::providers::embedding::VoyageAIEmbeddingProvider;

    #[test]
    fn test_voyageai_provider_creation() {
        let provider = VoyageAIEmbeddingProvider::new(
            "test-key".to_string(),
            Some("https://api.voyageai.com/v1".to_string()),
            "voyage-code-3".to_string(),
        );

        assert_eq!(provider.provider_name(), "voyageai");
        assert_eq!(provider.model(), "voyage-code-3");
        assert_eq!(provider.dimensions(), 1024);
        assert_eq!(provider.max_tokens(), 16000);
        assert_eq!(provider.api_key(), "test-key");
    }

    #[test]
    fn test_voyageai_dimensions_by_model() {
        let models_and_dims = vec![
            ("voyage-code-3", 1024),
            ("unknown-model", 1024), // fallback
        ];

        for (model, expected_dims) in models_and_dims {
            let provider =
                VoyageAIEmbeddingProvider::new("test-key".to_string(), None, model.to_string());
            assert_eq!(
                provider.dimensions(),
                expected_dims,
                "Model {} should have {} dimensions",
                model,
                expected_dims
            );
        }
    }

    #[test]
    fn test_voyageai_base_url() {
        let provider_no_url = VoyageAIEmbeddingProvider::new(
            "test-key".to_string(),
            None,
            "voyage-code-3".to_string(),
        );
        assert_eq!(provider_no_url.base_url(), "https://api.voyageai.com/v1");

        let provider_with_url = VoyageAIEmbeddingProvider::new(
            "test-key".to_string(),
            Some("https://custom.voyageai.com/v1".to_string()),
            "voyage-code-3".to_string(),
        );
        assert_eq!(
            provider_with_url.base_url(),
            "https://custom.voyageai.com/v1"
        );
    }
}

#[cfg(test)]
mod gemini_tests {
    use super::*;
    use mcp_context_browser::providers::embedding::GeminiEmbeddingProvider;

    #[test]
    fn test_gemini_provider_creation() {
        let provider = GeminiEmbeddingProvider::new(
            "test-key".to_string(),
            Some("https://generativelanguage.googleapis.com".to_string()),
            "gemini-embedding-001".to_string(),
        )
        .unwrap();

        assert_eq!(provider.provider_name(), "gemini");
        assert_eq!(provider.model(), "gemini-embedding-001");
        assert_eq!(provider.dimensions(), 768);
        assert_eq!(provider.max_tokens(), 2048);
        assert_eq!(provider.api_key(), "test-key");
    }

    #[test]
    fn test_gemini_dimensions_by_model() {
        let models_and_dims = vec![
            ("gemini-embedding-001", 768),
            ("text-embedding-004", 768),
            ("unknown-model", 768), // fallback
        ];

        for (model, expected_dims) in models_and_dims {
            let provider =
                GeminiEmbeddingProvider::new("test-key".to_string(), None, model.to_string())
                    .unwrap();
            assert_eq!(
                provider.dimensions(),
                expected_dims,
                "Model {} should have {} dimensions",
                model,
                expected_dims
            );
        }
    }

    #[test]
    fn test_gemini_api_model_name() {
        let provider = GeminiEmbeddingProvider::new(
            "test-key".to_string(),
            None,
            "models/gemini-embedding-001".to_string(),
        )
        .unwrap();

        // Should strip the "models/" prefix
        assert_eq!(provider.api_model_name(), "gemini-embedding-001");
    }

    #[test]
    fn test_gemini_base_url() {
        let provider_no_url = GeminiEmbeddingProvider::new(
            "test-key".to_string(),
            None,
            "gemini-embedding-001".to_string(),
        )
        .unwrap();
        assert_eq!(
            provider_no_url.base_url(),
            "https://generativelanguage.googleapis.com"
        );

        let provider_with_url = GeminiEmbeddingProvider::new(
            "test-key".to_string(),
            Some("https://custom.gemini.com".to_string()),
            "gemini-embedding-001".to_string(),
        )
        .unwrap();
        assert_eq!(provider_with_url.base_url(), "https://custom.gemini.com");
    }
}

#[cfg(test)]
mod provider_trait_tests {
    use super::*;
    use mcp_context_browser::providers::embedding::NullEmbeddingProvider;

    #[test]
    fn test_null_provider() {
        let provider = NullEmbeddingProvider::new();

        assert_eq!(provider.provider_name(), "null");
        assert_eq!(provider.dimensions(), 1);

        // Test embed single
        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(provider.embed("test"))
            .unwrap();
        assert_eq!(result.model, "null");
        assert_eq!(result.dimensions, 1);
        assert!(!result.vector.is_empty());

        // Test embed batch
        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(provider.embed_batch(&["test1".to_string(), "test2".to_string()]))
            .unwrap();
        assert_eq!(result.len(), 2);
        for embedding in result {
            assert_eq!(embedding.model, "null");
            assert_eq!(embedding.dimensions, 1);
            assert!(!embedding.vector.is_empty());
        }

        // Test empty batch
        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(provider.embed_batch(&[]))
            .unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_provider_consistency_validation() {
        // Test that all providers implement the trait consistently

        let null_provider = NullEmbeddingProvider::new();
        let openai_provider = OpenAIEmbeddingProvider::new(
            "test-key".to_string(),
            None,
            "text-embedding-3-small".to_string(),
        )
        .unwrap();
        let gemini_provider = GeminiEmbeddingProvider::new(
            "test-key".to_string(),
            None,
            "gemini-embedding-001".to_string(),
        )
        .unwrap();
        let voyageai_provider = VoyageAIEmbeddingProvider::new(
            "test-key".to_string(),
            None,
            "voyage-code-3".to_string(),
        );
        let ollama_provider = OllamaEmbeddingProvider::new(
            "http://localhost:11434".to_string(),
            "nomic-embed-text".to_string(),
        )
        .unwrap();

        let providers: Vec<&dyn EmbeddingProvider> = vec![
            &null_provider,
            &openai_provider,
            &gemini_provider,
            &voyageai_provider,
            &ollama_provider,
        ];

        // Test that all providers have non-empty names
        for provider in &providers {
            assert!(
                !provider.provider_name().is_empty(),
                "Provider name should not be empty"
            );
            assert!(
                provider.dimensions() > 0,
                "Provider dimensions should be positive"
            );
        }

        // Test provider-specific validations
        assert_eq!(null_provider.dimensions(), 1);
        assert_eq!(openai_provider.dimensions(), 1536);
        assert_eq!(gemini_provider.dimensions(), 768);
        assert_eq!(voyageai_provider.dimensions(), 1024);
        assert_eq!(ollama_provider.dimensions(), 768);

        // Test embed_batch with empty input (should return empty vec)
        for provider in &providers {
            let result = tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(provider.embed_batch(&[]))
                .unwrap();
            assert!(result.is_empty(), "Empty batch should return empty result");
        }
    }

    #[test]
    fn test_provider_url_configuration() {
        // Test URL configuration for providers that support custom URLs

        // OpenAI
        let openai_default = OpenAIEmbeddingProvider::new(
            "test-key".to_string(),
            None,
            "text-embedding-3-small".to_string(),
        )
        .unwrap();
        assert_eq!(openai_default.base_url(), "https://api.openai.com/v1");

        let openai_custom = OpenAIEmbeddingProvider::new(
            "test-key".to_string(),
            Some("https://custom.openai.com/v1".to_string()),
            "text-embedding-3-small".to_string(),
        )
        .unwrap();
        assert_eq!(openai_custom.base_url(), "https://custom.openai.com/v1");

        // Gemini
        let gemini_default = GeminiEmbeddingProvider::new(
            "test-key".to_string(),
            None,
            "gemini-embedding-001".to_string(),
        )
        .unwrap();
        assert_eq!(
            gemini_default.base_url(),
            "https://generativelanguage.googleapis.com"
        );

        let gemini_custom = GeminiEmbeddingProvider::new(
            "test-key".to_string(),
            Some("https://custom.gemini.com".to_string()),
            "gemini-embedding-001".to_string(),
        )
        .unwrap();
        assert_eq!(gemini_custom.base_url(), "https://custom.gemini.com");

        // VoyageAI
        let voyageai_default = VoyageAIEmbeddingProvider::new(
            "test-key".to_string(),
            None,
            "voyage-code-3".to_string(),
        );
        assert_eq!(voyageai_default.base_url(), "https://api.voyageai.com/v1");

        let voyageai_custom = VoyageAIEmbeddingProvider::new(
            "test-key".to_string(),
            Some("https://custom.voyageai.com".to_string()),
            "voyage-code-3".to_string(),
        );
        assert_eq!(voyageai_custom.base_url(), "https://custom.voyageai.com");
    }

    #[test]
    fn test_provider_model_validation() {
        // Test model-specific configurations

        // OpenAI models and their expected dimensions
        let openai_models = vec![
            ("text-embedding-3-small", 1536),
            ("text-embedding-3-large", 3072),
            ("text-embedding-ada-002", 1536),
            ("unknown-model", 1536), // fallback
        ];

        for (model, expected_dims) in openai_models {
            let provider =
                OpenAIEmbeddingProvider::new("test-key".to_string(), None, model.to_string())
                    .unwrap();
            assert_eq!(
                provider.dimensions(),
                expected_dims,
                "OpenAI model {} should have {} dimensions",
                model,
                expected_dims
            );
        }

        // Gemini models
        let gemini_models = vec![
            ("gemini-embedding-001", 768),
            ("models/gemini-embedding-001", 768), // with prefix
            ("text-embedding-004", 768),
            ("unknown-model", 768), // fallback
        ];

        for (model, expected_dims) in gemini_models {
            let provider =
                GeminiEmbeddingProvider::new("test-key".to_string(), None, model.to_string())
                    .unwrap();
            assert_eq!(
                provider.dimensions(),
                expected_dims,
                "Gemini model {} should have {} dimensions",
                model,
                expected_dims
            );
        }

        // Test Gemini model name stripping
        let gemini_with_prefix = GeminiEmbeddingProvider::new(
            "test-key".to_string(),
            None,
            "models/gemini-embedding-001".to_string(),
        )
        .unwrap();
        assert_eq!(gemini_with_prefix.api_model_name(), "gemini-embedding-001");

        // Ollama models
        let ollama_models = vec![
            ("nomic-embed-text", 768),
            ("all-minilm", 384),
            ("mxbai-embed-large", 1024),
            ("snowflake-arctic-embed", 768),
            ("unknown-model", 768), // fallback
        ];

        for (model, expected_dims) in ollama_models {
            let provider = OllamaEmbeddingProvider::new(
                "http://localhost:11434".to_string(),
                model.to_string(),
            )
            .unwrap();
            assert_eq!(
                provider.dimensions(),
                expected_dims,
                "Ollama model {} should have {} dimensions",
                model,
                expected_dims
            );
        }
    }

    #[test]
    fn test_provider_max_tokens_configuration() {
        // Test max tokens configuration for different models

        // OpenAI
        let openai_provider = OpenAIEmbeddingProvider::new(
            "test-key".to_string(),
            None,
            "text-embedding-3-small".to_string(),
        )
        .unwrap();
        assert_eq!(openai_provider.max_tokens(), 8192);

        // Gemini
        let gemini_provider = GeminiEmbeddingProvider::new(
            "test-key".to_string(),
            None,
            "gemini-embedding-001".to_string(),
        )
        .unwrap();
        assert_eq!(gemini_provider.max_tokens(), 2048);

        // VoyageAI
        let voyageai_provider = VoyageAIEmbeddingProvider::new(
            "test-key".to_string(),
            None,
            "voyage-code-3".to_string(),
        );
        assert_eq!(voyageai_provider.max_tokens(), 16000);

        // Ollama
        let ollama_provider = OllamaEmbeddingProvider::new(
            "http://localhost:11434".to_string(),
            "nomic-embed-text".to_string(),
        )
        .unwrap();
        assert_eq!(ollama_provider.max_tokens(), 8192);
    }

    #[test]
    fn test_provider_error_handling_consistency() {
        // Test that all providers handle errors consistently

        let null_provider = NullEmbeddingProvider::new();

        // Test empty text (should work for null provider)
        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(null_provider.embed(""));
        assert!(result.is_ok(), "Null provider should handle empty text");

        // Test very long text
        let long_text = "word ".repeat(10000);
        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(null_provider.embed(&long_text));
        assert!(result.is_ok(), "Null provider should handle long text");

        // Test batch with mixed empty/non-empty texts
        let texts = vec!["".to_string(), "short".to_string(), "a ".repeat(1000)];
        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(null_provider.embed_batch(&texts));
        assert!(
            result.is_ok(),
            "Null provider should handle mixed text lengths"
        );
        assert_eq!(
            result.unwrap().len(),
            3,
            "Should return embeddings for all inputs"
        );
    }

    #[test]
    fn test_provider_trait_compliance() {
        // Test that all providers properly implement the EmbeddingProvider trait

        // Only test NullEmbeddingProvider since others would require real API calls
        let null_provider = NullEmbeddingProvider::new();

        // Test basic trait methods
        assert!(
            !null_provider.provider_name().is_empty(),
            "NullProvider should have non-empty name"
        );
        assert!(
            null_provider.dimensions() > 0,
            "NullProvider should have positive dimensions"
        );

        // Test embed_batch with single item (should work)
        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(null_provider.embed_batch(&["test".to_string()]));
        assert!(
            result.is_ok(),
            "NullProvider should handle single item batch"
        );
        let embeddings = result.unwrap();
        assert_eq!(
            embeddings.len(),
            1,
            "NullProvider should return one embedding"
        );

        let embedding = &embeddings[0];
        assert!(
            !embedding.vector.is_empty(),
            "NullProvider should return non-empty vector"
        );
        assert_eq!(
            embedding.dimensions,
            null_provider.dimensions(),
            "NullProvider embedding dimensions should match provider"
        );

        // Test that other providers are structurally correct (just instantiation)
        let _openai_provider = OpenAIEmbeddingProvider::new(
            "test-key".to_string(),
            None,
            "text-embedding-3-small".to_string(),
        )
        .unwrap();
        let _gemini_provider = GeminiEmbeddingProvider::new(
            "test-key".to_string(),
            None,
            "gemini-embedding-001".to_string(),
        )
        .unwrap();
        let _voyageai_provider = VoyageAIEmbeddingProvider::new(
            "test-key".to_string(),
            None,
            "voyage-code-3".to_string(),
        );
        let _ollama_provider = OllamaEmbeddingProvider::new(
            "http://localhost:11434".to_string(),
            "nomic-embed-text".to_string(),
        )
        .unwrap();
    }

    #[test]
    fn test_provider_memory_safety() {
        // Test for potential memory issues and proper resource handling

        let provider = NullEmbeddingProvider::new();

        // Test with very large batch
        let large_batch: Vec<String> = (0..1000).map(|i| format!("text {}", i)).collect();
        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(provider.embed_batch(&large_batch));

        assert!(
            result.is_ok(),
            "Should handle large batches without memory issues"
        );
        let embeddings = result.unwrap();
        assert_eq!(
            embeddings.len(),
            1000,
            "Should return embeddings for all items in large batch"
        );

        // Test memory consistency
        for embedding in &embeddings {
            assert_eq!(
                embedding.vector.len(),
                1,
                "All embeddings should have consistent vector size"
            );
            assert_eq!(
                embedding.dimensions, 1,
                "All embeddings should have consistent dimensions"
            );
        }
    }

    #[cfg(test)]
    mod integration_tests {
        use super::*;

        /// Integration test for OpenAI API - only runs if OPENAI_API_KEY is set
        #[test]
        fn test_openai_real_api() {
            if std::env::var("OPENAI_LIVE_TEST").ok().as_deref() != Some("1") {
                println!(
                    "Skipping OpenAI integration test - set OPENAI_LIVE_TEST=1 to run against the real API"
                );
                return;
            }

            let api_key = match std::env::var("OPENAI_API_KEY") {
                Ok(key) => key,
                Err(_) => {
                    println!("Skipping OpenAI integration test - OPENAI_API_KEY not set");
                    return;
                }
            };

            println!("Running OpenAI integration test with real API...");

            let base_url = std::env::var("OPENAI_BASE_URL").ok();
            let provider = OpenAIEmbeddingProvider::new(
                api_key,
                base_url,
                "text-embedding-3-small".to_string(),
            )
            .unwrap();

            let runtime = tokio::runtime::Runtime::new().unwrap();

            // Test single embedding
            let result = runtime.block_on(provider.embed("Hello, world!"));
            assert!(result.is_ok(), "OpenAI embed should succeed");

            let embedding = result.unwrap();
            assert_eq!(embedding.model, "text-embedding-3-small");
            assert_eq!(embedding.dimensions, 1536);
            assert!(!embedding.vector.is_empty());
            assert_eq!(embedding.vector.len(), 1536);

            // Test batch embedding
            let texts = vec![
                "First test text".to_string(),
                "Second test text".to_string(),
                "Third test text".to_string(),
            ];
            let result = runtime.block_on(provider.embed_batch(&texts));
            assert!(result.is_ok(), "OpenAI batch embed should succeed");

            let embeddings = result.unwrap();
            assert_eq!(embeddings.len(), 3);
            for embedding in &embeddings {
                assert_eq!(embedding.model, "text-embedding-3-small");
                assert_eq!(embedding.dimensions, 1536);
                assert_eq!(embedding.vector.len(), 1536);
            }

            println!("✅ OpenAI integration test passed!");
        }

        /// Integration test for Gemini API - only runs if GEMINI_API_KEY is set
        #[test]
        fn test_gemini_real_api() {
            if let Ok(api_key) = std::env::var("GEMINI_API_KEY") {
                println!("Running Gemini integration test with real API...");

                let provider =
                    GeminiEmbeddingProvider::new(api_key, None, "gemini-embedding-001".to_string())
                        .unwrap();

                let runtime = tokio::runtime::Runtime::new().unwrap();

                // Test single embedding
                let result = runtime.block_on(provider.embed("Hello, world!"));
                assert!(result.is_ok(), "Gemini embed should succeed");

                let embedding = result.unwrap();
                assert_eq!(embedding.model, "gemini-embedding-001");
                assert_eq!(embedding.dimensions, 768);
                assert!(!embedding.vector.is_empty());
                assert_eq!(embedding.vector.len(), 768);

                // Test batch embedding
                let texts = vec![
                    "First test text".to_string(),
                    "Second test text".to_string(),
                ];
                let result = runtime.block_on(provider.embed_batch(&texts));
                assert!(result.is_ok(), "Gemini batch embed should succeed");

                let embeddings = result.unwrap();
                assert_eq!(embeddings.len(), 2);
                for embedding in &embeddings {
                    assert_eq!(embedding.model, "gemini-embedding-001");
                    assert_eq!(embedding.dimensions, 768);
                    assert_eq!(embedding.vector.len(), 768);
                }

                println!("✅ Gemini integration test passed!");
            } else {
                println!("Skipping Gemini integration test - GEMINI_API_KEY not set");
            }
        }

        /// Integration test for VoyageAI API - only runs if VOYAGE_API_KEY is set
        #[test]
        fn test_voyageai_real_api() {
            if let Ok(api_key) = std::env::var("VOYAGE_API_KEY") {
                println!("Running VoyageAI integration test with real API...");

                let provider =
                    VoyageAIEmbeddingProvider::new(api_key, None, "voyage-code-3".to_string());

                let runtime = tokio::runtime::Runtime::new().unwrap();

                // Test single embedding
                let result = runtime.block_on(provider.embed("Hello, world!"));
                assert!(result.is_ok(), "VoyageAI embed should succeed");

                let embedding = result.unwrap();
                assert_eq!(embedding.model, "voyage-code-3");
                assert_eq!(embedding.dimensions, 1024);
                assert!(!embedding.vector.is_empty());
                assert_eq!(embedding.vector.len(), 1024);

                // Test batch embedding
                let texts = vec![
                    "First test text".to_string(),
                    "Second test text".to_string(),
                ];
                let result = runtime.block_on(provider.embed_batch(&texts));
                assert!(result.is_ok(), "VoyageAI batch embed should succeed");

                let embeddings = result.unwrap();
                assert_eq!(embeddings.len(), 2);
                for embedding in &embeddings {
                    assert_eq!(embedding.model, "voyage-code-3");
                    assert_eq!(embedding.dimensions, 1024);
                    assert_eq!(embedding.vector.len(), 1024);
                }

                println!("✅ VoyageAI integration test passed!");
            } else {
                println!("Skipping VoyageAI integration test - VOYAGE_API_KEY not set");
            }
        }

        /// Integration test for Ollama - only runs if OLLAMA_URL is set
        #[test]
        fn test_ollama_real_api() {
            if let Ok(base_url) = std::env::var("OLLAMA_URL") {
                println!("Running Ollama integration test with real API...");

                let provider =
                    OllamaEmbeddingProvider::new(base_url, "nomic-embed-text".to_string()).unwrap();

                let runtime = tokio::runtime::Runtime::new().unwrap();

                // Test single embedding
                let result = runtime.block_on(provider.embed("Hello, world!"));
                assert!(result.is_ok(), "Ollama embed should succeed");

                let embedding = result.unwrap();
                assert_eq!(embedding.model, "nomic-embed-text");
                assert_eq!(embedding.dimensions, 768);
                assert!(!embedding.vector.is_empty());
                assert_eq!(embedding.vector.len(), 768);

                // Test batch embedding (Ollama does individual requests)
                let texts = vec![
                    "First test text".to_string(),
                    "Second test text".to_string(),
                ];
                let result = runtime.block_on(provider.embed_batch(&texts));
                assert!(result.is_ok(), "Ollama batch embed should succeed");

                let embeddings = result.unwrap();
                assert_eq!(embeddings.len(), 2);
                for embedding in &embeddings {
                    assert_eq!(embedding.model, "nomic-embed-text");
                    assert_eq!(embedding.dimensions, 768);
                    assert_eq!(embedding.vector.len(), 768);
                }

                println!("✅ Ollama integration test passed!");
            } else {
                println!("Skipping Ollama integration test - OLLAMA_URL not set");
            }
        }

        /// Performance test comparing all providers
        #[test]
        fn test_provider_performance_comparison() {
            println!("Running performance comparison test...");

            let null_provider = NullEmbeddingProvider::new();
            let runtime = tokio::runtime::Runtime::new().unwrap();

            let test_texts = vec![
                "This is a test sentence for performance benchmarking.".to_string(),
                "Another test sentence to measure embedding speed.".to_string(),
                "Performance testing with multiple sentences.".to_string(),
                "The quick brown fox jumps over the lazy dog.".to_string(),
                "Machine learning and artificial intelligence are transforming technology."
                    .to_string(),
            ];

            // Test NullProvider performance (baseline)
            let start = std::time::Instant::now();
            let result = runtime.block_on(null_provider.embed_batch(&test_texts));
            let null_duration = start.elapsed();

            assert!(
                result.is_ok(),
                "Null provider performance test should succeed"
            );
            let embeddings = result.unwrap();
            assert_eq!(embeddings.len(), test_texts.len());

            println!("📊 Performance Results:");
            println!("  Null Provider: {:?}", null_duration);
            println!(
                "  Throughput: {:.2} embeddings/sec",
                test_texts.len() as f64 / null_duration.as_secs_f64()
            );

            // Test with real providers if available
            if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
                if let Ok(openai_provider) = OpenAIEmbeddingProvider::new(
                    api_key,
                    None,
                    "text-embedding-3-small".to_string(),
                ) {
                    let start = std::time::Instant::now();
                    let result = runtime.block_on(openai_provider.embed_batch(&test_texts));
                    let openai_duration = start.elapsed();

                    if result.is_ok() {
                        println!("  OpenAI API: {:?}", openai_duration);
                        println!(
                            "  OpenAI Throughput: {:.2} embeddings/sec",
                            test_texts.len() as f64 / openai_duration.as_secs_f64()
                        );
                    }
                }
            }

            if let Ok(api_key) = std::env::var("GEMINI_API_KEY") {
                if let Ok(gemini_provider) =
                    GeminiEmbeddingProvider::new(api_key, None, "gemini-embedding-001".to_string())
                {
                    let start = std::time::Instant::now();
                    let result = runtime.block_on(gemini_provider.embed_batch(&test_texts));
                    let gemini_duration = start.elapsed();

                    if result.is_ok() {
                        println!("  Gemini API: {:?}", gemini_duration);
                        println!(
                            "  Gemini Throughput: {:.2} embeddings/sec",
                            test_texts.len() as f64 / gemini_duration.as_secs_f64()
                        );
                    }
                }
            }

            println!("✅ Performance comparison completed!");
        }
    }
}

#[cfg(test)]
mod factory_tests {
    use super::*;
    use mcp_context_browser::core::types::EmbeddingConfig;
    use mcp_context_browser::di::factory::{DefaultProviderFactory, ProviderFactory};

    #[test]
    fn test_supported_providers() {
        let factory = DefaultProviderFactory::new();
        let providers = factory.supported_embedding_providers();

        assert!(providers.contains(&"openai".to_string()));
        assert!(providers.contains(&"ollama".to_string()));
        assert!(providers.contains(&"voyageai".to_string()));
        assert!(providers.contains(&"gemini".to_string()));
        assert!(providers.contains(&"mock".to_string()));
    }

    #[test]
    fn test_create_openai_provider() {
        let factory = DefaultProviderFactory::new();
        let config = EmbeddingConfig {
            provider: "openai".to_string(),
            model: "text-embedding-3-small".to_string(),
            api_key: Some("test-key".to_string()),
            base_url: Some("https://api.openai.com/v1".to_string()),
            dimensions: Some(1536),
            max_tokens: Some(8192),
        };

        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(factory.create_embedding_provider(&config));

        assert!(result.is_ok());
        let provider = result.unwrap();
        assert_eq!(provider.provider_name(), "openai");
    }

    #[test]
    fn test_create_ollama_provider() {
        let factory = DefaultProviderFactory::new();
        let config = EmbeddingConfig {
            provider: "ollama".to_string(),
            model: "nomic-embed-text".to_string(),
            api_key: None,
            base_url: Some("http://localhost:11434".to_string()),
            dimensions: Some(768),
            max_tokens: Some(8192),
        };

        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(factory.create_embedding_provider(&config));

        assert!(result.is_ok());
        let provider = result.unwrap();
        assert_eq!(provider.provider_name(), "ollama");
    }

    #[test]
    fn test_create_voyageai_provider() {
        let factory = DefaultProviderFactory::new();
        let config = EmbeddingConfig {
            provider: "voyageai".to_string(),
            model: "voyage-code-3".to_string(),
            api_key: Some("test-key".to_string()),
            base_url: Some("https://api.voyageai.com/v1".to_string()),
            dimensions: Some(1024),
            max_tokens: Some(16000),
        };

        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(factory.create_embedding_provider(&config));

        assert!(result.is_ok());
        let provider = result.unwrap();
        assert_eq!(provider.provider_name(), "voyageai");
    }

    #[test]
    fn test_create_gemini_provider() {
        let factory = DefaultProviderFactory::new();
        let config = EmbeddingConfig {
            provider: "gemini".to_string(),
            model: "gemini-embedding-001".to_string(),
            api_key: Some("test-key".to_string()),
            base_url: Some("https://generativelanguage.googleapis.com".to_string()),
            dimensions: Some(768),
            max_tokens: Some(2048),
        };

        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(factory.create_embedding_provider(&config));

        assert!(result.is_ok());
        let provider = result.unwrap();
        assert_eq!(provider.provider_name(), "gemini");
    }

    #[test]
    fn test_create_unsupported_provider() {
        let factory = DefaultProviderFactory::new();
        let config = EmbeddingConfig {
            provider: "unsupported".to_string(),
            model: "test".to_string(),
            api_key: None,
            base_url: None,
            dimensions: None,
            max_tokens: None,
        };

        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(factory.create_embedding_provider(&config));

        assert!(result.is_err());
        match result {
            Err(Error::Config { message: msg }) => {
                assert!(msg.contains("Unsupported embedding provider"))
            }
            _ => panic!("Expected Config error"),
        }
    }

    #[test]
    fn test_create_provider_missing_api_key() {
        let factory = DefaultProviderFactory::new();
        let config = EmbeddingConfig {
            provider: "openai".to_string(),
            model: "text-embedding-3-small".to_string(),
            api_key: None, // Missing API key
            base_url: None,
            dimensions: None,
            max_tokens: None,
        };

        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(factory.create_embedding_provider(&config));

        assert!(result.is_err());
        match result {
            Err(Error::Config { message: msg }) => assert!(msg.contains("OpenAI API key required")),
            _ => panic!("Expected Config error"),
        }
    }
}
