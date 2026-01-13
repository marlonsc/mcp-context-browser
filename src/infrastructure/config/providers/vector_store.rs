use serde::{Deserialize, Serialize};
use validator::Validate;

/// Vector store provider configuration types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "provider")]
pub enum VectorStoreProviderConfig {
    #[serde(rename = "edgevec")]
    EdgeVec {
        #[serde(default)]
        max_vectors: Option<usize>,
        #[serde(default)]
        collection: Option<String>,
        #[serde(default)]
        hnsw_m: Option<usize>,
        #[serde(default)]
        hnsw_ef_construction: Option<usize>,
        #[serde(default)]
        distance_metric: Option<String>,
        #[serde(default)]
        use_quantization: Option<bool>,
    },
    #[serde(rename = "milvus")]
    Milvus {
        address: String,
        #[serde(default)]
        token: Option<String>,
        #[serde(default)]
        collection: Option<String>,
        #[serde(default)]
        dimensions: Option<usize>,
    },
    #[serde(rename = "pinecone")]
    Pinecone {
        api_key: String,
        environment: String,
        index_name: String,
        #[serde(default)]
        dimensions: Option<usize>,
    },
    #[serde(rename = "qdrant")]
    Qdrant {
        url: String,
        #[serde(default)]
        api_key: Option<String>,
        #[serde(default)]
        collection: Option<String>,
        #[serde(default)]
        dimensions: Option<usize>,
    },
    #[serde(rename = "in-memory")]
    InMemory {
        #[serde(default)]
        dimensions: Option<usize>,
    },
    #[serde(rename = "filesystem")]
    Filesystem {
        #[serde(default)]
        base_path: Option<String>,
        #[serde(default)]
        max_vectors_per_shard: Option<usize>,
        #[serde(default)]
        dimensions: Option<usize>,
        #[serde(default)]
        compression_enabled: Option<bool>,
        #[serde(default)]
        index_cache_size: Option<usize>,
        #[serde(default)]
        memory_mapping_enabled: Option<bool>,
    },
}

impl Validate for VectorStoreProviderConfig {
    fn validate(&self) -> std::result::Result<(), validator::ValidationErrors> {
        let mut errors = validator::ValidationErrors::new();
        match self {
            VectorStoreProviderConfig::Milvus { address, .. } => {
                if address.is_empty() {
                    errors.add("address", validator::ValidationError::new("length"));
                }
            }
            VectorStoreProviderConfig::Pinecone {
                api_key,
                environment,
                index_name,
                ..
            } => {
                if api_key.is_empty() {
                    errors.add("api_key", validator::ValidationError::new("length"));
                }
                if environment.is_empty() {
                    errors.add("environment", validator::ValidationError::new("length"));
                }
                if index_name.is_empty() {
                    errors.add("index_name", validator::ValidationError::new("length"));
                }
            }
            VectorStoreProviderConfig::Qdrant { url, .. } => {
                if url.is_empty() {
                    errors.add("url", validator::ValidationError::new("length"));
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
