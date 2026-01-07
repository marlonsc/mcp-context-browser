//! Example demonstrating how to use the FastEmbedProvider
//!
//! This example shows how to create a FastEmbedProvider and use it
//! to generate embeddings for text without external API dependencies.

use mcp_context_browser::core::types::EmbeddingConfig;
use mcp_context_browser::factory::DefaultProviderFactory;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 FastEmbedProvider Example");
    println!("============================");

    // Create a configuration for FastEmbed
    let config = EmbeddingConfig {
        provider: "fastembed".to_string(),
        model: "AllMiniLML6V2".to_string(),
        api_key: None,
        base_url: None,
        dimensions: Some(384),
        max_tokens: Some(512),
    };

    println!("📋 Configuration:");
    println!("   Provider: {}", config.provider);
    println!("   Model: {}", config.model);
    println!("   Dimensions: {:?}", config.dimensions);

    // Create the provider factory
    let factory = DefaultProviderFactory::new();

    println!("\n🏭 Creating FastEmbedProvider...");

    match factory.create_embedding_provider(&config).await {
        Ok(provider) => {
            println!("✅ FastEmbedProvider created successfully!");
            println!("   Provider name: {}", provider.provider_name());
            println!("   Dimensions: {}", provider.dimensions());

            // Test embedding generation
            println!("\n🔍 Testing embedding generation...");

            let test_texts = vec![
                "Hello, world!".to_string(),
                "This is a test of the FastEmbed provider.".to_string(),
                "Local embeddings without external dependencies!".to_string(),
            ];

            match provider.embed_batch(&test_texts).await {
                Ok(embeddings) => {
                    println!("✅ Successfully generated {} embeddings!", embeddings.len());

                    for (i, embedding) in embeddings.iter().enumerate() {
                        println!("   Embedding {}: {} dimensions, model: {}",
                                i + 1,
                                embedding.dimensions,
                                embedding.model);

                        // Show first 5 values of the vector
                        let preview: Vec<String> = embedding.vector.iter()
                            .take(5)
                            .map(|v| format!("{:.4}", v))
                            .collect();
                        println!("   Vector preview: [{}...]", preview.join(", "));
                    }

                    // Test single embedding
                    println!("\n🧪 Testing single embedding...");
                    match provider.embed("Single test text").await {
                        Ok(embedding) => {
                            println!("✅ Single embedding generated successfully!");
                            println!("   Dimensions: {}, Model: {}", embedding.dimensions, embedding.model);
                        }
                        Err(e) => {
                            println!("❌ Single embedding failed: {}", e);
                        }
                    }

                    // Test health check
                    println!("\n🏥 Testing health check...");
                    match provider.health_check().await {
                        Ok(_) => {
                            println!("✅ Health check passed!");
                        }
                        Err(e) => {
                            println!("❌ Health check failed: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("❌ Batch embedding failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to create FastEmbedProvider: {}", e);
            println!("💡 Make sure you have internet connection for model download on first run");
        }
    }

    println!("\n✨ Example completed!");
    Ok(())
}