# ADR 010: Hooks Subsystem with Agent-Backed Processing

## Status

**Proposed**(Planned for v0.2.0)

> Not yet implemented. Target crate structure for v0.2.0:
>
> -   `crates/mcb-domain/src/hooks.rs` - Hook domain types
> -   `crates/mcb-application/src/ports/providers/hooks.rs` - HookProcessor port trait
> -   `crates/mcb-application/src/use_cases/hooks.rs` - HookService
> -   `crates/mcb-providers/src/hooks/` - Hook provider implementations
> -   `crates/mcb-server/src/handlers/hook_tools.rs` - MCP tool handlers
> -   `crates/mcb-infrastructure/src/config/hooks.rs` - Hooks configuration
> -   `crates/mcb-infrastructure/src/di/hooks_registry.rs` - Hook processor registry
> -   EventBus exists in `crates/mcb-infrastructure/src/events/mod.rs`
> -   Requires ADR-009 memory integration for hook observations

## Context

Claude Code provides a hooks system for extending AI assistant behavior at lifecycle events (SessionStart, UserPromptSubmit, PreToolUse, PostToolUse, Stop). Currently, hooks are shell scripts with limited intelligence.

**Current limitations:**

-   Hooks are shell scripts with hardcoded rules
-   No semantic understanding of context
-   No integration with code search or session memory
-   Each hook operates in isolation
-   Complex decisions require manual rule maintenance

**Opportunity for integration:**

MCP Context Browser v0.2.0 already provides (via ADR 008 and ADR 009):

-   **Semantic code search**- understand code context
-   **Session memory**- recall past decisions and observations
-   **Hybrid search**- combine BM25 + vector similarity
-   **Event bus**- decoupled component communication
-   **Provider pattern**- pluggable implementations
-   **Actor pattern**- async message-based processing

**User demand:**

-   Intelligent hook processing with semantic context
-   Agent-backed decisions using Claude models
-   Integration with existing code search and memory
-   Policy-based filtering with semantic fallback
-   Zero-config for basic use, full customization available

## Decision

Implement a Hooks Subsystem that**maximally reuses existing infrastructure**:

### Component Reuse Strategy

| Existing Component | Hooks Reuse |
|-------------------|-------------|
| `SystemEvent` enum | Extend with `HookExecuted`, `HookBlocked` events |
| `EventBus` | Publish hook events for monitoring/admin UI |
| `ProviderRegistry` pattern | `HookProviderRegistry` for hook processors |
| `ServiceProvider/Factory` | `HookFactory` creates processors from config |
| `HttpClientProvider` | Anthropic API calls for agent processing |
| `MemoryProvider` (ADR 009) | Store hook observations, retrieve context |
| `SearchRepository` | Semantic search for code context in decisions |
| `HybridSearchActor` pattern | `HookProcessorActor` for async hook handling |
| `Error` enum | Extend with `Hook { message: String }` variant |
| `Observation` types (ADR 009) | Hook outputs become observations |

### Architecture Overview

```
Claude Code Session
        │
[Shell Hook] ─────────────────────┐
        │                         │
        ▼                         ▼
┌───────────────────────────────────────────────────┐
│ MCP Context Browser Server                        │
│                                                   │
│  ┌─────────────────────────────────────────────┐  │
│  │ HookService (application layer)             │  │
│  │   - Orchestrates hook processing            │  │
│  │   - Reuses: ContextService pattern          │  │
│  └─────────────────────────────────────────────┘  │
│           │                    │                  │
│           ▼                    ▼                  │
│  ┌────────────────┐   ┌────────────────────────┐  │
│  │ PolicyEngine   │   │ AgentProcessor         │  │
│  │ (rule-based)   │   │ (Claude SDK)           │  │
│  │ Fast path      │   │ Via HttpClientProvider │  │
│  └────────────────┘   └────────────────────────┘  │
│           │                    │                  │
│           ▼                    ▼                  │
│  ┌─────────────────────────────────────────────┐  │
│  │ Existing Infrastructure (REUSED)            │  │
│  │   - MemoryProvider → context retrieval      │  │
│  │   - SearchRepository → code search          │  │
│  │   - EventBus → publish hook events          │  │
│  │   - ChunkRepository → store observations    │  │
│  └─────────────────────────────────────────────┘  │
└───────────────────────────────────────────────────┘
```

### Key Design Principles

1.**Extend, don't duplicate**: Add to existing enums/traits rather than creating parallel structures
2.**Reuse providers**: Use existing `MemoryProvider`, `SearchRepository` for context
3.**Follow patterns**: Match `ContextService`, `IndexingService` patterns exactly
4.**Event-driven**: Integrate with `EventBus` like `IndexingService.start_event_listener()`
5.**Actor for async**: Use `mpsc`/`oneshot` pattern from `HybridSearchActor`

## Consequences

### Positive

-   **Minimal new code**: ~1500 LOC vs ~4000 LOC if built from scratch
-   **Consistent patterns**: Same DI, error handling, event patterns
-   **Unified platform**: Hooks integrate naturally with search + memory
-   **Tested infrastructure**: Reuses battle-tested components
-   **Easy maintenance**: Single codebase, shared improvements

### Negative

-   **Coupling**: Hooks depend on ADR 009 memory infrastructure
-   **Latency**: Agent calls add 500-2000ms for complex decisions
-   **API cost**: Claude API calls for agent processing

## Alternatives Considered

### Alternative 1: Standalone hooks service

-   **Pros**: Independent, no dependencies
-   **Cons**: Duplicates 80% of existing infrastructure
-   **Rejected**: Misses integration opportunity

### Alternative 2: Simple MCP tools only (no agent)

-   **Pros**: Fast, no API cost
-   **Cons**: Limited to rule-based decisions
-   **Rejected**: Doesn't meet semantic understanding requirement

## Implementation Notes

### Phase 1: Extend Domain Error

**Modify**: `crates/mcb-domain/src/error.rs`

```rust
// Add to Error enum
#[error("Hook processing error: {message}")]
Hook { message: String },

// Add constructor
impl Error {
    pub fn hook<S: Into<String>>(message: S) -> Self {
        Self::Hook { message: message.into() }
    }
}
```

### Phase 2: Extend SystemEvent

**Modify**: `crates/mcb-infrastructure/src/events/mod.rs`

```rust
// Add to SystemEvent enum
pub enum SystemEvent {
    // ... existing variants ...

    /// Hook was executed successfully
    HookExecuted {
        event_type: String,
        session_id: String,
        result: String,
        processing_time_ms: u64,
    },
    /// Hook blocked an operation
    HookBlocked {
        event_type: String,
        session_id: String,
        reason: String,
    },
    /// Hook processing failed
    HookFailed {
        event_type: String,
        session_id: String,
        error: String,
    },
}
```

### Phase 3: Hook Domain Types

**Create**: `crates/mcb-domain/src/hooks.rs`

```rust
//! Hook domain types
//!
//! Integrates with ADR 009 memory types for observation storage.

use serde::{Deserialize, Serialize};
use crate::domain::memory::{Observation, ObservationType};

/// Hook event types matching Claude Code lifecycle
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum HookEvent {
    SessionStart,
    UserPromptSubmit,
    PreToolUse,
    PostToolUse,
    Stop,
}

impl std::fmt::Display for HookEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SessionStart => write!(f, "session_start"),
            Self::UserPromptSubmit => write!(f, "user_prompt_submit"),
            Self::PreToolUse => write!(f, "pre_tool_use"),
            Self::PostToolUse => write!(f, "post_tool_use"),
            Self::Stop => write!(f, "stop"),
        }
    }
}

/// Hook processing result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum HookResult {
    Continue,
    Block,
    Skip,
}

/// Input for hook processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookInput {
    pub event: HookEvent,
    pub session_id: String,
    pub project: String,
    pub tool_name: Option<String>,
    pub tool_input: Option<serde_json::Value>,
    pub tool_output: Option<String>,
    pub user_prompt: Option<String>,
    pub session_type: Option<SessionType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionType {
    Init,
    Resume,
    Compact,
}

/// Output from hook processing
///
/// Observations are stored via MemoryProvider (ADR 009)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookOutput {
    pub result: HookResult,
    pub message: Option<String>,
    /// Observations to store (uses ADR 009 Observation type)
    pub observations: Vec<Observation>,
    pub metadata: HookMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HookMetadata {
    pub processing_time_ms: u64,
    pub agent_used: bool,
    pub policy_matched: Option<String>,
    pub cache_hit: bool,
}

/// Policy rule for fast-path decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    pub id: String,
    pub name: String,
    pub events: Vec<HookEvent>,
    pub condition: PolicyCondition,
    pub action: PolicyAction,
    pub priority: i32,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PolicyCondition {
    ToolMatches { pattern: String },
    FileMatches { pattern: String },
    ProjectMatches { pattern: String },
    And { conditions: Vec<PolicyCondition> },
    Or { conditions: Vec<PolicyCondition> },
    Not { condition: Box<PolicyCondition> },
    Always,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PolicyAction {
    Continue,
    Block { message: String },
    InjectContext { message: String },
    DelegateToAgent { instruction: String },
    StoreObservation { obs_type: ObservationType },
}
```

### Phase 4: Hook Provider Port

**Create**: `crates/mcb-application/src/ports/providers/hooks.rs`

```rust
//! Hook provider ports
//!
//! Follows the same pattern as EmbeddingProvider and VectorStoreProvider

use async_trait::async_trait;
use crate::domain::hooks::*;
use crate::domain::error::Result;

/// Hook processor interface
///
/// Pattern: Same as EmbeddingProvider trait
#[async_trait]
pub trait HookProcessor: Send + Sync {
    /// Process a hook event
    async fn process(&self, input: &HookInput) -> Result<HookOutput>;

    /// Provider name for logging/registry
    fn provider_name(&self) -> &str;

    /// Check if processor is available
    async fn is_available(&self) -> bool;
}

/// Policy storage interface
///
/// Can be implemented with SQLite (like MemoryProvider) or in-memory
#[async_trait]
pub trait PolicyStore: Send + Sync {
    async fn list_policies(&self) -> Result<Vec<PolicyRule>>;
    async fn get_policies_for_event(&self, event: &HookEvent) -> Result<Vec<PolicyRule>>;
    async fn upsert_policy(&self, policy: &PolicyRule) -> Result<()>;
    async fn delete_policy(&self, id: &str) -> Result<()>;
    fn provider_name(&self) -> &str;
}
```

### Phase 5: Hook Provider Registry

**Create**: `crates/mcb-infrastructure/src/di/hooks_registry.rs`

```rust
//! Hook provider registry
//!
//! Pattern: Exact copy of ProviderRegistry from registry.rs

use crate::domain::error::{Error, Result};
use crate::domain::ports::hooks::HookProcessor;
use dashmap::DashMap;
use std::sync::Arc;

/// Thread-safe hook processor registry using DashMap
///
/// Pattern: Same as ProviderRegistry
#[derive(Clone)]
pub struct HookProcessorRegistry {
    processors: Arc<DashMap<String, Arc<dyn HookProcessor>>>,
}

impl HookProcessorRegistry {
    pub fn new() -> Self {
        Self {
            processors: Arc::new(DashMap::new()),
        }
    }

    pub fn register(&self, name: String, processor: Arc<dyn HookProcessor>) -> Result<()> {
        if self.processors.contains_key(&name) {
            return Err(Error::generic(format!(
                "Hook processor '{}' already registered", name
            )));
        }
        self.processors.insert(name, processor);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Result<Arc<dyn HookProcessor>> {
        self.processors
            .get(name)
            .map(|p| Arc::clone(p.value()))
            .ok_or_else(|| Error::not_found(format!("Hook processor '{}' not found", name)))
    }

    pub fn remove(&self, name: &str) -> Result<()> {
        if self.processors.remove(name).is_some() {
            Ok(())
        } else {
            Err(Error::not_found(format!("Hook processor '{}' not found", name)))
        }
    }

    pub fn list(&self) -> Vec<String> {
        self.processors.iter().map(|p| p.key().clone()).collect()
    }
}

impl Default for HookProcessorRegistry {
    fn default() -> Self {
        Self::new()
    }
}
```

### Phase 6: Policy Engine Adapter

**Create**: `crates/mcb-providers/src/hooks/policy_engine.rs`

```rust
//! Policy engine for fast-path hook decisions

use regex::Regex;
use dashmap::DashMap;
use std::sync::Arc;
use crate::domain::hooks::*;
use crate::domain::ports::hooks::PolicyStore;
use crate::domain::error::Result;

pub struct PolicyEngine {
    store: Arc<dyn PolicyStore>,
    /// Compiled regex cache (pattern from HybridSearchEngine)
    pattern_cache: DashMap<String, Regex>,
}

impl PolicyEngine {
    pub fn new(store: Arc<dyn PolicyStore>) -> Self {
        Self {
            store,
            pattern_cache: DashMap::new(),
        }
    }

    /// Evaluate policies for input, return first matching action
    pub async fn evaluate(&self, input: &HookInput) -> Result<Option<(String, PolicyAction)>> {
        let mut policies = self.store.get_policies_for_event(&input.event).await?;
        policies.sort_by(|a, b| b.priority.cmp(&a.priority));

        for policy in policies {
            if !policy.enabled {
                continue;
            }
            if self.matches_condition(&policy.condition, input) {
                return Ok(Some((policy.id, policy.action)));
            }
        }

        Ok(None)
    }

    fn matches_condition(&self, condition: &PolicyCondition, input: &HookInput) -> bool {
        match condition {
            PolicyCondition::Always => true,

            PolicyCondition::ToolMatches { pattern } => {
                input.tool_name.as_ref()
                    .map(|t| self.matches_pattern(pattern, t))
                    .unwrap_or(false)
            }

            PolicyCondition::FileMatches { pattern } => {
                input.tool_input.as_ref()
                    .and_then(|inp| inp.get("path").or(inp.get("file_path")))
                    .and_then(|v| v.as_str())
                    .map(|p| self.matches_pattern(pattern, p))
                    .unwrap_or(false)
            }

            PolicyCondition::ProjectMatches { pattern } => {
                self.matches_pattern(pattern, &input.project)
            }

            PolicyCondition::And { conditions } => {
                conditions.iter().all(|c| self.matches_condition(c, input))
            }

            PolicyCondition::Or { conditions } => {
                conditions.iter().any(|c| self.matches_condition(c, input))
            }

            PolicyCondition::Not { condition } => {
                !self.matches_condition(condition, input)
            }
        }
    }

    fn matches_pattern(&self, pattern: &str, text: &str) -> bool {
        if let Some(regex) = self.pattern_cache.get(pattern) {
            return regex.is_match(text);
        }

        if let Ok(regex) = Regex::new(pattern) {
            let matches = regex.is_match(text);
            self.pattern_cache.insert(pattern.to_string(), regex);
            matches
        } else {
            text.contains(pattern)
        }
    }
}
```

### Phase 7: Claude Agent Processor

**Create**: `crates/mcb-providers/src/hooks/claude_processor.rs`

```rust
//! Claude agent processor using HttpClientProvider

use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;
use crate::adapters::http_client::HttpClientProvider;
use crate::domain::hooks::*;
use crate::domain::ports::hooks::HookProcessor;
use crate::domain::error::{Error, Result};

pub struct ClaudeAgentProcessor {
    http_client: Arc<dyn HttpClientProvider>,
    api_key: String,
    model: String,
    base_url: String,
    timeout: Duration,
}

impl ClaudeAgentProcessor {
    pub fn new(
        http_client: Arc<dyn HttpClientProvider>,
        api_key: String,
        model: Option<String>,
    ) -> Self {
        Self {
            http_client,
            api_key,
            model: model.unwrap_or_else(|| "claude-3-haiku-20240307".to_string()),
            base_url: "https://api.anthropic.com/v1/messages".to_string(),
            timeout: Duration::from_secs(10),
        }
    }

    fn build_prompt(&self, input: &HookInput, instruction: &str) -> String {
        let mut prompt = format!(
            "Hook Event: {}\nProject: {}\n",
            input.event, input.project
        );

        if let Some(tool) = &input.tool_name {
            prompt.push_str(&format!("Tool: {}\n", tool));
        }
        if let Some(ti) = &input.tool_input {
            prompt.push_str(&format!("Input: {}\n",
                serde_json::to_string_pretty(ti).unwrap_or_default()));
        }
        if let Some(to) = &input.tool_output {
            let truncated = if to.len() > 1000 { &to[..1000] } else { to };
            prompt.push_str(&format!("Output: {}\n", truncated));
        }

        prompt.push_str(&format!("\nInstruction: {}\n", instruction));
        prompt.push_str("\nRespond with JSON: {\"result\": \"continue\"|\"block\", \"message\": \"...\"}");

        prompt
    }

    async fn call_api(&self, prompt: &str) -> Result<serde_json::Value> {
        let client = self.http_client.client_with_timeout(self.timeout)
            .map_err(|e| Error::hook(format!("Failed to create client: {}", e)))?;

        let response = client
            .post(&self.base_url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&serde_json::json!({
                "model": self.model,
                "max_tokens": 1024,
                "messages": [{
                    "role": "user",
                    "content": prompt
                }]
            }))
            .send()
            .await
            .map_err(|e| Error::hook(format!("API request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::hook(format!("API error: {}", response.status())));
        }

        response.json().await
            .map_err(|e| Error::hook(format!("Failed to parse response: {}", e)))
    }
}

#[async_trait]
impl HookProcessor for ClaudeAgentProcessor {
    async fn process(&self, input: &HookInput) -> Result<HookOutput> {
        let instruction = match &input.event {
            HookEvent::PreToolUse => "Analyze this tool operation. Should it proceed?",
            HookEvent::PostToolUse => "Extract key observations from this tool output.",
            HookEvent::SessionStart => "Provide helpful context for this session.",
            HookEvent::UserPromptSubmit => "Analyze this user request.",
            HookEvent::Stop => "Summarize this session's work.",
        };

        let prompt = self.build_prompt(input, instruction);
        let response = self.call_api(&prompt).await?;

        // Parse response
        let content = response["content"][0]["text"]
            .as_str()
            .ok_or_else(|| Error::hook("Empty response"))?;

        let parsed: serde_json::Value = serde_json::from_str(content)
            .unwrap_or_else(|_| serde_json::json!({"result": "continue"}));

        let result = match parsed["result"].as_str() {
            Some("block") => HookResult::Block,
            Some("skip") => HookResult::Skip,
            _ => HookResult::Continue,
        };

        Ok(HookOutput {
            result,
            message: parsed["message"].as_str().map(String::from),
            observations: vec![],
            metadata: HookMetadata {
                agent_used: true,
                ..Default::default()
            },
        })
    }

    fn provider_name(&self) -> &str {
        "claude_agent"
    }

    async fn is_available(&self) -> bool {
        !self.api_key.is_empty()
    }
}
```

### Phase 8: Hook Service

**Create**: `crates/mcb-application/src/use_cases/hooks.rs`

```rust
//! Hook service
//!
//! Pattern: Same as ContextService and IndexingService

use std::sync::Arc;
use std::time::Instant;
use crate::domain::hooks::*;
use crate::domain::ports::hooks::{HookProcessor, PolicyStore};
use crate::domain::ports::memory::MemoryProvider;
use crate::domain::ports::SearchRepository;
use crate::adapters::hooks::policy_engine::PolicyEngine;
use crate::infrastructure::events::{SharedEventBus, SystemEvent};
use crate::domain::error::Result;
use dashmap::DashMap;

/// Hook service following ContextService pattern
pub struct HookService {
    policy_engine: Arc<PolicyEngine>,
    agent_processor: Option<Arc<dyn HookProcessor>>,
    /// Reuse ADR 009 memory provider for context
    memory_provider: Option<Arc<dyn MemoryProvider>>,
    /// Reuse existing search repository for code context
    search_repository: Option<Arc<dyn SearchRepository>>,
    /// Result cache
    cache: DashMap<u64, HookOutput>,
    config: HookServiceConfig,
}

#[derive(Debug, Clone)]
pub struct HookServiceConfig {
    pub agent_enabled: bool,
    pub cache_ttl_seconds: u64,
    pub max_context_tokens: usize,
}

impl Default for HookServiceConfig {
    fn default() -> Self {
        Self {
            agent_enabled: true,
            cache_ttl_seconds: 300,
            max_context_tokens: 2000,
        }
    }
}

impl HookService {
    /// Create new hook service with dependencies
    ///
    /// Pattern: Same constructor injection as ContextService
    pub fn new(
        policy_store: Arc<dyn PolicyStore>,
        agent_processor: Option<Arc<dyn HookProcessor>>,
        memory_provider: Option<Arc<dyn MemoryProvider>>,
        search_repository: Option<Arc<dyn SearchRepository>>,
        config: HookServiceConfig,
    ) -> Self {
        Self {
            policy_engine: Arc::new(PolicyEngine::new(policy_store)),
            agent_processor,
            memory_provider,
            search_repository,
            cache: DashMap::new(),
            config,
        }
    }

    /// Start event listener (pattern from IndexingService)
    pub fn start_event_listener(&self, event_bus: SharedEventBus) {
        let mut receiver = event_bus.subscribe();

        tokio::spawn(async move {
            while let Ok(event) = receiver.recv().await {
                // React to relevant system events
                if let SystemEvent::ConfigReloaded = event {
                    tracing::info!("[HOOKS] Config reloaded, clearing cache");
                    // Could reload policies here
                }
            }
        });
    }

    /// Process a hook event
    pub async fn process(&self, input: HookInput, event_bus: Option<&SharedEventBus>) -> Result<HookOutput> {
        let start = Instant::now();
        let mut metadata = HookMetadata::default();

        // Check cache first
        let cache_key = self.compute_cache_key(&input);
        if let Some(cached) = self.cache.get(&cache_key) {
            let mut output = cached.clone();
            output.metadata.cache_hit = true;
            return Ok(output);
        }

        // Try policy engine first (fast path)
        let output = if let Ok(Some((policy_id, action))) = self.policy_engine.evaluate(&input).await {
            metadata.policy_matched = Some(policy_id);
            self.execute_policy_action(&input, action, &mut metadata).await?
        } else if self.config.agent_enabled {
            // Delegate to agent (slow path)
            if let Some(agent) = &self.agent_processor {
                metadata.agent_used = true;
                agent.process(&input).await?
            } else {
                // No agent, default continue
                HookOutput {
                    result: HookResult::Continue,
                    message: None,
                    observations: vec![],
                    metadata: metadata.clone(),
                }
            }
        } else {
            // No policy, no agent - default continue
            HookOutput {
                result: HookResult::Continue,
                message: None,
                observations: vec![],
                metadata: metadata.clone(),
            }
        };

        metadata.processing_time_ms = start.elapsed().as_millis() as u64;
        let mut final_output = output;
        final_output.metadata = metadata.clone();

        // Cache result
        self.cache.insert(cache_key, final_output.clone());

        // Publish event
        if let Some(bus) = event_bus {
            let event = match &final_output.result {
                HookResult::Block => SystemEvent::HookBlocked {
                    event_type: input.event.to_string(),
                    session_id: input.session_id.clone(),
                    reason: final_output.message.clone().unwrap_or_default(),
                },
                _ => SystemEvent::HookExecuted {
                    event_type: input.event.to_string(),
                    session_id: input.session_id.clone(),
                    result: format!("{:?}", final_output.result),
                    processing_time_ms: metadata.processing_time_ms,
                },
            };
            let _ = bus.publish(event);
        }

        // Store observations via memory provider (ADR 009 integration)
        if let Some(memory) = &self.memory_provider {
            for obs in &final_output.observations {
                if let Err(e) = memory.store_observation(obs).await {
                    tracing::warn!("[HOOKS] Failed to store observation: {}", e);
                }
            }
        }

        Ok(final_output)
    }

    async fn execute_policy_action(
        &self,
        input: &HookInput,
        action: PolicyAction,
        metadata: &mut HookMetadata,
    ) -> Result<HookOutput> {
        match action {
            PolicyAction::Continue => Ok(HookOutput {
                result: HookResult::Continue,
                message: None,
                observations: vec![],
                metadata: metadata.clone(),
            }),

            PolicyAction::Block { message } => Ok(HookOutput {
                result: HookResult::Block,
                message: Some(message),
                observations: vec![],
                metadata: metadata.clone(),
            }),

            PolicyAction::InjectContext { message } => {
                // Optionally enhance with memory context
                let enhanced = if let Some(memory) = &self.memory_provider {
                    self.enhance_with_memory(&input.project, &message, memory).await
                } else {
                    message
                };

                Ok(HookOutput {
                    result: HookResult::Continue,
                    message: Some(enhanced),
                    observations: vec![],
                    metadata: metadata.clone(),
                })
            }

            PolicyAction::DelegateToAgent { instruction } => {
                if let Some(agent) = &self.agent_processor {
                    metadata.agent_used = true;
                    // Could pass instruction to agent here
                    agent.process(input).await
                } else {
                    Ok(HookOutput {
                        result: HookResult::Continue,
                        message: None,
                        observations: vec![],
                        metadata: metadata.clone(),
                    })
                }
            }

            PolicyAction::StoreObservation { obs_type } => {
                let obs = self.create_observation(input, obs_type);
                Ok(HookOutput {
                    result: HookResult::Continue,
                    message: None,
                    observations: vec![obs],
                    metadata: metadata.clone(),
                })
            }
        }
    }

    async fn enhance_with_memory(
        &self,
        project: &str,
        base_message: &str,
        memory: &Arc<dyn MemoryProvider>,
    ) -> String {
        // Use memory provider to get recent context
        let query = crate::domain::ports::memory::SearchQuery {
            project: Some(project.to_string()),
            limit: 5,
            ..Default::default()
        };

        if let Ok(results) = memory.search_observations(&query).await {
            if !results.is_empty() {
                let context: String = results.iter()
                    .map(|r| format!("- {}", r.title))
                    .collect::<Vec<_>>()
                    .join("\n");
                return format!("{}\n\nRecent context:\n{}", base_message, context);
            }
        }

        base_message.to_string()
    }

    fn create_observation(
        &self,
        input: &HookInput,
        obs_type: crate::domain::memory::ObservationType,
    ) -> crate::domain::memory::Observation {
        crate::domain::memory::Observation {
            id: 0,
            session_id: input.session_id.clone(),
            project: input.project.clone(),
            obs_type,
            title: format!("{} hook executed", input.event),
            subtitle: input.tool_name.clone(),
            facts: vec![],
            narrative: None,
            concepts: vec![],
            files_read: vec![],
            files_modified: vec![],
            prompt_number: None,
            discovery_tokens: 0,
            git_metadata: None,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    fn compute_cache_key(&self, input: &HookInput) -> u64 {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;

        let mut hasher = DefaultHasher::new();
        input.event.to_string().hash(&mut hasher);
        input.session_id.hash(&mut hasher);
        input.tool_name.hash(&mut hasher);
        hasher.finish()
    }
}
```

### Phase 9: MCP Tool Handlers

**Create**: `crates/mcb-server/src/handlers/hook_tools.rs`

```rust
//! MCP tool handlers for hooks
//!
//! Pattern: Same as IndexCodebaseHandler, SearchCodeHandler

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::application::hooks::HookService;
use crate::domain::hooks::*;
use crate::infrastructure::events::SharedEventBus;
use crate::domain::error::Result;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct HookProcessArgs {
    pub event: String,
    pub session_id: String,
    pub project: String,
    pub tool_name: Option<String>,
    pub tool_input: Option<serde_json::Value>,
    pub tool_output: Option<String>,
    pub user_prompt: Option<String>,
    pub session_type: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct HookDecideArgs {
    pub event: String,
    pub project: String,
    pub tool_name: Option<String>,
    pub file_path: Option<String>,
}

pub struct HookProcessHandler {
    hook_service: Arc<HookService>,
    event_bus: SharedEventBus,
}

impl HookProcessHandler {
    pub fn new(hook_service: Arc<HookService>, event_bus: SharedEventBus) -> Self {
        Self { hook_service, event_bus }
    }

    pub async fn handle(&self, args: HookProcessArgs) -> Result<serde_json::Value> {
        let input = HookInput {
            event: parse_event(&args.event)?,
            session_id: args.session_id,
            project: args.project,
            tool_name: args.tool_name,
            tool_input: args.tool_input,
            tool_output: args.tool_output,
            user_prompt: args.user_prompt,
            session_type: args.session_type.and_then(|s| parse_session_type(&s)),
        };

        let output = self.hook_service.process(input, Some(&self.event_bus)).await?;

        Ok(serde_json::json!({
            "result": match output.result {
                HookResult::Continue => "continue",
                HookResult::Block => "block",
                HookResult::Skip => "skip",
            },
            "message": output.message,
            "metadata": {
                "processing_time_ms": output.metadata.processing_time_ms,
                "agent_used": output.metadata.agent_used,
                "policy_matched": output.metadata.policy_matched,
                "cache_hit": output.metadata.cache_hit,
            }
        }))
    }
}

fn parse_event(s: &str) -> Result<HookEvent> {
    match s.to_lowercase().replace('_', "").as_str() {
        "sessionstart" => Ok(HookEvent::SessionStart),
        "userpromptsubmit" => Ok(HookEvent::UserPromptSubmit),
        "pretooluse" => Ok(HookEvent::PreToolUse),
        "posttooluse" => Ok(HookEvent::PostToolUse),
        "stop" => Ok(HookEvent::Stop),
        _ => Err(crate::domain::error::Error::invalid_argument(format!("Unknown event: {}", s))),
    }
}

fn parse_session_type(s: &str) -> Option<SessionType> {
    match s.to_lowercase().as_str() {
        "init" => Some(SessionType::Init),
        "resume" => Some(SessionType::Resume),
        "compact" => Some(SessionType::Compact),
        _ => None,
    }
}
```

### Phase 10: Configuration

**Create**: `crates/mcb-infrastructure/src/config/hooks.rs`

```rust
//! Hooks configuration
//!
//! Pattern: Same as other config modules

use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate, Default)]
pub struct HooksConfig {
    /// Enable hooks subsystem
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Enable agent-backed processing
    #[serde(default = "default_true")]
    pub agent_enabled: bool,

    /// Agent model
    #[serde(default = "default_model")]
    pub agent_model: String,

    /// Agent timeout in milliseconds
    #[serde(default = "default_timeout")]
    pub agent_timeout_ms: u64,

    /// Cache TTL in seconds
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl_seconds: u64,

    /// Path to policies file
    #[serde(default)]
    pub policies_path: Option<String>,
}

fn default_true() -> bool { true }
fn default_model() -> String { "claude-3-haiku-20240307".to_string() }
fn default_timeout() -> u64 { 10000 }
fn default_cache_ttl() -> u64 { 300 }
```

## Files Summary

### New Files (minimal)

| File | LOC | Purpose |
|------|-----|---------|
| `crates/mcb-domain/src/hooks.rs` | ~120 | Hook domain types |
| `crates/mcb-application/src/ports/providers/hooks.rs` | ~30 | HookProcessor trait |
| `crates/mcb-infrastructure/src/di/hooks_registry.rs` | ~50 | Registry (pattern copy) |
| `crates/mcb-providers/src/hooks/policy_engine.rs` | ~80 | Policy evaluation |
| `crates/mcb-providers/src/hooks/claude_processor.rs` | ~100 | Claude API via HttpClient |
| `crates/mcb-application/src/use_cases/hooks.rs` | ~180 | HookService (pattern copy) |
| `crates/mcb-server/src/handlers/hook_tools.rs` | ~80 | MCP handlers |
| `crates/mcb-infrastructure/src/config/hooks.rs` | ~40 | Configuration |

**Total**: ~680 LOC (vs ~4000 if built from scratch)

### Modified Files

| File | Change |
|------|--------|
| `crates/mcb-domain/src/error.rs` | Add `Hook` variant |
| `crates/mcb-infrastructure/src/events/mod.rs` | Add 3 hook events |
| `crates/mcb-domain/src/mod.rs` | Export hooks module |
| `crates/mcb-application/src/ports/providers/mod.rs` | Export HookProcessor |
| `crates/mcb-providers/src/lib.rs` | Export hooks providers |
| `crates/mcb-application/src/use_cases/mod.rs` | Export HookService |
| `crates/mcb-server/src/mcp_server.rs` | Register hook tools |
| `crates/mcb-infrastructure/src/config/mod.rs` | Export HooksConfig |

## Integration Points

### With ADR 008 (Git)

Hook inputs can include git context when available:

```rust
// In HookService, if git provider exists
if let Some(git) = &self.git_provider {
    input.git_context = git.current_branch(path).await.ok();
}
```

### With ADR 009 (Memory)

1.**Context retrieval**: Use `MemoryProvider.search_observations()` for context injection
2.**Observation storage**: Store hook observations via `MemoryProvider.store_observation()`
3.**Shared types**: Reuse `Observation`, `ObservationType` from memory domain

## Related ADRs

-   [ADR-001: Modular Crates Architecture](001-modular-crates-architecture.md) - HookProcessor follows trait-based DI
-   [ADR-002: Async-First Architecture](002-async-first-architecture.md) - Async hook processing
-   [ADR-007: Integrated Web Administration Interface](007-integrated-web-administration-interface.md) - Hook monitoring UI
-   [ADR-008: Git-Aware Semantic Indexing](008-git-aware-semantic-indexing-v0.2.0.md) - Git context in hooks
-   [ADR-009: Persistent Session Memory](009-persistent-session-memory-v0.2.0.md) - Hook observation storage
-   [ADR-012: Two-Layer DI Strategy](012-di-strategy-two-layer-approach.md) - Shaku DI for hook services
-   [ADR-013: Clean Architecture Crate Separation](013-clean-architecture-crate-separation.md) - Eight-crate organization

## References

-   [Claude Code Hooks Documentation](https://docs.anthropic.com/claude-code/hooks)
-   [Shaku Documentation](https://docs.rs/shaku) - DI framework (historical; see ADR-029)
-   Existing patterns: `crates/mcb-infrastructure/src/events/mod.rs`, `crates/mcb-infrastructure/src/di/registry.rs`, `crates/mcb-application/src/use_cases/context.rs`
