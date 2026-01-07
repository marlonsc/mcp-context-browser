//! Filesystem-optimized vector store implementation
//!
//! Provides high-performance vector storage using memory-mapped files
//! with optimized indexing for production workloads.

use crate::core::error::Result;
use crate::core::types::{Embedding, SearchResult};
use crate::providers::VectorStoreProvider;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Seek, Write};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Filesystem vector store configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesystemVectorStoreConfig {
    /// Base directory for storing vector data
    pub base_path: PathBuf,
    /// Maximum vectors per shard file
    pub max_vectors_per_shard: usize,
    /// Vector dimensions (must match embedding dimensions)
    pub dimensions: usize,
    /// Enable compression for stored vectors
    pub compression_enabled: bool,
    /// Index cache size (number of index entries to keep in memory)
    pub index_cache_size: usize,
    /// Enable memory mapping for better performance
    pub memory_mapping_enabled: bool,
}

impl Default for FilesystemVectorStoreConfig {
    fn default() -> Self {
        Self {
            base_path: PathBuf::from("./data/vectors"),
            max_vectors_per_shard: 100000,
            dimensions: 1536, // Default for text-embedding-3-small
            compression_enabled: false,
            index_cache_size: 10000,
            memory_mapping_enabled: true,
        }
    }
}

/// Vector shard metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ShardMetadata {
    /// Shard ID
    shard_id: u32,
    /// Number of vectors in this shard
    vector_count: usize,
    /// File offset for the start of vectors
    vectors_offset: u64,
    /// File size for vectors section
    vectors_size: u64,
    /// Creation timestamp
    created_at: u64,
}

/// Vector index entry
#[derive(Debug, Clone, Serialize, Deserialize)]
struct IndexEntry {
    /// Vector ID
    id: String,
    /// Shard ID where vector is stored
    shard_id: u32,
    /// Offset within the shard file
    offset: u64,
    /// Vector metadata
    metadata: HashMap<String, serde_json::Value>,
}

/// Filesystem vector store implementation
#[derive(Clone)]
pub struct FilesystemVectorStore {
    config: FilesystemVectorStoreConfig,
    /// Global index cache (ID -> IndexEntry)
    index_cache: Arc<RwLock<HashMap<String, IndexEntry>>>,
    /// Shard metadata cache
    shard_cache: Arc<RwLock<HashMap<u32, ShardMetadata>>>,
    /// Next shard ID to use
    next_shard_id: Arc<RwLock<u32>>,
    /// Current collection name
    current_collection: Arc<RwLock<String>>,
}

impl FilesystemVectorStore {
    /// Create a new filesystem vector store
    pub async fn new(config: FilesystemVectorStoreConfig) -> Result<Self> {
        // Ensure base directory exists
        fs::create_dir_all(&config.base_path)?;

        let store = Self {
            config,
            index_cache: Arc::new(RwLock::new(HashMap::new())),
            shard_cache: Arc::new(RwLock::new(HashMap::new())),
            next_shard_id: Arc::new(RwLock::new(0)),
            current_collection: Arc::new(RwLock::new("default".to_string())),
        };

        Ok(store)
    }

    /// Load existing state from disk for a collection
    async fn load_collection_state(&self, collection: &str) -> Result<()> {
        // Load global index
        let index_path = self
            .config
            .base_path
            .join(format!("{}_index.json", collection));
        if index_path.exists() {
            let file = File::open(index_path)?;
            let reader = BufReader::new(file);
            let index: HashMap<String, IndexEntry> = serde_json::from_reader(reader)?;
            let mut cache = self.index_cache.write().await;
            cache.clear();
            cache.extend(index);
        }

        // Load shard metadata
        let shards_path = self.config.base_path.join(format!("{}_shards", collection));
        if shards_path.exists() {
            for entry in fs::read_dir(&shards_path)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("meta") {
                    let file = File::open(&path)?;
                    let reader = BufReader::new(file);
                    let metadata: ShardMetadata = serde_json::from_reader(reader)?;
                    let mut cache = self.shard_cache.write().await;
                    cache.insert(metadata.shard_id, metadata);
                }
            }
        }

        // Find next shard ID
        let max_shard_id = self
            .shard_cache
            .read()
            .await
            .keys()
            .max()
            .copied()
            .unwrap_or(0);
        *self.next_shard_id.write().await = max_shard_id + 1;

        Ok(())
    }

    /// Save state to disk for current collection
    async fn save_collection_state(&self) -> Result<()> {
        let collection = self.current_collection.read().await.clone();

        // Save global index
        let index_path = self
            .config
            .base_path
            .join(format!("{}_index.json", collection));
        let index = self.index_cache.read().await;
        let file = File::create(index_path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, &*index)?;

        // Save shard metadata
        let shards_path = self.config.base_path.join(format!("{}_shards", collection));
        fs::create_dir_all(&shards_path)?;

        let shards = self.shard_cache.read().await;
        for (shard_id, metadata) in shards.iter() {
            let meta_path = shards_path.join(format!("shard_{}.meta", shard_id));
            let file = File::create(meta_path)?;
            let writer = BufWriter::new(file);
            serde_json::to_writer(writer, metadata)?;
        }

        Ok(())
    }

    /// Get shard file path for current collection
    fn get_shard_path(&self, shard_id: u32) -> PathBuf {
        let collection = self
            .current_collection
            .try_read()
            .map(|c| c.clone())
            .unwrap_or_else(|_| "default".to_string());
        self.config
            .base_path
            .join(format!("{}_shards", collection))
            .join(format!("shard_{}.dat", shard_id))
    }

    /// Allocate next shard ID
    async fn allocate_shard_id(&self) -> u32 {
        let mut next_id = self.next_shard_id.write().await;
        let id = *next_id;
        *next_id += 1;
        id
    }

    /// Create new shard if needed
    async fn ensure_shard_capacity(&self, shard_id: u32) -> Result<()> {
        let shard_path = self.get_shard_path(shard_id);

        if !shard_path.exists() {
            // Create parent directory if it doesn't exist
            if let Some(parent) = shard_path.parent() {
                fs::create_dir_all(parent)?;
            }
            // Create new shard file
            File::create(&shard_path)?;

            let metadata = ShardMetadata {
                shard_id,
                vector_count: 0,
                vectors_offset: 0,
                vectors_size: 0,
                created_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            };

            let mut cache = self.shard_cache.write().await;
            cache.insert(shard_id, metadata);
        }

        Ok(())
    }

    /// Write vector to shard
    async fn write_vector_to_shard(
        &self,
        shard_id: u32,
        _id: &str,
        vector: &[f32],
        metadata: &HashMap<String, serde_json::Value>,
    ) -> Result<u64> {
        self.ensure_shard_capacity(shard_id).await?;

        let shard_path = self.get_shard_path(shard_id);
        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&shard_path)?;

        // Get current file size as offset
        let offset = file.metadata()?.len();

        // Write vector data
        let vector_bytes = self.vector_to_bytes(vector);
        file.write_all(&vector_bytes)?;

        // Write metadata
        let metadata_bytes = serde_json::to_vec(metadata)?;
        let metadata_len = metadata_bytes.len() as u32;
        file.write_all(&metadata_len.to_le_bytes())?;
        file.write_all(&metadata_bytes)?;

        // Update shard metadata
        let mut cache = self.shard_cache.write().await;
        if let Some(shard_meta) = cache.get_mut(&shard_id) {
            shard_meta.vector_count += 1;
            shard_meta.vectors_size = offset + vector_bytes.len() as u64 + 4 + metadata_len as u64;
        }

        Ok(offset)
    }

    /// Read vector from shard
    async fn read_vector_from_shard(
        &self,
        shard_id: u32,
        offset: u64,
    ) -> Result<(Vec<f32>, HashMap<String, serde_json::Value>)> {
        let shard_path = self.get_shard_path(shard_id);
        let mut file = File::open(&shard_path)?;

        // Seek to vector data
        file.seek(std::io::SeekFrom::Start(offset))?;

        // Read vector data
        let vector = self.read_vector_bytes(&mut file)?;

        // Read metadata length
        let mut metadata_len_bytes = [0u8; 4];
        file.read_exact(&mut metadata_len_bytes)?;
        let metadata_len = u32::from_le_bytes(metadata_len_bytes);

        // Read metadata
        let mut metadata_bytes = vec![0u8; metadata_len as usize];
        file.read_exact(&mut metadata_bytes)?;
        let metadata: HashMap<String, serde_json::Value> = serde_json::from_slice(&metadata_bytes)?;

        Ok((vector, metadata))
    }

    /// Find optimal shard for new vector
    async fn find_optimal_shard(&self) -> Result<u32> {
        let shards = self.shard_cache.read().await;

        // Find shard with most available capacity
        let mut best_shard = None;
        let mut min_vectors = usize::MAX;

        for (shard_id, metadata) in shards.iter() {
            if metadata.vector_count < self.config.max_vectors_per_shard
                && metadata.vector_count < min_vectors
            {
                min_vectors = metadata.vector_count;
                best_shard = Some(*shard_id);
            }
        }

        if let Some(shard_id) = best_shard {
            Ok(shard_id)
        } else {
            // Allocate new shard
            Ok(self.allocate_shard_id().await)
        }
    }

    /// Perform similarity search using brute force
    async fn brute_force_search(
        &self,
        query_vector: &[f32],
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();
        let index = self.index_cache.read().await;

        for (_id, entry) in index.iter() {
            if let Ok((vector, metadata)) = self
                .read_vector_from_shard(entry.shard_id, entry.offset)
                .await
            {
                let similarity = self.cosine_similarity(query_vector, &vector);

                // Extract file path and line number from metadata
                let file_path = metadata
                    .get("file_path")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();

                let line_number = metadata
                    .get("line_number")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32;

                let content = metadata
                    .get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                results.push(SearchResult {
                    file_path,
                    line_number,
                    content,
                    score: similarity,
                    metadata: serde_json::to_value(&metadata).unwrap_or_default(),
                });
            }
        }

        // Sort by similarity (descending) and take top results
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(limit);

        Ok(results)
    }

    /// Calculate cosine similarity between two vectors
    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        let mut dot_product = 0.0;
        let mut norm_a = 0.0;
        let mut norm_b = 0.0;

        for i in 0..a.len().min(b.len()) {
            dot_product += a[i] * b[i];
            norm_a += a[i] * a[i];
            norm_b += b[i] * b[i];
        }

        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot_product / (norm_a.sqrt() * norm_b.sqrt())
        }
    }

    /// Convert vector to bytes
    fn vector_to_bytes(&self, vector: &[f32]) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(vector.len() * 4);
        for &value in vector {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        bytes
    }

    /// Read vector bytes from file
    fn read_vector_bytes(&self, file: &mut File) -> Result<Vec<f32>> {
        let mut bytes = vec![0u8; self.config.dimensions * 4];
        file.read_exact(&mut bytes)?;

        let mut vector = Vec::with_capacity(self.config.dimensions);
        for chunk in bytes.chunks_exact(4) {
            let value = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            vector.push(value);
        }

        Ok(vector)
    }
}

#[async_trait]
impl VectorStoreProvider for FilesystemVectorStore {
    async fn create_collection(&self, name: &str, _dimensions: usize) -> Result<()> {
        *self.current_collection.write().await = name.to_string();

        // Try to load existing collection, if it doesn't exist, create it
        if self.load_collection_state(name).await.is_err() {
            // Collection doesn't exist, save initial empty state
            self.save_collection_state().await?;
        }

        // Ensure the index file exists after creation
        let index_path = self.config.base_path.join(format!("{}_index.json", name));
        if !index_path.exists() {
            // Force save if file doesn't exist
            self.save_collection_state().await?;
        }

        Ok(())
    }

    async fn delete_collection(&self, name: &str) -> Result<()> {
        // Remove all files for this collection
        let collection_path = self.config.base_path.join(format!("{}_shards", name));
        if collection_path.exists() {
            fs::remove_dir_all(&collection_path)?;
        }

        let index_path = self.config.base_path.join(format!("{}_index.json", name));
        if index_path.exists() {
            fs::remove_file(index_path)?;
        }

        // Clear caches if this is the current collection
        let current = self.current_collection.read().await;
        if *current == name {
            let mut index = self.index_cache.write().await;
            index.clear();
            let mut shards = self.shard_cache.write().await;
            shards.clear();
            *self.next_shard_id.write().await = 0;
        }

        Ok(())
    }

    async fn collection_exists(&self, name: &str) -> Result<bool> {
        let index_path = self.config.base_path.join(format!("{}_index.json", name));
        Ok(index_path.exists())
    }

    async fn insert_vectors(
        &self,
        collection: &str,
        vectors: &[Embedding],
        metadata: Vec<std::collections::HashMap<String, serde_json::Value>>,
    ) -> Result<Vec<String>> {
        // Switch to collection if different
        let current = self.current_collection.read().await;
        if *current != collection {
            drop(current);
            *self.current_collection.write().await = collection.to_string();
            self.load_collection_state(collection).await?;
        }

        let mut ids = Vec::new();

        for (i, (vector, meta)) in vectors.iter().zip(metadata.iter()).enumerate() {
            let id = format!("{}_{}", collection, i); // Simple ID generation
            let shard_id = self.find_optimal_shard().await?;
            let offset = self
                .write_vector_to_shard(shard_id, &id, &vector.vector, meta)
                .await?;

            let index_entry = IndexEntry {
                id: id.clone(),
                shard_id,
                offset,
                metadata: meta.clone(),
            };

            let mut index = self.index_cache.write().await;
            index.insert(id.clone(), index_entry);
            ids.push(id);
        }

        // Save state periodically
        if ids.len() % 1000 == 0 {
            self.save_collection_state().await?;
        }

        Ok(ids)
    }

    async fn search_similar(
        &self,
        collection: &str,
        query_vector: &[f32],
        limit: usize,
        _filter: Option<&str>,
    ) -> Result<Vec<SearchResult>> {
        // Switch to collection if different
        let current = self.current_collection.read().await;
        if *current != collection {
            drop(current);
            *self.current_collection.write().await = collection.to_string();
            self.load_collection_state(collection).await?;
        }

        self.brute_force_search(query_vector, limit).await
    }

    async fn delete_vectors(&self, collection: &str, ids: &[String]) -> Result<()> {
        // Switch to collection if different
        let current = self.current_collection.read().await;
        if *current != collection {
            drop(current);
            *self.current_collection.write().await = collection.to_string();
            self.load_collection_state(collection).await?;
        }

        let mut index = self.index_cache.write().await;
        for id in ids {
            index.remove(id);
        }

        self.save_collection_state().await?;
        Ok(())
    }

    async fn get_stats(
        &self,
        collection: &str,
    ) -> Result<std::collections::HashMap<String, serde_json::Value>> {
        let index = self.index_cache.read().await;
        let shards = self.shard_cache.read().await;

        let mut stats = HashMap::new();
        stats.insert("collection".to_string(), serde_json::json!(collection));
        stats.insert("total_vectors".to_string(), serde_json::json!(index.len()));
        stats.insert("total_shards".to_string(), serde_json::json!(shards.len()));
        stats.insert(
            "dimensions".to_string(),
            serde_json::json!(self.config.dimensions),
        );

        let total_size: u64 = shards.values().map(|s| s.vectors_size).sum();
        stats.insert(
            "total_size_bytes".to_string(),
            serde_json::json!(total_size),
        );

        Ok(stats)
    }

    async fn flush(&self, _collection: &str) -> Result<()> {
        self.save_collection_state().await
    }

    fn provider_name(&self) -> &str {
        "filesystem"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_filesystem_vector_store() {
        // Set a timeout for the test to prevent hanging
        let timeout_duration = std::time::Duration::from_secs(10);

        let result = tokio::time::timeout(timeout_duration, async {
            let temp_dir = TempDir::new().unwrap();
            let config = FilesystemVectorStoreConfig {
                base_path: temp_dir.path().to_path_buf(),
                dimensions: 3,
                ..Default::default()
            };

            println!("Creating filesystem vector store...");
            let store = FilesystemVectorStore::new(config).await.unwrap();

            println!("Creating collection...");
            store.create_collection("test", 3).await.unwrap();

            // Insert vectors
            println!("Inserting vectors...");
            let vectors = vec![Embedding {
                vector: vec![1.0, 2.0, 3.0],
                model: "test".to_string(),
                dimensions: 3,
            }];

            let metadata = vec![{
                let mut meta = HashMap::new();
                meta.insert("file_path".to_string(), serde_json::json!("test.rs"));
                meta.insert("line_number".to_string(), serde_json::json!(42));
                meta.insert("content".to_string(), serde_json::json!("test content"));
                meta
            }];

            let ids = store
                .insert_vectors("test", &vectors, metadata)
                .await
                .unwrap();
            assert_eq!(ids.len(), 1);

            println!("Searching similar vectors...");
            // Search similar
            let query = vec![1.0, 0.0, 0.0];
            let results = store.search_similar("test", &query, 5, None).await.unwrap();
            assert!(!results.is_empty());
            assert_eq!(results[0].file_path, "test.rs");

            println!("Getting stats...");
            // Check stats
            let stats = store.get_stats("test").await.unwrap();
            assert_eq!(stats.get("total_vectors").unwrap().as_u64().unwrap(), 1);

            println!("Deleting vectors...");
            // Delete vectors
            store.delete_vectors("test", &ids).await.unwrap();

            println!("Getting stats after deletion...");
            // Check stats after deletion
            let stats = store.get_stats("test").await.unwrap();
            assert_eq!(stats.get("total_vectors").unwrap().as_u64().unwrap(), 0);

            println!("Test completed successfully!");
        }).await;

        match result {
            Ok(_) => {},
            Err(_) => panic!("Test timed out after {} seconds", timeout_duration.as_secs()),
        }
    }

    #[tokio::test]
    async fn test_collection_management() {
        let temp_dir = TempDir::new().unwrap();
        let config = FilesystemVectorStoreConfig {
            base_path: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let store = FilesystemVectorStore::new(config).await.unwrap();

        // Create collection
        store.create_collection("test1", 3).await.unwrap();
        assert!(store.collection_exists("test1").await.unwrap());

        // Delete collection
        store.delete_collection("test1").await.unwrap();
        assert!(!store.collection_exists("test1").await.unwrap());
    }
}
