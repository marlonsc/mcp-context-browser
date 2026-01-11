//! Tests for configuration example system
//!
//! Migrated from src/config_example.rs inline tests.
//! Tests configuration validation for providers.

use mcp_context_browser::config_example::{
    ConfigManager, EmbeddingProviderConfig, GlobalConfig, GlobalProviderConfig,
    ServerConfigExample, VectorStoreProviderConfig,
};

#[tokio::test]
async fn test_config_validation_openai() -> Result<(), Box<dyn std::error::Error>> {
    let config = GlobalConfig {
        server: ServerConfigExample::default(),
        providers: GlobalProviderConfig {
            embedding: EmbeddingProviderConfig::OpenAI {
                model: "text-embedding-3-small".to_string(),
                api_key: "sk-test".to_string(),
                base_url: None,
                dimensions: Some(1536),
                max_tokens: Some(8191),
            },
            vector_store: VectorStoreProviderConfig::Milvus {
                address: "http://localhost:19530".to_string(),
                token: None,
                collection: Some("test".to_string()),
                dimensions: Some(1536),
            },
        },
    };

    let manager = ConfigManager::new()?;
    assert!(manager.validate_config(&config).is_ok());
    Ok(())
}

#[tokio::test]
async fn test_config_validation_missing_api_key() -> Result<(), Box<dyn std::error::Error>> {
    let config = GlobalConfig {
        server: ServerConfigExample::default(),
        providers: GlobalProviderConfig {
            embedding: EmbeddingProviderConfig::OpenAI {
                model: "text-embedding-3-small".to_string(),
                api_key: "".to_string(), // Empty API key
                base_url: None,
                dimensions: Some(1536),
                max_tokens: Some(8191),
            },
            vector_store: VectorStoreProviderConfig::Milvus {
                address: "http://localhost:19530".to_string(),
                token: None,
                collection: Some("test".to_string()),
                dimensions: Some(1536),
            },
        },
    };

    let manager = ConfigManager::new()?;
    assert!(manager.validate_config(&config).is_err());
    Ok(())
}

#[tokio::test]
async fn test_config_validation_voyageai() -> Result<(), Box<dyn std::error::Error>> {
    let config = GlobalConfig {
        server: ServerConfigExample::default(),
        providers: GlobalProviderConfig {
            embedding: EmbeddingProviderConfig::VoyageAI {
                model: "voyage-code-3".to_string(),
                api_key: "voyage-test-key".to_string(),
                dimensions: Some(1024),
                max_tokens: Some(32000),
            },
            vector_store: VectorStoreProviderConfig::Milvus {
                address: "http://localhost:19530".to_string(),
                token: None,
                collection: Some("test".to_string()),
                dimensions: Some(1024),
            },
        },
    };

    let manager = ConfigManager::new()?;
    assert!(manager.validate_config(&config).is_ok());
    Ok(())
}

#[tokio::test]
async fn test_config_validation_voyageai_missing_api_key() -> Result<(), Box<dyn std::error::Error>>
{
    let config = GlobalConfig {
        server: ServerConfigExample::default(),
        providers: GlobalProviderConfig {
            embedding: EmbeddingProviderConfig::VoyageAI {
                model: "voyage-code-3".to_string(),
                api_key: "".to_string(), // Empty API key
                dimensions: Some(1024),
                max_tokens: Some(32000),
            },
            vector_store: VectorStoreProviderConfig::Milvus {
                address: "http://localhost:19530".to_string(),
                token: None,
                collection: Some("test".to_string()),
                dimensions: Some(1024),
            },
        },
    };

    let manager = ConfigManager::new()?;
    assert!(manager.validate_config(&config).is_err());
    Ok(())
}

#[tokio::test]
async fn test_config_validation_mock_provider() -> Result<(), Box<dyn std::error::Error>> {
    let config = GlobalConfig {
        server: ServerConfigExample::default(),
        providers: GlobalProviderConfig {
            embedding: EmbeddingProviderConfig::Mock {
                dimensions: Some(128),
                max_tokens: Some(512),
            },
            vector_store: VectorStoreProviderConfig::Milvus {
                address: "http://localhost:19530".to_string(),
                token: None,
                collection: Some("test".to_string()),
                dimensions: Some(128),
            },
        },
    };

    let manager = ConfigManager::new()?;
    assert!(manager.validate_config(&config).is_ok());
    Ok(())
}

#[tokio::test]
async fn test_config_validation_pinecone() -> Result<(), Box<dyn std::error::Error>> {
    let config = GlobalConfig {
        server: ServerConfigExample::default(),
        providers: GlobalProviderConfig {
            embedding: EmbeddingProviderConfig::OpenAI {
                model: "text-embedding-3-small".to_string(),
                api_key: "sk-test".to_string(),
                base_url: None,
                dimensions: Some(1536),
                max_tokens: Some(8191),
            },
            vector_store: VectorStoreProviderConfig::Pinecone {
                api_key: "pinecone-test-key".to_string(),
                environment: "us-east-1".to_string(),
                index_name: "test-index".to_string(),
                dimensions: Some(1536),
            },
        },
    };

    let manager = ConfigManager::new()?;
    assert!(manager.validate_config(&config).is_ok());
    Ok(())
}

#[tokio::test]
async fn test_config_validation_pinecone_missing_fields() -> Result<(), Box<dyn std::error::Error>>
{
    // Missing API key
    let config = GlobalConfig {
        server: ServerConfigExample::default(),
        providers: GlobalProviderConfig {
            embedding: EmbeddingProviderConfig::OpenAI {
                model: "text-embedding-3-small".to_string(),
                api_key: "sk-test".to_string(),
                base_url: None,
                dimensions: Some(1536),
                max_tokens: Some(8191),
            },
            vector_store: VectorStoreProviderConfig::Pinecone {
                api_key: "".to_string(), // Empty
                environment: "us-east-1".to_string(),
                index_name: "test-index".to_string(),
                dimensions: Some(1536),
            },
        },
    };

    let manager = ConfigManager::new()?;
    assert!(manager.validate_config(&config).is_err());

    // Missing environment
    let config = GlobalConfig {
        server: ServerConfigExample::default(),
        providers: GlobalProviderConfig {
            embedding: EmbeddingProviderConfig::OpenAI {
                model: "text-embedding-3-small".to_string(),
                api_key: "sk-test".to_string(),
                base_url: None,
                dimensions: Some(1536),
                max_tokens: Some(8191),
            },
            vector_store: VectorStoreProviderConfig::Pinecone {
                api_key: "pinecone-test-key".to_string(),
                environment: "".to_string(), // Empty
                index_name: "test-index".to_string(),
                dimensions: Some(1536),
            },
        },
    };

    assert!(manager.validate_config(&config).is_err());

    // Missing index name
    let config = GlobalConfig {
        server: ServerConfigExample::default(),
        providers: GlobalProviderConfig {
            embedding: EmbeddingProviderConfig::OpenAI {
                model: "text-embedding-3-small".to_string(),
                api_key: "sk-test".to_string(),
                base_url: None,
                dimensions: Some(1536),
                max_tokens: Some(8191),
            },
            vector_store: VectorStoreProviderConfig::Pinecone {
                api_key: "pinecone-test-key".to_string(),
                environment: "us-east-1".to_string(),
                index_name: "".to_string(), // Empty
                dimensions: Some(1536),
            },
        },
    };

    assert!(manager.validate_config(&config).is_err());
    Ok(())
}

#[tokio::test]
async fn test_config_validation_invalid_server_port() -> Result<(), Box<dyn std::error::Error>> {
    let config = GlobalConfig {
        server: ServerConfigExample {
            host: "127.0.0.1".to_string(),
            port: 0, // Invalid port
        },
        providers: GlobalProviderConfig {
            embedding: EmbeddingProviderConfig::Mock {
                dimensions: Some(128),
                max_tokens: Some(512),
            },
            vector_store: VectorStoreProviderConfig::Milvus {
                address: "http://localhost:19530".to_string(),
                token: None,
                collection: Some("test".to_string()),
                dimensions: Some(128),
            },
        },
    };

    let manager = ConfigManager::new()?;
    assert!(manager.validate_config(&config).is_err());
    Ok(())
}

#[test]
fn test_server_config_default() {
    let config = ServerConfigExample::default();
    assert_eq!(config.host, "127.0.0.1");
    assert_eq!(config.port, 3000);
}

#[tokio::test]
async fn test_config_validation_milvus_empty_address() -> Result<(), Box<dyn std::error::Error>> {
    let config = GlobalConfig {
        server: ServerConfigExample::default(),
        providers: GlobalProviderConfig {
            embedding: EmbeddingProviderConfig::Mock {
                dimensions: Some(128),
                max_tokens: Some(512),
            },
            vector_store: VectorStoreProviderConfig::Milvus {
                address: "".to_string(), // Empty address
                token: None,
                collection: Some("test".to_string()),
                dimensions: Some(128),
            },
        },
    };

    let manager = ConfigManager::new()?;
    assert!(manager.validate_config(&config).is_err());
    Ok(())
}
