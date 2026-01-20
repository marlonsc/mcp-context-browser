//! Unit tests for browse value objects
//!
//! Moved from inline tests in src/value_objects/browse.rs
//! per test organization standards.

use mcb_domain::value_objects::browse::{CollectionInfo, FileInfo};

#[test]
fn test_collection_info_new() {
    let info = CollectionInfo::new("test-collection", 100, 10, Some(1705680000), "milvus");

    assert_eq!(info.name, "test-collection");
    assert_eq!(info.vector_count, 100);
    assert_eq!(info.file_count, 10);
    assert_eq!(info.last_indexed, Some(1705680000));
    assert_eq!(info.provider, "milvus");
}

#[test]
fn test_collection_info_serialization() {
    let info = CollectionInfo::new("test", 50, 5, None, "in_memory");
    let json = serde_json::to_string(&info).expect("serialization should succeed");
    let deserialized: CollectionInfo =
        serde_json::from_str(&json).expect("deserialization should succeed");

    assert_eq!(info, deserialized);
}

#[test]
fn test_file_info_new() {
    let info = FileInfo::new("src/main.rs", 5, "rust", Some(1024));

    assert_eq!(info.path, "src/main.rs");
    assert_eq!(info.chunk_count, 5);
    assert_eq!(info.language, "rust");
    assert_eq!(info.size_bytes, Some(1024));
}

#[test]
fn test_file_info_serialization() {
    let info = FileInfo::new("lib.rs", 3, "rust", None);
    let json = serde_json::to_string(&info).expect("serialization should succeed");
    let deserialized: FileInfo =
        serde_json::from_str(&json).expect("deserialization should succeed");

    assert_eq!(info, deserialized);
}
