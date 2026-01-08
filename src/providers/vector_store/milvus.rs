//! Milvus vector store provider implementation

use crate::core::error::{Error, Result};
use crate::core::types::{Embedding, SearchResult};
use crate::providers::VectorStoreProvider;
use async_trait::async_trait;
use milvus::client::Client;
use milvus::schema::{CollectionSchemaBuilder, FieldSchema};
use std::collections::HashMap;

/// Milvus vector store provider implementation
pub struct MilvusVectorStoreProvider {
    client: Client,
}

impl MilvusVectorStoreProvider {
    /// Create a new Milvus vector store provider
    pub async fn new(address: String, _token: Option<String>) -> Result<Self> {
        // Extract host and port from URL for Milvus client
        let endpoint = if let Some(stripped) = address.strip_prefix("http://") {
            stripped.to_string()
        } else if let Some(stripped) = address.strip_prefix("https://") {
            stripped.to_string()
        } else {
            address
        };

        let client = Client::new(endpoint)
            .await
            .map_err(|e| Error::vector_db(format!("Failed to connect to Milvus: {}", e)))?;

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
                false, // Don't use auto_id so we can provide our own IDs
            ))
            .add_field(FieldSchema::new_float_vector(
                "vector",
                "feature field",
                dimensions as i64,
            ))
            .add_field(FieldSchema::new_varchar("file_path", "file path", 512))
            .add_field(FieldSchema::new_int64("line_number", "line number"))
            .add_field(FieldSchema::new_varchar("content", "content", 65535))
            .build()
            .map_err(|e| Error::vector_db(format!("Failed to create schema: {}", e)))?;

        self.client
            .create_collection(schema.clone(), None)
            .await
            .map_err(|e| Error::vector_db(format!("Failed to create collection: {}", e)))?;

        // Create index on the vector field for efficient search
        let collection_instance = self.client.get_collection(name).await.map_err(|e| {
            Error::vector_db(format!("Failed to get collection for indexing: {}", e))
        })?;

        use milvus::index::{IndexParams, IndexType, MetricType};
        use std::collections::HashMap;

        let index_params = IndexParams::new(
            "vector_index".to_string(),
            IndexType::IvfFlat,
            MetricType::L2,
            HashMap::from([("nlist".to_string(), "1024".to_string())]),
        );

        collection_instance
            .create_index("vector", index_params)
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
            if vector.vector.len() != expected_dims {
                return Err(Error::vector_db(format!(
                    "Vector at index {} has {} elements but should have {} (dimensions)",
                    i,
                    vector.vector.len(),
                    expected_dims
                )));
            }
        }

        // Get collection instance
        let collection_instance = self.client.get_collection(collection).await.map_err(|e| {
            Error::vector_db(format!("Failed to get collection '{}': {}", collection, e))
        })?;

        // Prepare data for insertion
        let mut ids = Vec::new();
        let mut vectors_data = Vec::new();
        let mut file_paths = Vec::new();
        let mut line_numbers = Vec::new();
        let mut contents = Vec::new();

        for (i, (embedding, meta)) in vectors.iter().zip(metadata.iter()).enumerate() {
            ids.push((i + 1) as i64);
            vectors_data.extend_from_slice(&embedding.vector);

            let file_path = meta
                .get("file_path")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();

            let line_number = meta
                .get("line_number")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);

            let content = meta
                .get("content")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            file_paths.push(file_path);
            line_numbers.push(line_number);
            contents.push(content);
        }

        // Validate schema has required fields
        let schema = collection_instance.schema();
        let required_fields = ["id", "vector", "file_path", "line_number", "content"];
        for field_name in &required_fields {
            if schema.get_field(field_name).is_none() {
                return Err(Error::vector_db(format!(
                    "Required field '{}' not found in collection '{}' schema",
                    field_name, collection
                )));
            }
        }

        // Create FieldColumns for insertion using schema fields
        use milvus::data::FieldColumn;

        let id_column = FieldColumn::new(
            schema.get_field("id").unwrap(), // Safe after validation above
            ids.clone(),
        );

        let vector_column = FieldColumn::new(
            schema.get_field("vector").unwrap(), // Safe after validation above
            vectors_data,
        );

        let file_path_column = FieldColumn::new(
            schema.get_field("file_path").unwrap(), // Safe after validation above
            file_paths,
        );

        let line_number_column = FieldColumn::new(
            schema.get_field("line_number").unwrap(), // Safe after validation above
            line_numbers,
        );

        let content_column = FieldColumn::new(
            schema.get_field("content").unwrap(), // Safe after validation above
            contents,
        );

        let columns = vec![
            id_column,
            vector_column,
            file_path_column,
            line_number_column,
            content_column,
        ];

        // Insert using collection instance
        collection_instance
            .insert(columns, None)
            .await
            .map_err(|e| Error::vector_db(format!("Failed to insert vectors: {}", e)))?;

        // Return IDs as strings
        Ok(ids.iter().map(|id| id.to_string()).collect())
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

        // Get collection instance
        let collection_instance = self.client.get_collection(collection).await.map_err(|e| {
            Error::vector_db(format!("Failed to get collection '{}': {}", collection, e))
        })?;

        // Check if collection has data before attempting search
        let stats = self.get_stats(collection).await?;
        let vector_count = stats
            .get("vectors_count")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        if vector_count == 0 {
            // Empty collection, return no results
            return Ok(Vec::new());
        }

        // Ensure collection is loaded
        let load_percent = collection_instance.get_load_percent().await.unwrap_or(0);

        if load_percent < 100
            && let Err(_e) = collection_instance.load(1).await
        {
            return Ok(Vec::new());
        }

        // Perform search using collection instance
        use milvus::collection::SearchOption;
        use milvus::index::MetricType;
        use milvus::value::Value;

        let search_option = SearchOption::new();
        let search_results = collection_instance
            .search(
                vec![Value::from(query_vector.to_vec())],
                "vector",
                limit as i32,
                MetricType::L2,
                vec!["id", "file_path", "line_number", "content"],
                &search_option,
            )
            .await
            .map_err(|e| Error::vector_db(format!("Failed to search: {}", e)))?;

        // Convert results to our format
        let mut results = Vec::new();

        for search_result in search_results {
            // Extract field columns from search result
            let id_column = search_result
                .field
                .iter()
                .find(|fc| fc.name == "id")
                .ok_or_else(|| {
                    Error::vector_db("id field not found in search result".to_string())
                })?;

            let file_path_column = search_result
                .field
                .iter()
                .find(|fc| fc.name == "file_path")
                .ok_or_else(|| {
                    Error::vector_db("file_path field not found in search result".to_string())
                })?;

            let line_number_column = search_result
                .field
                .iter()
                .find(|fc| fc.name == "line_number")
                .ok_or_else(|| {
                    Error::vector_db("line_number field not found in search result".to_string())
                })?;

            let content_column = search_result
                .field
                .iter()
                .find(|fc| fc.name == "content")
                .ok_or_else(|| {
                    Error::vector_db("content field not found in search result".to_string())
                })?;

            // Extract data from columns
            let ids: Vec<i64> = id_column
                .value
                .clone()
                .try_into()
                .map_err(|e| Error::vector_db(format!("Failed to extract ids: {:?}", e)))?;

            let file_paths: Vec<String> =
                file_path_column.value.clone().try_into().map_err(|e| {
                    Error::vector_db(format!("Failed to extract file_paths: {:?}", e))
                })?;

            let line_numbers: Vec<i64> =
                line_number_column.value.clone().try_into().map_err(|e| {
                    Error::vector_db(format!("Failed to extract line_numbers: {:?}", e))
                })?;

            let contents: Vec<String> =
                content_column.value.clone().try_into().map_err(|e| {
                    Error::vector_db(format!("Failed to extract contents: {:?}", e))
                })?;

            let scores = &search_result.score;

            // Create SearchResult for each match
            for (i, _) in ids.iter().enumerate() {
                let distance = scores.get(i).copied().unwrap_or(0.0);
                // Convert L2 distance to similarity score (higher is better)
                // Using exponential decay: score = exp(-distance)
                let score = (-distance).exp();

                results.push(SearchResult {
                    file_path: file_paths
                        .get(i)
                        .cloned()
                        .unwrap_or_else(|| "unknown".to_string()),
                    line_number: line_numbers.get(i).copied().unwrap_or(0) as u32,
                    content: contents.get(i).cloned().unwrap_or_else(|| "".to_string()),
                    score,
                    metadata: serde_json::json!({
                        "source": "milvus",
                        "id": ids[i],
                        "collection": collection
                    }),
                });
            }
        }

        Ok(results)
    }

    async fn delete_vectors(&self, collection: &str, ids: &[String]) -> Result<()> {
        // Get collection instance
        let collection_instance = self
            .client
            .get_collection(collection)
            .await
            .map_err(|e| Error::vector_db(format!("Failed to get collection: {}", e)))?;

        // Convert string IDs to i64 for Milvus
        let id_numbers: Vec<i64> = ids.iter().filter_map(|id| id.parse::<i64>().ok()).collect();

        if id_numbers.is_empty() {
            return Ok(()); // Nothing to delete
        }

        // Create delete expression
        let id_list = id_numbers
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(", ");

        let delete_expr = format!("id in [{}]", id_list);

        // Delete using collection instance
        collection_instance
            .delete(&delete_expr, None)
            .await
            .map_err(|e| Error::vector_db(format!("Failed to delete vectors: {}", e)))?;

        Ok(())
    }

    async fn get_stats(&self, collection: &str) -> Result<HashMap<String, serde_json::Value>> {
        // Get collection instance
        let collection_instance = self.client.get_collection(collection).await.map_err(|e| {
            Error::vector_db(format!("Failed to get collection '{}': {}", collection, e))
        })?;

        // Count entities using the correct query signature from the official example
        // First ensure collection is loaded (required for queries)
        let load_percent = collection_instance.get_load_percent().await.unwrap_or(0);
        if load_percent < 100 {
            let _ = collection_instance.load(1).await; // Ignore errors for now
        }

        // query::<_, [&str; 0]>(expr, []) - empty array for partition names means all partitions
        let entity_count = match collection_instance
            .query::<_, [&str; 0]>("id >= 0", [])
            .await
        {
            Ok(query_results) => query_results.first().map(|col| col.len()).unwrap_or(0),
            Err(_) => 0, // If query fails, collection might be empty or have issues
        };

        let mut stats = HashMap::new();
        stats.insert("collection".to_string(), serde_json::json!(collection));
        stats.insert("status".to_string(), serde_json::json!("active"));
        stats.insert("vectors_count".to_string(), serde_json::json!(entity_count));
        stats.insert("provider".to_string(), serde_json::json!("milvus"));
        Ok(stats)
    }

    async fn flush(&self, collection: &str) -> Result<()> {
        // Flush collection to ensure data persistence
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
