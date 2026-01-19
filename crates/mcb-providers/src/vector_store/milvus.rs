//! Milvus vector store provider implementation
//!
//! High-performance cloud vector database using Milvus.
//! Supports production-scale vector storage with automatic indexing and distributed search.

use crate::constants::{
    MILVUS_FIELD_VARCHAR_MAX_LENGTH, MILVUS_IVFFLAT_NLIST, MILVUS_METADATA_VARCHAR_MAX_LENGTH,
};
use crate::utils::JsonExt;
use async_trait::async_trait;
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::providers::{VectorStoreAdmin, VectorStoreProvider};
use mcb_domain::value_objects::{Embedding, SearchResult};
use milvus::client::Client;
use milvus::data::FieldColumn;
use milvus::proto::schema::DataType;
use milvus::schema::{CollectionSchemaBuilder, FieldSchema};
use milvus::value::{Value, ValueVec};
use std::borrow::Cow;
use std::collections::HashMap;

/// Milvus vector store provider implementation
pub struct MilvusVectorStoreProvider {
    client: Client,
}

/// Default connection timeout in seconds
const DEFAULT_TIMEOUT_SECS: u64 = 10;

impl MilvusVectorStoreProvider {
    /// Helper method to convert Milvus errors to domain errors
    fn map_milvus_error<T, E: std::fmt::Display>(
        result: std::result::Result<T, E>,
        operation: &str,
    ) -> Result<T> {
        result.map_err(|e| Error::vector_db(format!("Failed to {}: {}", operation, e)))
    }

    /// Create a new Milvus vector store provider
    ///
    /// # Arguments
    /// * `address` - Milvus server address (e.g., "http://localhost:19530")
    /// * `token` - Optional authentication token
    /// * `timeout_secs` - Connection timeout in seconds (default: 10)
    pub async fn new(
        address: String,
        _token: Option<String>,
        timeout_secs: Option<u64>,
    ) -> Result<Self> {
        // Ensure the address has a scheme (required by tonic transport)
        let endpoint = if address.starts_with("http://") || address.starts_with("https://") {
            address
        } else {
            format!("http://{}", address)
        };

        let timeout = timeout_secs.unwrap_or(DEFAULT_TIMEOUT_SECS);
        let timeout_duration = std::time::Duration::from_secs(timeout);

        let client = tokio::time::timeout(timeout_duration, Client::new(endpoint.clone()))
            .await
            .map_err(|_| {
                Error::vector_db(format!(
                    "Milvus connection timed out after {} seconds",
                    timeout
                ))
            })?
            .map_err(|e| {
                Error::vector_db(format!(
                    "Failed to connect to Milvus at {}: {}",
                    endpoint, e
                ))
            })?;

        Ok(Self { client })
    }
}

#[async_trait]
impl VectorStoreAdmin for MilvusVectorStoreProvider {
    async fn collection_exists(&self, name: &str) -> Result<bool> {
        Self::map_milvus_error(self.client.has_collection(name).await, "check collection")
    }

    async fn get_stats(&self, collection: &str) -> Result<HashMap<String, serde_json::Value>> {
        let stats = self
            .client
            .get_collection_stats(collection)
            .await
            .map_err(|e| {
                Error::vector_db(format!(
                    "Failed to get stats for collection '{}': {}",
                    collection, e
                ))
            })?;

        let mut result = HashMap::new();
        result.insert("collection".to_string(), serde_json::json!(collection));
        result.insert("status".to_string(), serde_json::json!("active"));

        if let Some(count_str) = stats.get("row_count") {
            if let Ok(count) = count_str.parse::<i64>() {
                result.insert("vectors_count".to_string(), serde_json::json!(count));
            }
        }

        result.insert("provider".to_string(), serde_json::json!("milvus"));
        Ok(result)
    }

    async fn flush(&self, collection: &str) -> Result<()> {
        // Retry flush with backoff to handle rate limiting
        let mut last_error = None;
        for attempt in 0..3 {
            match self.client.flush_collections(vec![collection]).await {
                Ok(_) => return Ok(()),
                Err(e) => {
                    let err_str = e.to_string();
                    if err_str.contains("RateLimit") || err_str.contains("rate limit") {
                        tracing::debug!("Flush attempt {} rate limited, retrying...", attempt + 1);
                        last_error = Some(e);
                        tokio::time::sleep(std::time::Duration::from_millis(
                            1000 * (attempt + 1) as u64,
                        ))
                        .await;
                        continue;
                    }
                    return Err(Error::vector_db(format!(
                        "Failed to flush collection: {}",
                        e
                    )));
                }
            }
        }

        if let Some(e) = last_error {
            return Err(Error::vector_db(format!(
                "Failed to flush collection after retries: {}",
                e
            )));
        }

        Ok(())
    }

    fn provider_name(&self) -> &str {
        "milvus"
    }
}

#[async_trait]
impl VectorStoreProvider for MilvusVectorStoreProvider {
    async fn create_collection(&self, name: &str, dimensions: usize) -> Result<()> {
        let schema = CollectionSchemaBuilder::new(name, &format!("Collection for {}", name))
            .add_field(FieldSchema::new_primary_int64(
                "id",
                "primary key field",
                true, // Use auto_id
            ))
            .add_field(FieldSchema::new_float_vector(
                "vector",
                "feature field",
                dimensions as i64,
            ))
            .add_field(FieldSchema::new_varchar(
                "file_path",
                "file path",
                MILVUS_FIELD_VARCHAR_MAX_LENGTH,
            ))
            .add_field(FieldSchema::new_int64("start_line", "start line"))
            .add_field(FieldSchema::new_varchar(
                "content",
                "content",
                MILVUS_METADATA_VARCHAR_MAX_LENGTH,
            ))
            .build()
            .map_err(|e| Error::vector_db(format!("Failed to create schema: {}", e)))?;

        Self::map_milvus_error(
            self.client.create_collection(schema, None).await,
            "create collection",
        )?;

        // Wait for Milvus to sync collection metadata
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        // Create index on the vector field for efficient search
        use milvus::index::{IndexParams, IndexType, MetricType};

        let index_params = IndexParams::new(
            "vector_index".to_string(),
            IndexType::IvfFlat,
            MetricType::L2,
            HashMap::from([("nlist".to_string(), MILVUS_IVFFLAT_NLIST.to_string())]),
        );

        // Retry index creation with backoff to handle eventual consistency
        let mut last_error = None;
        for attempt in 0..3 {
            match self
                .client
                .create_index(name, "vector", index_params.clone())
                .await
            {
                Ok(()) => {
                    last_error = None;
                    break;
                }
                Err(e) => {
                    let err_str = e.to_string();
                    if err_str.contains("CollectionNotExists")
                        || err_str.contains("collection not found")
                    {
                        tracing::debug!(
                            "Index creation attempt {} failed (collection not ready), retrying...",
                            attempt + 1
                        );
                        last_error = Some(e);
                        tokio::time::sleep(std::time::Duration::from_millis(
                            500 * (attempt + 1) as u64,
                        ))
                        .await;
                        continue;
                    }
                    return Err(Error::vector_db(format!("Failed to create index: {}", e)));
                }
            }
        }

        if let Some(e) = last_error {
            return Err(Error::vector_db(format!(
                "Failed to create index after retries: {}",
                e
            )));
        }

        Ok(())
    }

    async fn delete_collection(&self, name: &str) -> Result<()> {
        Self::map_milvus_error(self.client.drop_collection(name).await, "delete collection")?;
        Ok(())
    }

    async fn insert_vectors(
        &self,
        collection: &str,
        vectors: &[Embedding],
        metadata: Vec<HashMap<String, serde_json::Value>>,
    ) -> Result<Vec<String>> {
        if vectors.is_empty() {
            return Err(Error::vector_db(
                "No vectors provided for insertion".to_string(),
            ));
        }

        if vectors.len() != metadata.len() {
            return Err(Error::vector_db(format!(
                "Vectors ({}) and metadata ({}) arrays must have the same length",
                vectors.len(),
                metadata.len()
            )));
        }

        // Validate all vectors have the same dimensions
        let expected_dims = vectors[0].dimensions;
        for (i, vector) in vectors.iter().enumerate() {
            if vector.dimensions != expected_dims {
                return Err(Error::vector_db(format!(
                    "Vector at index {} has dimensions {} but expected {}",
                    i, vector.dimensions, expected_dims
                )));
            }
        }

        // Prepare data for insertion
        let mut vectors_flat = Vec::new();
        let mut file_paths = Vec::new();
        let mut start_lines = Vec::new();
        let mut contents = Vec::new();

        for (embedding, meta) in vectors.iter().zip(metadata.iter()) {
            vectors_flat.extend_from_slice(&embedding.vector);

            let file_path = meta.string_or("file_path", "unknown");
            let start_line = meta
                .opt_i64("start_line")
                .or_else(|| meta.opt_i64("line_number"))
                .unwrap_or(0);
            let content = meta.string_or("content", "");

            file_paths.push(file_path);
            start_lines.push(start_line);
            contents.push(content);
        }

        // With auto_id: true, we don't provide the "id" column
        let vector_column = FieldColumn {
            name: "vector".to_string(),
            dtype: DataType::FloatVector,
            value: ValueVec::Float(vectors_flat),
            dim: expected_dims as i64,
            max_length: 0,
            is_dynamic: false,
        };
        let file_path_column = FieldColumn {
            name: "file_path".to_string(),
            dtype: DataType::VarChar,
            value: ValueVec::String(file_paths),
            dim: 1,
            max_length: MILVUS_FIELD_VARCHAR_MAX_LENGTH,
            is_dynamic: false,
        };
        let start_line_column = FieldColumn {
            name: "start_line".to_string(),
            dtype: DataType::Int64,
            value: ValueVec::Long(start_lines),
            dim: 1,
            max_length: 0,
            is_dynamic: false,
        };
        let content_column = FieldColumn {
            name: "content".to_string(),
            dtype: DataType::VarChar,
            value: ValueVec::String(contents),
            dim: 1,
            max_length: MILVUS_METADATA_VARCHAR_MAX_LENGTH,
            is_dynamic: false,
        };

        let columns = vec![
            vector_column,
            file_path_column,
            start_line_column,
            content_column,
        ];

        let res = Self::map_milvus_error(
            self.client.insert(collection, columns, None).await,
            "insert vectors",
        )?;

        // Return IDs as strings from the result
        let ids = match res.i_ds {
            Some(ids) => match ids.id_field {
                Some(milvus::proto::schema::i_ds::IdField::IntId(int_ids)) => {
                    int_ids.data.iter().map(|id| id.to_string()).collect()
                }
                Some(milvus::proto::schema::i_ds::IdField::StrId(str_ids)) => str_ids.data.clone(),
                None => Vec::new(),
            },
            None => Vec::new(),
        };

        Ok(ids)
    }

    async fn search_similar(
        &self,
        collection: &str,
        query_vector: &[f32],
        limit: usize,
        _filter: Option<&str>,
    ) -> Result<Vec<SearchResult>> {
        if query_vector.is_empty() {
            return Err(Error::vector_db("Query vector cannot be empty".to_string()));
        }

        if limit == 0 {
            return Ok(Vec::new());
        }

        // Ensure collection is loaded - handle missing collection gracefully
        if let Err(e) = self.client.load_collection(collection, None).await {
            let err_str = e.to_string();
            if err_str.contains("CollectionNotExists")
                || err_str.contains("collection not found")
                || err_str.contains("not exist")
            {
                tracing::debug!(
                    "Collection '{}' does not exist, returning empty results",
                    collection
                );
                return Ok(Vec::new());
            }
            return Err(Error::vector_db(format!(
                "Failed to load collection '{}': {}",
                collection, e
            )));
        }

        use milvus::query::SearchOptions;

        let search_options = SearchOptions::new()
            .limit(limit)
            .output_fields(vec![
                "id".to_string(),
                "file_path".to_string(),
                "start_line".to_string(),
                "content".to_string(),
            ])
            .add_param("metric_type", "L2");

        let search_results = match self
            .client
            .search(
                collection,
                vec![Value::FloatArray(Cow::Borrowed(query_vector))],
                Some(search_options),
            )
            .await
        {
            Ok(results) => results,
            Err(e) => {
                let err_str = e.to_string();
                if err_str.contains("no IDs") || err_str.contains("empty") {
                    return Ok(Vec::new());
                }
                return Err(Error::vector_db(format!("Failed to search: {}", e)));
            }
        };

        // Convert results to our format
        let mut results = Vec::new();

        for search_result in search_results {
            let scores = &search_result.score;
            let ids = &search_result.id;

            // Map columns by name for easy access
            let mut columns_map = HashMap::new();
            for column in &search_result.field {
                columns_map.insert(column.name.as_str(), column);
            }

            for (i, id_val) in ids.iter().enumerate() {
                let distance = scores.get(i).copied().unwrap_or(0.0);
                let score = (-distance).exp();

                let id_str = match id_val {
                    Value::Long(id) => id.to_string(),
                    Value::String(id) => id.to_string(),
                    _ => "unknown".to_string(),
                };

                let file_path = columns_map
                    .get("file_path")
                    .and_then(|col| col.get(i))
                    .map(|v| match v {
                        Value::String(s) => s.to_string(),
                        _ => "unknown".to_string(),
                    })
                    .unwrap_or_else(|| "unknown".to_string());

                let start_line = columns_map
                    .get("start_line")
                    .or_else(|| columns_map.get("line_number"))
                    .and_then(|col| col.get(i))
                    .map(|v| match v {
                        Value::Long(n) => n as u32,
                        _ => 0,
                    })
                    .unwrap_or(0);

                let content = columns_map
                    .get("content")
                    .and_then(|col| col.get(i))
                    .map(|v| match v {
                        Value::String(s) => s.to_string(),
                        _ => "".to_string(),
                    })
                    .unwrap_or_default();

                results.push(SearchResult {
                    id: id_str,
                    file_path,
                    start_line,
                    content,
                    score: score as f64,
                    language: "unknown".to_string(),
                });
            }
        }

        Ok(results)
    }

    async fn delete_vectors(&self, collection: &str, ids: &[String]) -> Result<()> {
        use milvus::mutate::DeleteOptions;
        use milvus::value::ValueVec;

        // Convert string IDs to i64 for Milvus
        let id_numbers: Vec<i64> = ids.iter().filter_map(|id| id.parse::<i64>().ok()).collect();

        if id_numbers.is_empty() {
            return Ok(()); // Nothing to delete
        }

        let options = DeleteOptions::with_ids(ValueVec::Long(id_numbers));

        Self::map_milvus_error(
            self.client.delete(collection, &options).await,
            "delete vectors",
        )?;

        Ok(())
    }

    async fn get_vectors_by_ids(
        &self,
        collection: &str,
        ids: &[String],
    ) -> Result<Vec<SearchResult>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        // Ensure collection is loaded
        self.client
            .load_collection(collection, None)
            .await
            .map_err(|e| {
                Error::vector_db(format!("Failed to load collection '{}': {}", collection, e))
            })?;

        // Construct expression for query
        let expr = format!("id in [{}]", ids.join(","));

        use milvus::query::QueryOptions;
        let mut query_options = QueryOptions::new();
        query_options = query_options.output_fields(vec![
            "id".to_string(),
            "file_path".to_string(),
            "start_line".to_string(),
            "content".to_string(),
        ]);

        let query_results = Self::map_milvus_error(
            self.client.query(collection, &expr, &query_options).await,
            "query by IDs",
        )?;

        // Convert results to our format
        let mut results = Vec::new();

        // Map columns by name
        let mut columns_map = HashMap::new();
        for column in &query_results {
            columns_map.insert(column.name.as_str(), column);
        }

        let row_count = if let Some(col) = query_results.first() {
            col.len()
        } else {
            0
        };

        for i in 0..row_count {
            let id_str = columns_map
                .get("id")
                .and_then(|col| col.get(i))
                .map(|v| match v {
                    Value::Long(id) => id.to_string(),
                    Value::String(id) => id.to_string(),
                    _ => "unknown".to_string(),
                })
                .unwrap_or_else(|| "unknown".to_string());

            let file_path = columns_map
                .get("file_path")
                .and_then(|col| col.get(i))
                .map(|v| match v {
                    Value::String(s) => s.to_string(),
                    _ => "unknown".to_string(),
                })
                .unwrap_or_else(|| "unknown".to_string());

            let start_line = columns_map
                .get("start_line")
                .or_else(|| columns_map.get("line_number"))
                .and_then(|col| col.get(i))
                .map(|v| match v {
                    Value::Long(n) => n as u32,
                    _ => 0,
                })
                .unwrap_or(0);

            let content = columns_map
                .get("content")
                .and_then(|col| col.get(i))
                .map(|v| match v {
                    Value::String(s) => s.to_string(),
                    _ => "".to_string(),
                })
                .unwrap_or_default();

            results.push(SearchResult {
                id: id_str,
                file_path,
                start_line,
                content,
                score: 1.0,
                language: "unknown".to_string(),
            });
        }

        Ok(results)
    }

    async fn list_vectors(&self, collection: &str, limit: usize) -> Result<Vec<SearchResult>> {
        if limit == 0 {
            return Ok(Vec::new());
        }

        // Ensure collection is loaded
        self.client
            .load_collection(collection, None)
            .await
            .map_err(|e| {
                Error::vector_db(format!("Failed to load collection '{}': {}", collection, e))
            })?;

        let expr = "id >= 0".to_string();
        use milvus::query::QueryOptions;
        let mut query_options = QueryOptions::new();
        query_options = query_options.limit(limit as i64).output_fields(vec![
            "id".to_string(),
            "file_path".to_string(),
            "start_line".to_string(),
            "content".to_string(),
        ]);

        let query_results = Self::map_milvus_error(
            self.client.query(collection, &expr, &query_options).await,
            "list vectors",
        )?;

        // Convert results to our format
        let mut results = Vec::new();

        // Map columns by name
        let mut columns_map = HashMap::new();
        for column in &query_results {
            columns_map.insert(column.name.as_str(), column);
        }

        let row_count = if let Some(col) = query_results.first() {
            col.len()
        } else {
            0
        };

        for i in 0..row_count {
            let id_str = columns_map
                .get("id")
                .and_then(|col| col.get(i))
                .map(|v| match v {
                    Value::Long(id) => id.to_string(),
                    Value::String(id) => id.to_string(),
                    _ => "unknown".to_string(),
                })
                .unwrap_or_else(|| "unknown".to_string());

            let file_path = columns_map
                .get("file_path")
                .and_then(|col| col.get(i))
                .map(|v| match v {
                    Value::String(s) => s.to_string(),
                    _ => "unknown".to_string(),
                })
                .unwrap_or_else(|| "unknown".to_string());

            let start_line = columns_map
                .get("start_line")
                .or_else(|| columns_map.get("line_number"))
                .and_then(|col| col.get(i))
                .map(|v| match v {
                    Value::Long(n) => n as u32,
                    _ => 0,
                })
                .unwrap_or(0);

            let content = columns_map
                .get("content")
                .and_then(|col| col.get(i))
                .map(|v| match v {
                    Value::String(s) => s.to_string(),
                    _ => "".to_string(),
                })
                .unwrap_or_default();

            results.push(SearchResult {
                id: id_str,
                file_path,
                start_line,
                content,
                score: 1.0,
                language: "unknown".to_string(),
            });
        }

        Ok(results)
    }
}

// ============================================================================
// Auto-registration via linkme distributed slice
// ============================================================================

use std::sync::Arc;

use mcb_application::ports::registry::{
    VECTOR_STORE_PROVIDERS, VectorStoreProviderConfig, VectorStoreProviderEntry,
};

/// Factory function for creating Milvus vector store provider instances.
fn milvus_factory(
    config: &VectorStoreProviderConfig,
) -> std::result::Result<Arc<dyn VectorStoreProvider>, String> {
    let uri = config
        .uri
        .clone()
        .unwrap_or_else(|| "http://localhost:19530".to_string());
    let token = config.api_key.clone();

    // Create Milvus client synchronously using block_on
    let provider = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current()
            .block_on(async { MilvusVectorStoreProvider::new(uri, token, None).await })
    })
    .map_err(|e| format!("Failed to create Milvus provider: {e}"))?;

    Ok(Arc::new(provider))
}

#[linkme::distributed_slice(VECTOR_STORE_PROVIDERS)]
static MILVUS_PROVIDER: VectorStoreProviderEntry = VectorStoreProviderEntry {
    name: "milvus",
    description: "Milvus distributed vector database",
    factory: milvus_factory,
};
