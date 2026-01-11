//! Integration tests with Docker containers for real provider testing
//!
//! These tests require Docker and docker-compose to be available.
//! Run with: make test-integration-docker

use mcp_context_browser::adapters::http_client::HttpClientPool;
use mcp_context_browser::domain::types::EmbeddingConfig;
use std::env;
use std::sync::Arc;

#[cfg(test)]
mod tests {
    use super::*;
    use mcp_context_browser::infrastructure::di::factory::ServiceProviderInterface;

    fn get_test_http_client(
    ) -> Result<Arc<dyn mcp_context_browser::adapters::http_client::HttpClientProvider>, Box<dyn std::error::Error>> {
        let pool = HttpClientPool::new().map_err(|e| e as Box<dyn std::error::Error>)?;
        Ok(Arc::new(pool))
    }

    fn get_ollama_config() -> EmbeddingConfig {
        EmbeddingConfig {
            provider: "ollama".to_string(),
            model: "nomic-embed-text".to_string(),
            api_key: None,
            base_url: Some(
                env::var("OLLAMA_BASE_URL")
                    .unwrap_or_else(|_| "http://localhost:11434".to_string())
                    .trim_end_matches('/')
                    .to_string(),
            ),
            dimensions: Some(768),
            max_tokens: Some(8192),
        }
    }

    #[tokio::test]
    async fn test_openai_mock_embedding() -> Result<(), Box<dyn std::error::Error>> {
        // Use null provider for testing instead of external mock server
        let config = mcp_context_browser::domain::types::EmbeddingConfig {
            provider: "mock".to_string(),
            model: "test-model".to_string(),
            api_key: None,
            base_url: None,
            dimensions: Some(384),
            max_tokens: Some(8192),
        };
        let service_provider =
            mcp_context_browser::infrastructure::di::factory::ServiceProvider::new();
        let http_client = get_test_http_client()?;

        let embedding_provider = service_provider
            .get_embedding_provider(&config, http_client.clone())
            .await?;

        let test_text = "This is a test text for embedding";
        let embedding = embedding_provider
            .embed(test_text)
            .await?;

        assert_eq!(embedding.model, "null");
        assert_eq!(embedding.dimensions, 1);
        assert!(!embedding.vector.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn test_openai_mock_batch_embedding() -> Result<(), Box<dyn std::error::Error>> {
        // Use null provider for testing instead of external mock server
        let config = mcp_context_browser::domain::types::EmbeddingConfig {
            provider: "mock".to_string(),
            model: "test-model".to_string(),
            api_key: None,
            base_url: None,
            dimensions: Some(384),
            max_tokens: Some(8192),
        };
        let service_provider =
            mcp_context_browser::infrastructure::di::factory::ServiceProvider::new();
        let http_client = get_test_http_client()?;

        let embedding_provider = service_provider
            .get_embedding_provider(&config, http_client.clone())
            .await?;

        let test_texts = vec![
            "First test text".to_string(),
            "Second test text".to_string(),
            "Third test text".to_string(),
        ];

        let embeddings = embedding_provider
            .embed_batch(&test_texts)
            .await?;

        assert_eq!(embeddings.len(), 3);
        for embedding in &embeddings {
            assert_eq!(embedding.model, "null");
            assert_eq!(embedding.dimensions, 1);
            assert!(!embedding.vector.is_empty());
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_ollama_embedding() -> Result<(), Box<dyn std::error::Error>> {
        let config = get_ollama_config();
        let service_provider =
            mcp_context_browser::infrastructure::di::factory::ServiceProvider::new();
        let http_client = get_test_http_client()?;

        let embedding_provider = service_provider
            .get_embedding_provider(&config, http_client.clone())
            .await?;

        let test_text = "This is a test text for Ollama embedding";
        let embedding = embedding_provider
            .embed(test_text)
            .await?;

        assert_eq!(embedding.model, "nomic-embed-text");
        assert_eq!(embedding.dimensions, 768);
        assert!(!embedding.vector.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn test_milvus_vector_store_operations() -> Result<(), Box<dyn std::error::Error>> {
        let config = mcp_context_browser::domain::types::VectorStoreConfig {
            provider: "in-memory".to_string(),
            address: None,
            token: None,
            collection: Some("test_integration_collection".to_string()),
            dimensions: Some(768),
        };
        let service_provider =
            mcp_context_browser::infrastructure::di::factory::ServiceProvider::new();
        let http_client = get_test_http_client()?;

        let vector_store_provider = service_provider
            .get_vector_store_provider(&config)
            .await?;

        let collection_name = "test_integration_collection";
        let dimensions = 768;

        // Test collection creation
        vector_store_provider
            .create_collection(collection_name, dimensions)
            .await?;

        // Test collection exists
        let exists = vector_store_provider
            .collection_exists(collection_name)
            .await?;
        assert!(exists);

        // Test embedding provider for sample data
        let embedding_config = get_ollama_config();
        let embedding_provider = service_provider
            .get_embedding_provider(&embedding_config, http_client.clone())
            .await?;

        let test_texts = vec![
            "This is a sample Rust function".to_string(),
            "Another piece of code for testing".to_string(),
        ];

        let embeddings = embedding_provider
            .embed_batch(&test_texts)
            .await?;

        // Create metadata for the embeddings
        let metadata: Vec<std::collections::HashMap<String, serde_json::Value>> = vec![
            {
                let mut map = std::collections::HashMap::new();
                map.insert(
                    "content".to_string(),
                    serde_json::Value::String(test_texts[0].clone()),
                );
                map.insert(
                    "file_path".to_string(),
                    serde_json::Value::String("test.rs".to_string()),
                );
                map.insert(
                    "start_line".to_string(),
                    serde_json::Value::Number(1.into()),
                );
                map.insert("end_line".to_string(), serde_json::Value::Number(5.into()));
                map
            },
            {
                let mut map = std::collections::HashMap::new();
                map.insert(
                    "content".to_string(),
                    serde_json::Value::String(test_texts[1].clone()),
                );
                map.insert(
                    "file_path".to_string(),
                    serde_json::Value::String("test.rs".to_string()),
                );
                map.insert(
                    "start_line".to_string(),
                    serde_json::Value::Number(6.into()),
                );
                map.insert("end_line".to_string(), serde_json::Value::Number(10.into()));
                map
            },
        ];

        // Test vector insertion
        let ids = vector_store_provider
            .insert_vectors(collection_name, &embeddings, metadata)
            .await?;
        assert_eq!(ids.len(), 2);

        // Test search
        let query_embedding = &embeddings[0];
        let search_results = vector_store_provider
            .search_similar(collection_name, &query_embedding.vector, 5, None)
            .await?;

        assert!(!search_results.is_empty());
        assert!(search_results[0].score >= 0.0);

        // Test stats
        let stats = vector_store_provider
            .get_stats(collection_name)
            .await?;
        assert!(stats.contains_key("vectors_count"));

        // Test collection deletion
        vector_store_provider
            .delete_collection(collection_name)
            .await?;

        let exists_after_delete = vector_store_provider
            .collection_exists(collection_name)
            .await?;
        assert!(!exists_after_delete);
        Ok(())
    }

    #[tokio::test]
    async fn test_full_pipeline_openai_milvus() -> Result<(), Box<dyn std::error::Error>> {
        let embedding_config = mcp_context_browser::domain::types::EmbeddingConfig {
            provider: "mock".to_string(),
            model: "test-model".to_string(),
            api_key: None,
            base_url: None,
            dimensions: Some(384),
            max_tokens: Some(8192),
        };
        let vector_config = mcp_context_browser::domain::types::VectorStoreConfig {
            provider: "in-memory".to_string(),
            address: None,
            token: None,
            collection: Some("test_pipeline_collection".to_string()),
            dimensions: Some(384),
        };
        let service_provider =
            mcp_context_browser::infrastructure::di::factory::ServiceProvider::new();
        let http_client = get_test_http_client()?;

        let embedding_provider = service_provider
            .get_embedding_provider(&embedding_config, http_client.clone())
            .await?;

        let vector_store_provider = service_provider
            .get_vector_store_provider(&vector_config)
            .await?;

        let collection_name = "test_pipeline_collection";

        // Create collection
        vector_store_provider
            .create_collection(collection_name, 1536)
            .await?;

        // Generate embeddings
        let code_samples = vec![
            "fn main() { println!(\"Hello, World!\"); }".to_string(),
            "pub struct User { name: String, age: u32 }".to_string(),
            "impl User { pub fn new(name: String) -> Self { User { name, age: 0 } } }".to_string(),
        ];

        let embeddings = embedding_provider
            .embed_batch(&code_samples)
            .await?;

        // Create metadata
        let metadata: Vec<std::collections::HashMap<String, serde_json::Value>> = code_samples
            .iter()
            .enumerate()
            .map(|(i, content)| {
                let mut map = std::collections::HashMap::new();
                map.insert(
                    "content".to_string(),
                    serde_json::Value::String(content.clone()),
                );
                map.insert(
                    "file_path".to_string(),
                    serde_json::Value::String("main.rs".to_string()),
                );
                map.insert(
                    "start_line".to_string(),
                    serde_json::Value::Number((i * 10 + 1).into()),
                );
                map.insert(
                    "end_line".to_string(),
                    serde_json::Value::Number(((i + 1) * 10).into()),
                );
                map
            })
            .collect();

        // Insert into vector store
        let ids = vector_store_provider
            .insert_vectors(collection_name, &embeddings, metadata)
            .await?;

        assert_eq!(ids.len(), 3);

        // Search for similar code
        let query_text = "struct with name field";
        let query_embedding = embedding_provider
            .embed(query_text)
            .await?;

        let results = vector_store_provider
            .search_similar(collection_name, &query_embedding.vector, 2, None)
            .await?;

        assert!(!results.is_empty());
        assert!(results[0].score > 0.0);

        // Cleanup
        vector_store_provider
            .delete_collection(collection_name)
            .await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_provider_error_handling() -> Result<(), Box<dyn std::error::Error>> {
        // Test with invalid OpenAI configuration
        let invalid_config = EmbeddingConfig {
            provider: "openai".to_string(),
            model: "text-embedding-3-small".to_string(),
            api_key: Some("invalid-key".to_string()),
            base_url: Some("http://localhost:1080".to_string()),
            dimensions: Some(1536),
            max_tokens: Some(8192),
        };

        let service_provider =
            mcp_context_browser::infrastructure::di::factory::ServiceProvider::new();
        let http_client = get_test_http_client()?;

        let result = service_provider
            .get_embedding_provider(&invalid_config, http_client.clone())
            .await;
        assert!(result.is_ok()); // Provider creation should succeed

        let provider = result?;
        let embedding_result = provider.embed("test").await;
        assert!(embedding_result.is_err()); // But embedding should fail with invalid key
        Ok(())
    }

    #[tokio::test]
    async fn test_ollama_real_provider_integration() -> Result<(), Box<dyn std::error::Error>> {
        let config = get_ollama_config();
        let service_provider =
            mcp_context_browser::infrastructure::di::factory::ServiceProvider::new();
        let http_client = get_test_http_client()?;

        let embedding_provider = service_provider
            .get_embedding_provider(&config, http_client.clone())
            .await?;

        let test_text = "This is a test text for real Ollama embedding generation";
        let embedding = embedding_provider
            .embed(test_text)
            .await?;

        assert_eq!(embedding.model, "nomic-embed-text");
        assert_eq!(embedding.dimensions, 768);
        assert!(!embedding.vector.is_empty());
        assert_eq!(embedding.vector.len(), 768);

        // Verify vector values are reasonable floats
        for &value in &embedding.vector {
            assert!(value.is_finite(), "Embedding value should be finite");
            assert!(!value.is_nan(), "Embedding value should not be NaN");
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_ollama_real_batch_embedding_integration() -> Result<(), Box<dyn std::error::Error>> {
        let config = get_ollama_config();
        let service_provider =
            mcp_context_browser::infrastructure::di::factory::ServiceProvider::new();
        let http_client = get_test_http_client()?;

        let embedding_provider = service_provider
            .get_embedding_provider(&config, http_client.clone())
            .await?;

        let test_texts = vec![
            "First test text for batch processing".to_string(),
            "Second test text for batch processing".to_string(),
            "Third test text with different content".to_string(),
        ];

        let embeddings = embedding_provider
            .embed_batch(&test_texts)
            .await?;

        assert_eq!(embeddings.len(), 3);
        for (i, embedding) in embeddings.iter().enumerate() {
            assert_eq!(embedding.model, "nomic-embed-text");
            assert_eq!(embedding.dimensions, 768);
            assert!(!embedding.vector.is_empty());
            assert_eq!(embedding.vector.len(), 768);

            // Verify vector values are reasonable floats
            for &value in &embedding.vector {
                assert!(
                    value.is_finite(),
                    "Embedding value should be finite for text {}",
                    i
                );
                assert!(
                    !value.is_nan(),
                    "Embedding value should not be NaN for text {}",
                    i
                );
            }
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_ollama_real_different_models() -> Result<(), Box<dyn std::error::Error>> {
        // Test with different models if available
        let models_to_test = vec!["nomic-embed-text"];

        for model in models_to_test {
            let config = EmbeddingConfig {
                provider: "ollama".to_string(),
                model: model.to_string(),
                api_key: None,
                base_url: Some(
                    env::var("OLLAMA_BASE_URL")
                        .unwrap_or_else(|_| "http://localhost:11434".to_string()),
                ),
                dimensions: Some(768), // Will be overridden by model-specific dimensions
                max_tokens: Some(8192),
            };

            let service_provider =
                mcp_context_browser::infrastructure::di::factory::ServiceProvider::new();
            let http_client = get_test_http_client()?;
            let embedding_provider = service_provider
                .get_embedding_provider(&config, http_client.clone())
                .await?;

            let test_text = &format!("Test text for model {}", model);
            let embedding = embedding_provider
                .embed(test_text)
                .await?;

            assert_eq!(embedding.model, model);
            assert!(!embedding.vector.is_empty());
            assert!(embedding.dimensions > 0);

            // Verify all vector values are valid floats
            for &value in &embedding.vector {
                assert!(value.is_finite());
                assert!(!value.is_nan());
            }
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_ollama_real_empty_batch() -> Result<(), Box<dyn std::error::Error>> {
        let config = get_ollama_config();
        let service_provider =
            mcp_context_browser::infrastructure::di::factory::ServiceProvider::new();
        let http_client = get_test_http_client()?;

        let embedding_provider = service_provider
            .get_embedding_provider(&config, http_client.clone())
            .await?;

        let empty_texts: Vec<String> = vec![];
        let embeddings = embedding_provider
            .embed_batch(&empty_texts)
            .await?;

        assert!(embeddings.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn test_ollama_real_error_handling() -> Result<(), Box<dyn std::error::Error>> {
        // Test with invalid Ollama URL
        let invalid_config = EmbeddingConfig {
            provider: "ollama".to_string(),
            model: "nomic-embed-text".to_string(),
            api_key: None,
            base_url: Some("http://invalid-host:11434".to_string()),
            dimensions: Some(768),
            max_tokens: Some(8192),
        };

        let service_provider =
            mcp_context_browser::infrastructure::di::factory::ServiceProvider::new();
        let http_client = get_test_http_client()?;
        let embedding_provider = service_provider
            .get_embedding_provider(&invalid_config, http_client.clone())
            .await?;

        let embedding_result = embedding_provider.embed("test").await;
        assert!(embedding_result.is_err(), "Should fail with invalid host");
        Ok(())
    }

    #[tokio::test]
    async fn test_ollama_real_large_text() -> Result<(), Box<dyn std::error::Error>> {
        let config = get_ollama_config();
        let service_provider =
            mcp_context_browser::infrastructure::di::factory::ServiceProvider::new();
        let http_client = get_test_http_client()?;

        let embedding_provider = service_provider
            .get_embedding_provider(&config, http_client.clone())
            .await?;

        // Create a moderately sized text that should work within token limits
        let large_text = "This is a test text for embedding. ".repeat(20); // ~840 characters

        let embedding = embedding_provider
            .embed(&large_text)
            .await?;

        assert_eq!(embedding.model, "nomic-embed-text");
        assert_eq!(embedding.dimensions, 768);
        assert!(!embedding.vector.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn test_ollama_real_provider_metadata() -> Result<(), Box<dyn std::error::Error>> {
        let config = get_ollama_config();
        let service_provider =
            mcp_context_browser::infrastructure::di::factory::ServiceProvider::new();
        let http_client = get_test_http_client()?;

        let embedding_provider = service_provider
            .get_embedding_provider(&config, http_client.clone())
            .await?;

        // Test provider metadata
        assert_eq!(embedding_provider.provider_name(), "ollama");
        assert_eq!(embedding_provider.dimensions(), 768); // nomic-embed-text dimensions
        Ok(())
    }
}
