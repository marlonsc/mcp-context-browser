//! Unit tests for Codebase entities
//!
//! Tests the CodebaseSnapshot and FileSnapshot entities, including
//! change tracking functionality.

use mcb_domain::entities::codebase::{CodebaseSnapshot, FileSnapshot, SnapshotChanges};
use std::collections::HashMap;

#[test]
fn test_file_snapshot_creation() {
    let file_snapshot = FileSnapshot {
        path: "src/lib.rs".to_string(),
        modified_at: 1641081600, // 2022-01-02 00:00:00 UTC
        size: 2048,
        hash: "def456".to_string(),
        language: "rust".to_string(),
    };

    assert_eq!(file_snapshot.path, "src/lib.rs");
    assert_eq!(file_snapshot.modified_at, 1641081600);
    assert_eq!(file_snapshot.size, 2048);
    assert_eq!(file_snapshot.hash, "def456");
    assert_eq!(file_snapshot.language, "rust");
}

#[test]
fn test_codebase_snapshot_creation() {
    let mut files = HashMap::new();
    files.insert(
        "src/main.rs".to_string(),
        FileSnapshot {
            path: "src/main.rs".to_string(),
            modified_at: 1640995200,
            size: 1024,
            hash: "abc123".to_string(),
            language: "rust".to_string(),
        },
    );

    let snapshot = CodebaseSnapshot {
        id: "snapshot-001".to_string(),
        created_at: 1640995200,
        collection: "my-project".to_string(),
        files: files.clone(),
        total_files: 1,
        total_size: 1024,
    };

    assert_eq!(snapshot.id, "snapshot-001");
    assert_eq!(snapshot.created_at, 1640995200);
    assert_eq!(snapshot.collection, "my-project");
    assert_eq!(snapshot.total_files, 1);
    assert_eq!(snapshot.total_size, 1024);
    assert_eq!(snapshot.files.len(), 1);
    assert!(snapshot.files.contains_key("src/main.rs"));
}

#[test]
fn test_codebase_snapshot_multiple_files() {
    let mut files = HashMap::new();

    files.insert(
        "src/main.rs".to_string(),
        FileSnapshot {
            path: "src/main.rs".to_string(),
            modified_at: 1640995200,
            size: 1024,
            hash: "abc123".to_string(),
            language: "rust".to_string(),
        },
    );

    files.insert(
        "src/lib.rs".to_string(),
        FileSnapshot {
            path: "src/lib.rs".to_string(),
            modified_at: 1641081600,
            size: 2048,
            hash: "def456".to_string(),
            language: "rust".to_string(),
        },
    );

    files.insert(
        "Cargo.toml".to_string(),
        FileSnapshot {
            path: "Cargo.toml".to_string(),
            modified_at: 1640995200,
            size: 512,
            hash: "toml123".to_string(),
            language: "toml".to_string(),
        },
    );

    let snapshot = CodebaseSnapshot {
        id: "multi-file-snapshot".to_string(),
        created_at: 1641081600,
        collection: "test-project".to_string(),
        files: files.clone(),
        total_files: 3,
        total_size: 3584, // 1024 + 2048 + 512
    };

    assert_eq!(snapshot.total_files, 3);
    assert_eq!(snapshot.total_size, 3584);
    assert_eq!(snapshot.files.len(), 3);
    assert!(snapshot.files.contains_key("src/main.rs"));
    assert!(snapshot.files.contains_key("src/lib.rs"));
    assert!(snapshot.files.contains_key("Cargo.toml"));
}

#[test]
fn test_snapshot_changes_empty() {
    let changes = SnapshotChanges {
        added: vec![],
        modified: vec![],
        removed: vec![],
    };

    assert!(!changes.has_changes());
    assert_eq!(changes.total_changes(), 0);
}

#[test]
fn test_snapshot_changes_with_additions() {
    let changes = SnapshotChanges {
        added: vec!["new_file.rs".to_string(), "another.rs".to_string()],
        modified: vec![],
        removed: vec![],
    };

    assert!(changes.has_changes());
    assert_eq!(changes.total_changes(), 2);
    assert_eq!(changes.added.len(), 2);
    assert!(changes.modified.is_empty());
    assert!(changes.removed.is_empty());
}

#[test]
fn test_snapshot_changes_mixed() {
    let changes = SnapshotChanges {
        added: vec!["new.rs".to_string()],
        modified: vec!["changed.rs".to_string(), "updated.rs".to_string()],
        removed: vec!["deleted.rs".to_string()],
    };

    assert!(changes.has_changes());
    assert_eq!(changes.total_changes(), 4);
    assert_eq!(changes.added.len(), 1);
    assert_eq!(changes.modified.len(), 2);
    assert_eq!(changes.removed.len(), 1);
}

#[test]
fn test_snapshot_changes_only_modifications() {
    let changes = SnapshotChanges {
        added: vec![],
        modified: vec![
            "modified1.rs".to_string(),
            "modified2.rs".to_string(),
            "modified3.rs".to_string(),
        ],
        removed: vec![],
    };

    assert!(changes.has_changes());
    assert_eq!(changes.total_changes(), 3);
    assert!(changes.added.is_empty());
    assert_eq!(changes.modified.len(), 3);
    assert!(changes.removed.is_empty());
}

#[test]
fn test_snapshot_changes_only_removals() {
    let changes = SnapshotChanges {
        added: vec![],
        modified: vec![],
        removed: vec!["gone1.rs".to_string(), "gone2.rs".to_string()],
    };

    assert!(changes.has_changes());
    assert_eq!(changes.total_changes(), 2);
    assert!(changes.added.is_empty());
    assert!(changes.modified.is_empty());
    assert_eq!(changes.removed.len(), 2);
}
