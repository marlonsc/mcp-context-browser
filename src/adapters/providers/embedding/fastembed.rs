//! FastEmbed local embedding provider implementation
//!
//! This provider uses the fastembed library to generate embeddings locally
//! without requiring external API calls. It uses ONNX models for inference.

use crate::domain::error::{Error, Result};
use crate::domain::ports::EmbeddingProvider;
use crate::domain::types::Embedding;
use async_trait::async_trait;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use tokio::sync::{mpsc, oneshot};

/// Messages for the FastEmbed actor
enum FastEmbedMessage {
    EmbedBatch {
        texts: Vec<String>,
        tx: oneshot::Sender<Result<Vec<Embedding>>>,
    },
}

/// FastEmbed local embedding provider using Actor pattern to eliminate locks
pub struct FastEmbedProvider {
    sender: mpsc::Sender<FastEmbedMessage>,
    model_name: String,
}

impl FastEmbedProvider {
    /// Create a new FastEmbed provider with the default model (AllMiniLML6V2)
    pub fn new() -> Result<Self> {
        Self::with_model(EmbeddingModel::AllMiniLML6V2)
    }

    /// Create a new FastEmbed provider with a specific model
    pub fn with_model(model: EmbeddingModel) -> Result<Self> {
        let init_options = InitOptions::new(model.clone()).with_show_download_progress(true);
        Self::with_options(init_options)
    }

    /// Create a new FastEmbed provider with custom initialization options
    pub fn with_options(init_options: InitOptions) -> Result<Self> {
        let model_name = format!("{:?}", init_options.model_name);
        let text_embedding = TextEmbedding::try_new(init_options).map_err(|e| {
            Error::embedding(format!("Failed to initialize FastEmbed model: {}", e))
        })?;

        let (tx, rx) = mpsc::channel(100);
        let mut actor = FastEmbedActor::new(rx, text_embedding, model_name.clone());
        tokio::spawn(async move {
            actor.run().await;
        });

        Ok(Self {
            sender: tx,
            model_name,
        })
    }

    /// Get the model name
    pub fn model(&self) -> &str {
        &self.model_name
    }

    /// Get the maximum tokens supported by this model (approximate)
    pub fn max_tokens(&self) -> usize {
        // Most FastEmbed models support around 512 tokens
        // This is a conservative estimate
        512
    }
}

#[async_trait]
impl EmbeddingProvider for FastEmbedProvider {
    async fn embed(&self, text: &str) -> Result<Embedding> {
        let embeddings = self.embed_batch(&[text.to_string()]).await?;
        embeddings
            .into_iter()
            .next()
            .ok_or_else(|| Error::embedding("No embedding returned from FastEmbed".to_string()))
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .sender
            .send(FastEmbedMessage::EmbedBatch {
                texts: texts.to_vec(),
                tx,
            })
            .await;
        rx.await
            .unwrap_or_else(|_| Err(Error::internal("Actor closed")))
    }

    fn dimensions(&self) -> usize {
        // AllMiniLML6V2 has 384 dimensions
        // We'll query the model for actual dimensions
        // For now, return a reasonable default that matches AllMiniLML6V2
        384
    }

    fn provider_name(&self) -> &str {
        "fastembed"
    }

    async fn health_check(&self) -> Result<()> {
        // Try a simple embedding to verify the model works
        self.embed("health check").await?;
        Ok(())
    }
}

impl Clone for FastEmbedProvider {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            model_name: self.model_name.clone(),
        }
    }
}

struct FastEmbedActor {
    receiver: mpsc::Receiver<FastEmbedMessage>,
    model: TextEmbedding,
    model_name: String,
}

impl FastEmbedActor {
    fn new(
        receiver: mpsc::Receiver<FastEmbedMessage>,
        model: TextEmbedding,
        model_name: String,
    ) -> Self {
        Self {
            receiver,
            model,
            model_name,
        }
    }

    async fn run(&mut self) {
        while let Some(msg) = self.receiver.recv().await {
            match msg {
                FastEmbedMessage::EmbedBatch { texts, tx } => {
                    let text_refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();
                    let embeddings_result = self.model.embed(text_refs, None);
                    let result = match embeddings_result {
                        Ok(res) => Ok(res
                            .into_iter()
                            .map(|v| Embedding {
                                vector: v.clone(),
                                model: self.model_name.clone(),
                                dimensions: v.len(),
                            })
                            .collect()),
                        Err(e) => Err(Error::embedding(format!(
                            "FastEmbed embedding failed: {}",
                            e
                        ))),
                    };
                    let _ = tx.send(result);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fastembed_provider_creation() {
        let provider = FastEmbedProvider::new();
        assert!(provider.is_ok());
    }

    #[tokio::test]
    async fn test_fastembed_provider_dimensions(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let provider = FastEmbedProvider::new()?;
        assert_eq!(provider.dimensions(), 384);
        assert_eq!(provider.provider_name(), "fastembed");
        Ok(())
    }

    #[tokio::test]
    async fn test_fastembed_provider_embed() -> std::result::Result<(), Box<dyn std::error::Error>>
    {
        let provider = FastEmbedProvider::new()?;
        let embedding = provider.embed("test text").await?;
        assert_eq!(embedding.dimensions, 384);
        assert_eq!(embedding.model, "AllMiniLML6V2");
        assert_eq!(embedding.vector.len(), 384);
        Ok(())
    }

    #[tokio::test]
    async fn test_fastembed_provider_embed_batch(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let provider = FastEmbedProvider::new()?;
        let texts = vec!["test text 1".to_string(), "test text 2".to_string()];
        let embeddings = provider.embed_batch(&texts).await?;
        assert_eq!(embeddings.len(), 2);

        for embedding in embeddings {
            assert_eq!(embedding.dimensions, 384);
            assert_eq!(embedding.model, "AllMiniLML6V2");
            assert_eq!(embedding.vector.len(), 384);
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_fastembed_provider_health_check(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let provider = FastEmbedProvider::new()?;
        provider.health_check().await?;
        Ok(())
    }
}
