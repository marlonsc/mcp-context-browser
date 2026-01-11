# API Surface Analysis

This document provides an overview of the public API surface of the MCP Context Browser.

## Public Modules

### Core Library Modules

- adapters
- admin
- application
- chunking
- config_example
- daemon
- domain
- infrastructure
- server
- snapshot
- sync

### Public Re-exports

- domain::error::{Error, Result}
- domain::types::*
- server::builder::McpServerBuilder
- server::init::run_server
- server::mcp_server::McpServer

## Public Functions

## Public Types

### Data Structures

- NodeExtractionRule 
- LanguageConfig 
- NodeExtractionRuleBuilder 
- IntelligentChunker;
- GenericFallbackChunker<'a> 
- RustProcessor 
- ".to_string(),
- PythonProcessor 
- JavaScriptProcessor 
- JavaProcessor 

### Enums

- McpError 
- CompatibilityResult 
- SessionState 
- SessionError 
- TransportMode 

## API Stability

### Current Status
- **Version**: 0.1.0 (First Stable Release)
- **Stability**: Experimental - APIs may change
- **Compatibility**: Breaking changes expected until 1.0.0

### Public API Commitments
- MCP protocol interface stability
- Core semantic search functionality
- Provider abstraction interfaces

*Generated automatically from source code analysis on: 2026-01-11 21:51:55 UTC*
