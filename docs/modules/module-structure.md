# Module Structure

This document shows the hierarchical structure of modules in the MCP Context Browser.

## Module Tree

```
mcp-context-browser/
├── main.rs (entry point)
├── lib.rs (library exports)
├── core/ (core types and utilities)
│   ├── error.rs
│   ├── types.rs
│   ├── cache.rs
│   ├── limits.rs
│   └── rate_limit.rs
├── config.rs (configuration)
├── providers/ (provider implementations)
│   ├── mod.rs
│   ├── embedding/
│   └── vector_store/
├── services/ (business logic)
│   ├── mod.rs
│   ├── context.rs
│   ├── indexing.rs
│   └── search.rs
├── server/ (MCP protocol server)
│   └── mod.rs
├── metrics/ (monitoring)
│   ├── mod.rs
│   ├── http_server.rs
│   └── system.rs
├── sync/ (cross-process coordination)
│   └── mod.rs
├── daemon/ (background processes)
│   └── mod.rs
└── snapshot/ (change tracking)
    └── mod.rs
```

## Analysis

This is a simplified module structure generated as fallback when cargo-modules analysis is not available. The actual structure may be more complex with additional submodules and dependencies.

*Generated automatically on: 2026-01-07 21:25:37 UTC*
