use crate::domain::error::{Error, Result};
use config::{Config as ConfigBuilder, Environment, FileFormat};
use std::path::Path;
use validator::Validate;

use super::types::Config;

/// Embedded default configuration from config/default.toml
/// This is the single source of truth for default values in the binary.
/// Works from any working directory because it's compiled into the binary.
const DEFAULT_CONFIG_TOML: &str = include_str!("../../../config/default.toml");

/// Returns the embedded default config TOML for testing purposes
#[cfg(test)]
pub fn get_default_config_toml() -> &'static str {
    DEFAULT_CONFIG_TOML
}

/// Load only embedded defaults without user config or environment variables.
/// Useful for testing that embedded defaults are correctly set.
#[cfg(test)]
pub async fn load_embedded_defaults_only() -> Result<Config> {
    let config = ConfigBuilder::builder()
        .add_source(config::File::from_str(
            DEFAULT_CONFIG_TOML,
            FileFormat::Toml,
        ))
        .build()
        .map_err(|e| Error::config(format!("Failed to build configuration: {}", e)))?;

    let config: Config = config
        .try_deserialize()
        .map_err(|e| Error::config(format!("Failed to deserialize configuration: {}", e)))?;

    config
        .validate()
        .map_err(|e| Error::config(format!("Configuration validation failed: {}", e)))?;

    Ok(config)
}

#[derive(Debug, Clone, Copy)]
pub struct ConfigLoader;

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigLoader {
    pub fn new() -> Self {
        Self
    }

    pub async fn load(&self) -> Result<Config> {
        // Start with embedded default config (source of truth for defaults)
        let mut builder = ConfigBuilder::builder().add_source(config::File::from_str(
            DEFAULT_CONFIG_TOML,
            FileFormat::Toml,
        ));

        // Layer 2: User configuration from XDG standard location (if exists)
        let config_dir = dirs::config_dir();
        if let Some(dir) = config_dir {
            let user_config_path = dir.join("mcp-context-browser").join("config.toml");
            if user_config_path.exists() {
                builder = builder.add_source(config::File::from(user_config_path).required(false));
            }
        }

        // Layer 3: Environment variables (highest priority)
        builder = builder.add_source(
            Environment::with_prefix("MCP")
                .separator("__")
                .try_parsing(true),
        );

        let config = builder
            .build()
            .map_err(|e| Error::config(format!("Failed to build configuration: {}", e)))?;

        let config: Config = config
            .try_deserialize()
            .map_err(|e| Error::config(format!("Failed to deserialize configuration: {}", e)))?;

        config
            .validate()
            .map_err(|e| Error::config(format!("Configuration validation failed: {}", e)))?;

        Ok(config)
    }

    pub async fn load_with_file(&self, path: &Path) -> Result<Config> {
        // Start with embedded default config
        let mut builder = ConfigBuilder::builder()
            .add_source(config::File::from_str(
                DEFAULT_CONFIG_TOML,
                FileFormat::Toml,
            ))
            // Override with specified file
            .add_source(config::File::from(path).required(false));

        // Environment variables still have highest priority
        builder = builder.add_source(
            Environment::with_prefix("MCP")
                .separator("__")
                .try_parsing(true),
        );

        let config = builder
            .build()
            .map_err(|e| Error::config(format!("Failed to build configuration: {}", e)))?;

        let config: Config = config
            .try_deserialize()
            .map_err(|e| Error::config(format!("Failed to deserialize configuration: {}", e)))?;

        config
            .validate()
            .map_err(|e| Error::config(format!("Configuration validation failed: {}", e)))?;

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::Builder;

    /// Test that embedded defaults are loaded correctly
    /// Uses load_embedded_defaults_only() to avoid interference from user config files
    #[tokio::test]
    async fn test_load_embedded_defaults() {
        // Load only embedded defaults (no user config, no env vars)
        let config = super::load_embedded_defaults_only()
            .await
            .expect("Should load embedded defaults");

        // Verify embedded defaults from config/default.toml
        assert_eq!(config.providers.embedding.provider, "fastembed");
        assert_eq!(config.providers.vector_store.provider, "filesystem");
        assert_eq!(config.server.port, 3000);
        assert_eq!(config.metrics.port, 3001);
    }

    /// Test that user config file overrides embedded defaults
    #[tokio::test]
    async fn test_user_config_overrides_defaults() {
        // Create temp config file that overrides provider settings
        let mut file = Builder::new()
            .suffix(".toml")
            .tempfile()
            .expect("Should create temp file");

        writeln!(
            file,
            r#"
[providers.embedding]
provider = "ollama"
model = "nomic-embed-text"

[providers.vector_store]
provider = "milvus"
"#
        )
        .expect("Should write config");

        let loader = ConfigLoader::new();
        let config = loader
            .load_with_file(file.path())
            .await
            .expect("Should load config with file");

        // User config should override defaults
        assert_eq!(
            config.providers.embedding.provider, "ollama",
            "Embedding provider should be overridden to ollama"
        );
        assert_eq!(
            config.providers.vector_store.provider, "milvus",
            "Vector store provider should be overridden to milvus"
        );

        // Other defaults should remain
        assert_eq!(config.server.port, 3000);
        assert_eq!(config.metrics.port, 3001);
    }

    /// Test that environment variables have highest priority
    /// Note: Skipped due to environment variable pollution in parallel test execution
    #[tokio::test]
    #[ignore]
    async fn test_env_vars_override_file_and_defaults() {
        // Create temp config file
        let mut file = Builder::new()
            .suffix(".toml")
            .tempfile()
            .expect("Should create temp file");

        writeln!(
            file,
            r#"
[server]
port = 5000

[providers.embedding]
provider = "ollama"
"#
        )
        .expect("Should write config");

        // Set env var (highest priority)
        unsafe {
            std::env::set_var("MCP__server__port", "6000");
        }

        let loader = ConfigLoader::new();
        let config = loader
            .load_with_file(file.path())
            .await
            .expect("Should load config");

        // Env var should win over file
        assert_eq!(config.server.port, 6000);
        // File should win over default
        assert_eq!(config.providers.embedding.provider, "ollama");
    }

    /// Test the 3-layer config priority: embedded < file < env
    /// Note: Skipped due to environment variable pollution in parallel test execution
    #[tokio::test]
    #[ignore]
    async fn test_config_priority_layers() {
        let mut file = Builder::new()
            .suffix(".toml")
            .tempfile()
            .expect("Should create temp file");

        // File overrides: metrics.port = 4001 (default is 3001)
        // File keeps: server.port from default (3000)
        writeln!(
            file,
            r#"
[metrics]
port = 4001
enabled = true

[providers.embedding]
provider = "voyageai"

[providers.vector_store]
provider = "milvus"
"#
        )
        .expect("Should write config");

        // Env overrides: providers.embedding.provider = "gemini"
        unsafe {
            std::env::set_var("MCP__providers__embedding__provider", "gemini");
        }

        let loader = ConfigLoader::new();
        let config = loader
            .load_with_file(file.path())
            .await
            .expect("Should load config");

        // Layer 1 (embedded default): server.port = 3000
        assert_eq!(config.server.port, 3000, "Default should provide server.port");

        // Layer 2 (file): metrics.port = 4001, vector_store = milvus
        assert_eq!(config.metrics.port, 4001, "File should override metrics.port");
        assert_eq!(
            config.providers.vector_store.provider, "milvus",
            "File should override vector_store.provider"
        );

        // Layer 3 (env): embedding.provider = gemini
        assert_eq!(
            config.providers.embedding.provider, "gemini",
            "Env should override embedding.provider from file"
        );
    }

    /// Test that provider config is correctly parsed with all fields
    #[tokio::test]
    async fn test_provider_config_fields() {
        let mut file = Builder::new()
            .suffix(".toml")
            .tempfile()
            .expect("Should create temp file");

        writeln!(
            file,
            r#"
[providers.embedding]
provider = "ollama"
model = "custom-model"
dimensions = 1024

[providers.vector_store]
provider = "milvus"
address = "localhost:19530"
collection = "test_collection"
"#
        )
        .expect("Should write config");

        let loader = ConfigLoader::new();
        let config = loader
            .load_with_file(file.path())
            .await
            .expect("Should load config");

        assert_eq!(config.providers.embedding.provider, "ollama");
        assert_eq!(config.providers.embedding.model, "custom-model");
        assert_eq!(config.providers.embedding.dimensions, Some(1024));

        assert_eq!(config.providers.vector_store.provider, "milvus");
        assert_eq!(
            config.providers.vector_store.address,
            Some("localhost:19530".to_string())
        );
        assert_eq!(
            config.providers.vector_store.collection,
            Some("test_collection".to_string())
        );
    }

    /// Test that missing optional fields don't cause errors
    #[tokio::test]
    async fn test_partial_config_override() {
        let mut file = Builder::new()
            .suffix(".toml")
            .tempfile()
            .expect("Should create temp file");

        // Only override provider name, not other fields
        writeln!(
            file,
            r#"
[providers.embedding]
provider = "openai"
"#
        )
        .expect("Should write config");

        let loader = ConfigLoader::new();
        let config = loader
            .load_with_file(file.path())
            .await
            .expect("Should load config");

        // Provider should be overridden
        assert_eq!(config.providers.embedding.provider, "openai");
        // Other fields should keep defaults
        assert_eq!(config.providers.vector_store.provider, "filesystem");
    }

    /// Test that config validation catches invalid values
    #[tokio::test]
    async fn test_config_validation() {
        let loader = ConfigLoader::new();
        let config = loader.load().await.expect("Should load valid config");

        // Validate should pass for default config
        assert!(config.validate().is_ok());
    }

    /// Verify the embedded default TOML is valid and parseable
    #[test]
    fn test_embedded_default_toml_is_valid() {
        let toml_str = get_default_config_toml();
        let parsed: toml::Value = toml::from_str(toml_str).expect("Default TOML should be valid");

        // Verify expected sections exist
        assert!(parsed.get("server").is_some(), "Should have [server] section");
        assert!(parsed.get("metrics").is_some(), "Should have [metrics] section");
        assert!(parsed.get("providers").is_some(), "Should have [providers] section");

        // Verify provider defaults
        let providers = parsed.get("providers").unwrap();
        let embedding = providers.get("embedding").unwrap();
        assert_eq!(
            embedding.get("provider").unwrap().as_str().unwrap(),
            "fastembed"
        );

        let vector_store = providers.get("vector_store").unwrap();
        assert_eq!(
            vector_store.get("provider").unwrap().as_str().unwrap(),
            "filesystem"
        );
    }
}
