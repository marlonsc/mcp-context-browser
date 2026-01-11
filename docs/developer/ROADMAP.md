# Development Roadmap

## Overview

This roadmap outlines the development of MCP Context Browser, a drop-in replacement for claude-context with enhanced capabilities for semantic code search.

---

## Current Status

### v0.1.0 - First Stable Release ✅ RELEASED

**Status**: Production-Ready
**Release Date**: January 2026

MCP Context Browser v0.1.0 is the first stable release, providing a complete drop-in replacement for claude-context with superior performance and expanded capabilities.

#### Achievements

- ✅ Full MCP protocol implementation (4 tools)
- ✅ 14 languages with AST parsing (Rust, Python, JS/TS, Go, Java, C, C++, C#, Ruby, PHP, Swift, Kotlin, Scala, Haskell)
- ✅ 6 embedding providers (OpenAI, VoyageAI, Ollama, Gemini, FastEmbed, Mock)
- ✅ 6 vector stores (Milvus, EdgeVec, In-Memory, Filesystem, Encrypted, Null)
- ✅ claude-context environment variable compatibility
- ✅ 391+ tests with comprehensive coverage
- ✅ JWT authentication and rate limiting
- ✅ Clean architecture with trait-based dependency injection
- ✅ HTTP transport foundation for future enhancements
- ✅ Systemd service integration
- ✅ Binary auto-respawn mechanism

---

## Upcoming Releases

### v0.2.0 - Git-Aware Semantic Indexing 🚧 PLANNED

**Status**: Planning Complete (ADR-008)
**Priority**: High
**Estimated Effort**: 10 phases

#### Vision

Transform MCP Context Browser from a filesystem-based indexer to a git-aware semantic search platform supporting monorepos, multiple branches, commit history, and change impact analysis.

#### Objectives

1. **Project-Relative Indexing**: Indexes remain valid even if directory is moved
2. **Git Monorepo Support**: Multiple projects, submodules, branches, remotes
3. **Change Impact Understanding**: Automatic analysis of changes between commits/branches

#### New Capabilities

| Capability | Description |
|------------|-------------|
| Repository ID | Portable identification via root commit hash |
| Multi-Branch Indexing | Index main, HEAD, and current branch (configurable) |
| Commit History | Last 50 commits indexed by default |
| Submodule Support | Recursive indexing as separate projects |
| Project Detection | Auto-detect Cargo, npm, Python, Go, Maven projects in monorepos |
| Impact Analysis | Semantic analysis of change impact between refs |

#### New MCP Tools

| Tool | Purpose |
|------|---------|
| `index_git_repository` | Index repository with branch awareness |
| `search_branch` | Search within specific branch |
| `compare_branches` | Compare code between branches |
| `analyze_impact` | Analyze change impact between refs |
| `list_repositories` | List indexed repositories |

#### Technical Details

- **New Dependency**: git2 (libgit2 bindings)
- **New Files**: ~12 source files
- **Estimated LOC**: ~2500
- **ADR**: [008-git-aware-semantic-indexing-v0.2.0](../adr/008-git-aware-semantic-indexing-v0.2.0.md)

#### Configuration Defaults

| Setting | Default | Override |
|---------|---------|----------|
| Branches | main, HEAD, current | Per-repo via `.mcp-context.toml` |
| History depth | 50 commits | Per-repo |
| Submodules | Recursive indexing | Per-repo |

---

### v0.3.0 - Advanced Code Intelligence 📋 FUTURE

**Status**: Conceptual
**Priority**: Medium

#### Objectives

- Symbol extraction and cross-referencing
- Call graph analysis
- Dependency impact mapping
- Code similarity detection
- Refactoring suggestions

---

### v0.4.0 - Enterprise Features 📋 FUTURE

**Status**: Conceptual
**Priority**: Medium

#### Objectives

- Multi-tenant support
- Advanced RBAC with team permissions
- SSO integration (SAML, OIDC)
- Audit logging enhancements
- Cost tracking and optimization

---

### v1.0.0 - Production Enterprise 📋 FUTURE

**Status**: Conceptual
**Priority**: High

#### Objectives

- Full enterprise feature set
- SLA guarantees
- Professional support model
- Compliance certifications (SOC 2, ISO 27001)
- High availability clustering

---

## Version History

| Version | Status | Key Features |
|---------|--------|--------------|
| v0.0.1 | Released | Initial prototype |
| v0.0.2 | Released | Core architecture |
| v0.0.3 | Released | Production foundation |
| v0.0.4 | Released | Documentation excellence, clean architecture |
| v0.1.0 | **Released** | First stable release, claude-context replacement |
| v0.2.0 | **Planned** | Git-aware semantic indexing |
| v0.3.0 | Future | Advanced code intelligence |
| v0.4.0 | Future | Enterprise features |
| v1.0.0 | Future | Production enterprise |

---

## Implementation Principles

### Development Practices

1. **ADR-Driven Development**: Architectural decisions documented before implementation
2. **Test-First**: Core functionality developed with comprehensive tests
3. **Clean Architecture**: Separation of concerns with trait-based DI
4. **Documentation First**: Documentation updated with each code change
5. **Security by Design**: Security considerations in every component

### Quality Gates

All releases must pass:

- [ ] All tests pass (unit, integration, e2e)
- [ ] Code coverage meets targets (>85%)
- [ ] Clippy lint clean
- [ ] Security audit clean
- [ ] Performance benchmarks maintained
- [ ] Documentation complete and accurate

---

## Cross-References

- **Architecture**: [ARCHITECTURE.md](../architecture/ARCHITECTURE.md)
- **Contributing**: [CONTRIBUTING.md](./CONTRIBUTING.md)
- **ADR Index**: [docs/adr/README.md](../adr/README.md)
- **Version History**: [VERSION_HISTORY.md](../VERSION_HISTORY.md)
- **Deployment**: [DEPLOYMENT.md](../operations/DEPLOYMENT.md)
