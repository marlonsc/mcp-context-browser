//! Merkle tree implementation for efficient incremental file change detection
//!
//! Provides hierarchical hashing that allows detecting changes with minimal computation
//! and enables efficient incremental synchronization across large codebases.

use crate::core::error::{Error, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// Merkle tree node representing either a file or directory
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MerkleNode {
    /// Leaf node representing a file with its hash
    File {
        name: String,
        hash: String,
        size: u64,
    },
    /// Internal node representing a directory with child nodes
    Directory {
        name: String,
        hash: String,
        children: HashMap<String, MerkleNode>,
    },
}

impl MerkleNode {
    /// Get the hash of this node
    pub fn hash(&self) -> &str {
        match self {
            MerkleNode::File { hash, .. } => hash,
            MerkleNode::Directory { hash, .. } => hash,
        }
    }

    /// Get the name of this node
    pub fn name(&self) -> &str {
        match self {
            MerkleNode::File { name, .. } => name,
            MerkleNode::Directory { name, .. } => name,
        }
    }

    /// Check if this is a file node
    pub fn is_file(&self) -> bool {
        matches!(self, MerkleNode::File { .. })
    }

    /// Check if this is a directory node
    pub fn is_directory(&self) -> bool {
        matches!(self, MerkleNode::Directory { .. })
    }
}

/// Merkle tree for hierarchical change detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleTree {
    root: MerkleNode,
}

impl MerkleTree {
    /// Create a new Merkle tree from a directory
    pub fn from_directory(path: &std::path::Path) -> Result<Self> {
        let root = Self::build_node(path, "")?;
        Ok(Self { root })
    }

    /// Get the root hash of the tree
    pub fn root_hash(&self) -> &str {
        self.root.hash()
    }

    /// Compare with another Merkle tree to find changes
    pub fn diff(&self, other: &MerkleTree) -> MerkleDiff {
        Self::diff_nodes(&self.root, &other.root, Vec::new())
    }

    /// Build a Merkle node from filesystem
    fn build_node(path: &std::path::Path, name: &str) -> Result<MerkleNode> {
        let metadata = std::fs::metadata(path).map_err(|e| Error::Io {
            source: std::io::Error::other(format!(
                "Failed to read metadata for {}: {}",
                path.display(),
                e
            )),
        })?;

        if metadata.is_file() {
            let content = std::fs::read(path).map_err(|e| {
                Error::generic(format!("Failed to read file {}: {}", path.display(), e))
            })?;

            let hash = Self::hash_content(&content);
            let name = if name.is_empty() {
                path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
            } else {
                name.to_string()
            };

            Ok(MerkleNode::File {
                name,
                hash,
                size: metadata.len(),
            })
        } else if metadata.is_dir() {
            let mut children = HashMap::new();
            let mut child_hashes = Vec::new();

            let entries = std::fs::read_dir(path).map_err(|e| {
                Error::generic(format!(
                    "Failed to read directory {}: {}",
                    path.display(),
                    e
                ))
            })?;

            for entry in entries {
                let entry: std::fs::DirEntry = entry.map_err(|e| {
                    Error::generic(format!("Failed to read directory entry: {}", e))
                })?;
                let entry_path = entry.path();

                // Skip hidden files and directories
                if let Some(file_name) = entry_path.file_name()
                    && file_name.to_string_lossy().starts_with('.')
                {
                    continue;
                }

                // Skip common non-source files
                if Self::should_skip_entry(&entry_path) {
                    continue;
                }

                let entry_name = entry_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();

                let child_node = Self::build_node(&entry_path, &entry_name)?;
                child_hashes.push(child_node.hash().to_string());
                children.insert(entry_name, child_node);
            }

            // Sort hashes for consistent ordering
            child_hashes.sort();
            let combined_hash = Self::hash_combined(&child_hashes);

            let name = if name.is_empty() {
                path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
            } else {
                name.to_string()
            };

            Ok(MerkleNode::Directory {
                name,
                hash: combined_hash,
                children,
            })
        } else {
            Err(Error::generic(format!(
                "Unsupported file type: {}",
                path.display()
            )))
        }
    }

    /// Check if filesystem entry should be skipped
    fn should_skip_entry(path: &std::path::Path) -> bool {
        let file_name = path.file_name().unwrap_or_default().to_string_lossy();

        // Skip common non-source files
        if file_name.ends_with(".log")
            || file_name.ends_with(".tmp")
            || file_name.ends_with(".cache")
            || file_name.ends_with(".lock")
            || file_name.starts_with('.')
        {
            return true;
        }

        // Skip build artifacts
        let path_str = path.to_string_lossy();
        if path_str.contains("/target/")
            || path_str.contains("/node_modules/")
            || path_str.contains("/.git/")
            || path_str.contains("/dist/")
            || path_str.contains("/build/")
            || path_str.contains("/.cargo/")
        {
            return true;
        }

        false
    }

    /// Calculate hash of file content
    fn hash_content(content: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content);
        format!("{:x}", hasher.finalize())
    }

    /// Calculate hash of combined child hashes
    fn hash_combined(hashes: &[String]) -> String {
        let mut hasher = Sha256::new();
        for hash in hashes {
            hasher.update(hash.as_bytes());
        }
        format!("{:x}", hasher.finalize())
    }

    /// Compare two Merkle nodes and return differences
    fn diff_nodes(left: &MerkleNode, right: &MerkleNode, path: Vec<String>) -> MerkleDiff {
        match (left, right) {
            (
                MerkleNode::File {
                    name: left_name,
                    hash: left_hash,
                    ..
                },
                MerkleNode::File {
                    name: right_name,
                    hash: right_hash,
                    ..
                },
            ) => {
                if left_name != right_name {
                    return MerkleDiff::default();
                }

                if left_hash != right_hash {
                    let mut diff = MerkleDiff::default();
                    diff.modified.push(path);
                    diff
                } else {
                    MerkleDiff::default()
                }
            }
            (
                MerkleNode::Directory {
                    name: left_name,
                    children: left_children,
                    ..
                },
                MerkleNode::Directory {
                    name: right_name,
                    children: right_children,
                    ..
                },
            ) => {
                if left_name != right_name {
                    return MerkleDiff::default();
                }

                let mut diff = MerkleDiff::default();

                // Find added and modified files/directories
                for (name, left_child) in left_children {
                    let mut current_path = path.clone();
                    current_path.push(name.clone());

                    if let Some(right_child) = right_children.get(name) {
                        // Both exist, check recursively
                        let child_diff = Self::diff_nodes(left_child, right_child, current_path);
                        diff.added.extend(child_diff.added);
                        diff.modified.extend(child_diff.modified);
                        diff.removed.extend(child_diff.removed);
                    } else {
                        // Removed in right (present in left, absent in right)
                        diff.removed.push(current_path);
                    }
                }

                // Find added files/directories (present in right, absent in left)
                for name in right_children.keys() {
                    if !left_children.contains_key(name) {
                        let mut current_path = path.clone();
                        current_path.push(name.clone());
                        diff.added.push(current_path);
                    }
                }

                diff
            }
            _ => {
                // Type mismatch (file vs directory) - treat as modification
                let mut diff = MerkleDiff::default();
                diff.modified.push(path);
                diff
            }
        }
    }
}

/// Differences between two Merkle trees
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MerkleDiff {
    /// Paths that were added
    pub added: Vec<Vec<String>>,
    /// Paths that were modified
    pub modified: Vec<Vec<String>>,
    /// Paths that were removed
    pub removed: Vec<Vec<String>>,
}

impl MerkleDiff {
    /// Check if there are any changes
    pub fn has_changes(&self) -> bool {
        !self.added.is_empty() || !self.modified.is_empty() || !self.removed.is_empty()
    }

    /// Get total number of changes
    pub fn total_changes(&self) -> usize {
        self.added.len() + self.modified.len() + self.removed.len()
    }

    /// Convert path vectors to string paths
    pub fn added_paths(&self) -> Vec<String> {
        self.added.iter().map(|path| path.join("/")).collect()
    }

    /// Convert path vectors to string paths
    pub fn modified_paths(&self) -> Vec<String> {
        self.modified.iter().map(|path| path.join("/")).collect()
    }

    /// Convert path vectors to string paths
    pub fn removed_paths(&self) -> Vec<String> {
        self.removed.iter().map(|path| path.join("/")).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_merkle_tree_creation() {
        let temp_dir = TempDir::new().unwrap();
        let tree = MerkleTree::from_directory(temp_dir.path()).unwrap();
        assert!(!tree.root_hash().is_empty());
    }

    #[test]
    fn test_identical_trees_no_diff() {
        let temp_dir = TempDir::new().unwrap();

        // Create a file
        std::fs::write(temp_dir.path().join("test.txt"), "content").unwrap();

        let tree1 = MerkleTree::from_directory(temp_dir.path()).unwrap();
        let tree2 = MerkleTree::from_directory(temp_dir.path()).unwrap();

        let diff = tree1.diff(&tree2);
        assert!(!diff.has_changes());
    }

    #[test]
    fn test_modified_file_detection() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        // Create file with initial content
        std::fs::write(&file_path, "content1").unwrap();
        let tree1 = MerkleTree::from_directory(temp_dir.path()).unwrap();

        // Modify file
        std::fs::write(&file_path, "content2").unwrap();
        let tree2 = MerkleTree::from_directory(temp_dir.path()).unwrap();

        let diff = tree1.diff(&tree2);
        assert!(diff.has_changes());
        assert_eq!(diff.modified.len(), 1);
        assert_eq!(diff.modified[0], vec!["test.txt"]);
    }

    #[test]
    fn test_added_file_detection() {
        let temp_dir = TempDir::new().unwrap();

        let tree1 = MerkleTree::from_directory(temp_dir.path()).unwrap();

        // Add file
        std::fs::write(temp_dir.path().join("test.txt"), "content").unwrap();
        let tree2 = MerkleTree::from_directory(temp_dir.path()).unwrap();

        let diff = tree1.diff(&tree2);
        assert!(diff.has_changes());
        assert_eq!(diff.added.len(), 1);
        assert_eq!(diff.added[0], vec!["test.txt"]);
    }
}
