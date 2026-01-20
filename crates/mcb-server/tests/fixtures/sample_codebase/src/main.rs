//! MCP Context Browser - Main Entry Point
//!
//! This is the main entry point for the MCP Context Browser server
//! that provides semantic code search capabilities.

mod cache;
mod chunking;
mod di;
mod embedding;
mod error;
mod handlers;
mod vector_store;

use std::sync::Arc;

/// Application configuration
pub struct Config {
    pub host: String,
    pub port: u16,
    pub embedding_provider: String,
    pub vector_store_provider: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            embedding_provider: "null".to_string(),
            vector_store_provider: "memory".to_string(),
        }
    }
}

/// Main function - entry point
fn main() {
    println!("MCP Context Browser starting...");

    let config = Config::default();
    println!("Server: {}:{}", config.host, config.port);
    println!("Embedding: {}", config.embedding_provider);
    println!("Vector Store: {}", config.vector_store_provider);

    // In real implementation:
    // 1. Initialize DI container
    // 2. Start HTTP server
    // 3. Register MCP protocol handlers
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 8080);
    }
}
