# Architecture Decision Records

This directory contains all Architecture Decision Records (ADRs) for the MCP Context Browser project.

## Current ADRs

### Core Architecture

\1-   [ADR 001: Provider Pattern Architecture](001-provider-pattern-architecture.md)
\1-   [ADR 002: Async-First Architecture](002-async-first-architecture.md)
\1-   [ADR 003: C4 Model Documentation](003-c4-model-documentation.md)
\1-   [ADR 004: Multi-Provider Strategy](004-multi-provider-strategy.md)

### Documentation & Quality

\1-   [ADR 005: Documentation Excellence](005-documentation-excellence.md)
\1-   [ADR 006: Code Audit and Architecture Improvements](006-code-audit-and-improvements.md)
\1-   [ADR 007: Integrated Web Administration Interface](007-integrated-web-administration-interface.md)

### v0.2.0 Features (Planned)

\1-   [ADR 008: Git-Aware Semantic Indexing v0.2.0](008-git-aware-semantic-indexing-v0.2.0.md)
\1-   [ADR 009: Persistent Session Memory v0.2.0](009-persistent-session-memory-v0.2.0.md)
\1-   [ADR 010: Hooks Subsystem with Agent-Backed Processing](010-hooks-subsystem-agent-backed.md)

### Clean Architecture & DI (v0.1.1)

\1-   [ADR 011: HTTP Transport Request/Response Pattern](011-http-transport-request-response-pattern.md)
\1-   [ADR 012: Two-Layer DI Strategy](012-di-strategy-two-layer-approach.md) - Shaku modules + runtime factories
\1-   [ADR 013: Clean Architecture Crate Separation](013-clean-architecture-crate-separation.md) - Seven-crate workspace organization

## ADR Status Legend

| Status | Meaning |
|--------|---------|
| Proposed | Under discussion |
| Accepted | Approved and to be implemented |
| Implemented | Completed in codebase |
| Deprecated | No longer relevant |
| Superseded | Replaced by another ADR |

## Creating New ADRs

Use the sequential numbering format: `XXX-descriptive-name.md`

See [ADR Template](../architecture/ARCHITECTURE.md#adr-template) for the standard format
