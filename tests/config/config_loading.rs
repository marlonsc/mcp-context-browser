use mcp_context_browser::config::ConfigLoader;
use std::env;
use std::io::Write;
use tempfile::Builder;

#[tokio::test]
async fn test_config_loader_priority() {
    // Set some env vars (lowercase keys to match config structure)
    // Note: config-rs doesn't auto-lowercase, so use lowercase key names
    unsafe {
        env::set_var("MCP__server__port", "4000");
        env::set_var("MCP__metrics__port", "4001");
    }

    // Create a temp config file with .toml extension
    let mut file = Builder::new().suffix(".toml").tempfile().unwrap();

    writeln!(
        file,
        r#"
[server]
port = 5000
host = "0.0.0.0"

[metrics]
port = 5001
enabled = true
"#
    )
    .unwrap();

    // Load config
    let loader = ConfigLoader::new();
    let config = loader.load_with_file(file.path()).await.unwrap();

    // Cleanup env vars to prevent test pollution
    unsafe {
        env::remove_var("MCP__server__port");
        env::remove_var("MCP__metrics__port");
    }

    assert_eq!(config.server.port, 4000); // Env priority
    assert_eq!(config.server.host, "0.0.0.0"); // File fallback
    assert_eq!(config.metrics.port, 4001); // Env priority
}
