# Development Roadmap

## Overview

This roadmap outlines the development of MCP Context Browser, a drop-in replacement for Claude-context with enhanced capabilities for semantic code search.

---

## Current Status

### v0.1.0 - First Stable Release ✅ RELEASED

**Status**: Production-Ready
**Release Date**: January 2026

MCP Context Browser v0.1.0 is the first stable release, providing a complete drop-in replacement for Claude-context with superior performance and expanded capabilities.

#### Achievements

\1-   ✅ Full MCP protocol implementation (4 tools)
\1-   ✅ 12 languages with AST parsing (Rust, Python, JS/TS, Go, Java, C, C++, C#, Ruby, PHP, Swift, Kotlin)
\1-   ✅ 6 embedding providers (OpenAI, VoyageAI, Ollama, Gemini, FastEmbed, Null)
\1-   ✅ 6 vector stores (Milvus, EdgeVec, In-Memory, Filesystem, Encrypted, Null)
\1-   ✅ Claude-context environment variable compatibility
\1-   ✅ 790+ tests with comprehensive coverage (100% pass rate)
\1-   ✅ JWT authentication and rate limiting
\1-   ✅ Clean architecture with trait-based dependency injection
\1-   ✅ HTTP transport foundation for future enhancements
\1-   ✅ Systemd service integration
\1-   ✅ Binary auto-respawn mechanism

---

## Upcoming Releases

### v0.2.0 - Git-Aware Indexing + Persistent Session Memory 🚧 PLANNED

**Status**: Planning Complete (ADR-008, ADR-009)
**Priority**: High
**Estimated Effort**: 20 phases (10 git + 10 memory)

#### Vision

Transform MCP Context Browser into a comprehensive development platform combining git-aware semantic search with persistent cross-session memory, enabling both powerful code navigation and context preservation across Claude Code sessions.

#### Objectives

1.**Git-Aware Semantic Indexing**(ADR-008)

\1-   Project-relative indexing (portable)
\1-   Multi-branch support with commit history
\1-   Change impact analysis
\1-   Monorepo and submodule support

2.**Persistent Session Memory**(ADR-009)

\1-   Cross-session observation storage
\1-   Semantic search over past decisions and work
\1-   Token-efficient progressive disclosure (3-layer workflow)
\1-   Context injection for session continuity

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
\1-  **New Dependency**: git2 (libgit2 bindings)
\1-  **New Files**: ~12 source files
\1-  **Estimated LOC**: ~2500
\1-  **ADR**: [008-git-aware-semantic-indexing-v0.2.0](../adr/008-git-aware-semantic-indexing-v0.2.0.md)

**Session Memory:**
\1-  **New Dependency**: sqlx (SQLite support)
\1-  **New Files**: ~15 source files
\1-  **Estimated LOC**: ~3000
\1-  **ADR**: [009-persistent-session-memory-v0.2.0](../adr/009-persistent-session-memory-v0.2.0.md)

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

\1-  **AST Enhancement**: Extend existing tree-sitter integration with symbol extraction
\1-  **Graph Database**: Consider Neo4j or in-memory graph for relationships
\1-  **Incremental Updates**: Update graph on file changes (not full reindex)
\1-  **MCP Tools**: New tools for symbol search, call graph queries, similarity detection

#### Success Metrics

\1-   Symbol extraction: <1s for 10,000 LOC
\1-   Cross-reference lookup: <100ms
\1-   Call graph generation: <5s for large projects
\1-   Similarity detection: >90% accuracy

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

\1-  **Multi-Tenancy Architecture**: Tenant isolation at database and provider level
\1-  **Authentication Layer**: OAuth2/OIDC integration with configurable providers
\1-  **Audit System**: Structured logging with tamper-proof audit trail
\1-  **Metrics per Tenant**: Prometheus labels for tenant-specific monitoring
\1-  **Web Admin UI**: Expand ADR-007 admin interface with tenant management

#### Success Metrics

\1-   Tenant isolation: 100% (no cross-tenant data leakage)
\1-   SSO integration: <5 min setup time
\1-   Audit completeness: 100% of API calls logged
\1-   Admin UI availability: 99.9% uptime

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

\1-  **HA Architecture**: Active-active deployment across multiple regions
\1-  **Automated Backup**: Continuous backup with 99.999% durability
\1-  **Monitoring & Alerting**: Comprehensive observability with PagerDuty integration
\1-  **Compliance Framework**: Automated compliance checking and reporting
\1-  **Documentation**: Professional documentation with support portal
\1-  **Certification Process**: Third-party security audits and certifications

#### Success Metrics

\1-   Uptime: 99.9% (measured monthly)
\1-   Response time: P95 <200ms for search queries
\1-   Support SLA: <15 min for critical issues
\1-   Compliance: 100% audit pass rate
\1-   Recovery Time Objective (RTO): <1 hour
\1-   Recovery Point Objective (RPO): <15 minutes

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
| v0.1.0 |**Current**| Documentation excellence, clean architecture, first stable release |
| v0.2.0 | Planned | Git-aware semantic indexing, persistent session memory |
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

\1-   [ ] All tests pass (unit, integration, e2e)
\1-   [ ] Code coverage meets targets (>85%)
\1-   [ ] Clippy lint clean
\1-   [ ] Security audit clean
\1-   [ ] Performance benchmarks maintained
\1-   [ ] Documentation complete and accurate

---

## Cross-References

\1-  **Architecture**: [ARCHITECTURE.md](../architecture/ARCHITECTURE.md)
\1-  **Contributing**: [CONTRIBUTING.md](./CONTRIBUTING.md)
\1-  **ADR Index**: [docs/ADR/README.md](../adr/README.md)
\1-  **Version History**: [VERSION_HISTORY.md](../VERSION_HISTORY.md)
\1-  **Deployment**: [DEPLOYMENT.md](../operations/DEPLOYMENT.md)
