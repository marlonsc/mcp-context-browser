# ADR 008: Git-Aware Semantic Indexing v0.2.0

## Status

**Proposed**(Planned for v0.2.0)

> Not yet implemented. Key dependencies:
>
> -   git2 crate not in Cargo.toml
> -   No `crates/mcb-domain/src/git.rs` or `crates/mcb-providers/src/git/`
> -   No GitProvider trait or implementation
> -   Blocking: Requires git2 dependency and new module structure
>
> **Target crate structure (v0.2.0)**:
>
> -   `crates/mcb-domain/src/git.rs` - Git domain types
> -   `crates/mcb-application/src/ports/providers/git.rs` - GitProvider trait
> -   `crates/mcb-providers/src/git/` - git2 implementation
> -   `crates/mcb-application/src/use_cases/git_indexing.rs` - Git-aware indexing service

## Context

MCP Context Browser v0.1.0 provides efficient semantic code search but lacks version control system awareness. This limits its usefulness in real-world scenarios:

**Current problems:**

-   Indexes are based on filesystem paths, breaking if directory is moved
-   No distinction between branches - search mixes code from different contexts
-   Change detection based on file mtime, not actual commits
-   No support for monorepos with multiple projects
-   No commit history indexing
-   No change impact analysis

**User demand:**

-   Developers work with large monorepos (Uber, Google, Meta patterns)
-   Need to search code in specific branch
-   Need to understand impact of changes before merge
-   Need to index submodules as separate projects

## Decision

Implement full git integration in mcb v0.2.0 with:

1.**Repository identification by root commit**(portable)
2.**Multi-branch indexing**(main + HEAD + current by default)
3.**Commit history**(last 50 by default)
4.**Submodule detection**with recursive indexing
5.**Project detection**in monorepos
6.**Impact analysis**between commits/branches

**Library chosen**: git2 (libgit2 bindings)

-   Mature, battle-tested, widely used
-   Stable and well-documented API
-   Superior performance to gitoxide (still in development)

## Consequences

### Positive

-   **Portability**: Indexes survive directory moves/renames
-   **Precise context**: Search within specific branch
-   **Monorepo support**: Enterprises can use with large codebases
-   **Impact analysis**: Prevents bugs before merge
-   **History**: Search in previous versions of code

### Negative

-   **Complexity**: Adds ~12 new files, ~2500 LOC
-   **Dependency**: git2 adds libgit2 as native dependency
-   **Storage**: Per-branch indexes increase disk usage
-   **Performance**: Git operations add latency

## Alternatives Considered

### Alternative 1: gitoxide (pure Rust)

-   **Pros**: Pure Rust, no native dependency
-   **Cons**: API still unstable, fewer features
-   **Rejected**: Risk of breaking changes

### Alternative 2: Shell commands (git CLI)

-   **Pros**: Always available, no dependency
-   **Cons**: Subprocess overhead, output parsing
-   **Rejected**: Poor performance for frequent operations

### Alternative 3: Keep without git

-   **Pros**: Simplicity
-   **Cons**: Does not meet user demand
-   **Rejected**: Essential feature for adoption

## Implementation Notes

### Phase 1: Domain Model Extension

**Create**: `crates/mcb-domain/src/git.rs`

```rust
use serde::{Deserialize, Serialize};

/// Portable repository identity based on first commit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryId {
    /// SHA-1 of first commit (immutable identifier)
    pub root_commit_hash: String,
    /// Current remote origin URL (for display/caching)
    pub remote_url: Option<String>,
    /// Human-readable name
    pub name: String,
}

/// Branch reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchRef {
    pub repository_id: String,
    pub name: String,
    pub commit_hash: String,
    pub is_default: bool,
}

/// Lightweight commit metadata for index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitInfo {
    pub hash: String,
    pub short_hash: String,
    pub author: String,
    pub timestamp: u64,
    pub message_summary: String,
}

/// Submodule reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmoduleRef {
    pub path: String,
    pub url: String,
    pub commit_hash: String,
}

/// Project detected within monorepo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRef {
    pub repository_id: String,
    pub path: String,
    pub project_type: ProjectType,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProjectType {
    CargoWorkspaceMember,
    NpmPackage,
    PythonProject,
    GoModule,
    MavenModule,
    GradleModule,
    Generic,
}

/// Git diff between commits/branches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitDiff {
    pub from: CommitInfo,
    pub to: CommitInfo,
    pub files_added: Vec<String>,
    pub files_modified: Vec<String>,
    pub files_deleted: Vec<String>,
}

/// Git metadata attached to CodeChunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitChunkMetadata {
    pub repository_id: String,
    pub branch: String,
    pub commit_hash: String,
    pub relative_path: String,
    pub project: Option<String>,
    pub introduced_commit: Option<String>,
    pub last_modified_commit: Option<String>,
}
```

### Phase 2: Git Provider Port/Adapter

**Create**: `crates/mcb-application/src/ports/providers/git.rs`

```rust
use async_trait::async_trait;
use std::path::Path;
use crate::domain::git::*;

#[async_trait]
pub trait GitProvider: Send + Sync {
    /// Detect if path is inside a git repository
    async fn detect_repository(&self, path: &Path) -> Result<Option<RepositoryId>>;

    /// Get repository info from a path
    async fn get_repository_info(&self, path: &Path) -> Result<RepositoryInfo>;

    /// List branches with optional filtering
    async fn list_branches(&self, repo_path: &Path) -> Result<Vec<BranchRef>>;

    /// Get current branch
    async fn current_branch(&self, repo_path: &Path) -> Result<BranchRef>;

    /// Get commits in range
    async fn get_commits(
        &self,
        repo_path: &Path,
        since: Option<&str>,
        limit: usize
    ) -> Result<Vec<CommitInfo>>;

    /// Get diff between two refs
    async fn diff(&self, repo_path: &Path, from: &str, to: &str) -> Result<GitDiff>;

    /// List submodules
    async fn list_submodules(&self, repo_path: &Path) -> Result<Vec<SubmoduleRef>>;

    /// Get file content at specific commit
    async fn file_at_commit(
        &self,
        repo_path: &Path,
        file: &str,
        commit: &str
    ) -> Result<String>;

    /// Detect projects within repository
    async fn detect_projects(&self, repo_path: &Path) -> Result<Vec<ProjectRef>>;

    /// Provider name for logging
    fn provider_name(&self) -> &str;
}
```

**Create**: `crates/mcb-providers/src/git/git2_provider.rs`

```rust
use git2::{Repository, BranchType, DiffOptions};
use dashmap::DashMap;
use std::path::{Path, PathBuf};

pub struct Git2Provider {
    /// Cache for repository handles
    repo_cache: DashMap<PathBuf, RepositoryId>,
}

impl Git2Provider {
    pub fn new() -> Self {
        Self {
            repo_cache: DashMap::new(),
        }
    }

    /// Open repository with caching
    fn open_repo(&self, path: &Path) -> Result<Repository> {
        // Use tokio::task::spawn_blocking for git2 operations
        Repository::discover(path).map_err(Into::into)
    }
}
```

### Phase 3: Repository Manager Service

**Create**: `crates/mcb-application/src/use_cases/repository.rs`

```rust
use dashmap::DashMap;
use std::path::PathBuf;
use std::sync::Arc;
use crate::domain::ports::git::GitProvider;
use crate::domain::git::*;

pub struct RepositoryManager {
    git_provider: Arc<dyn GitProvider>,
    /// Indexed repositories (RepositoryId -> metadata)
    repositories: DashMap<String, RepositoryState>,
    /// Branch indexing state
    branch_states: DashMap<String, BranchState>,
}

pub struct RepositoryState {
    pub id: RepositoryId,
    pub root_path: PathBuf,
    pub branches: Vec<String>,
    pub default_branch: String,
    pub submodules: Vec<SubmoduleRef>,
    pub projects: Vec<ProjectRef>,
    pub last_indexed: Option<u64>,
}

pub struct BranchState {
    pub branch_ref: BranchRef,
    pub indexed_commit: Option<String>,
    pub last_indexed: Option<u64>,
}

impl RepositoryManager {
    pub async fn register(&self, path: &Path) -> Result<RepositoryState>;
    pub async fn get_state(&self, repo_id: &str) -> Option<RepositoryState>;
    pub async fn update_branch_state(&self, repo_id: &str, branch: &str, commit: &str);
}
```

### Phase 4: Git-Aware Snapshot Manager

**Create**: `crates/mcb-infrastructure/src/snapshot/git_snapshot.rs`

```rust
use std::path::Path;
use std::sync::Arc;
use crate::domain::ports::git::GitProvider;

pub struct GitSnapshotManager {
    git_provider: Arc<dyn GitProvider>,
}

pub struct GitChangeset {
    pub commit_range: (String, String),
    pub added_files: Vec<String>,
    pub modified_files: Vec<String>,
    pub deleted_files: Vec<String>,
}

impl GitSnapshotManager {
    /// Get changed files since last indexed commit
    pub async fn get_changed_files(
        &self,
        repo_path: &Path,
        branch: &str,
        last_indexed_commit: Option<&str>,
    ) -> Result<GitChangeset> {
        let current = self.git_provider.current_branch(repo_path).await?;

        match last_indexed_commit {
            Some(old) => {
                let diff = self.git_provider.diff(repo_path, old, &current.commit_hash).await?;
                Ok(GitChangeset::from_diff(diff))
            }
            None => {
                // First indexing: all tracked files
                Ok(GitChangeset::full_index(repo_path).await?)
            }
        }
    }
}
```

### Phase 5: Schema Extensions

**Modify**: `crates/mcb-domain/src/entities/code_chunk.rs`

```rust
// Add to existing CodeChunk struct
pub struct CodeChunk {
    // ... existing fields ...

    /// Git-specific metadata (None for non-git codebases)
    pub git_metadata: Option<GitChunkMetadata>,
}
```

**Extended metadata JSON structure:**

```json
{
  "content": "fn main() { ... }",
  "file_path": "src/main.rs",
  "start_line": 10,
  "end_line": 25,
  "language": "Rust",
  "git": {
    "repository_id": "abc123def456...",
    "branch": "main",
    "commit": "def456...",
    "project": "my-project",
    "introduced_commit": "ghi789...",
    "last_modified_commit": "def456..."
  }
}
```

**Collection naming strategy:**

| Pattern | Purpose |
|---------|---------|
| `{repo_id}_{branch}` | Branch-specific search |
| `{repo_id}_all` | Cross-branch search |
| `{repo_id}_{commit_short}` | Point-in-time snapshot (optional) |

### Phase 6: Git Indexing Service

**Create**: `crates/mcb-application/src/use_cases/git_indexing.rs`

```rust
pub struct GitIndexingService {
    repository_manager: Arc<RepositoryManager>,
    context_service: Arc<ContextService>,
    git_snapshot_manager: GitSnapshotManager,
    chunker: IntelligentChunker,
}

pub struct IndexingOptions {
    pub branches: BranchSelection,
    pub include_submodules: bool,
    pub include_history: bool,
    pub history_depth: Option<usize>,
}

pub enum BranchSelection {
    Current,
    All,
    Default,  // main + HEAD + current
    Specific(Vec<String>),
}

impl GitIndexingService {
    /// Index a git repository with full context
    pub async fn index_repository(
        &self,
        repo_path: &Path,
        options: IndexingOptions,
    ) -> Result<IndexingStats> {
        // 1. Detect/register repository
        let repo_state = self.repository_manager.register(repo_path).await?;

        // 2. Determine branches to index
        let branches = self.resolve_branches(&repo_state, &options.branches);

        // 3. Index each branch
        for branch in branches {
            self.index_branch(&repo_state, &branch, &options).await?;
        }

        // 4. Handle submodules if requested
        if options.include_submodules {
            for submodule in &repo_state.submodules {
                self.index_submodule(&repo_state, submodule).await?;
            }
        }

        Ok(stats)
    }
}
```

### Phase 7: History Indexing

**Strategy to avoid index explosion:**

```rust
pub struct HistoryIndexingStrategy {
    /// Max commits to index per branch (default: 50)
    pub max_commits: usize,
    /// Only index commits touching specific file patterns
    pub file_patterns: Vec<String>,
    /// Sample rate: index every N commits (default: 1)
    pub sample_rate: usize,
}
```

### Phase 8: Impact Analysis

**Create**: `crates/mcb-application/src/use_cases/impact.rs`

```rust
pub struct ImpactAnalyzer {
    repository_manager: Arc<RepositoryManager>,
    git_provider: Arc<dyn GitProvider>,
    context_service: Arc<ContextService>,
}

pub struct ImpactAnalysis {
    pub from_ref: String,
    pub to_ref: String,
    pub direct_changes: Vec<FileChange>,
    pub impacted_files: Vec<ImpactedFile>,
    pub impact_score: f32,  // 0-1 estimate of change scope
}

pub struct ImpactedFile {
    pub file_path: String,
    pub relationship: ImpactRelationship,
    pub confidence: f32,
}

pub enum ImpactRelationship {
    DirectDependency,
    TransitiveDependency,
    SemanticallySimilar,
    SharedImports,
}

impl ImpactAnalyzer {
    /// Analyze impact of changes between two refs
    pub async fn analyze_impact(
        &self,
        repo_path: &Path,
        from: &str,
        to: &str,
    ) -> Result<ImpactAnalysis> {
        let diff = self.git_provider.diff(repo_path, from, to).await?;

        let mut analysis = ImpactAnalysis::new(from, to);

        for file in &diff.files_modified {
            let related = self.find_related_code(repo_path, file).await?;
            analysis.add_impact(file, related);
        }

        Ok(analysis)
    }
}
```

### Phase 9: MCP Tools

**Create**: `crates/mcb-server/src/handlers/git_tools.rs`

| Tool | Description | Parameters |
|------|-------------|------------|
| `index_git_repository` | Index repository with branch awareness | path, branches?, include_submodules?, include_history? |
| `search_branch` | Search within specific branch | query, repository?, branch?, limit? |
| `compare_branches` | Compare code between branches | path, from_branch, to_branch |
| `analyze_impact` | Analyze change impact | path, from_ref, to_ref |
| `list_repositories` | List indexed repositories | - |

### Phase 10: Configuration

**Create**: `crates/mcb-infrastructure/src/config/git.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct GitConfig {
    /// Enable git integration (default: true)
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Default branches to index: ["main", "HEAD"]
    #[serde(default = "default_branches")]
    pub default_branches: Vec<String>,

    /// Include submodules by default (default: true)
    #[serde(default = "default_true")]
    pub include_submodules: bool,

    /// History indexing settings
    #[serde(default)]
    pub history: HistoryConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryConfig {
    /// Enable history indexing (default: true)
    pub enabled: bool,
    /// Max commits to index (default: 50)
    pub max_commits: usize,
    /// Sample rate: index every N commits (default: 1)
    pub sample_rate: usize,
}

fn default_branches() -> Vec<String> {
    vec!["main".to_string(), "HEAD".to_string()]
}
```

## Dependencies

Add to `Cargo.toml`:

```toml
git2 = "0.20"
```

## Files to Create

| File | Purpose |
|------|---------|
| `crates/mcb-domain/src/git.rs` | Git domain types |
| `crates/mcb-application/src/ports/providers/git.rs` | GitProvider trait |
| `crates/mcb-providers/src/git/mod.rs` | Git module |
| `crates/mcb-providers/src/git/git2_provider.rs` | git2 implementation |
| `crates/mcb-application/src/use_cases/repository.rs` | Repository manager |
| `crates/mcb-application/src/use_cases/git_indexing.rs` | Git-aware indexing |
| `crates/mcb-application/src/use_cases/impact.rs` | Impact analysis |
| `crates/mcb-infrastructure/src/snapshot/git_snapshot.rs` | Git-based change detection |
| `crates/mcb-server/src/handlers/git_tools.rs` | MCP git tools |
| `crates/mcb-infrastructure/src/config/git.rs` | Git configuration |

## Files to Modify

| File | Change |
|------|--------|
| `crates/mcb-providers/Cargo.toml` | Add `git2 = "0.20"` dependency |
| `crates/mcb-domain/src/entities/code_chunk.rs` | Add `git_metadata` field to CodeChunk |
| `crates/mcb-application/src/ports/providers/mod.rs` | Export GitProvider |
| `crates/mcb-domain/src/mod.rs` | Export git module |
| `crates/mcb-providers/src/lib.rs` | Export git provider |
| `crates/mcb-application/src/use_cases/mod.rs` | Export repository, git_indexing, impact |
| `crates/mcb-application/src/use_cases/indexing.rs` | Integrate with GitIndexingService |
| `crates/mcb-infrastructure/src/snapshot/mod.rs` | Export git_snapshot |
| `crates/mcb-server/src/mcp_server.rs` | Register new tools |
| `crates/mcb-infrastructure/src/config/mod.rs` | Export git config |

## Success Metrics

| Metric | Before | Target v0.2.0 |
|--------|--------|---------------|
| Portability | Filesystem path | Root commit ID |
| Multi-branch | No | Yes |
| Submodules | No | Yes |
| History | No | 50 commits |
| Impact | No | Yes |

## Configuration Defaults

| Setting | Default | Override |
|---------|---------|----------|
| Branches | main, HEAD, current | Per-repo |
| History depth | 50 commits | Per-repo |
| Submodules | Recursive indexing | Per-repo |

## Related ADRs

-   [ADR-001: Provider Pattern Architecture](001-provider-pattern-architecture.md) - Provider patterns for GitProvider
-   [ADR-002: Async-First Architecture](002-async-first-architecture.md) - Async git operations
-   [ADR-004: Multi-Provider Strategy](004-multi-provider-strategy.md) - Provider routing
-   [ADR-009: Persistent Session Memory](009-persistent-session-memory-v0.2.0.md) - Git-tagged memory entries
-   [ADR-012: Two-Layer DI Strategy](012-di-strategy-two-layer-approach.md) - DI for git providers
-   [ADR-013: Clean Architecture Crate Separation](013-clean-architecture-crate-separation.md) - Crate organization

## References

-   [git2 crate](https://docs.rs/git2/)
-   [libgit2](https://libgit2.org/)
-   [Shaku Documentation](https://docs.rs/shaku)
