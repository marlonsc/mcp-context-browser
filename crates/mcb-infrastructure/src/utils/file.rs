//! Async file I/O utilities
//!
//! Consolidates common patterns for JSON serialization, file writing, and error handling.
//! Provides reusable async helpers that reduce boilerplate across the codebase.

use mcb_domain::error::{Error, Result};
use std::path::Path;

/// Async file utilities for common I/O patterns
///
/// Provides convenience methods that combine multiple operations with proper
/// error handling and consistent error messages.
pub struct FileUtils;

impl FileUtils {
    /// Write JSON to file with proper error handling
    ///
    /// Serializes value to JSON and writes atomically with descriptive error.
    ///
    /// # Arguments
    /// * `path` - The file path to write to
    /// * `value` - The value to serialize and write
    /// * `context` - Description for error messages (e.g., "config file")
    pub async fn write_json<T: serde::Serialize, P: AsRef<Path>>(
        path: P,
        value: &T,
        context: &str,
    ) -> Result<()> {
        let content = serde_json::to_string_pretty(value).map_err(|e| Error::Infrastructure {
            message: format!("Failed to serialize {}: {}", context, e),
            source: Some(Box::new(e)),
        })?;

        tokio::fs::write(path.as_ref(), content)
            .await
            .map_err(|e| Error::io_with_source(format!("Failed to write {}", context), e))?;

        Ok(())
    }

    /// Read JSON from file with proper error handling
    ///
    /// # Arguments
    /// * `path` - The file path to read from
    /// * `context` - Description for error messages (e.g., "config file")
    pub async fn read_json<T: serde::de::DeserializeOwned, P: AsRef<Path>>(
        path: P,
        context: &str,
    ) -> Result<T> {
        let content = tokio::fs::read_to_string(path.as_ref())
            .await
            .map_err(|e| Error::io_with_source(format!("Failed to read {}", context), e))?;

        serde_json::from_str(&content).map_err(|e| Error::Infrastructure {
            message: format!("Failed to parse {}: {}", context, e),
            source: Some(Box::new(e)),
        })
    }

    /// Ensure directory exists and write file
    ///
    /// Creates parent directories if needed, then writes the content.
    ///
    /// # Arguments
    /// * `path` - The file path to write to
    /// * `content` - The content bytes to write
    /// * `context` - Description for error messages
    pub async fn ensure_dir_write<P: AsRef<Path>>(
        path: P,
        content: &[u8],
        context: &str,
    ) -> Result<()> {
        if let Some(parent) = path.as_ref().parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                Error::io_with_source(format!("Failed to create directory for {}", context), e)
            })?;
        }

        tokio::fs::write(path.as_ref(), content)
            .await
            .map_err(|e| Error::io_with_source(format!("Failed to write {}", context), e))?;

        Ok(())
    }

    /// Ensure directory exists and write JSON
    ///
    /// Creates parent directories if needed, serializes value, and writes to file.
    ///
    /// # Arguments
    /// * `path` - The file path to write to
    /// * `value` - The value to serialize and write
    /// * `context` - Description for error messages
    pub async fn ensure_dir_write_json<T: serde::Serialize, P: AsRef<Path>>(
        path: P,
        value: &T,
        context: &str,
    ) -> Result<()> {
        let content = serde_json::to_string_pretty(value).map_err(|e| Error::Infrastructure {
            message: format!("Failed to serialize {}: {}", context, e),
            source: Some(Box::new(e)),
        })?;

        Self::ensure_dir_write(path, content.as_bytes(), context).await
    }

    /// Check if path exists (async wrapper)
    pub async fn exists<P: AsRef<Path>>(path: P) -> bool {
        tokio::fs::metadata(path.as_ref()).await.is_ok()
    }

    /// Read file if exists, return None otherwise
    ///
    /// Useful for optional files where missing file is not an error.
    pub async fn read_if_exists<P: AsRef<Path>>(path: P) -> Result<Option<Vec<u8>>> {
        match tokio::fs::read(path.as_ref()).await {
            Ok(content) => Ok(Some(content)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(Error::io_with_source("Failed to read file", e)),
        }
    }

    /// Read file as string if exists, return None otherwise
    pub async fn read_string_if_exists<P: AsRef<Path>>(path: P) -> Result<Option<String>> {
        match tokio::fs::read_to_string(path.as_ref()).await {
            Ok(content) => Ok(Some(content)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(Error::io_with_source("Failed to read file", e)),
        }
    }
}
