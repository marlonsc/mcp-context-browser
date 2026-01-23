//! Session Management
//!
//! Provides session isolation for MCB server connections.
//! Each client connection can have its own session context,
//! allowing for collection namespace prefixing and isolation.

use dashmap::DashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::Instant;

/// Session manager for tracking client connections
///
/// Maintains a map of session IDs to session contexts, enabling
/// per-connection isolation of collections and state.
#[derive(Debug, Default)]
pub struct SessionManager {
    sessions: DashMap<String, SessionContext>,
}

/// Context for a single client session
#[derive(Debug, Clone)]
pub struct SessionContext {
    /// Unique session identifier
    pub id: String,

    /// Prefix to apply to collection names for isolation
    pub collection_prefix: String,

    /// When this session was created
    pub created_at: Instant,

    /// Last access time
    pub last_access: Instant,
}

impl SessionContext {
    /// Create a new session context
    ///
    /// The collection prefix is generated from a hash of the full session ID,
    /// ensuring unique prefixes even for session IDs that share common prefixes
    /// (e.g., "claude_uuid1" vs "claude_uuid2").
    pub fn new(id: &str) -> Self {
        let now = Instant::now();

        // Hash the full session ID to create a unique prefix
        // This ensures different sessions always get different prefixes
        let mut hasher = DefaultHasher::new();
        id.hash(&mut hasher);
        let hash = hasher.finish();

        // Use lowercase hex for Milvus collection name compatibility
        // Format: s_{12-char-hex} (e.g., "s_a1b2c3d4e5f6")
        let collection_prefix = format!("s_{:012x}", hash & 0xFFFFFFFFFFFF);

        Self {
            id: id.to_string(),
            collection_prefix,
            created_at: now,
            last_access: now,
        }
    }

    /// Touch the session to update last access time
    pub fn touch(&mut self) {
        self.last_access = Instant::now();
    }
}

impl SessionManager {
    /// Create a new session manager
    pub fn new() -> Self {
        Self {
            sessions: DashMap::new(),
        }
    }

    /// Get or create a session context for the given session ID
    pub fn get_or_create(&self, session_id: &str) -> SessionContext {
        self.sessions
            .entry(session_id.to_string())
            .or_insert_with(|| SessionContext::new(session_id))
            .clone()
    }

    /// Get a session context if it exists
    pub fn get(&self, session_id: &str) -> Option<SessionContext> {
        self.sessions.get(session_id).map(|r| r.clone())
    }

    /// Remove a session
    pub fn remove(&self, session_id: &str) -> Option<SessionContext> {
        self.sessions.remove(session_id).map(|(_, v)| v)
    }

    /// Prefix a collection name with the session's collection prefix
    ///
    /// If no session ID is provided, returns the collection name unchanged.
    pub fn prefix_collection(&self, session_id: Option<&str>, collection: &str) -> String {
        match session_id {
            Some(id) => {
                let ctx = self.get_or_create(id);
                format!("{}_{}", ctx.collection_prefix, collection)
            }
            None => collection.to_string(),
        }
    }

    /// Get the number of active sessions
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    /// Clean up sessions older than the given duration
    pub fn cleanup_old_sessions(&self, max_age: std::time::Duration) {
        let now = Instant::now();
        self.sessions
            .retain(|_, ctx| now.duration_since(ctx.last_access) < max_age);
    }
}

/// Create a shared session manager
pub fn create_session_manager() -> Arc<SessionManager> {
    Arc::new(SessionManager::new())
}

// Tests moved to crates/mcb-server/tests/integration/operating_modes_integration.rs
// See: test_session_manager_* tests for coverage
