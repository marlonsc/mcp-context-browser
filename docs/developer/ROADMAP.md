# Development Roadmap

## Overview

This roadmap outlines the development of MCP Context Browser, a drop-in replacement for Claude-context with enhanced capabilities for semantic code search.

---

## Current Status

### v0.1.2 - Provider Modernization + Validation Tooling 🚀 CURRENT

**Status**: In Development
**Release Date**: January 18, 2026

MCP Context Browser v0.1.2 modernizes provider registration using compile-time linkme distributed slices and introduces the mcb-validate crate scaffolding.

#### Achievements

**Provider Modernization:**

-   ✅ All 15 providers migrated to linkme distributed slices (compile-time registration)
-   ✅ 4 pure linkme registries (embedding, vector store, cache, language)
-   ✅ Zero runtime overhead (provider discovery at compile time)
-   ✅ Eliminated inventory dependency (removed from Cargo.toml)

**Architecture Validation Scaffolding (mcb-validate):**

-   ✅ New mcb-validate crate (8th crate in workspace)
-   ✅ Phase 1: Linters verified (17/17 tests pass)
-   ✅ Phase 2: AST verified (26/26 tests pass)
-   ✅ Phase 3: Rule Engines verified (30/30 tests pass)
-   ✅ 12 migration validation rules (YAML files in rules/migration/)
-   ❌ Phases 4-7: Not started (directories do not exist)

**Admin UI Code Browser:**

-   ✅ VectorStoreBrowser trait in mcb-domain (ports layer)
-   ✅ CollectionInfo value object for browse metadata
-   ✅ 6 provider implementations (Milvus, InMemory, Filesystem, EdgeVec, Null, Encrypted)
-   ✅ REST API handlers (list collections, files, chunks)
-   ✅ 3 UI pages (collections grid, files list, code chunks)
-   ✅ Prism.js syntax highlighting in code viewer
-   ✅ Nav links added to all admin pages

**Verification Date**: 2026-01-18 via `make test`. See `docs/developer/IMPLEMENTATION_STATUS.md`.

**Maintained from v0.1.1:**

-   ✅ 790+ tests with comprehensive coverage (100% pass rate)
-   ✅ 6 embedding providers (OpenAI, VoyageAI, Ollama, Gemini, FastEmbed, Null)
-   ✅ 3 vector stores (In-Memory, Encrypted, Null)
-   ✅ 12 languages with AST parsing support
-   ✅ Clean architecture with trait-based dependency injection

#### Technical Metrics

-   **Source Files**: 340 Rust files (↑ from ~300 in v0.1.1)
-   **Test Suite**: 790+ tests passing (maintained)
-   **Crates**: 8 (7 from v0.1.1 + mcb-validate)
-   **Validation Rules**: 12 YAML migration rules created
-   **Provider Registration**: Compile-time via linkme (inventory removed)
-   **mcb-validate Status**: Phases 1-3 verified (73 tests), Phases 4-7 not started

---

## Recent Releases

### v0.1.0 - First Stable Release ✅ RELEASED

**Status**: Production-Ready
**Release Date**: January 11, 2026

MCP Context Browser v0.1.0 is the first stable release, providing a complete drop-in replacement for Claude-context with superior performance and expanded capabilities.

#### Achievements

-   ✅ Full MCP protocol implementation (4 tools)
-   ✅ 12 languages with AST parsing (Rust, Python, JS/TS, Go, Java, C, C++, C#, Ruby, PHP, Swift, Kotlin)
-   ✅ 6 embedding providers (OpenAI, VoyageAI, Ollama, Gemini, FastEmbed, Null)
-   ✅ 3 vector stores (In-Memory, Encrypted, Null)
-   ✅ Claude-context environment variable compatibility
-   ✅ 790+ tests with comprehensive coverage (100% pass rate)
-   ✅ JWT authentication and rate limiting
-   ✅ Clean architecture with trait-based dependency injection
-   ✅ HTTP transport foundation for future enhancements
-   ✅ Systemd service integration
-   ✅ Binary auto-respawn mechanism

---

## Upcoming Releases

### v0.2.0 - Git-Aware Indexing + Session Memory + Advanced Browser 🚧 PLANNED

**Status**: Planning Complete (ADR-008, ADR-009, ADR-028)
**Priority**: High
**Estimated Effort**: 25 phases (10 git + 10 memory + 5 browser)

#### Vision

Transform MCP Context Browser into a comprehensive development platform combining git-aware semantic search with persistent cross-session memory and IDE-like code browsing, enabling powerful code navigation and context preservation across Claude Code sessions.

#### Objectives

1.**Git-Aware Semantic Indexing**(ADR-008)

-   Project-relative indexing (portable)
-   Multi-branch support with commit history
-   Change impact analysis
-   Monorepo and submodule support

2.**Persistent Session Memory**(ADR-009)

-   Cross-session observation storage
-   Semantic search over past decisions and work
-   Token-efficient progressive disclosure (3-layer workflow)
-   Context injection for session continuity

3.**Advanced Code Browser UI**(ADR-028)

-   Tree view navigation with collapsible directories
-   Full syntax highlighting with chunk boundary markers
-   Inline search Result highlighting
-   Keyboard shortcuts and dark mode
-   Real-time SSE updates during indexing

#### New Capabilities - Git Integration

| Capability | Description |
|------------|-------------|
| Repository ID | Portable identification via root commit hash |
| Multi-Branch Indexing | Index main, HEAD, and current branch (configurable) |
| Commit History | Last 50 commits indexed by default |
| Submodule Support | Recursive indexing as separate projects |
| Project Detection | Auto-detect Cargo, npm, Python, Go, Maven projects in monorepos |
| Impact Analysis | Semantic analysis of change impact between refs |

#### New Capabilities - Session Memory

| Capability | Description |
|------------|-------------|
| Observation Storage | Persistent storage of tool outputs with metadata |
| Session Summaries | Comprehensive session-level summaries |
| Hybrid Search | Search memory using BM25 + vector embeddings |
| Progressive Disclosure | 3-layer workflow: search → timeline → get_observations (10x token savings) |
| Context Injection | Automatic context generation for SessionStart hooks |
| Git-Tagged Memory | Observations tagged with git context (branch, commit) |

#### New Capabilities - Code Browser

| Capability | Description |
|------------|-------------|
| Tree View Navigation | Collapsible directory tree for large codebases |
| Enhanced Code Display | Tree-sitter highlighting with chunk boundaries |
| Search Integration | Inline semantic search results with similarity scores |
| Keyboard Navigation | Vim-like shortcuts (j/k scroll, Enter to open) |
| Real-time Updates | SSE events for indexing progress and updates |
| Dark Mode | CSS variable-based theming |

#### New MCP Tools - Git

| Tool | Purpose |
|------|---------|
| `index_git_repository` | Index repository with branch awareness |
| `search_branch` | Search within specific branch |
| `compare_branches` | Compare code between branches |
| `analyze_impact` | Analyze change impact between refs |
| `list_repositories` | List indexed repositories |

#### New MCP Tools - Memory

| Tool | Purpose |
|------|---------|
| `search` | Step 1: Search memory index (token-efficient) |
| `timeline` | Step 2: Get chronological context around results |
| `get_observations` | Step 3: Fetch full details for filtered IDs |
| `store_observation` | Store tool observation (PostToolUse hook) |
| `inject_context` | Generate context for SessionStart hook |

#### Technical Details

**Git Integration:**

-   **New Dependency**: git2 (libgit2 bindings)
-   **New Files**: ~12 source files
-   **Estimated LOC**: ~2500
-   **ADR**: [008-git-aware-semantic-indexing-v0.2.0](../adr/008-git-aware-semantic-indexing-v0.2.0.md)

**Session Memory:**

-   **New Dependency**: sqlx (SQLite support)
-   **New Files**: ~15 source files
-   **Estimated LOC**: ~3000
-   **ADR**: [009-persistent-session-memory-v0.2.0](../adr/009-persistent-session-memory-v0.2.0.md)

**Code Browser:**

-   **New Dependencies**: Alpine.js (CDN)
-   **New Files**: ~6 source files
-   **Estimated LOC**: ~1500
-   **ADR**: [028-advanced-code-browser-v020](../adr/028-advanced-code-browser-v020.md)
-   **Foundation**: v0.1.2 basic browse (already implemented)

#### Configuration Defaults

**Git Settings:**

| Setting | Default | Override |
|---------|---------|----------|
| Branches | main, HEAD, current | Per-repo via `.mcp-context.toml` |
| History depth | 50 commits | Per-repo |
| Submodules | Recursive indexing | Per-repo |

**Memory Settings:**

| Setting | Default | Override |
|---------|---------|----------|
| Database | ~/.MCP-context-browser/memory.db | Global config |
| Observation types | decision, bugfix, feature | Per-project |
| Observation limit | 20 | Per-request |
| Date range | 30 days | Per-request |
| SDK compression | Disabled | Opt-in |

---

### v0.3.0 - Advanced Code Intelligence 📋 FUTURE

**Status**: Conceptual
**Priority**: Medium
**Dependencies**: v0.2.0 completion

#### Vision

Enhance semantic code search with deep code intelligence features, enabling advanced analysis beyond keyword and semantic matching. This version focuses on understanding code relationships and providing actionable insights for refactoring and optimization.

#### Objectives

| Feature | Description | Benefit |
|---------|-------------|---------|
|**Symbol Extraction**| Extract and index all symbols (functions, classes, variables) | Navigate code by symbols, not just files |
|**Cross-Referencing**| Build symbol usage graph across codebase | "Find all usages" with precision |
|**Call Graph Analysis**| Map function call relationships | Understand execution paths |
|**Dependency Mapping**| Visualize module and package dependencies | Identify refactoring opportunities |
|**Code Similarity**| Detect duplicate or similar code patterns | Reduce code duplication |
|**Refactoring Suggestions**| AI-powered refactoring recommendations | Improve code quality |

#### Technical Approach

-   **AST Enhancement**: Extend existing tree-sitter integration with symbol extraction
-   **Graph Database**: Consider Neo4j or in-memory graph for relationships
-   **Incremental Updates**: Update graph on file changes (not full reindex)
-   **MCP Tools**: New tools for symbol search, call graph queries, similarity detection

#### Success Metrics

-   Symbol extraction: <1s for 10,000 LOC
-   Cross-reference lookup: <100ms
-   Call graph generation: <5s for large projects
-   Similarity detection: >90% accuracy

---

### v0.4.0 - Enterprise Features 📋 FUTURE

**Status**: Conceptual
**Priority**: Medium
**Dependencies**: v0.3.0 completion

#### Vision

Transform MCP Context Browser into an enterprise-ready platform with multi-tenancy, advanced security, compliance features, and comprehensive administrative capabilities suitable for large organizations.

#### Objectives

| Feature | Description | Benefit |
|---------|-------------|---------|
|**Multi-Tenant Support**| Isolated workspaces with resource quotas | Support multiple teams/projects |
|**Advanced RBAC**| Role-based access control with team permissions | Granular security control |
|**SSO Integration**| SAML 2.0, OIDC, Active Directory support | Enterprise authentication |
|**Enhanced Audit Logging**| Comprehensive activity tracking with retention | Compliance and security |
|**Cost Tracking**| Per-tenant API usage and cost allocation | Budget management |
|**Admin Dashboard**| Web-based administrative interface | Centralized management |

#### Technical Approach

-   **Multi-Tenancy Architecture**: Tenant isolation at database and provider level
-   **Authentication Layer**: OAuth2/OIDC integration with configurable providers
-   **Audit System**: Structured logging with tamper-proof audit trail
-   **Metrics per Tenant**: Prometheus labels for tenant-specific monitoring
-   **Web Admin UI**: Expand ADR-007 admin interface with tenant management

#### Success Metrics

-   Tenant isolation: 100% (no cross-tenant data leakage)
-   SSO integration: <5 min setup time
-   Audit completeness: 100% of API calls logged
-   Admin UI availability: 99.9% uptime

---

### v1.0.0 - Production Enterprise 📋 FUTURE

**Status**: Conceptual
**Priority**: High
**Dependencies**: v0.4.0 completion

#### Vision

Deliver a fully production-ready enterprise platform with SLA guarantees, professional support, compliance certifications, and high-availability deployment options suitable for mission-critical use cases in large enterprises.

#### Objectives

| Feature | Description | Benefit |
|---------|-------------|---------|
|**Full Enterprise Feature Set**| All v0.2.0-v0.4.0 features polished and hardened | Production-grade reliability |
|**SLA Guarantees**| 99.9% uptime commitment with monitoring | Business continuity |
|**Professional Support**| 24/7 support with response time SLAs | Enterprise peace of mind |
|**Compliance Certifications**| SOC 2 Type II, ISO 27001, GDPR | Regulatory compliance |
|**High Availability**| Multi-region deployment with automatic failover | Zero downtime |
|**Disaster Recovery**| Backup/restore with point-in-time recovery | Data protection |

#### Technical Approach

-   **HA Architecture**: Active-active deployment across multiple regions
-   **Automated Backup**: Continuous backup with 99.999% durability
-   **Monitoring & Alerting**: Comprehensive observability with PagerDuty integration
-   **Compliance Framework**: Automated compliance checking and reporting
-   **Documentation**: Professional documentation with support portal
-   **Certification Process**: Third-party security audits and certifications

#### Success Metrics

-   Uptime: 99.9% (measured monthly)
-   Response time: P95 <200ms for search queries
-   Support SLA: <15 min for critical issues
-   Compliance: 100% audit pass rate
-   Recovery Time Objective (RTO): <1 hour
-   Recovery Point Objective (RPO): <15 minutes

#### Certification Timeline

| Certification | Timeline | Estimated Cost |
|---------------|----------|----------------|
| SOC 2 Type I | Months 1-3 | $25k-$50k |
| SOC 2 Type II | Months 4-9 | $50k-$100k |
| ISO 27001 | Months 6-12 | $30k-$75k |
| GDPR Compliance | Months 1-6 | $10k-$25k |

---

## Version History

| Version | Status | Key Features |
|---------|--------|--------------|
| v0.0.1 | Released | Initial prototype |
| v0.0.2 | Released | Core architecture |
| v0.0.3 | Released | Production foundation |
| v0.1.0 | Released | Documentation excellence, clean architecture, first stable release |
| v0.1.1 | Released | Modular crate architecture (7 crates), DI foundation |
| v0.1.2 | **Current** | Linkme provider registration, mcb-validate Phases 1-3, Admin UI Browse |
| v0.2.0 | Planned | Git-aware indexing, session memory, advanced code browser |
| v0.3.0 | Future | Advanced code intelligence |
| v0.4.0 | Future | Enterprise features |
| v1.0.0 | Future | Production enterprise |

---

## Implementation Principles

### Development Practices

1.**ADR-Driven Development**: Architectural decisions documented before implementation
2.**Test-First**: Core functionality developed with comprehensive tests
3.**Clean Architecture**: Separation of concerns with trait-based DI
4.**Documentation First**: Documentation updated with each code change
5.**Security by Design**: Security considerations in every component

### Quality Gates

All releases must pass:

-   [ ] All tests pass (unit, integration, e2e)
-   [ ] Code coverage meets targets (>85%)
-   [ ] Clippy lint clean
-   [ ] Security audit clean
-   [ ] Performance benchmarks maintained
-   [ ] Documentation complete and accurate

---

## Cross-References

-   **Architecture**: [ARCHITECTURE.md](../architecture/ARCHITECTURE.md)
-   **Contributing**: [CONTRIBUTING.md](./CONTRIBUTING.md)
-   **ADR Index**: [docs/ADR/README.md](../adr/README.md)
-   **Version History**: [VERSION_HISTORY.md](../VERSION_HISTORY.md)
-   **Deployment**: [DEPLOYMENT.md](../operations/DEPLOYMENT.md)
