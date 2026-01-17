# Quickstart Guide

Get MCP Context Browser v0.1.1 running in 5 minutes.

## 1. Download

### Pre-built Binary (Recommended)

```bash

# Linux x86_64
curl -LO https://github.com/marlonsc/mcp-context-browser/releases/latest/download/mcp-context-browser-linux-x86_64.tar.gz
tar xzf mcp-context-browser-linux-x86_64.tar.gz
sudo mv mcp-context-browser /usr/local/bin/
```

### From Source

```bash
git clone https://github.com/marlonsc/mcp-context-browser.git
cd mcp-context-browser
make build-release

# Binary at: ./target/release/mcp-context-browser
```

## 2. Configure

### Option A: OpenAI (Cloud)

```bash
export EMBEDDING_PROVIDER=openai
export OPENAI_API_KEY=sk-your-key-here
```

### Option B: Ollama (Local, Free)

```bash

# Start Ollama
ollama serve &
ollama pull nomic-embed-text

# Configure
export EMBEDDING_PROVIDER=ollama
export OLLAMA_BASE_URL=http://localhost:11434
```

### Option C: FastEmbed (Local, No Setup)

```bash
export EMBEDDING_PROVIDER=fastembed

# No API key needed - uses local models
```

## 3. Connect to Claude Desktop

Add to `~/.config/Claude/claude_desktop_config.json` (Linux) or `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS):

```json
{
  "mcpServers": {
    "context": {
      "command": "/usr/local/bin/mcp-context-browser",
      "args": [],
      "env": {
        "EMBEDDING_PROVIDER": "openai",
        "OPENAI_API_KEY": "sk-your-key-here",
        "VECTOR_STORE_PROVIDER": "in-memory"
      }
    }
  }
}
```

## 4. Use in Claude

Restart Claude Desktop, then ask:

> "Index my project at /path/to/myproject"

Claude will use the `index_codebase` tool to index your code.

Then search:

> "Find where user authentication is handled"

Claude will use `search_code` to find relevant code.

## Available MCP Tools

| Tool | What it does |
|------|--------------|
| `index_codebase` | Index a directory for semantic search |
| `search_code` | Search indexed code with natural language |
| `get_indexing_status` | Check indexing progress |
| `clear_index` | Remove indexed data |

## Supported Languages (12)

Rust, Python, JavaScript, TypeScript, Go, Java, C, C++, C#, Ruby, PHP, Swift, Kotlin

## Vector Store Options

| Provider | Best for | Setup |
|----------|----------|-------|
| `in-memory` | Development, testing | None |
| `filesystem` | Persistent storage | None |
| `milvus` | Production scale | Docker/Kubernetes |
| `edgevec` | Embedded use | None |

### Using Milvus

```bash

# Start Milvus with Docker
docker run -d --name milvus -p 19530:19530 milvusdb/milvus:latest

# Configure
export VECTOR_STORE_PROVIDER=milvus
export MILVUS_ADDRESS=http://localhost:19530
```

## Troubleshooting

### "API key required"

Set the appropriate environment variable:

```bash
export OPENAI_API_KEY=sk-...

# or
export VOYAGE_API_KEY=...
```

### "Connection refused"

Check if Ollama/Milvus is running:

```bash
curl http://localhost:11434/api/version  # Ollama
curl http://localhost:19530/v1/vector/health  # Milvus
```

### Claude doesn't see the tools

1.  Check config file location
2.  Restart Claude Desktop
3.  Look for errors in Claude's logs

## Next Steps

\1-   [Migration Guide](../migration/FROM_CLAUDE_CONTEXT.md) - If coming from Claude-context
\1-   [Architecture](../architecture/ARCHITECTURE.md) - Understanding the system
\1-   [ADR Index](../adr/README.md) - Architectural decisions
\1-   [Version History](../VERSION_HISTORY.md) - Complete version history
\1-   [Roadmap](../developer/ROADMAP.md) - Upcoming features including v0.2.0 Git-Aware Indexing
