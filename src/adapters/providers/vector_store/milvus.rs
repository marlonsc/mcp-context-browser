//! Milvus vector store provider implementation

use crate::domain::error::{Error, Result};
use crate::domain::ports::VectorStoreProvider;
use crate::domain::types::{Embedding, SearchResult};
use async_trait::async_trait;
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
        // The Milvus SDK expects a full URI like "http://localhost:19530"
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
            .add_field(FieldSchema::new_varchar("file_path", "file path", 512))
            .add_field(FieldSchema::new_int64("start_line", "start line"))
            .add_field(FieldSchema::new_varchar("content", "content", 65535))
            .build()
            .map_err(|e| Error::vector_db(format!("Failed to create schema: {}", e)))?;

        self.client
            .create_collection(schema, None)
            .await
            .map_err(|e| Error::vector_db(format!("Failed to create collection: {}", e)))?;

        // Create index on the vector field for efficient search
        use milvus::index::{IndexParams, IndexType, MetricType};

        let index_params = IndexParams::new(
            "vector_index".to_string(),
            IndexType::IvfFlat,
            MetricType::L2,
            HashMap::from([("nlist".to_string(), "1024".to_string())]),
        );

        self.client
            .create_index(name, "vector", index_params)
            .await
            .map_err(|e| Error::vector_db(format!("Failed to create index: {}", e)))?;

        Ok(())
    }

    async fn delete_collection(&self, name: &str) -> Result<()> {
        self.client
            .drop_collection(name)
            .await
            .map_err(|e| Error::vector_db(format!("Failed to delete collection: {}", e)))?;

        Ok(())
    }

    async fn collection_exists(&self, name: &str) -> Result<bool> {
        self.client
            .has_collection(name)
            .await
            .map_err(|e| Error::vector_db(format!("Failed to check collection: {}", e)))
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

            let file_path = meta
                .get("file_path")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();

            let start_line = meta
                .get("start_line")
                .or_else(|| meta.get("line_number")) // Backward compatibility
                .and_then(|v| v.as_i64())
                .unwrap_or(0);

            let content = meta
                .get("content")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

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
            max_length: 512,
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
            max_length: 65535,
            is_dynamic: false,
        };

        let columns = vec![
            vector_column,
            file_path_column,
            start_line_column,
            content_column,
        ];

        // Insert directly using client
        let res = self
            .client
            .insert(collection, columns, None)
            .await
            .map_err(|e| Error::vector_db(format!("Failed to insert vectors: {}", e)))?;

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

        // Ensure collection is loaded
        self.client
            .load_collection(collection, None)
            .await
            .map_err(|e| {
                Error::vector_db(format!("Failed to load collection '{}': {}", collection, e))
            })?;

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
                // Handle empty collection case - Milvus returns error for no results
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
                    .unwrap_or_else(|| "".to_string());

                results.push(SearchResult {
                    id: id_str,
                    file_path,
                    start_line,
                    content,
                    score,
                    metadata: serde_json::json!({
                        "source": "milvus",
                        "collection": collection
                    }),
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

        // Delete using client
        self.client
            .delete(collection, &options)
            .await
            .map_err(|e| Error::vector_db(format!("Failed to delete vectors: {}", e)))?;

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

        let query_results = self
            .client
            .query(collection, &expr, &query_options)
            .await
            .map_err(|e| Error::vector_db(format!("Failed to query by IDs: {}", e)))?;

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
                .unwrap_or_else(|| "".to_string());

            results.push(SearchResult {
                id: id_str,
                file_path,
                start_line,
                content,
                score: 1.0,
                metadata: serde_json::json!({
                    "source": "milvus",
                    "collection": collection
                }),
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

        let query_results = self
            .client
            .query(collection, &expr, &query_options)
            .await
            .map_err(|e| Error::vector_db(format!("Failed to list vectors: {}", e)))?;

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
                .unwrap_or_else(|| "".to_string());

            results.push(SearchResult {
                id: id_str,
                file_path,
                start_line,
                content,
                score: 1.0,
                metadata: serde_json::json!({
                    "source": "milvus",
                    "collection": collection
                }),
            });
        }

        Ok(results)
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
        self.client
            .flush_collections(vec![collection])
            .await
            .map_err(|e| Error::vector_db(format!("Failed to flush collection: {}", e)))?;

        Ok(())
    }

    fn provider_name(&self) -> &str {
        "milvus"
    }
}
