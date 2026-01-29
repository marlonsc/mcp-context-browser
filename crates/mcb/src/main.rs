//! MCP Context Browser - Entry Point
//!
//! Binary entry point for the MCP Context Browser server.
//! Lives in the `mcb` facade crate to avoid doc output filename collision
//! with the `mcb` library crate (cargo issue #6313).
//!
//! ## Operating Modes
//!
//! | Mode | Command | Description |
//! |------|---------|-------------|
//! | **Standalone** | `mcb` (config: `mode.type = "standalone"`) | Local providers, stdio transport |
//! | **Server** | `mcb --server` | HTTP daemon, accepts client connections |
//! | **Client** | `mcb` (config: `mode.type = "client"`) | Connects to server via HTTP |

// Force-link mcb-providers to ensure linkme inventory registrations are included
extern crate mcb_providers;

use clap::Parser;
use mcb_server::run;

/// Command line interface for MCP Context Browser
#[derive(Parser, Debug)]
#[command(name = "mcb")]
#[command(about = "MCP Context Browser - Semantic Code Search Server")]
#[command(version)]
pub struct Cli {
    /// Path to configuration file
    #[arg(short, long)]
    pub config: Option<std::path::PathBuf>,

    /// Run as server daemon (HTTP + optional stdio)
    ///
    /// When this flag is set, MCB runs as a server daemon that accepts
    /// connections from MCB clients. Without this flag, MCB checks the
    /// config file to determine if it should run in standalone or client mode.
    #[arg(long, help = "Run as server daemon")]
    pub server: bool,
}

/// Main entry point for the MCP Context Browser
///
/// Dispatches to the appropriate mode based on CLI flags and configuration:
/// - `--server` flag: Run as HTTP server daemon
/// - Config `mode.type = "standalone"`: Run with local providers (default)
/// - Config `mode.type = "client"`: Connect to remote server
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    run(cli.config.as_deref(), cli.server).await
}
