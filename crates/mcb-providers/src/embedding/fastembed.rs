//! FastEmbed Local Embedding Provider
//!
//! Implements the EmbeddingProvider port using the fastembed library for local
//! embedding generation. Uses ONNX models for inference without external API calls.

use async_trait::async_trait;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use tokio::sync::{mpsc, oneshot};

use mcb_domain::error::{Error, Result};
use mcb_domain::ports::providers::EmbeddingProvider;
use mcb_domain::value_objects::Embedding;

use crate::constants::EMBEDDING_DIMENSION_FASTEMBED_DEFAULT;

/// Messages for the FastEmbed actor
enum FastEmbedMessage {
    EmbedBatch {
        texts: Vec<String>,
        tx: oneshot::Sender<Result<Vec<Embedding>>>,
    },
}

/// FastEmbed local embedding provider using Actor pattern
///
/// Uses the Actor pattern to eliminate locks and ensure thread-safe access
/// to the underlying ONNX model. The model is initialized once and processes
/// embedding requests through a channel.
///
/// ## Example
///
/// ```rust,no_run
/// use mcb_providers::embedding::FastEmbedProvider;
///
/// let provider = FastEmbedProvider::new().expect("Failed to initialize");
/// // provider is now ready to embed texts locally
/// ```
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
        512
    }
}

#[async_trait]
impl EmbeddingProvider for FastEmbedProvider {
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>> {
        let (tx, rx) = oneshot::channel();
        self.sender
            .send(FastEmbedMessage::EmbedBatch {
                texts: texts.to_vec(),
                tx,
            })
            .await
            .map_err(|_| Error::embedding("FastEmbed actor channel closed"))?;

        rx.await
            .unwrap_or_else(|_| Err(Error::embedding("FastEmbed actor closed")))
    }

    fn dimensions(&self) -> usize {
        // AllMiniLML6V2 has 384 dimensions
        EMBEDDING_DIMENSION_FASTEMBED_DEFAULT
    }

    fn provider_name(&self) -> &str {
        "fastembed"
    }

    async fn health_check(&self) -> Result<()> {
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

/// Internal actor that processes embedding requests
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
                        Ok(res) => {
                            let model_name = self.model_name.clone();
                            Ok(res
                                .into_iter()
                                .map(|v| {
                                    let dimensions = v.len();
                                    Embedding {
                                        vector: v,
                                        model: model_name.clone(),
                                        dimensions,
                                    }
                                })
                                .collect())
                        }
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

// ============================================================================
// Auto-registration via linkme distributed slice
// ============================================================================

use std::sync::Arc;

use mcb_application::ports::registry::{
    EMBEDDING_PROVIDERS, EmbeddingProviderConfig, EmbeddingProviderEntry,
};
use mcb_domain::ports::providers::EmbeddingProvider as EmbeddingProviderPort;

/// Parse model name string to EmbeddingModel enum
fn parse_embedding_model(model_name: &str) -> EmbeddingModel {
    match model_name.to_lowercase().as_str() {
        "bgesmallenq" | "bge-small-en-q" => EmbeddingModel::BGESmallENV15Q,
        "bgesmallen" | "bge-small-en" => EmbeddingModel::BGESmallENV15,
        "bgesmallzh" | "bge-small-zh" => EmbeddingModel::BGESmallZHV15,
        "bgebasenq" | "bge-base-en-q" => EmbeddingModel::BGEBaseENV15Q,
        "bgebaseen" | "bge-base-en" => EmbeddingModel::BGEBaseENV15,
        "bgelargeen" | "bge-large-en" => EmbeddingModel::BGELargeENV15Q,
        "multilingual-e5-small" => EmbeddingModel::MultilingualE5Small,
        "multilingual-e5-large" => EmbeddingModel::MultilingualE5Large,
        _ => EmbeddingModel::AllMiniLML6V2, // default
    }
}

/// Factory function for creating FastEmbed provider instances.
fn fastembed_factory(
    config: &EmbeddingProviderConfig,
) -> std::result::Result<Arc<dyn EmbeddingProviderPort>, String> {
    let model_name = config
        .model
        .clone()
        .unwrap_or_else(|| "AllMiniLML6V2".to_string());

    let model = parse_embedding_model(&model_name);
    let provider = FastEmbedProvider::with_model(model)
        .map_err(|e| format!("Failed to create FastEmbed provider: {e}"))?;

    Ok(Arc::new(provider))
}

#[linkme::distributed_slice(EMBEDDING_PROVIDERS)]
static FASTEMBED_PROVIDER: EmbeddingProviderEntry = EmbeddingProviderEntry {
    name: "fastembed",
    description: "FastEmbed local provider (AllMiniLML6V2, BGESmallEN, etc.)",
    factory: fastembed_factory,
};
