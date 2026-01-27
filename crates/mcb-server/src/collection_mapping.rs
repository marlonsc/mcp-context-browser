//! Collection Name Mapping Manager
//!
//! Manages the mapping between user-friendly collection names (with hyphens)
//! and Milvus-compatible names (with underscores and timestamp suffix).
//!
//! Stores mapping in `~/.config/mcb/collection_mapping.json`
//!
//! Example:
//! ```json
//! {
//!   "mcp-context-browser": "mcp_context_browser_20260126_143021",
//!   "my-project": "my_project_20260126_143022"
//! }
//! ```

use mcb_domain::error::{Error, Result};
use std::collections::HashMap;
use std::path::PathBuf;

/// Collection name mapping file name
const MAPPING_FILENAME: &str = "collection_mapping.json";

/// Gets the default mapping file path (~/.config/mcb/collection_mapping.json)
fn get_mapping_file_path() -> Result<PathBuf> {
    let config_dir =
        dirs::config_dir().ok_or_else(|| Error::io("Unable to determine config directory"))?;

    let mcb_config = config_dir.join("mcb");
    Ok(mcb_config.join(MAPPING_FILENAME))
}

/// Load the collection name mapping from disk
fn load_mapping() -> Result<HashMap<String, String>> {
    let mapping_path = get_mapping_file_path()?;

    if !mapping_path.exists() {
        return Ok(HashMap::new());
    }

    let content = std::fs::read_to_string(&mapping_path)
        .map_err(|e| Error::io(format!("Failed to read mapping file: {}", e)))?;

    serde_json::from_str(&content)
        .map_err(|e| Error::io(format!("Failed to parse mapping file: {}", e)))
}

/// Save the collection name mapping to disk
fn save_mapping(mapping: &HashMap<String, String>) -> Result<()> {
    let mapping_path = get_mapping_file_path()?;

    // Ensure directory exists
    if let Some(parent) = mapping_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| Error::io(format!("Failed to create config directory: {}", e)))?;
    }

    let json = serde_json::to_string_pretty(mapping)
        .map_err(|e| Error::io(format!("Failed to serialize mapping: {}", e)))?;

    std::fs::write(&mapping_path, json)
        .map_err(|e| Error::io(format!("Failed to write mapping file: {}", e)))
}

/// Generate a Milvus-compatible name from a user-friendly collection name
fn generate_milvus_name(user_name: &str) -> String {
    // Replace hyphens with underscores
    let normalized = user_name.replace('-', "_").to_lowercase();

    // Add timestamp suffix to prevent collisions
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();

    let timestamp = format!("{}", now.as_secs() % 1_000_000); // Last 6 digits

    format!("{}_{}", normalized, timestamp)
}

/// Get or create a Milvus-compatible collection name
///
/// # Arguments
/// * `user_name` - User-provided collection name (e.g., "mcp-context-browser")
///
/// # Returns
/// * `String` - Milvus-compatible name (stored in mapping)
///
/// # Example
/// ```ignore
/// let milvus_name = map_collection_name("mcp-context-browser")?;
/// // Returns: "mcp_context_browser_143021" (with mapping stored)
/// ```
pub fn map_collection_name(user_name: &str) -> Result<String> {
    let mut mapping = load_mapping()?;

    // Return existing mapping if available
    if let Some(milvus_name) = mapping.get(user_name) {
        return Ok(milvus_name.clone());
    }

    // Generate new mapping
    let milvus_name = generate_milvus_name(user_name);
    mapping.insert(user_name.to_string(), milvus_name.clone());

    // Persist the mapping
    save_mapping(&mapping)?;

    Ok(milvus_name)
}

/// Get all known collections (user-friendly names)
///
/// # Returns
/// * `Vec<String>` - List of user-provided collection names
pub fn list_collections() -> Result<Vec<String>> {
    let mapping = load_mapping()?;
    let mut collections: Vec<String> = mapping.keys().cloned().collect();
    collections.sort();
    Ok(collections)
}

/// Get the reverse mapping (Milvus name → user name)
///
/// # Returns
/// * `HashMap<String, String>` - Mapping from Milvus names to user names
pub fn get_reverse_mapping() -> Result<HashMap<String, String>> {
    let mapping = load_mapping()?;
    let reversed = mapping
        .into_iter()
        .map(|(user, milvus)| (milvus, user))
        .collect();
    Ok(reversed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_milvus_name() {
        let result = generate_milvus_name("mcp-context-browser");
        assert!(result.starts_with("mcp_context_browser_"));
        assert!(result.len() > "mcp_context_browser_".len());
    }

    #[test]
    fn test_generate_milvus_name_lowercase() {
        let result = generate_milvus_name("MyProject");
        assert!(!result.contains(|c: char| c.is_uppercase()));
    }

    #[test]
    fn test_generate_milvus_name_hyphens_converted() {
        let result = generate_milvus_name("my-project-name");
        assert!(!result.contains('-'));
        assert!(result.contains('_'));
    }
}
