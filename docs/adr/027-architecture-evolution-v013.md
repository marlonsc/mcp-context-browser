# ADR 027: Architecture Evolution v0.1.3 - Onion/Clean Enhancement

## Status

**Proposed**

> Inspired by kamu-cli's production Onion/Clean Architecture patterns. Extends [ADR 013](013-clean-architecture-crate-separation.md) (Clean Architecture Crate Separation) and [ADR 024](024-simplified-dependency-injection.md) (Simplified Dependency Injection) without breaking backward compatibility.

## Context

MCB v0.1.2 established a SOLID Clean Architecture foundation with:

-   8-crate separation (mcb-domain, mcb-application, mcb-providers, mcb-infrastructure, mcb-server, mcb, mcb-validate)
-   20+ port traits in mcb-application (EmbeddingProvider, VectorStoreProvider, etc.)
-   Linkme-based provider auto-registration (15+ providers)
-   Handle-based DI with runtime provider switching (ADR 024)
-   790+ tests with architectural validation via mcb-validate

### Analysis of kamu-cli's Onion/Clean Architecture

Analysis of the [kamu-cli](https://github.com/kamu-data/kamu-cli) production codebase revealed opportunities to evolve MCB without rewriting:

| Aspect | MCB Current | kamu-cli Pattern | Opportunity |
|--------|-------------|------------------|-------------|
| Module Organization | By layer (entities/, ports/, services/) | By bounded context (workspace/, indexing/, search/) | Feature-centric navigation |
| Engine Contracts | Implicit via providers | Explicit engine traits | Plugin ecosystem |
| Indexing | Full re-index | Incremental with checkpoints | 90%+ time reduction |
| Operability | Binary only | Node mode with Helm | Kubernetes deployment |
| Quality | Unit tests only | Relevance tests | Search quality gates |

### Problems Addressed

1.  **Layer-centric organization**: Finding code by feature requires knowing which layer it belongs to
2.  **Implicit engine contracts**: Providers are loosely coupled without formal engine semantics
3.  **Full re-indexing**: Unchanged files are re-processed unnecessarily
4.  **Limited operability**: No standard deployment patterns for production
5.  **No quality metrics**: Search relevance changes go undetected

## Decision

Adopt a phased evolution plan (Phases 0-5) to enhance MCB's architecture while maintaining backward compatibility:

### Phase 0: Baseline & Acceptance Criteria

-   Document layer boundaries in ARCHITECTURE_BOUNDARIES.md
-   Define golden acceptance tests (index repo, run queries, validate latency <200ms)
-   No MCP API changes

### Phase 1: Bounded Contexts Within Layers

Organize mcb-domain and mcb-application by feature modules instead of pure layer folders:

**Bounded Contexts:**

-   `workspace/` - Config, roots, ignore rules, multi-root support
-   `indexing/` - Index state, progress, checkpoints
-   `chunking/` - Language strategies, chunk types
-   `search/` - Query parsing, ranking, Result aggregation
-   `telemetry/` - Events, tracing, metrics

**Module Structure:**

```
mcb-application/src/
├── workspace/
│   ├── mod.rs
│   ├── ports.rs      # WorkspaceConfigProvider
│   └── service.rs    # WorkspaceService
├── indexing/
│   ├── mod.rs
│   ├── ports.rs      # IndexStateStore, CodeExtractor
│   └── service.rs    # IndexingService
├── chunking/
│   ├── mod.rs
│   ├── ports.rs      # Chunker, LanguageChunkingProvider
│   └── service.rs    # ChunkingOrchestrator
├── search/
│   ├── mod.rs
│   ├── ports.rs      # VectorIndex, Ranker, Embedder
│   └── service.rs    # SearchService, ContextService
├── telemetry/
│   ├── mod.rs
│   ├── ports.rs      # EventBusProvider, MetricsCollector
│   └── service.rs    # TelemetryService
├── shared/
│   ├── ports/        # CacheProvider, CryptoProvider (cross-cutting)
│   └── registry/     # Linkme registries
└── lib.rs            # Re-exports for backward compatibility
```

### Phase 2: Explicit Engine Contracts

Define formal engine traits in the domain layer:

```rust
// mcb-application/src/indexing/ports.rs
#[async_trait]
pub trait IndexStateStore: Send + Sync {
    async fn get_checkpoint(&self, collection: &str) -> Result<Option<IndexCheckpoint>>;
    async fn save_checkpoint(&self, collection: &str, checkpoint: IndexCheckpoint) -> Result<()>;
    async fn get_file_fingerprint(&self, path: &Path) -> Result<Option<FileFingerprint>>;
    async fn save_file_fingerprint(&self, path: &Path, fp: FileFingerprint) -> Result<()>;
}

// mcb-application/src/search/ports.rs
#[async_trait]
pub trait Ranker: Send + Sync {
    async fn rank(&self, query: &str, candidates: Vec<SearchCandidate>) -> Result<Vec<RankedResult>>;
    fn ranker_name(&self) -> &str;
}
```

**Engine Implementations:**

-   `IndexStateStore`: SQLite (default), In-Memory (testing), RocksDB (feature-gated)
-   `Ranker`: CosineRanker, HybridRanker (BM25 + semantic), MMRRanker, LLMReranker (feature-gated)

**Unified Config:**

```toml
[engines.embedding]
type = "ollama"
model = "nomic-embed-text"

[engines.vector_store]
type = "edgevec"
path = "./data/vectors"

[engines.ranker]
type = "hybrid"
semantic_weight = 0.7
```

### Phase 3: Incremental Indexing Pipeline

Implement checkpoint/resume for indexing:

```rust
pub struct FileFingerprint {
    pub mtime: SystemTime,
    pub size: u64,
    pub content_hash: Option<[u8; 32]>,  // SHA-256 of first 4KB
}

impl IndexingService {
    pub async fn index_incremental(&self, collection: &str, root: &Path) -> Result<IndexStats> {
        let checkpoint = self.state_store.get_checkpoint(collection).await?;
        let files_to_process = self.compute_changed_files(root, &checkpoint).await?;

        for batch in files_to_process.chunks(100) {
            self.index_batch(batch).await?;
            self.state_store.save_checkpoint(collection, current_checkpoint).await?;
        }

        self.garbage_collect_removed(collection, root).await?;
        Ok(stats)
    }
}
```

**Capabilities:**

-   Fingerprint-based change detection (mtime + size + partial hash)
-   Checkpoint/resume on crash
-   Garbage collection for removed files
-   Metrics: index time, chunks processed, reuse rate

### Phase 4: Node Mode Operability

Enable production deployment patterns:

**CLI Subcommands:**

```bash
mcb serve              # MCP server (existing behavior)
mcb index --watch      # Watch filesystem and update index
mcb doctor             # Environment checks
```

**Health Endpoints:**

```rust
#[get("/healthz")]
pub fn health() -> Status { Status::Ok }

#[get("/readyz")]
pub async fn ready(ctx: &State<AppContext>) -> Status {
    if ctx.is_ready().await { Status::Ok } else { Status::ServiceUnavailable }
}

#[get("/metrics")]
pub fn metrics() -> String { /* Prometheus format */ }
```

**Deployment Artifacts:**

-   Dockerfile (distroless base)
-   docker-compose.yml for local development
-   Helm chart (helm/mcb-context-browser/)

### Phase 5: Relevance Testing

Add search quality gates to CI:

```yaml

# examples/queries.yaml
-   query: "how does authentication work"
  collection: "rust-repo"
  expected_files:
    -   "src/auth.rs"
    -   "src/middleware/auth.rs"
  min_recall_at_5: 0.8

-   query: "database connection handling"
  collection: "rust-repo"
  expected_files:
    -   "src/db/pool.rs"
  min_recall_at_5: 0.6
```

**CI Integration:**

-   Index example repositories
-   Run relevance test suite
-   Fail if recall@k drops below threshold
-   Report per-query metrics

## Consequences

### Positive

-   **Feature-centric navigation**: Domain experts find code by feature (search/, indexing/) not layer
-   **Plugin ecosystem**: Engine contracts enable third-party implementations
-   **90%+ faster re-indexing**: Incremental indexing skips unchanged files
-   **Kubernetes-ready**: Node mode with Helm chart for production deployment
-   **Quality gates**: Relevance tests prevent search regression

### Negative

-   **More files**: Bounded context modules add directory structure
-   **Learning curve**: Team must understand bounded context organization
-   **CI time**: Relevance tests add ~2-3 minutes to pipeline
-   **Rust boilerplate**: New engine traits require implementations across providers

### Neutral

-   **Backward compatible**: No MCP API changes, old config keys still work
-   **Gradual adoption**: Each phase is independent, can be adopted incrementally
-   **Test suite intact**: All 790+ existing tests continue to pass

## Implementation Notes

See implementation prompt: `thoughts/prompts/PROMPT_V013_IMPLEMENTATION.md`

**Estimated Scope:**

-   Phase 0: 1 PR (baseline)
-   Phase 1: 3-4 PRs (bounded contexts)
-   Phase 2: 4-5 PRs (engine contracts)
-   Phase 3: 3 PRs (incremental indexing)
-   Phase 4: 3 PRs (node mode)
-   Phase 5: 2 PRs (relevance testing)
-   **Total**: ~16-18 PRs

## Related ADRs

-   [ADR 013: Clean Architecture Crate Separation](013-clean-architecture-crate-separation.md) - **Extended** by this ADR (adds bounded contexts within layers)
-   [ADR 024: Simplified Dependency Injection](024-simplified-dependency-injection.md) - **Extended** by this ADR (formalizes engine contracts using handle pattern)
-   [ADR 008: Git-Aware Semantic Indexing v0.2.0](008-git-aware-semantic-indexing-v0.2.0.md) - **Prepared for** by this ADR (incremental indexing foundation)

## References

-   [kamu-cli](https://github.com/kamu-data/kamu-cli) - Production Onion/Clean Architecture reference
-   [Clean Architecture by Robert C. Martin](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)
-   [Onion Architecture by Jeffrey Palermo](https://jeffreypalermo.com/2008/07/the-onion-architecture-part-1/)
-   [Domain-Driven Design by Eric Evans](https://domainlanguage.com/ddd/)
