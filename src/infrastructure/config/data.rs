use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use validator::Validate;

/// Data directory configuration using XDG standard locations
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct DataConfig {
    /// Base data directory (XDG_DATA_HOME standard)
    /// Default: ~/.local/share/mcp-context-browser
    /// Stores: snapshots, config history, encryption keys, circuit breaker state, etc.
    #[validate(length(min = 1))]
    pub base_dir: String,

    /// Snapshots directory (code snapshots and project state)
    /// Default: {base_dir}/snapshots
    pub snapshots_dir: Option<String>,

    /// Configuration history directory (audit trail of config changes)
    /// Default: {base_dir}/config-history
    pub config_history_dir: Option<String>,

    /// Encryption keys directory (master key and encryption materials)
    /// Default: {base_dir}/encryption
    pub encryption_keys_dir: Option<String>,

    /// Circuit breaker state directory (failover and health state)
    /// Default: {base_dir}/circuit-breakers
    pub circuit_breakers_dir: Option<String>,
}

impl Default for DataConfig {
    fn default() -> Self {
        Self {
            base_dir: "~/.local/share/mcp-context-browser".to_string(),
            snapshots_dir: None,
            config_history_dir: None,
            encryption_keys_dir: None,
            circuit_breakers_dir: None,
        }
    }
}

impl DataConfig {
    /// Get the actual snapshots directory path (expanded from ~)
    pub fn snapshots_path(&self) -> PathBuf {
        expand_path(
            self.snapshots_dir
                .as_ref()
                .unwrap_or(&format!("{}/snapshots", self.base_dir)),
        )
    }

    /// Get the actual config history directory path (expanded from ~)
    pub fn config_history_path(&self) -> PathBuf {
        expand_path(
            self.config_history_dir
                .as_ref()
                .unwrap_or(&format!("{}/config-history", self.base_dir)),
        )
    }

    /// Get the actual encryption keys directory path (expanded from ~)
    pub fn encryption_keys_path(&self) -> PathBuf {
        expand_path(
            self.encryption_keys_dir
                .as_ref()
                .unwrap_or(&format!("{}/encryption", self.base_dir)),
        )
    }

    /// Get the actual circuit breakers directory path (expanded from ~)
    pub fn circuit_breakers_path(&self) -> PathBuf {
        expand_path(
            self.circuit_breakers_dir
                .as_ref()
                .unwrap_or(&format!("{}/circuit-breakers", self.base_dir)),
        )
    }

    /// Get the base directory path (expanded from ~)
    pub fn base_path(&self) -> PathBuf {
        expand_path(&self.base_dir)
    }
}

/// Expand ~ to home directory
fn expand_path(path: &str) -> PathBuf {
    if path.starts_with("~/") || path == "~" {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let rest = if path == "~" { "" } else { &path[2..] };
        home.join(rest)
    } else {
        PathBuf::from(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_path() {
        let expanded = expand_path("~/.local/share/mcp-context-browser");
        assert!(expanded
            .to_string_lossy()
            .contains(".local/share/mcp-context-browser"));
    }

    #[test]
    fn test_expand_path_home_shortcut() {
        let expanded = expand_path("~");
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        assert_eq!(expanded, home);
    }

    #[test]
    fn test_expand_path_absolute() {
        let expanded = expand_path("/tmp/test");
        assert_eq!(expanded, PathBuf::from("/tmp/test"));
    }
}
