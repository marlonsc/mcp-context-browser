//! Merkle tree tests
//!
//! Tests migrated from src/infrastructure/merkle.rs

use mcp_context_browser::infrastructure::merkle::MerkleTree;
use tempfile::TempDir;

#[test]
fn test_merkle_tree_creation() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let tree = MerkleTree::from_directory(temp_dir.path())?;
    assert!(!tree.root_hash().is_empty());
    Ok(())
}

#[test]
fn test_identical_trees_no_diff() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;

    // Create a file
    std::fs::write(temp_dir.path().join("test.txt"), "content")?;

    let tree1 = MerkleTree::from_directory(temp_dir.path())?;
    let tree2 = MerkleTree::from_directory(temp_dir.path())?;

    let diff = tree1.diff(&tree2);
    assert!(!diff.has_changes());
    Ok(())
}

#[test]
fn test_modified_file_detection() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let file_path = temp_dir.path().join("test.txt");

    // Create file with initial content
    std::fs::write(&file_path, "content1")?;
    let tree1 = MerkleTree::from_directory(temp_dir.path())?;

    // Modify file
    std::fs::write(&file_path, "content2")?;
    let tree2 = MerkleTree::from_directory(temp_dir.path())?;

    let diff = tree1.diff(&tree2);
    assert!(diff.has_changes());
    assert_eq!(diff.modified.len(), 1);
    assert_eq!(diff.modified[0], vec!["test.txt"]);
    Ok(())
}

#[test]
fn test_added_file_detection() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;

    let tree1 = MerkleTree::from_directory(temp_dir.path())?;

    // Add file
    std::fs::write(temp_dir.path().join("test.txt"), "content")?;
    let tree2 = MerkleTree::from_directory(temp_dir.path())?;

    let diff = tree1.diff(&tree2);
    assert!(diff.has_changes());
    assert_eq!(diff.added.len(), 1);
    assert_eq!(diff.added[0], vec!["test.txt"]);
    Ok(())
}
