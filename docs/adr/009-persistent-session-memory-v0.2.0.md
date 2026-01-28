# ADR 009: Persistent Session Memory v0.2.0

## Status

**Proposed**(Planned for v0.2.0)

> Not yet implemented. Target crate structure for v0.2.0:
>
> -   `crates/mcb-domain/src/memory.rs` - Memory domain types
> -   `crates/mcb-application/src/ports/providers/memory.rs` - MemoryProvider port trait
> -   `crates/mcb-application/src/use_cases/session.rs` - Session manager service
> -   `crates/mcb-application/src/use_cases/memory_search.rs` - Memory search service
> -   `crates/mcb-application/src/use_cases/context_injection.rs` - Context injection
> -   `crates/mcb-providers/src/memory/` - Memory provider implementations
> -   `crates/mcb-server/src/handlers/memory_tools.rs` - MCP tool handlers
> -   `crates/mcb-infrastructure/src/config/memory.rs` - Memory configuration
> -   Requires ADR-008 git integration for git-tagged observations

## Context

MCP Context Browser v0.1.0 provides semantic code search but lacks session-level memory persistence. Each Claude Code session starts fresh, losing valuable context:

**Current problems:**

-   No persistence of tool observations across sessions
-   No session summaries or decision tracking
-   No context injection for session continuity
-   No semantic search over past work
-   Token waste re-discovering solved problems
-   No ROI metrics on context discovery

**User demand:**

-   Developers need cross-session memory (like Claude-mem provides)
-   Need to recall past decisions and rationale
-   Need semantic search over session history
-   Need progressive disclosure (index → context → details)
-   Need token-efficient context injection

**Reference implementation**: Claude-mem v8.5.2 demonstrates these features work well in practice with TypeScript + SQLite + Chroma architecture.

## Decision

Implement persistent session memory in mcb v0.2.0 by porting Claude-mem's core architecture to Rust:

1.**Observation storage**via existing vector store infrastructure
2.**Session management**with lifecycle tracking
3.**Memory compression**via configurable summarization
4.**Hybrid search**combining existing vector search with BM25
5.**3-layer workflow**(search → timeline → get_observations)
6.**Context injection**for SessionStart hook integration
7.**Progressive disclosure**with token cost visibility

**Key design choice**: Leverage existing mcb infrastructure (provider pattern, vector stores, hybrid search) rather than duplicating Claude-mem's SQLite + Chroma approach.

### Architecture Overview

```
Claude Code Session
        ↓
[Hook Integration] (PostToolUse → save observation)
        ↓
┌─────────────────────────────────────────┐
│ MCP Context Browser Server              │
│ ├── MemoryService (new)                 │
│ ├── SessionManager (new)                │
│ ├── ObservationStore (reuses VectorStore)│
│ └── HybridSearch (existing)             │
└─────────────────────────────────────────┘
        ↓
┌─────────────────────────────────────────┐
│ Storage Layer                           │
│ ├── Vector Store (observations)         │
│ └── SQLite/Milvus (metadata)           │
└─────────────────────────────────────────┘
```

## Consequences

### Positive

-   **Context preservation**: Past decisions, fixes, and discoveries survive sessions
-   **Token efficiency**: 10x savings via 3-layer progressive disclosure
-   **Unified platform**: Single MCP server for code search + session memory
-   **Infrastructure reuse**: Leverages existing vector stores, hybrid search
-   **Git integration**: Memory entries tagged with git context (ADR-008)
-   **Admin UI**: Session dashboard in web interface (ADR-007)

### Negative

-   **Complexity**: Adds ~15 new files, ~3000 LOC
-   **Storage growth**: Per-session observations increase disk usage
-   **Hook dependency**: Requires external hook setup for full functionality
-   **Compression model**: Needs configured embedding provider

## Alternatives Considered

### Alternative 1: Use Claude-mem directly as plugin

-   **Pros**: Proven, feature-complete
-   **Cons**: Separate service, no integration with code search
-   **Rejected**: Missed opportunity for unified platform

### Alternative 2: SQLite-only storage (like Claude-mem)

-   **Pros**: Simpler, proven approach
-   **Cons**: Duplicates existing vector infrastructure
-   **Rejected**: Leverage existing providers

### Alternative 3: Defer to v0.3.0

-   **Pros**: Focus v0.2.0 on git only
-   **Cons**: Delays high-value feature
-   **Rejected**: Memory complements git context well

## Implementation Notes

### Phase 1: Domain Model

**Create**: `crates/mcb-domain/src/memory.rs`

```rust
use serde::{Deserialize, Serialize};
use crate::domain::git::GitChunkMetadata;

/// Observation type classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ObservationType {
    Decision,
    Bugfix,
    Feature,
    Refactor,
    Discovery,
    Change,
}

impl std::fmt::Display for ObservationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Decision => write!(f, "decision"),
            Self::Bugfix => write!(f, "bugfix"),
            Self::Feature => write!(f, "feature"),
            Self::Refactor => write!(f, "refactor"),
            Self::Discovery => write!(f, "discovery"),
            Self::Change => write!(f, "change"),
        }
    }
}

/// Single memory observation (compressed tool output)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    /// Unique identifier
    pub id: u64,
    /// Session this observation belongs to
    pub session_id: String,
    /// Project/repository name
    pub project: String,
    /// Observation classification
    pub obs_type: ObservationType,
    /// Short title (< 100 chars)
    pub title: String,
    /// Optional subtitle for context
    pub subtitle: Option<String>,
    /// Key facts as bullet points
    pub facts: Vec<String>,
    /// Human-readable narrative
    pub narrative: Option<String>,
    /// Abstract concepts for semantic search
    pub concepts: Vec<String>,
    /// Files read during this observation
    pub files_read: Vec<String>,
    /// Files modified during this observation
    pub files_modified: Vec<String>,
    /// Which prompt number in session
    pub prompt_number: Option<u32>,
    /// Tokens spent discovering this (ROI metric)
    pub discovery_tokens: u64,
    /// Git context if available
    pub git_metadata: Option<GitChunkMetadata>,
    /// Creation timestamp (Unix epoch)
    pub created_at: u64,
}

/// Session-level summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    /// Unique identifier
    pub id: u64,
    /// Session being summarized
    pub session_id: String,
    /// Project/repository name
    pub project: String,
    /// What was the user's request?
    pub request: Option<String>,
    /// What was investigated?
    pub investigated: Option<String>,
    /// Key learnings
    pub learned: Option<String>,
    /// What was completed?
    pub completed: Option<String>,
    /// Recommended next steps
    pub next_steps: Option<String>,
    /// Additional notes
    pub notes: Option<String>,
    /// All files read in session
    pub files_read: Vec<String>,
    /// All files edited in session
    pub files_edited: Vec<String>,
    /// Cumulative discovery tokens
    pub discovery_tokens: u64,
    /// Git context at session end
    pub git_metadata: Option<GitChunkMetadata>,
    /// Creation timestamp
    pub created_at: u64,
}

/// Active session state (in-memory)
#[derive(Debug, Clone)]
pub struct ActiveSession {
    /// Database ID
    pub db_id: u64,
    /// Claude Code session ID
    pub session_id: String,
    /// Project name
    pub project: String,
    /// Current user prompt
    pub user_prompt: Option<String>,
    /// Pending observations awaiting compression
    pub pending_observations: Vec<PendingObservation>,
    /// Last prompt number
    pub last_prompt_number: u32,
    /// Session start time
    pub start_time: u64,
    /// Cumulative input tokens
    pub input_tokens: u64,
    /// Cumulative output tokens
    pub output_tokens: u64,
}

/// Raw observation before compression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingObservation {
    pub id: u64,
    pub tool_name: String,
    pub tool_input: serde_json::Value,
    pub tool_output: String,
    pub prompt_number: u32,
    pub created_at: u64,
}

/// User prompt history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPrompt {
    pub id: u64,
    pub session_id: String,
    pub prompt_number: u32,
    pub prompt_text: String,
    pub created_at: u64,
}

/// Search result in index format (token-efficient)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservationIndex {
    pub id: u64,
    pub obs_type: ObservationType,
    pub title: String,
    pub project: String,
    pub created_at: u64,
    /// Estimated tokens to fetch full details
    pub estimated_tokens: u32,
}

/// Timeline context around an anchor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineContext {
    pub anchor_id: u64,
    pub before: Vec<ObservationIndex>,
    pub anchor: ObservationIndex,
    pub after: Vec<ObservationIndex>,
}
```

### Phase 2: Memory Provider Port

**Create**: `crates/mcb-application/src/ports/providers/memory.rs`

```rust
use async_trait::async_trait;
use crate::domain::memory::*;
use crate::domain::Result;

/// Memory storage provider interface
#[async_trait]
pub trait MemoryProvider: Send + Sync {
    // === Observation Operations ===

    /// Store a new observation
    async fn store_observation(&self, obs: &Observation) -> Result<u64>;

    /// Get observation by ID
    async fn get_observation(&self, id: u64) -> Result<Option<Observation>>;

    /// Get multiple observations by IDs (batch)
    async fn get_observations(&self, ids: &[u64]) -> Result<Vec<Observation>>;

    /// Search observations with filters
    async fn search_observations(&self, query: &SearchQuery) -> Result<Vec<ObservationIndex>>;

    // === Session Operations ===

    /// Create new session
    async fn create_session(&self, session: &ActiveSession) -> Result<u64>;

    /// Get session by ID
    async fn get_session(&self, session_id: &str) -> Result<Option<ActiveSession>>;

    /// Update session state
    async fn update_session(&self, session: &ActiveSession) -> Result<()>;

    /// Store session summary
    async fn store_summary(&self, summary: &SessionSummary) -> Result<u64>;

    /// Get session summary
    async fn get_summary(&self, session_id: &str) -> Result<Option<SessionSummary>>;

    // === Prompt Operations ===

    /// Store user prompt
    async fn store_prompt(&self, prompt: &UserPrompt) -> Result<u64>;

    /// Get prompts for session
    async fn get_prompts(&self, session_id: &str) -> Result<Vec<UserPrompt>>;

    // === Pending Queue ===

    /// Queue pending observation for compression
    async fn queue_pending(&self, pending: &PendingObservation) -> Result<u64>;

    /// Get pending observations for session
    async fn get_pending(&self, session_id: &str) -> Result<Vec<PendingObservation>>;

    /// Remove pending observation after compression
    async fn remove_pending(&self, id: u64) -> Result<()>;

    /// Provider name for logging
    fn provider_name(&self) -> &str;
}

/// Search query with filters
#[derive(Debug, Clone, Default)]
pub struct SearchQuery {
    /// Full-text search query
    pub query: Option<String>,
    /// Filter by project
    pub project: Option<String>,
    /// Filter by observation type
    pub obs_type: Option<ObservationType>,
    /// Filter by concepts
    pub concepts: Vec<String>,
    /// Filter by files
    pub files: Vec<String>,
    /// Date range start (Unix epoch)
    pub date_start: Option<u64>,
    /// Date range end (Unix epoch)
    pub date_end: Option<u64>,
    /// Result limit
    pub limit: usize,
    /// Result offset
    pub offset: usize,
    /// Order by
    pub order_by: SearchOrder,
}

#[derive(Debug, Clone, Default)]
pub enum SearchOrder {
    #[default]
    Relevance,
    DateDesc,
    DateAsc,
}
```

### Phase 3: Memory Storage Adapter

**Create**: `crates/mcb-providers/src/memory/sqlite_memory.rs`

```rust
use async_trait::async_trait;
use sqlx::{SqlitePool, Row};
use crate::domain::memory::*;
use crate::domain::ports::memory::*;

pub struct SqliteMemoryProvider {
    pool: SqlitePool,
}

impl SqliteMemoryProvider {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = SqlitePool::connect(database_url).await?;
        let provider = Self { pool };
        provider.run_migrations().await?;
        Ok(provider)
    }

    async fn run_migrations(&self) -> Result<()> {
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS observations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT NOT NULL,
                project TEXT NOT NULL,
                obs_type TEXT NOT NULL,
                title TEXT NOT NULL,
                subtitle TEXT,
                facts TEXT NOT NULL DEFAULT '[]',
                narrative TEXT,
                concepts TEXT NOT NULL DEFAULT '[]',
                files_read TEXT NOT NULL DEFAULT '[]',
                files_modified TEXT NOT NULL DEFAULT '[]',
                prompt_number INTEGER,
                discovery_tokens INTEGER NOT NULL DEFAULT 0,
                git_metadata TEXT,
                created_at INTEGER NOT NULL,
                FOREIGN KEY (session_id) REFERENCES sessions(session_id)
            );

            CREATE INDEX IF NOT EXISTS idx_obs_session ON observations(session_id);
            CREATE INDEX IF NOT EXISTS idx_obs_project ON observations(project);
            CREATE INDEX IF NOT EXISTS idx_obs_created ON observations(created_at);

            CREATE VIRTUAL TABLE IF NOT EXISTS observations_fts USING fts5(
                title, narrative, facts, concepts,
                content='observations',
                content_rowid='id'
            );
        "#).execute(&self.pool).await?;

        // Additional tables for sessions, summaries, prompts, pending...
        Ok(())
    }
}

#[async_trait]
impl MemoryProvider for SqliteMemoryProvider {
    async fn store_observation(&self, obs: &Observation) -> Result<u64> {
        let facts_json = serde_json::to_string(&obs.facts)?;
        let concepts_json = serde_json::to_string(&obs.concepts)?;
        let files_read_json = serde_json::to_string(&obs.files_read)?;
        let files_modified_json = serde_json::to_string(&obs.files_modified)?;
        let git_json = obs.git_metadata.as_ref()
            .map(|g| serde_json::to_string(g))
            .transpose()?;

        let id = sqlx::query(r#"
            INSERT INTO observations (
                session_id, project, obs_type, title, subtitle,
                facts, narrative, concepts, files_read, files_modified,
                prompt_number, discovery_tokens, git_metadata, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#)
        .bind(&obs.session_id)
        .bind(&obs.project)
        .bind(obs.obs_type.to_string())
        .bind(&obs.title)
        .bind(&obs.subtitle)
        .bind(&facts_json)
        .bind(&obs.narrative)
        .bind(&concepts_json)
        .bind(&files_read_json)
        .bind(&files_modified_json)
        .bind(obs.prompt_number)
        .bind(obs.discovery_tokens as i64)
        .bind(&git_json)
        .bind(obs.created_at as i64)
        .execute(&self.pool)
        .await?
        .last_insert_rowid() as u64;

        // Update FTS index
        self.update_fts_index(id, obs).await?;

        Ok(id)
    }

    async fn search_observations(&self, query: &SearchQuery) -> Result<Vec<ObservationIndex>> {
        let mut sql = String::from(r#"
            SELECT o.id, o.obs_type, o.title, o.project, o.created_at
            FROM observations o
        "#);

        let mut conditions = Vec::new();
        let mut use_fts = false;

        if let Some(ref q) = query.query {
            sql.push_str(" JOIN observations_fts fts ON fts.rowid = o.id");
            conditions.push(format!("fts MATCH '{}'", q.replace("'", "''")));
            use_fts = true;
        }

        if let Some(ref project) = query.project {
            conditions.push(format!("o.project = '{}'", project));
        }

        if let Some(ref obs_type) = query.obs_type {
            conditions.push(format!("o.obs_type = '{}'", obs_type));
        }

        if !conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&conditions.join(" AND "));
        }

        sql.push_str(match query.order_by {
            SearchOrder::Relevance if use_fts => " ORDER BY rank",
            SearchOrder::DateDesc | SearchOrder::Relevance => " ORDER BY o.created_at DESC",
            SearchOrder::DateAsc => " ORDER BY o.created_at ASC",
        });

        sql.push_str(&format!(" LIMIT {} OFFSET {}", query.limit, query.offset));

        let rows = sqlx::query(&sql)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(|row| ObservationIndex {
            id: row.get::<i64, _>("id") as u64,
            obs_type: parse_obs_type(row.get("obs_type")),
            title: row.get("title"),
            project: row.get("project"),
            created_at: row.get::<i64, _>("created_at") as u64,
            estimated_tokens: 500, // Approximate
        }).collect())
    }

    fn provider_name(&self) -> &str {
        "sqlite_memory"
    }

    // ... implement remaining methods ...
}
```

### Phase 4: Session Manager Service

**Create**: `crates/mcb-application/src/use_cases/session.rs`

```rust
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::broadcast;
use crate::domain::memory::*;
use crate::domain::ports::memory::MemoryProvider;

pub struct SessionManager {
    /// Memory storage provider
    memory_provider: Arc<dyn MemoryProvider>,
    /// Active sessions (in-memory cache)
    active_sessions: DashMap<String, ActiveSession>,
    /// Event broadcaster for real-time updates
    event_tx: broadcast::Sender<SessionEvent>,
}

#[derive(Debug, Clone)]
pub enum SessionEvent {
    SessionStarted { session_id: String, project: String },
    ObservationStored { session_id: String, observation_id: u64 },
    SessionEnded { session_id: String, summary_id: Option<u64> },
}

impl SessionManager {
    pub fn new(memory_provider: Arc<dyn MemoryProvider>) -> Self {
        let (event_tx, _) = broadcast::channel(1024);
        Self {
            memory_provider,
            active_sessions: DashMap::new(),
            event_tx,
        }
    }

    /// Start or resume a session
    pub async fn start_session(
        &self,
        session_id: &str,
        project: &str,
    ) -> Result<ActiveSession> {
        // Check for existing session
        if let Some(existing) = self.active_sessions.get(session_id) {
            return Ok(existing.clone());
        }

        // Check database for previous session
        if let Some(db_session) = self.memory_provider.get_session(session_id).await? {
            self.active_sessions.insert(session_id.to_string(), db_session.clone());
            return Ok(db_session);
        }

        // Create new session
        let session = ActiveSession {
            db_id: 0,
            session_id: session_id.to_string(),
            project: project.to_string(),
            user_prompt: None,
            pending_observations: Vec::new(),
            last_prompt_number: 0,
            start_time: now_epoch(),
            input_tokens: 0,
            output_tokens: 0,
        };

        let db_id = self.memory_provider.create_session(&session).await?;
        let session = ActiveSession { db_id, ..session };

        self.active_sessions.insert(session_id.to_string(), session.clone());

        let _ = self.event_tx.send(SessionEvent::SessionStarted {
            session_id: session_id.to_string(),
            project: project.to_string(),
        });

        Ok(session)
    }

    /// Store a tool observation
    pub async fn store_observation(
        &self,
        session_id: &str,
        observation: Observation,
    ) -> Result<u64> {
        let obs_id = self.memory_provider.store_observation(&observation).await?;

        let _ = self.event_tx.send(SessionEvent::ObservationStored {
            session_id: session_id.to_string(),
            observation_id: obs_id,
        });

        Ok(obs_id)
    }

    /// End session and generate summary
    pub async fn end_session(
        &self,
        session_id: &str,
        summary: Option<SessionSummary>,
    ) -> Result<Option<u64>> {
        let summary_id = if let Some(s) = summary {
            Some(self.memory_provider.store_summary(&s).await?)
        } else {
            None
        };

        self.active_sessions.remove(session_id);

        let _ = self.event_tx.send(SessionEvent::SessionEnded {
            session_id: session_id.to_string(),
            summary_id,
        });

        Ok(summary_id)
    }

    /// Subscribe to session events
    pub fn subscribe(&self) -> broadcast::Receiver<SessionEvent> {
        self.event_tx.subscribe()
    }
}
```

### Phase 5: Memory Search Service

**Create**: `crates/mcb-application/src/use_cases/memory_search.rs`

```rust
use std::sync::Arc;
use crate::adapters::hybrid_search::HybridSearchEngine;
use crate::domain::memory::*;
use crate::domain::ports::memory::{MemoryProvider, SearchQuery};

pub struct MemorySearchService {
    memory_provider: Arc<dyn MemoryProvider>,
    hybrid_search: Arc<HybridSearchEngine>,
}

impl MemorySearchService {
    pub fn new(
        memory_provider: Arc<dyn MemoryProvider>,
        hybrid_search: Arc<HybridSearchEngine>,
    ) -> Self {
        Self { memory_provider, hybrid_search }
    }

    /// Step 1: Search and return index (token-efficient)
    pub async fn search(&self, query: SearchQuery) -> Result<Vec<ObservationIndex>> {
        // Use hybrid search combining vector similarity + BM25
        self.memory_provider.search_observations(&query).await
    }

    /// Step 2: Get timeline context around anchor
    pub async fn timeline(
        &self,
        anchor_id: u64,
        depth_before: usize,
        depth_after: usize,
        project: Option<&str>,
    ) -> Result<TimelineContext> {
        let anchor = self.memory_provider.get_observation(anchor_id).await?
            .ok_or_else(|| Error::NotFound("Observation not found"))?;

        let before_query = SearchQuery {
            project: project.map(String::from),
            date_end: Some(anchor.created_at - 1),
            limit: depth_before,
            order_by: SearchOrder::DateDesc,
            ..Default::default()
        };

        let after_query = SearchQuery {
            project: project.map(String::from),
            date_start: Some(anchor.created_at + 1),
            limit: depth_after,
            order_by: SearchOrder::DateAsc,
            ..Default::default()
        };

        let before = self.memory_provider.search_observations(&before_query).await?;
        let after = self.memory_provider.search_observations(&after_query).await?;

        Ok(TimelineContext {
            anchor_id,
            before,
            anchor: observation_to_index(&anchor),
            after,
        })
    }

    /// Step 3: Get full observation details (batch)
    pub async fn get_observations(&self, ids: &[u64]) -> Result<Vec<Observation>> {
        self.memory_provider.get_observations(ids).await
    }

    /// Shortcut: Find decisions
    pub async fn decisions(&self, project: Option<&str>, limit: usize) -> Result<Vec<ObservationIndex>> {
        self.search(SearchQuery {
            project: project.map(String::from),
            obs_type: Some(ObservationType::Decision),
            limit,
            ..Default::default()
        }).await
    }

    /// Shortcut: Find changes for a file
    pub async fn changes_for_file(&self, file: &str, limit: usize) -> Result<Vec<ObservationIndex>> {
        self.search(SearchQuery {
            files: vec![file.to_string()],
            limit,
            ..Default::default()
        }).await
    }
}
```

### Phase 6: Context Injection Service

**Create**: `crates/mcb-application/src/use_cases/context_injection.rs`

```rust
use std::sync::Arc;
use crate::application::memory_search::MemorySearchService;
use crate::domain::memory::*;
use crate::domain::ports::memory::SearchQuery;

pub struct ContextInjectionService {
    search_service: Arc<MemorySearchService>,
}

#[derive(Debug, Clone)]
pub struct ContextInjectionConfig {
    /// Observation types to include
    pub observation_types: Vec<ObservationType>,
    /// Concepts to filter by
    pub concepts: Vec<String>,
    /// Max observations to include
    pub observation_limit: usize,
    /// Max sessions to include
    pub session_limit: usize,
    /// Date range in days
    pub date_range_days: Option<u32>,
}

impl Default for ContextInjectionConfig {
    fn default() -> Self {
        Self {
            observation_types: vec![
                ObservationType::Decision,
                ObservationType::Bugfix,
                ObservationType::Feature,
            ],
            concepts: Vec::new(),
            observation_limit: 20,
            session_limit: 5,
            date_range_days: Some(30),
        }
    }
}

impl ContextInjectionService {
    pub fn new(search_service: Arc<MemorySearchService>) -> Self {
        Self { search_service }
    }

    /// Generate context for SessionStart hook
    pub async fn generate_context(
        &self,
        project: &str,
        config: &ContextInjectionConfig,
    ) -> Result<String> {
        let mut output = String::new();

        // Header
        output.push_str(&format!(
            "# [{}] recent context, {}\n\n",
            project,
            format_date_now()
        ));

        // Legend
        output.push_str("**Legend:**session-request | bugfix | feature | refactor | change | discovery | decision\n\n");

        // Column key
        output.push_str("**Column Key**:\n");
        output.push_str("-**Read**: Tokens to read this observation (cost to learn it now)\n");
        output.push_str("-**Work**: Tokens spent on work that produced this record\n\n");

        // Build query
        let query = SearchQuery {
            project: Some(project.to_string()),
            limit: config.observation_limit,
            date_start: config.date_range_days.map(|d| now_epoch() - (d as u64 * 86400)),
            ..Default::default()
        };

        let observations = self.search_service.search(query).await?;

        // Group by date
        let grouped = group_by_date(&observations);

        // Format each group
        for (date, obs) in grouped {
            output.push_str(&format!("### {}\n\n", date));
            output.push_str("| ID | Time | T | Title | Read | Work |\n");
            output.push_str("|----|------|---|-------|------|------|\n");

            for o in obs {
                output.push_str(&format!(
                    "| #{} | {} | {} | {} | ~{} | |\n",
                    o.id,
                    format_time(o.created_at),
                    type_emoji(&o.obs_type),
                    truncate(&o.title, 60),
                    o.estimated_tokens
                ));
            }

            output.push_str("\n");
        }

        // Token economics
        let total_tokens: u32 = observations.iter().map(|o| o.estimated_tokens).sum();
        output.push_str(&format!(
            "**Context Economics**:\n- Loading: {} observations ({} tokens to read)\n",
            observations.len(),
            total_tokens
        ));

        output.push_str("\nUse MCP tools (search, get_observations) to fetch full observations on-demand.\n");

        Ok(output)
    }
}

fn type_emoji(t: &ObservationType) -> &'static str {
    match t {
        ObservationType::Decision => "decision",
        ObservationType::Bugfix => "bugfix",
        ObservationType::Feature => "feature",
        ObservationType::Refactor => "refactor",
        ObservationType::Discovery => "discovery",
        ObservationType::Change => "change",
    }
}
```

### Phase 7: MCP Tools

**Create**: `crates/mcb-server/src/handlers/memory_tools.rs`

```rust
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::application::memory_search::MemorySearchService;
use crate::application::session::SessionManager;
use crate::application::context_injection::ContextInjectionService;

/// Tool: search - Step 1 of 3-layer workflow
#[derive(Debug, Deserialize)]
pub struct SearchInput {
    pub query: Option<String>,
    pub project: Option<String>,
    #[serde(rename = "type")]
    pub entity_type: Option<String>,
    pub obs_type: Option<String>,
    pub concepts: Option<Vec<String>>,
    pub files: Option<Vec<String>>,
    pub date_start: Option<String>,
    pub date_end: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub order_by: Option<String>,
}

/// Tool: timeline - Step 2 of 3-layer workflow
#[derive(Debug, Deserialize)]
pub struct TimelineInput {
    /// Anchor observation ID (or query to find it)
    pub anchor: Option<u64>,
    pub query: Option<String>,
    pub depth_before: Option<usize>,
    pub depth_after: Option<usize>,
    pub project: Option<String>,
}

/// Tool: get_observations - Step 3 of 3-layer workflow
#[derive(Debug, Deserialize)]
pub struct GetObservationsInput {
    pub ids: Vec<u64>,
    pub project: Option<String>,
}

/// Tool: store_observation - PostToolUse hook integration
#[derive(Debug, Deserialize)]
pub struct StoreObservationInput {
    pub session_id: String,
    pub tool_name: String,
    pub tool_input: serde_json::Value,
    pub tool_output: String,
    pub project: String,
}

/// Tool: inject_context - SessionStart hook integration
#[derive(Debug, Deserialize)]
pub struct InjectContextInput {
    pub project: String,
    pub session_id: Option<String>,
    pub observation_types: Option<Vec<String>>,
    pub observation_limit: Option<usize>,
}

pub struct MemoryToolsHandler {
    search_service: Arc<MemorySearchService>,
    session_manager: Arc<SessionManager>,
    context_service: Arc<ContextInjectionService>,
}

impl MemoryToolsHandler {
    pub async fn handle_search(&self, input: SearchInput) -> Result<serde_json::Value> {
        let query = SearchQuery {
            query: input.query,
            project: input.project,
            obs_type: input.obs_type.and_then(|s| parse_obs_type(&s)),
            concepts: input.concepts.unwrap_or_default(),
            files: input.files.unwrap_or_default(),
            limit: input.limit.unwrap_or(20),
            offset: input.offset.unwrap_or(0),
            ..Default::default()
        };

        let results = self.search_service.search(query).await?;

        // Format as token-efficient table
        Ok(serde_json::json!({
            "count": results.len(),
            "results": results,
            "hint": "Use timeline(anchor=ID) for context, get_observations(ids=[...]) for details"
        }))
    }

    pub async fn handle_timeline(&self, input: TimelineInput) -> Result<serde_json::Value> {
        let anchor_id = match (input.anchor, input.query) {
            (Some(id), _) => id,
            (None, Some(q)) => {
                // Auto-find anchor via search
                let results = self.search_service.search(SearchQuery {
                    query: Some(q),
                    limit: 1,
                    ..Default::default()
                }).await?;
                results.first()
                    .ok_or_else(|| Error::NotFound("No matching observation"))?
                    .id
            }
            (None, None) => return Err(Error::InvalidInput("anchor or query required")),
        };

        let context = self.search_service.timeline(
            anchor_id,
            input.depth_before.unwrap_or(3),
            input.depth_after.unwrap_or(3),
            input.project.as_deref(),
        ).await?;

        Ok(serde_json::to_value(context)?)
    }

    pub async fn handle_get_observations(&self, input: GetObservationsInput) -> Result<serde_json::Value> {
        let observations = self.search_service.get_observations(&input.ids).await?;
        Ok(serde_json::to_value(observations)?)
    }

    pub async fn handle_inject_context(&self, input: InjectContextInput) -> Result<serde_json::Value> {
        let config = ContextInjectionConfig {
            observation_limit: input.observation_limit.unwrap_or(20),
            ..Default::default()
        };

        let context = self.context_service.generate_context(&input.project, &config).await?;

        Ok(serde_json::json!({
            "context": context
        }))
    }
}
```

### Phase 8: MCP Tool Registration

**Modify**: `crates/mcb-server/src/mcp_server.rs`

```rust
// Add to tool registration
fn register_memory_tools(&self) -> Vec<ToolInfo> {
    vec![
        ToolInfo {
            name: "__IMPORTANT".to_string(),
            description: "3-LAYER WORKFLOW (ALWAYS FOLLOW):\n\

                1. search(query) -> Get index with IDs (~50-100 tokens/result)\n\
                2. timeline(anchor=ID) -> Get context around interesting results\n\
                3. get_observations([IDs]) -> Fetch full details ONLY for filtered IDs\n\

                NEVER fetch full details without filtering first. 10x token savings.".to_string(),
            input_schema: json!({"type": "object", "properties": {}}),
        },
        ToolInfo {
            name: "search".to_string(),
            description: "Step 1: Search memory. Returns index with IDs. \
                Params: query, limit, project, type, obs_type, dateStart, dateEnd, offset, orderBy".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string"},
                    "project": {"type": "string"},
                    "obs_type": {"type": "string"},
                    "limit": {"type": "integer"},
                    "offset": {"type": "integer"}
                }
            }),
        },
        ToolInfo {
            name: "timeline".to_string(),
            description: "Step 2: Get context around results. \
                Params: anchor (observation ID) OR query (finds anchor automatically), \
                depth_before, depth_after, project".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "anchor": {"type": "integer"},
                    "query": {"type": "string"},
                    "depth_before": {"type": "integer"},
                    "depth_after": {"type": "integer"},
                    "project": {"type": "string"}
                }
            }),
        },
        ToolInfo {
            name: "get_observations".to_string(),
            description: "Step 3: Fetch full details for filtered IDs. \
                Params: ids (array of observation IDs, required), orderBy, limit, project".to_string(),
            input_schema: json!({
                "type": "object",
                "required": ["ids"],
                "properties": {
                    "ids": {
                        "type": "array",
                        "items": {"type": "integer"},
                        "description": "Array of observation IDs to fetch"
                    },
                    "project": {"type": "string"}
                }
            }),
        },
        ToolInfo {
            name: "store_observation".to_string(),
            description: "Store tool observation (for PostToolUse hook integration)".to_string(),
            input_schema: json!({
                "type": "object",
                "required": ["session_id", "tool_name", "project"],
                "properties": {
                    "session_id": {"type": "string"},
                    "tool_name": {"type": "string"},
                    "tool_input": {"type": "object"},
                    "tool_output": {"type": "string"},
                    "project": {"type": "string"}
                }
            }),
        },
        ToolInfo {
            name: "inject_context".to_string(),
            description: "Generate context for SessionStart hook injection".to_string(),
            input_schema: json!({
                "type": "object",
                "required": ["project"],
                "properties": {
                    "project": {"type": "string"},
                    "session_id": {"type": "string"},
                    "observation_limit": {"type": "integer"}
                }
            }),
        },
    ]
}
```

### Phase 9: Configuration

**Create**: `crates/mcb-infrastructure/src/config/memory.rs`

```rust
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct MemoryConfig {
    /// Enable memory features (default: true)
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// SQLite database path (default: ~/.mcb/memory.db)
    #[serde(default = "default_database_path")]
    pub database_path: String,

    /// Enable vector embeddings for semantic search (default: true)
    #[serde(default = "default_enabled")]
    pub enable_vector_search: bool,

    /// Context injection settings
    #[serde(default)]
    pub context: ContextConfig,

    /// Compression settings
    #[serde(default)]
    pub compression: CompressionConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContextConfig {
    /// Observation types to include in context
    #[serde(default = "default_observation_types")]
    pub observation_types: Vec<String>,

    /// Max observations per context injection
    #[serde(default = "default_observation_limit")]
    pub observation_limit: usize,

    /// Max sessions to reference
    #[serde(default = "default_session_limit")]
    pub session_limit: usize,

    /// Date range in days (default: 30)
    #[serde(default = "default_date_range")]
    pub date_range_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompressionConfig {
    /// Enable automatic compression via SDK agent (default: false)
    /// When false, observations are stored as-is
    #[serde(default)]
    pub enable_sdk_compression: bool,

    /// Model for compression (default: claude-3-haiku-20240307)
    #[serde(default = "default_compression_model")]
    pub model: String,

    /// Max tokens per compression batch
    #[serde(default = "default_max_tokens")]
    pub max_batch_tokens: usize,
}

fn default_enabled() -> bool { true }
fn default_database_path() -> String { "~/.mcb/memory.db".to_string() }
fn default_observation_types() -> Vec<String> {
    vec!["decision".to_string(), "bugfix".to_string(), "feature".to_string()]
}
fn default_observation_limit() -> usize { 20 }
fn default_session_limit() -> usize { 5 }
fn default_date_range() -> u32 { 30 }
fn default_compression_model() -> String { "claude-3-haiku-20240307".to_string() }
fn default_max_tokens() -> usize { 8000 }
```

### Phase 10: Admin UI Integration

**Modify**: `crates/mcb-server/src/admin/web/templates/` - Add memory dashboard

```html
<!-- memory.html -->
<div class="memory-dashboard">
  <h2>Session Memory</h2>

  <div class="stats-cards">
    <div class="stat-card">
      <span class="stat-value" id="total-observations">0</span>
      <span class="stat-label">Total Observations</span>
    </div>
    <div class="stat-card">
      <span class="stat-value" id="total-sessions">0</span>
      <span class="stat-label">Sessions</span>
    </div>
    <div class="stat-card">
      <span class="stat-value" id="tokens-saved">0</span>
      <span class="stat-label">Tokens Saved</span>
    </div>
  </div>

  <div class="recent-observations">
    <h3>Recent Observations</h3>
    <table id="observations-table">
      <thead>
        <tr>
          <th>ID</th>
          <th>Type</th>
          <th>Title</th>
          <th>Project</th>
          <th>Time</th>
        </tr>
      </thead>
      <tbody></tbody>
    </table>
  </div>

  <div class="search-panel">
    <h3>Search Memory</h3>
    <input type="text" id="memory-search" placeholder="Search observations...">
    <div id="search-results"></div>
  </div>
</div>
```

## Dependencies

Add to `Cargo.toml`:

```toml
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite"] }
```

## Files to Create

| File | Purpose |
|------|---------|
| `crates/mcb-domain/src/memory.rs` | Memory domain types |
| `crates/mcb-application/src/ports/providers/memory.rs` | MemoryProvider trait |
| `crates/mcb-providers/src/memory/mod.rs` | Memory providers module |
| `crates/mcb-providers/src/memory/sqlite_memory.rs` | SQLite implementation |
| `crates/mcb-application/src/use_cases/session.rs` | Session manager service |
| `crates/mcb-application/src/use_cases/memory_search.rs` | Memory search service |
| `crates/mcb-application/src/use_cases/context_injection.rs` | Context generation |
| `crates/mcb-server/src/handlers/memory_tools.rs` | MCP tool handlers |
| `crates/mcb-infrastructure/src/config/memory.rs` | Memory configuration |
| `crates/mcb-server/src/admin/web/templates/memory.html` | Admin dashboard |

## Files to Modify

| File | Change |
|------|--------|
| `crates/mcb-providers/Cargo.toml` | Add `sqlx` dependency |
| `crates/mcb-domain/src/mod.rs` | Export memory module |
| `crates/mcb-application/src/ports/providers/mod.rs` | Export MemoryProvider |
| `crates/mcb-providers/src/lib.rs` | Export memory providers |
| `crates/mcb-application/src/use_cases/mod.rs` | Export session, memory_search, context_injection |
| `crates/mcb-server/src/mcp_server.rs` | Register memory tools |
| `crates/mcb-infrastructure/src/config/mod.rs` | Export memory config |
| `crates/mcb-infrastructure/src/di/modules/mod.rs` | Wire memory services |
| `crates/mcb-server/src/admin/routes.rs` | Add memory dashboard route |

## Integration with ADR-008 (Git)

Memory observations automatically capture git context when available:

```rust
// When storing observation
let git_metadata = if let Some(git_provider) = &self.git_provider {
    let repo = git_provider.detect_repository(project_path).await?;
    repo.map(|r| GitChunkMetadata {
        repository_id: r.root_commit_hash,
        branch: current_branch,
        commit_hash: head_commit,
        ..Default::default()
    })
} else {
    None
};
```

## Success Metrics

| Metric | Before | Target v0.2.0 |
|--------|--------|---------------|
| Cross-session memory | No | Yes |
| Observation storage | No | Yes |
| Session summaries | No | Yes |
| Semantic search | Code only | Code + Memory |
| Context injection | No | Yes |
| Token efficiency | N/A | 10x via 3-layer |

## Configuration Defaults

| Setting | Default | Override |
|---------|---------|----------|
| Database | ~/.mcb/memory.db | Per-instance |
| Observation types | decision, bugfix, feature | Per-project |
| Observation limit | 20 | Per-request |
| Date range | 30 days | Per-request |
| SDK compression | Disabled | Opt-in |

## Related ADRs

-   [ADR-001: Provider Pattern Architecture](001-provider-pattern-architecture.md) - MemoryProvider follows trait-based DI
-   [ADR-002: Async-First Architecture](002-async-first-architecture.md) - Async storage operations
-   [ADR-004: Multi-Provider Strategy](004-multi-provider-strategy.md) - Memory provider routing
-   [ADR-007: Integrated Web Administration Interface](007-integrated-web-administration-interface.md) - Memory dashboard UI
-   [ADR-008: Git-Aware Semantic Indexing](008-git-aware-semantic-indexing-v0.2.0.md) - Git-tagged observations
-   [ADR-010: Hooks Subsystem](010-hooks-subsystem-agent-backed.md) - Hook observation storage
-   [ADR-012: Two-Layer DI Strategy](012-di-strategy-two-layer-approach.md) - Shaku DI for memory services
-   [ADR-013: Clean Architecture Crate Separation](013-clean-architecture-crate-separation.md) - Eight-crate organization

## References

-   [Claude-mem v8.5.2](https://github.com/thedotmack/claude-mem) - Reference implementation
-   [Shaku Documentation](https://docs.rs/shaku) - DI framework
