//! Filesystem-optimized vector store implementation
//!
//! Provides high-performance vector storage using memory-mapped files
//! with optimized indexing for production workloads.

use crate::constants::{
    FILESYSTEM_BYTES_PER_DIMENSION, FILESYSTEM_VECTOR_STORE_INDEX_CACHE_SIZE,
    FILESYSTEM_VECTOR_STORE_MAX_PER_SHARD,
};
use crate::utils::JsonExt;
use async_trait::async_trait;
use dashmap::DashMap;
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::providers::{VectorStoreAdmin, VectorStoreBrowser, VectorStoreProvider};
use mcb_domain::value_objects::{CollectionInfo, Embedding, FileInfo, SearchResult};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io::{Read, Seek, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

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
            max_vectors_per_shard: FILESYSTEM_VECTOR_STORE_MAX_PER_SHARD,
            dimensions: 1536, // Default for text-embedding-3-small
            compression_enabled: false,
            index_cache_size: FILESYSTEM_VECTOR_STORE_INDEX_CACHE_SIZE,
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
    #[allow(dead_code)]
    vectors_offset: u64,
    /// File size for vectors section
    vectors_size: u64,
    /// Creation timestamp
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    metadata: HashMap<String, serde_json::Value>,
}

/// Filesystem vector store implementation
#[derive(Clone)]
pub struct FilesystemVectorStore {
    config: FilesystemVectorStoreConfig,
    /// Global index cache ((collection, ID) -> IndexEntry)
    index_cache: Arc<DashMap<(String, String), IndexEntry>>,
    /// Shard metadata cache ((collection, shard_id) -> ShardMetadata)
    shard_cache: Arc<DashMap<(String, u32), ShardMetadata>>,
    /// Next shard ID to use per collection
    next_shard_ids: Arc<DashMap<String, Arc<AtomicU32>>>,
}

// File utility helpers (inlined from infrastructure)
mod file_utils {
    use mcb_domain::error::{Error, Result};
    use serde::{Serialize, de::DeserializeOwned};
    use std::path::Path;

    pub async fn exists(path: &Path) -> bool {
        tokio::fs::metadata(path).await.is_ok()
    }

    pub async fn read_json<T: DeserializeOwned>(path: &Path, description: &str) -> Result<T> {
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| Error::io(format!("Failed to read {}: {}", description, e)))?;
        serde_json::from_str(&content)
            .map_err(|e| Error::internal(format!("Failed to parse {}: {}", description, e)))
    }

    pub async fn write_json<T: Serialize>(path: &Path, data: &T, description: &str) -> Result<()> {
        let content = serde_json::to_string_pretty(data)
            .map_err(|e| Error::internal(format!("Failed to serialize {}: {}", description, e)))?;
        tokio::fs::write(path, content)
            .await
            .map_err(|e| Error::io(format!("Failed to write {}: {}", description, e)))
    }

    pub async fn ensure_dir_write(path: &Path, data: &[u8], description: &str) -> Result<()> {
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                Error::io(format!(
                    "Failed to create directory for {}: {}",
                    description, e
                ))
            })?;
        }
        tokio::fs::write(path, data)
            .await
            .map_err(|e| Error::io(format!("Failed to write {}: {}", description, e)))
    }

    pub async fn ensure_dir_write_json<T: Serialize>(
        path: &Path,
        data: &T,
        description: &str,
    ) -> Result<()> {
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                Error::io(format!(
                    "Failed to create directory for {}: {}",
                    description, e
                ))
            })?;
        }
        write_json(path, data, description).await
    }
}

impl FilesystemVectorStore {
    /// Create a new filesystem vector store
    pub async fn new(config: FilesystemVectorStoreConfig) -> Result<Self> {
        // Ensure base directory exists
        tokio::fs::create_dir_all(&config.base_path)
            .await
            .map_err(|e| Error::io(format!("Failed to create base directory: {}", e)))?;

        let store = Self {
            config,
            index_cache: Arc::new(DashMap::new()),
            shard_cache: Arc::new(DashMap::new()),
            next_shard_ids: Arc::new(DashMap::new()),
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
        if file_utils::exists(&index_path).await {
            let index: HashMap<String, IndexEntry> =
                file_utils::read_json(&index_path, "collection index").await?;
            for (id, entry) in index {
                self.index_cache.insert((collection.to_string(), id), entry);
            }
        }

        // Load shard metadata
        let shards_path = self.config.base_path.join(format!("{}_shards", collection));
        if shards_path.exists() {
            let mut entries = tokio::fs::read_dir(&shards_path)
                .await
                .map_err(|e| Error::io(format!("Failed to read shards directory: {}", e)))?;

            while let Some(entry) = entries
                .next_entry()
                .await
                .map_err(|e| Error::io(format!("Failed to read directory entry: {}", e)))?
            {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("meta") {
                    let metadata: ShardMetadata =
                        file_utils::read_json(&path, "shard metadata").await?;
                    self.shard_cache
                        .insert((collection.to_string(), metadata.shard_id), metadata);
                }
            }
        }

        // Find next shard ID
        let max_shard_id = self
            .shard_cache
            .iter()
            .filter(|r| r.key().0 == collection)
            .map(|r| r.value().shard_id)
            .max()
            .unwrap_or(0);

        self.next_shard_ids.insert(
            collection.to_string(),
            Arc::new(AtomicU32::new(max_shard_id + 1)),
        );

        Ok(())
    }

    /// Save state to disk for a collection
    async fn save_collection_state(&self, collection: &str) -> Result<()> {
        // Save global index
        let index_path = self
            .config
            .base_path
            .join(format!("{}_index.json", collection));
        let index: HashMap<String, IndexEntry> = self
            .index_cache
            .iter()
            .filter(|r| r.key().0 == collection)
            .map(|r| (r.key().1.clone(), r.value().clone()))
            .collect();
        file_utils::write_json(&index_path, &index, "collection index").await?;

        // Save shard metadata
        let shards_path = self.config.base_path.join(format!("{}_shards", collection));
        for r in self.shard_cache.iter() {
            let (c, shard_id) = r.key();
            if c == collection {
                let meta_path = shards_path.join(format!("shard_{}.meta", shard_id));
                file_utils::ensure_dir_write_json(&meta_path, r.value(), "shard metadata").await?;
            }
        }
        Ok(())
    }

    /// Get shard file path for a collection
    fn get_shard_path(&self, collection: &str, shard_id: u32) -> PathBuf {
        self.config
            .base_path
            .join(format!("{}_shards", collection))
            .join(format!("shard_{}.dat", shard_id))
    }

    /// Allocate next shard ID
    fn allocate_shard_id(&self, collection: &str) -> u32 {
        let entry = self
            .next_shard_ids
            .entry(collection.to_string())
            .or_insert_with(|| Arc::new(AtomicU32::new(0)));
        entry.value().fetch_add(1, Ordering::SeqCst)
    }

    /// Create new shard if needed
    async fn ensure_shard_capacity(&self, collection: &str, shard_id: u32) -> Result<()> {
        let shard_path = self.get_shard_path(collection, shard_id);

        if !file_utils::exists(&shard_path).await {
            // Create shard file with empty content
            file_utils::ensure_dir_write(&shard_path, &[], "shard file").await?;

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
            self.shard_cache
                .insert((collection.to_string(), shard_id), metadata);
        }
        Ok(())
    }

    /// Write vector to shard
    async fn write_vector_to_shard(
        &self,
        collection: &str,
        shard_id: u32,
        _id: &str,
        vector: &[f32],
        metadata: &HashMap<String, serde_json::Value>,
    ) -> Result<u64> {
        self.ensure_shard_capacity(collection, shard_id).await?;

        let shard_path = self.get_shard_path(collection, shard_id);

        let vector_bytes = self.vector_to_bytes(vector);
        let metadata_bytes = serde_json::to_vec(metadata)
            .map_err(|e| Error::internal(format!("Failed to serialize metadata: {}", e)))?;
        let metadata_len = metadata_bytes.len() as u32;

        let (offset, total_shard_size) = tokio::task::spawn_blocking(move || {
            let mut file = std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(&shard_path)?;

            let offset = file.metadata()?.len();
            file.seek(std::io::SeekFrom::End(0))?;
            file.write_all(&vector_bytes)?;
            file.write_all(&metadata_len.to_le_bytes())?;
            file.write_all(&metadata_bytes)?;

            let total_size = file.metadata()?.len();
            Ok::<_, std::io::Error>((offset, total_size))
        })
        .await
        .map_err(|e| Error::internal(format!("Blocking task failed: {}", e)))?
        .map_err(|e| Error::io(format!("Failed to write to shard: {}", e)))?;

        // Update shard metadata
        if let Some(mut shard_meta) = self
            .shard_cache
            .get_mut(&(collection.to_string(), shard_id))
        {
            shard_meta.vector_count += 1;
            shard_meta.vectors_size = total_shard_size;
        }

        Ok(offset)
    }

    /// Read vector from shard
    async fn read_vector_from_shard(
        &self,
        collection: &str,
        shard_id: u32,
        offset: u64,
    ) -> Result<(Vec<f32>, HashMap<String, serde_json::Value>)> {
        let shard_path = self.get_shard_path(collection, shard_id);
        let dimensions = self.config.dimensions;

        tokio::task::spawn_blocking(move || {
            let mut file = std::fs::File::open(&shard_path)?;
            file.seek(std::io::SeekFrom::Start(offset))?;

            // Read vector data
            let mut bytes = vec![0u8; dimensions * FILESYSTEM_BYTES_PER_DIMENSION];
            file.read_exact(&mut bytes)?;
            let mut vector = Vec::with_capacity(dimensions);
            for chunk in bytes.chunks_exact(FILESYSTEM_BYTES_PER_DIMENSION) {
                let value = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                vector.push(value);
            }

            // Read metadata length
            let mut metadata_len_bytes = [0u8; 4];
            file.read_exact(&mut metadata_len_bytes)?;
            let metadata_len = u32::from_le_bytes(metadata_len_bytes);

            // Read metadata
            let mut metadata_bytes = vec![0u8; metadata_len as usize];
            file.read_exact(&mut metadata_bytes)?;
            let metadata: HashMap<String, serde_json::Value> =
                serde_json::from_slice(&metadata_bytes)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

            Ok((vector, metadata))
        })
        .await
        .map_err(|e| Error::internal(format!("Blocking task failed: {}", e)))?
        .map_err(|e: std::io::Error| Error::io(format!("Failed to read from shard: {}", e)))
    }

    /// Find optimal shard for new vector
    fn find_optimal_shard(&self, collection: &str) -> u32 {
        // Find shard with most available capacity
        let mut best_shard = None;
        let mut min_vectors = usize::MAX;

        for r in self.shard_cache.iter() {
            let (c, shard_id) = r.key();
            if c == collection {
                let metadata = r.value();
                if metadata.vector_count < self.config.max_vectors_per_shard
                    && metadata.vector_count < min_vectors
                {
                    min_vectors = metadata.vector_count;
                    best_shard = Some(*shard_id);
                }
            }
        }

        if let Some(shard_id) = best_shard {
            shard_id
        } else {
            // Allocate new shard
            self.allocate_shard_id(collection)
        }
    }

    /// Perform similarity search using brute force
    async fn brute_force_search(
        &self,
        collection: &str,
        query_vector: &[f32],
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();

        // Collect index entries first to avoid holding DashMap iterator across await points
        let entries: Vec<IndexEntry> = self
            .index_cache
            .iter()
            .filter(|r| r.key().0 == collection)
            .map(|r| r.value().clone())
            .collect();

        for entry in entries {
            if let Ok((vector, metadata)) = self
                .read_vector_from_shard(collection, entry.shard_id, entry.offset)
                .await
            {
                let similarity = self.cosine_similarity(query_vector, &vector);

                let file_path = metadata.string_or("file_path", "unknown");
                let start_line = metadata
                    .opt_u64("start_line")
                    .or_else(|| metadata.opt_u64("line_number"))
                    .unwrap_or(0) as u32;
                let content = metadata.string_or("content", "");
                let language = metadata.string_or("language", "unknown");

                results.push(SearchResult {
                    id: entry.id.clone(),
                    file_path,
                    start_line,
                    content,
                    score: similarity as f64,
                    language,
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
        let (dot_product, norm_a, norm_b) = a
            .iter()
            .zip(b.iter())
            .fold((0.0, 0.0, 0.0), |(dot, na, nb), (&x, &y)| {
                (dot + x * y, na + x * x, nb + y * y)
            });

        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot_product / (norm_a.sqrt() * norm_b.sqrt())
        }
    }

    /// Convert vector to bytes
    fn vector_to_bytes(&self, vector: &[f32]) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(vector.len() * FILESYSTEM_BYTES_PER_DIMENSION);
        for &value in vector {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        bytes
    }
}

#[async_trait]
impl VectorStoreAdmin for FilesystemVectorStore {
    async fn collection_exists(&self, name: &str) -> Result<bool> {
        let index_path = self.config.base_path.join(format!("{}_index.json", name));
        Ok(index_path.exists())
    }

    async fn get_stats(&self, collection: &str) -> Result<HashMap<String, serde_json::Value>> {
        // Ensure state is loaded
        if !self.next_shard_ids.contains_key(collection) {
            self.load_collection_state(collection).await?;
        }

        let mut stats = HashMap::new();
        stats.insert("collection".to_string(), serde_json::json!(collection));

        let total_vectors = self
            .index_cache
            .iter()
            .filter(|r| r.key().0 == collection)
            .count();
        stats.insert(
            "total_vectors".to_string(),
            serde_json::json!(total_vectors),
        );

        let total_shards = self
            .shard_cache
            .iter()
            .filter(|r| r.key().0 == collection)
            .count();
        stats.insert("total_shards".to_string(), serde_json::json!(total_shards));

        stats.insert(
            "dimensions".to_string(),
            serde_json::json!(self.config.dimensions),
        );

        let total_size: u64 = self
            .shard_cache
            .iter()
            .filter(|r| r.key().0 == collection)
            .map(|r| r.value().vectors_size)
            .sum();

        stats.insert(
            "total_size_bytes".to_string(),
            serde_json::json!(total_size),
        );

        Ok(stats)
    }

    async fn flush(&self, collection: &str) -> Result<()> {
        self.save_collection_state(collection).await
    }

    fn provider_name(&self) -> &str {
        "filesystem"
    }
}

#[async_trait]
impl VectorStoreProvider for FilesystemVectorStore {
    async fn create_collection(&self, name: &str, _dimensions: usize) -> Result<()> {
        // Try to load existing collection, if it doesn't exist, create it
        if !self.collection_exists(name).await? {
            // Collection doesn't exist, save initial empty state
            self.save_collection_state(name).await?;
        } else {
            self.load_collection_state(name).await?;
        }

        Ok(())
    }

    async fn delete_collection(&self, name: &str) -> Result<()> {
        // Remove all files for this collection
        let collection_path = self.config.base_path.join(format!("{}_shards", name));
        if collection_path.exists() {
            tokio::fs::remove_dir_all(&collection_path)
                .await
                .map_err(|e| Error::io(format!("Failed to delete collection shards: {}", e)))?;
        }

        let index_path = self.config.base_path.join(format!("{}_index.json", name));
        if index_path.exists() {
            tokio::fs::remove_file(index_path)
                .await
                .map_err(|e| Error::io(format!("Failed to delete collection index: {}", e)))?;
        }

        // Clear caches
        self.index_cache.retain(|k, _| k.0 != name);
        self.shard_cache.retain(|k, _| k.0 != name);
        self.next_shard_ids.remove(name);

        Ok(())
    }

    async fn insert_vectors(
        &self,
        collection: &str,
        vectors: &[Embedding],
        metadata: Vec<std::collections::HashMap<String, serde_json::Value>>,
    ) -> Result<Vec<String>> {
        // Ensure state is loaded
        if !self.next_shard_ids.contains_key(collection) {
            self.load_collection_state(collection).await?;
        }

        let mut ids = Vec::new();

        for (i, (vector, meta)) in vectors.iter().zip(metadata.iter()).enumerate() {
            let id = format!(
                "{}_{}_{}",
                collection,
                i,
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos()
            );
            let shard_id = self.find_optimal_shard(collection);
            let offset = self
                .write_vector_to_shard(collection, shard_id, &id, &vector.vector, meta)
                .await?;

            let index_entry = IndexEntry {
                id: id.clone(),
                shard_id,
                offset,
                metadata: meta.clone(),
            };

            self.index_cache
                .insert((collection.to_string(), id.clone()), index_entry);
            ids.push(id);
        }

        // Save state
        self.save_collection_state(collection).await?;

        Ok(ids)
    }

    async fn search_similar(
        &self,
        collection: &str,
        query_vector: &[f32],
        limit: usize,
        _filter: Option<&str>,
    ) -> Result<Vec<SearchResult>> {
        // Ensure state is loaded
        if !self.next_shard_ids.contains_key(collection) {
            self.load_collection_state(collection).await?;
        }

        self.brute_force_search(collection, query_vector, limit)
            .await
    }

    async fn delete_vectors(&self, collection: &str, ids: &[String]) -> Result<()> {
        // Ensure state is loaded
        if !self.next_shard_ids.contains_key(collection) {
            self.load_collection_state(collection).await?;
        }

        // Remove from index
        for id in ids {
            self.index_cache
                .remove(&(collection.to_string(), id.clone()));
        }

        // Save state
        self.save_collection_state(collection).await?;
        Ok(())
    }

    async fn get_vectors_by_ids(
        &self,
        collection: &str,
        ids: &[String],
    ) -> Result<Vec<SearchResult>> {
        // Ensure state is loaded
        if !self.next_shard_ids.contains_key(collection) {
            self.load_collection_state(collection).await?;
        }

        let mut results = Vec::new();
        for id in ids {
            if let Some(entry) = self.index_cache.get(&(collection.to_string(), id.clone())) {
                if let Ok((_, metadata)) = self
                    .read_vector_from_shard(collection, entry.shard_id, entry.offset)
                    .await
                {
                    let file_path = metadata.string_or("file_path", "unknown");
                    let start_line = metadata
                        .opt_u64("start_line")
                        .or_else(|| metadata.opt_u64("line_number"))
                        .unwrap_or(0) as u32;
                    let content = metadata.string_or("content", "");
                    let language = metadata.string_or("language", "unknown");

                    results.push(SearchResult {
                        id: id.clone(),
                        file_path,
                        start_line,
                        content,
                        score: 1.0,
                        language,
                    });
                }
            }
        }
        Ok(results)
    }

    async fn list_vectors(&self, collection: &str, limit: usize) -> Result<Vec<SearchResult>> {
        // Ensure state is loaded
        if !self.next_shard_ids.contains_key(collection) {
            self.load_collection_state(collection).await?;
        }

        let mut results = Vec::new();
        let entries: Vec<_> = self
            .index_cache
            .iter()
            .filter(|r| r.key().0 == collection)
            .take(limit)
            .map(|r| (r.key().1.clone(), r.value().clone()))
            .collect();

        for (id, entry) in entries {
            if let Ok((_vector, metadata)) = self
                .read_vector_from_shard(collection, entry.shard_id, entry.offset)
                .await
            {
                let file_path = metadata.string_or("file_path", "unknown");
                let start_line = metadata
                    .opt_u64("start_line")
                    .or_else(|| metadata.opt_u64("line_number"))
                    .unwrap_or(0) as u32;
                let content = metadata.string_or("content", "");
                let language = metadata.string_or("language", "unknown");

                results.push(SearchResult {
                    id,
                    file_path,
                    start_line,
                    content,
                    score: 1.0,
                    language,
                });
            }
        }
        Ok(results)
    }
}

#[async_trait]
impl VectorStoreBrowser for FilesystemVectorStore {
    async fn list_collections(&self) -> Result<Vec<CollectionInfo>> {
        // Find all collection index files
        let mut collections = Vec::new();

        let entries = tokio::fs::read_dir(&self.config.base_path)
            .await
            .map_err(|e| Error::io(format!("Failed to read base directory: {}", e)))?;

        let mut entries = entries;
        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| Error::io(format!("Failed to read directory entry: {}", e)))?
        {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.ends_with("_index.json") {
                    let collection_name = name.trim_end_matches("_index.json");

                    // Get stats for this collection
                    let stats = self.get_stats(collection_name).await.unwrap_or_default();
                    let vector_count = stats
                        .get("total_vectors")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);

                    // Count unique files from index cache
                    let file_paths: HashSet<String> = self
                        .index_cache
                        .iter()
                        .filter(|r| r.key().0 == collection_name)
                        .filter_map(|r| {
                            r.value()
                                .metadata
                                .get("file_path")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string())
                        })
                        .collect();
                    let file_count = file_paths.len() as u64;

                    collections.push(CollectionInfo::new(
                        collection_name,
                        vector_count,
                        file_count,
                        None,
                        self.provider_name(),
                    ));
                }
            }
        }

        Ok(collections)
    }

    async fn list_file_paths(&self, collection: &str, limit: usize) -> Result<Vec<FileInfo>> {
        // Ensure state is loaded
        if !self.next_shard_ids.contains_key(collection) {
            self.load_collection_state(collection).await?;
        }

        // Aggregate file info from index cache
        let mut file_map: HashMap<String, (u32, String)> = HashMap::new();

        for entry in self.index_cache.iter() {
            if entry.key().0 == collection {
                if let Some(file_path) = entry
                    .value()
                    .metadata
                    .get("file_path")
                    .and_then(|v| v.as_str())
                {
                    let language = entry
                        .value()
                        .metadata
                        .get("language")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string();

                    let e = file_map
                        .entry(file_path.to_string())
                        .or_insert((0, language));
                    e.0 += 1; // Increment chunk count
                }
            }
        }

        let files: Vec<FileInfo> = file_map
            .into_iter()
            .take(limit)
            .map(|(path, (chunk_count, language))| FileInfo::new(path, chunk_count, language, None))
            .collect();

        Ok(files)
    }

    async fn get_chunks_by_file(
        &self,
        collection: &str,
        file_path: &str,
    ) -> Result<Vec<SearchResult>> {
        // Ensure state is loaded
        if !self.next_shard_ids.contains_key(collection) {
            self.load_collection_state(collection).await?;
        }

        let mut results = Vec::new();

        // Find all entries for this file
        let entries: Vec<_> = self
            .index_cache
            .iter()
            .filter(|r| {
                r.key().0 == collection
                    && r.value()
                        .metadata
                        .get("file_path")
                        .and_then(|v| v.as_str())
                        .is_some_and(|p| p == file_path)
            })
            .map(|r| (r.key().1.clone(), r.value().clone()))
            .collect();

        for (id, entry) in entries {
            if let Ok((_, metadata)) = self
                .read_vector_from_shard(collection, entry.shard_id, entry.offset)
                .await
            {
                let start_line = metadata
                    .opt_u64("start_line")
                    .or_else(|| metadata.opt_u64("line_number"))
                    .unwrap_or(0) as u32;
                let content = metadata.string_or("content", "");
                let language = metadata.string_or("language", "unknown");

                results.push(SearchResult {
                    id,
                    file_path: file_path.to_string(),
                    start_line,
                    content,
                    score: 1.0,
                    language,
                });
            }
        }

        // Sort by start_line
        results.sort_by_key(|r| r.start_line);

        Ok(results)
    }
}

// ============================================================================
// Auto-registration via linkme distributed slice
// ============================================================================

use mcb_application::ports::registry::{
    VECTOR_STORE_PROVIDERS, VectorStoreProviderConfig, VectorStoreProviderEntry,
};

/// Factory function for creating filesystem vector store provider instances.
fn filesystem_factory(
    config: &VectorStoreProviderConfig,
) -> std::result::Result<Arc<dyn VectorStoreProvider>, String> {
    let base_path = config
        .uri
        .clone()
        .unwrap_or_else(|| "./data/vectors".to_string());
    let dimensions = config.dimensions.unwrap_or(1536);

    let fs_config = FilesystemVectorStoreConfig {
        base_path: std::path::PathBuf::from(base_path),
        dimensions,
        ..Default::default()
    };

    // Create store synchronously using block_in_place for the async constructor
    let store = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current()
            .block_on(async { FilesystemVectorStore::new(fs_config).await })
    })
    .map_err(|e| format!("Failed to create filesystem store: {e}"))?;

    Ok(Arc::new(store))
}

#[linkme::distributed_slice(VECTOR_STORE_PROVIDERS)]
static FILESYSTEM_PROVIDER: VectorStoreProviderEntry = VectorStoreProviderEntry {
    name: "filesystem",
    description: "Filesystem-based vector store (persistent, sharded)",
    factory: filesystem_factory,
};
