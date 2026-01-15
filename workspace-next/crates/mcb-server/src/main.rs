//! MCP Context Browser Server - Clean Architecture Implementation
//!
//! Enterprise semantic code search server following Clean Architecture principles.
//! Features dependency injection, domain-driven design, and MCP protocol compliance.
//!
//! Architecture:
//! - Domain layer: Core business logic and contracts (mcb-domain)
//! - Infrastructure: Cross-cutting concerns and external integrations (mcb-infrastructure)
//! - Server: Transport and protocol layer (mcb-server)

use clap::Parser;
use mcb_server::run_server;

/// Command line interface for MCP Context Browser Server
#[derive(Parser, Debug)]
#[command(name = "mcb-server")]
#[command(about = "MCP Context Browser - Clean Architecture Semantic Code Search Server")]
#[command(version)]
struct Cli {
    /// Path to configuration file
    #[arg(short, long)]
    config: Option<std::path::PathBuf>,
}

/// Main entry point for the MCP Context Browser server
///
/// Parses command line arguments and starts the MCP server with dependency injection.
/// The server follows Clean Architecture principles with clear separation of concerns.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    run_server(cli.config.as_deref()).await
}