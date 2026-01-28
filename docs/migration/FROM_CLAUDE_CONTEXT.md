# Migration Guide: From Claude-context to mcb

This guide helps you migrate from [zilliztech/Claude-context](https://github.com/zilliztech/claude-context) to mcb.

## Why Migrate?

| Feature | Claude-context | mcb |
|---------|----------------|---------------------|
|**Runtime**| Node.js 20-23 | Native Rust binary |
|**MCP Tools**| 4 tools | 4 tools (same interface) |
|**Hybrid Search**| BM25 + vector | BM25 + vector |
|**Embedding Providers**| 4 | 6 (+ FastEmbed, Mock) |
|**Vector Stores**| 2 (Milvus/Zilliz) | 3 (In-Memory, Encrypted, Null) |
|**Languages**| 13+ | 13 (Rust, Python, JS/TS, Go, Java, C, C++, C#, Ruby, PHP, Swift, Kotlin) |
|**Performance**| Node.js interpreter | Native compiled |
|**Dependencies**| npm packages | Single binary |

## Quick Migration

### Step 1: Install mcb

```bash

# Download the latest release
curl -LO https://github.com/your-org/mcb/releases/latest/download/mcb-linux-x86_64.tar.gz
tar xzf mcb-linux-x86_64.tar.gz
sudo mv mcb /usr/local/bin/
```

### Step 2: Keep Your Environment Variables

mcb is fully compatible with Claude-context environment variables:

| Claude-context | mcb | Status |
|----------------|---------------------|--------|
| `OPENAI_API_KEY` | `OPENAI_API_KEY` | Direct support |
| `VOYAGE_API_KEY` | `VOYAGE_API_KEY` | Direct support |
| `OLLAMA_BASE_URL` | `OLLAMA_BASE_URL` | Direct support |
| `GEMINI_API_KEY` | `GEMINI_API_KEY` | Direct support |
| `MILVUS_TOKEN` | `MILVUS_TOKEN` | Direct support |
| `MILVUS_ADDRESS` | `MILVUS_ADDRESS` | Direct support |

**No changes required to your existing environment variables!**

### Step 3: Update Claude Desktop Configuration

Replace the Claude-context entry in your `claude_desktop_config.json`:

**Before (Claude-context):**

```json
{
  "mcpServers": {
    "context": {
      "command": "npx",
      "args": ["-y", "@anthropic/claude-context"],
      "env": {
        "OPENAI_API_KEY": "sk-...",
        "MILVUS_ADDRESS": "http://localhost:19530"
      }
    }
  }
}
```

**After (mcb):**

```json
{
  "mcpServers": {
    "context": {
      "command": "/usr/local/bin/mcb",
      "args": [],
      "env": {
        "OPENAI_API_KEY": "sk-...",
        "MILVUS_ADDRESS": "http://localhost:19530"
      }
    }
  }
}
```

### Step 4: Verify Installation

```bash

# Check version
mcb --version

# Test with your existing index

# The MCP tools work identically to claude-context
```

## MCP Tools Comparison

Both tools provide the same 4 MCP tools with identical interfaces:

| Tool | Description | Compatibility |
|------|-------------|---------------|
| `index_codebase` | Index a directory with AST-aware chunking | 100% compatible |
| `search_code` | Semantic + BM25 hybrid search | 100% compatible |
| `get_indexing_status` | Check indexing progress | 100% compatible |
| `clear_index` | Remove indexed data | 100% compatible |

## Provider Selection

### Embedding Providers

Set `EMBEDDING_PROVIDER` environment variable:

```bash

# OpenAI (default)
EMBEDDING_PROVIDER=openai
OPENAI_API_KEY=sk-...

# VoyageAI (code-optimized)
EMBEDDING_PROVIDER=voyageai
VOYAGE_API_KEY=...

# Ollama (local)
EMBEDDING_PROVIDER=ollama
OLLAMA_BASE_URL=http://localhost:11434

# Gemini
EMBEDDING_PROVIDER=gemini
GEMINI_API_KEY=...

# FastEmbed (local, no API key)
EMBEDDING_PROVIDER=fastembed
```

### Vector Store Providers

Set `VECTOR_STORE_PROVIDER` environment variable:

```bash

# Milvus/Zilliz (default)
VECTOR_STORE_PROVIDER=milvus
MILVUS_ADDRESS=http://localhost:19530
MILVUS_TOKEN=...  # optional

# In-Memory (development)
VECTOR_STORE_PROVIDER=in-memory

# Filesystem (persistent, no external deps)
VECTOR_STORE_PROVIDER=filesystem

# EdgeVec (embedded vector store)
VECTOR_STORE_PROVIDER=edgevec
```

## Language Support

Both tools support the same core languages. mcb v0.1.0 now includes:

**Original (matching Claude-context):**

-   Rust, Python, JavaScript, TypeScript
-   Go, Java, C, C++, C#

**Added in v0.1.0:**

-   Ruby, PHP, Swift, Kotlin

## Differences

### Improvements in mcb

1.**No Node.js required**- Single binary, no npm/npx
2.**Faster startup**- Native compilation vs interpreter
3.**Lower memory**- Rust memory efficiency
4.**More providers**- FastEmbed (local), EdgeVec, Filesystem stores
5.**Enterprise features**- JWT auth, rate limiting, encryption at rest

### Behavioral Differences

| Aspect | Claude-context | mcb |
|--------|----------------|---------------------|
| Config format | convict.js schema | TOML config |
| Config location | `~/.context/config.json` | `~/.context/config.toml` |
| Default model (OpenAI) | text-embedding-3-small | text-embedding-3-small |
| Default model (VoyageAI) | voyage-code-3 | voyage-code-3 |

## Troubleshooting

### "API key required" errors

Ensure environment variables are set:

```bash
export OPENAI_API_KEY=sk-...

# or
export VOYAGE_API_KEY=...
```

### Connection to Milvus fails

Check Milvus is running and accessible:

```bash
curl http://localhost:19530/v1/vector/health
```

### Index not found after migration

Re-index your codebase:

```

# In Claude Desktop, use the index_codebase tool
```

## Getting Help

-   GitHub Issues: <https://github.com/your-org/mcb/issues>
-   Documentation: <https://github.com/your-org/mcb/docs>

## Rollback

If you need to temporarily rollback:

```json
{
  "mcpServers": {
    "context": {
      "command": "npx",
      "args": ["-y", "@anthropic/claude-context"]
    }
  }
}
```

Your environment variables work with both tools.
