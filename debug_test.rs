use std::fs;
use tempfile::TempDir;
use mcp_context_browser::providers::vector_store::filesystem::{FilesystemVectorStore, FilesystemVectorStoreConfig};

#[tokio::test]
async fn debug_collection_management() {
    let temp_dir = TempDir::new().unwrap();
    let config = FilesystemVectorStoreConfig {
        base_path: temp_dir.path().to_path_buf(),
        ..Default::default()
    };

    let store = FilesystemVectorStore::new(config).await.unwrap();
    println!("Temp dir: {:?}", temp_dir.path());

    // Create collection
    println!("Creating collection test1...");
    store.create_collection("test1", 3).await.unwrap();
    
    // Check if index file exists
    let index_path = temp_dir.path().join("test1_index.json");
    println!("Index path: {:?}", index_path);
    println!("Index exists: {}", index_path.exists());
    
    // List files in temp dir
    if let Ok(entries) = fs::read_dir(temp_dir.path()) {
        println!("Files in temp dir:");
        for entry in entries {
            if let Ok(entry) = entry {
                println!("  {:?}", entry.path());
            }
        }
    }
    
    assert!(store.collection_exists("test1").await.unwrap());
}
