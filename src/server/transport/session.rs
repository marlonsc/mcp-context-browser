//! Session Manager for HTTP Transport
//!
//! Manages client sessions with support for resumption via Last-Event-ID.

use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

use super::config::SessionConfig;

/// Session state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionState {
    /// Session is being initialized
    Initializing,
    /// Session is active and accepting requests
    Active,
    /// Client disconnected, session can be resumed
    Suspended,
    /// Session is terminated
    Terminated,
}

/// Buffered message for session resumption
#[derive(Debug, Clone)]
pub struct BufferedMessage {
    /// Unique event ID for SSE resumption
    pub event_id: String,
    /// The message content
    pub message: serde_json::Value,
    /// When the message was created
    pub timestamp: Instant,
}

/// Client session with state for reconnection support
#[derive(Debug, Clone)]
pub struct McpSession {
    /// Unique session identifier
    pub id: String,
    /// Server version at session creation
    pub server_version: String,
    /// Session creation time
    pub created_at: Instant,
    /// Last activity timestamp
    pub last_activity: Instant,
    /// Messages buffered for resumption
    pub message_buffer: Vec<BufferedMessage>,
    /// Last event ID sent to client
    pub last_event_id: Option<String>,
    /// Client info from initialization
    pub client_info: Option<serde_json::Value>,
    /// Session state
    pub state: SessionState,
}

impl McpSession {
    /// Create a new session
    fn new(server_version: &str) -> Self {
        let id = generate_session_id();
        Self {
            id,
            server_version: server_version.to_string(),
            created_at: Instant::now(),
            last_activity: Instant::now(),
            message_buffer: Vec::new(),
            last_event_id: None,
            client_info: None,
            state: SessionState::Initializing,
        }
    }

    /// Mark session as active
    fn activate(&mut self) {
        self.state = SessionState::Active;
        self.last_activity = Instant::now();
    }

    /// Check if session is expired
    fn is_expired(&self, ttl: Duration) -> bool {
        self.last_activity.elapsed() > ttl
    }
}

/// Thread-safe session manager
pub struct SessionManager {
    sessions: DashMap<String, McpSession>,
    config: SessionConfig,
    server_version: String,
    cleanup_running: Arc<RwLock<bool>>,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new(config: SessionConfig, server_version: String) -> Self {
        Self {
            sessions: DashMap::new(),
            config,
            server_version,
            cleanup_running: Arc::new(RwLock::new(false)),
        }
    }

    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(
            SessionConfig::default(),
            env!("CARGO_PKG_VERSION").to_string(),
        )
    }

    /// Create a new session
    pub fn create_session(&self) -> Result<McpSession, SessionError> {
        // Check max sessions limit
        if self.sessions.len() >= self.config.max_sessions() {
            return Err(SessionError::MaxSessionsReached);
        }

        let session = McpSession::new(&self.server_version);
        let id = session.id.clone();
        self.sessions.insert(id, session.clone());

        debug!("Created session: {}", session.id);
        Ok(session)
    }

    /// Get a session by ID
    pub fn get_session(&self, session_id: &str) -> Option<McpSession> {
        self.sessions.get(session_id).map(|s| s.clone())
    }

    /// Activate a session (after initialization completes)
    pub fn activate_session(&self, session_id: &str) -> Result<(), SessionError> {
        if let Some(mut session) = self.sessions.get_mut(session_id) {
            session.activate();
            Ok(())
        } else {
            Err(SessionError::SessionNotFound)
        }
    }

    /// Update session activity (touch)
    pub fn touch_session(&self, session_id: &str) -> bool {
        if let Some(mut session) = self.sessions.get_mut(session_id) {
            session.last_activity = Instant::now();
            true
        } else {
            false
        }
    }

    /// Set client info for a session
    pub fn set_client_info(&self, session_id: &str, info: serde_json::Value) -> bool {
        if let Some(mut session) = self.sessions.get_mut(session_id) {
            session.client_info = Some(info);
            true
        } else {
            false
        }
    }

    /// Buffer a message for resumption
    pub fn buffer_message(&self, session_id: &str, message: serde_json::Value) -> Option<String> {
        if !self.config.resumption_enabled {
            return None;
        }

        if let Some(mut session) = self.sessions.get_mut(session_id) {
            let event_id = Uuid::new_v4().to_string();

            // Trim buffer if at capacity
            while session.message_buffer.len() >= self.config.resumption_buffer_size {
                session.message_buffer.remove(0);
            }

            session.message_buffer.push(BufferedMessage {
                event_id: event_id.clone(),
                message,
                timestamp: Instant::now(),
            });
            session.last_event_id = Some(event_id.clone());

            Some(event_id)
        } else {
            None
        }
    }

    /// Get buffered messages after a given event ID (for resumption)
    pub fn get_messages_after(
        &self,
        session_id: &str,
        last_event_id: &str,
    ) -> Vec<BufferedMessage> {
        if let Some(session) = self.sessions.get(session_id) {
            let mut found = false;
            session
                .message_buffer
                .iter()
                .filter(|m| {
                    if found {
                        return true;
                    }
                    if m.event_id == last_event_id {
                        found = true;
                    }
                    false
                })
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Suspend a session (client disconnected but can resume)
    pub fn suspend_session(&self, session_id: &str) -> bool {
        if let Some(mut session) = self.sessions.get_mut(session_id) {
            session.state = SessionState::Suspended;
            true
        } else {
            false
        }
    }

    /// Terminate a session
    pub fn terminate_session(&self, session_id: &str) -> bool {
        self.sessions.remove(session_id).is_some()
    }

    /// Get active session count
    pub fn active_session_count(&self) -> usize {
        self.sessions
            .iter()
            .filter(|s| s.state == SessionState::Active)
            .count()
    }

    /// Get total session count
    pub fn total_session_count(&self) -> usize {
        self.sessions.len()
    }

    /// Start background cleanup task
    pub async fn start_cleanup_task(&self) {
        let mut running = self.cleanup_running.write().await;
        if *running {
            warn!("Session cleanup task already running");
            return;
        }
        *running = true;
        drop(running);

        let sessions = self.sessions.clone();
        let ttl = Duration::from_secs(self.config.ttl_secs);
        let cleanup_running = Arc::clone(&self.cleanup_running);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;

                let running = cleanup_running.read().await;
                if !*running {
                    break;
                }
                drop(running);

                // Remove expired sessions
                let before = sessions.len();
                sessions.retain(|_, session| !session.is_expired(ttl));
                let removed = before - sessions.len();

                if removed > 0 {
                    info!("Cleaned up {} expired sessions", removed);
                }
            }
        });

        info!("Session cleanup task started");
    }

    /// Stop the cleanup task
    pub async fn stop_cleanup_task(&self) {
        let mut running = self.cleanup_running.write().await;
        *running = false;
        info!("Session cleanup task stopped");
    }
}

/// Generate a secure session ID
fn generate_session_id() -> String {
    format!("mcp_{}", Uuid::new_v4().to_string().replace('-', ""))
}

/// Session manager errors
#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("Maximum concurrent sessions reached")]
    MaxSessionsReached,
    #[error("Session not found or expired")]
    SessionNotFound,
    #[error("Session already terminated")]
    SessionTerminated,
    #[error("Invalid session state for operation")]
    InvalidState,
}

// Extension trait to get max_sessions from config
impl SessionConfig {
    fn max_sessions(&self) -> usize {
        // Default from transport config
        1000
    }
}

/// Shared session manager
pub type SharedSessionManager = Arc<SessionManager>;

/// Create a shared session manager
pub fn create_shared_session_manager(
    config: SessionConfig,
    server_version: String,
) -> SharedSessionManager {
    Arc::new(SessionManager::new(config, server_version))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let manager = SessionManager::with_defaults();
        let session = manager.create_session().unwrap();

        assert!(session.id.starts_with("mcp_"));
        assert_eq!(session.state, SessionState::Initializing);
    }

    #[test]
    fn test_session_activation() {
        let manager = SessionManager::with_defaults();
        let session = manager.create_session().unwrap();

        manager.activate_session(&session.id).unwrap();

        let updated = manager.get_session(&session.id).unwrap();
        assert_eq!(updated.state, SessionState::Active);
    }

    #[test]
    fn test_session_touch() {
        let manager = SessionManager::with_defaults();
        let session = manager.create_session().unwrap();

        std::thread::sleep(std::time::Duration::from_millis(10));

        assert!(manager.touch_session(&session.id));

        let updated = manager.get_session(&session.id).unwrap();
        assert!(updated.last_activity > session.last_activity);
    }

    #[test]
    fn test_message_buffering() {
        let manager = SessionManager::with_defaults();
        let session = manager.create_session().unwrap();

        let msg = serde_json::json!({"test": "message"});
        let event_id = manager.buffer_message(&session.id, msg.clone());

        assert!(event_id.is_some());

        let updated = manager.get_session(&session.id).unwrap();
        assert_eq!(updated.message_buffer.len(), 1);
        assert_eq!(updated.message_buffer[0].message, msg);
    }

    #[test]
    fn test_session_termination() {
        let manager = SessionManager::with_defaults();
        let session = manager.create_session().unwrap();

        assert!(manager.terminate_session(&session.id));
        assert!(manager.get_session(&session.id).is_none());
    }
}
