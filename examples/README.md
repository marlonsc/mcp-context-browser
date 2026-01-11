# Examples

This directory contains example applications demonstrating MCP Context Browser v0.1.0 capabilities.

## Available Examples

### 1. Configuration Demo (`config_demo.rs`)

Demonstrates the professional configuration system with TOML schema validation.

```bash
cargo run --example config_demo
```

**Features demonstrated:**

-   TOML configuration loading
-   Environment variable interpolation
-   Provider configuration (OpenAI, VoyageAI, Ollama, Gemini)
-   Vector store configuration (Milvus, InMemory, Filesystem)
-   Configuration validation and error handling

### 2. Multi-Provider Routing (`advanced_routing.rs`)

Demonstrates the multi-provider routing system with intelligent failover.

```bash
cargo run --example advanced_routing
```

**Features demonstrated:**

-   Provider registry initialization
-   Router configuration with multiple providers
-   Selection strategy implementation
-   Health monitoring integration
-   Circuit breaker patterns
-   Automatic failover between providers

### 3. FastEmbed Provider (`fastembed_example.rs`)

Demonstrates local embedding generation without external API dependencies.

```bash
cargo run --example fastembed_example
```

**Features demonstrated:**

-   FastEmbed provider configuration
-   Local model loading (AllMiniLML6V2)
-   Embedding generation without external APIs
-   Provider factory usage
-   Dimension and token configuration

## Running Examples

### Prerequisites

-   Rust 1.70+
-   Cargo build tools

### Environment Setup

Some examples require environment variables:

```bash
# For OpenAI provider
export OPENAI_API_KEY="your-api-key"

# For VoyageAI provider
export VOYAGEAI_API_KEY="your-api-key"

# For Ollama provider (local)
export OLLAMA_URL="http://localhost:11434"
```

### Build and Run

```bash
# Build all examples
cargo build --examples

# Run specific example
cargo run --example config_demo
cargo run --example advanced_routing
cargo run --example fastembed_example

# Run with environment file
source .env && cargo run --example config_demo
```

## Example Output

### config_demo

```text
Configuration Demo - Professional Config System
================================================
Loading configuration from config.toml...
Embedding Provider: openai
Vector Store: milvus
```

### advanced_routing

```text
MCP Context Browser - Multi-Provider Strategy Demo
====================================================
Registering Providers...
Provider 1: openai (healthy)
Provider 2: ollama (healthy)
Routing request through: openai
```

### fastembed_example

```text
FastEmbedProvider Example
============================
Configuration:
   Provider: fastembed
   Model: AllMiniLML6V2
   Dimensions: 384
```

## Creating New Examples

1.  Create a new `.rs` file in the `examples/` directory
2.  Add the `#[tokio::main]` attribute for async examples
3.  Document the example with doc comments (`//!`)
4.  Include error handling and user feedback
5.  Test the example before committing

### Example Template

```rust
//! Example demonstrating [feature name]
//!
//! Run with: cargo run --example [example_name]

use mcp_context_browser::core::error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Example - [Feature Name]");
    println!("=========================");

    // Your example code here

    Ok(())
}
```

---

## Cross-References

-   **Architecture**: [ARCHITECTURE.md](../docs/architecture/ARCHITECTURE.md)
-   **Contributing**: [CONTRIBUTING.md](../docs/developer/CONTRIBUTING.md)
-   **Tests**: [tests/](../tests/)
-   **Module Documentation**: [docs/modules/](../docs/modules/)
