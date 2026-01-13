use serde::{Deserialize, Serialize};
use validator::Validate;

/// Embedding provider configuration types (similar to Claude Context)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "provider")]
pub enum EmbeddingProviderConfig {
    #[serde(rename = "openai")]
    OpenAI {
        model: String,
        api_key: String,
        #[serde(default)]
        base_url: Option<String>,
        #[serde(default)]
        dimensions: Option<usize>,
        #[serde(default)]
        max_tokens: Option<usize>,
    },
    #[serde(rename = "ollama")]
    Ollama {
        model: String,
        #[serde(default)]
        host: Option<String>,
        #[serde(default)]
        dimensions: Option<usize>,
        #[serde(default)]
        max_tokens: Option<usize>,
    },
    #[serde(rename = "voyageai")]
    VoyageAI {
        model: String,
        api_key: String,
        #[serde(default)]
        dimensions: Option<usize>,
        #[serde(default)]
        max_tokens: Option<usize>,
    },
    #[serde(rename = "gemini")]
    Gemini {
        model: String,
        api_key: String,
        #[serde(default)]
        base_url: Option<String>,
        #[serde(default)]
        dimensions: Option<usize>,
        #[serde(default)]
        max_tokens: Option<usize>,
    },
    #[serde(rename = "mock")]
    Mock {
        #[serde(default)]
        dimensions: Option<usize>,
        #[serde(default)]
        max_tokens: Option<usize>,
    },
    #[serde(rename = "fastembed")]
    FastEmbed {
        #[serde(default)]
        model: Option<String>,
        #[serde(default)]
        dimensions: Option<usize>,
        #[serde(default)]
        max_tokens: Option<usize>,
    },
}

impl Validate for EmbeddingProviderConfig {
    fn validate(&self) -> std::result::Result<(), validator::ValidationErrors> {
        let mut errors = validator::ValidationErrors::new();
        match self {
            EmbeddingProviderConfig::OpenAI { model, api_key, .. } => {
                if model.is_empty() {
                    errors.add("model", validator::ValidationError::new("length"));
                }
                if api_key.is_empty() {
                    errors.add("api_key", validator::ValidationError::new("length"));
                }
            }
            EmbeddingProviderConfig::Ollama { model, .. } => {
                if model.is_empty() {
                    errors.add("model", validator::ValidationError::new("length"));
                }
            }
            EmbeddingProviderConfig::VoyageAI { model, api_key, .. } => {
                if model.is_empty() {
                    errors.add("model", validator::ValidationError::new("length"));
                }
                if api_key.is_empty() {
                    errors.add("api_key", validator::ValidationError::new("length"));
                }
            }
            EmbeddingProviderConfig::Gemini { model, api_key, .. } => {
                if model.is_empty() {
                    errors.add("model", validator::ValidationError::new("length"));
                }
                if api_key.is_empty() {
                    errors.add("api_key", validator::ValidationError::new("length"));
                }
            }
            _ => {}
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
