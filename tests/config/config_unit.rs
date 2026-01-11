//! Unit tests for configuration system components
//!
//! Tests configuration data structures, validation, and loading.

use mcp_context_browser::infrastructure::config::Config;
use validator::Validate;

#[cfg(test)]
mod config_structure_tests {
    use super::*;

    #[test]
    fn test_config_field_access() {
        let config = Config::default();
        assert_eq!(config.name, "MCP Context Browser");
    }

    #[test]
    fn test_config_default_values() {
        let config = Config::default();
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 3000);
    }

    #[test]
    fn test_config_serialization_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let config = Config::default();
        let serialized = serde_json::to_string(&config)?;
        let deserialized: Config = serde_json::from_str(&serialized)?;
        assert_eq!(config.name, deserialized.name);
        assert_eq!(config.server.host, deserialized.server.host);
        assert_eq!(config.server.port, deserialized.server.port);
        Ok(())
    }
}

#[cfg(test)]
mod config_validation_tests {
    use super::*;

    #[test]
    fn test_config_required_fields() {
        let config = Config::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_field_constraints() {
        let config = Config::default();
        assert!(config.server.port > 0);
    }

    #[test]
    fn test_config_cross_field_validation() {
        let config = Config::default();
        assert!(!config.server.host.is_empty());
        assert!(!config.name.is_empty());
    }
}
