//! In-Memory Ring Buffer Logging (Actor Pattern)
//!
//! Provides a tracing Layer that captures logs into a circular buffer
//! for real-time access via the admin dashboard.
//!
//! Uses the Actor pattern with mpsc channels - NO LOCKS.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::layer::Context;
use tracing_subscriber::Layer;

/// A single log entry stored in the ring buffer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Timestamp when the log was created
    pub timestamp: DateTime<Utc>,
    /// Log level (ERROR, WARN, INFO, DEBUG, TRACE)
    pub level: String,
    /// Target module/component
    pub target: String,
    /// Log message
    pub message: String,
    /// Optional structured fields
    pub fields: Option<String>,
}

impl LogEntry {
    /// Create a new log entry with current timestamp
    pub fn new(level: Level, target: &str, message: String, fields: Option<String>) -> Self {
        Self {
            timestamp: Utc::now(),
            level: level.to_string(),
            target: target.to_string(),
            message,
            fields,
        }
    }
}

/// Messages for the log buffer actor
pub enum LogBufferMessage {
    /// Push a new log entry
    Push(LogEntry),
    /// Get all entries
    GetAll(oneshot::Sender<Vec<LogEntry>>),
    /// Get the most recent N entries
    GetRecent(usize, oneshot::Sender<Vec<LogEntry>>),
    /// Get entries filtered by level
    GetByLevel(String, oneshot::Sender<Vec<LogEntry>>),
    /// Clear all entries
    Clear,
    /// Get current count
    GetCount(oneshot::Sender<usize>),
}

/// Handle for the log buffer actor
#[derive(Clone)]
pub struct LogBuffer {
    sender: mpsc::Sender<LogBufferMessage>,
}

impl Default for LogBuffer {
    fn default() -> Self {
        Self::new(1000)
    }
}

impl LogBuffer {
    /// Create a new log buffer handle and start the actor
    pub fn new(capacity: usize) -> Self {
        let (tx, rx) = mpsc::channel(1000);
        let mut actor = LogBufferActor::new(rx, capacity);
        tokio::spawn(async move {
            actor.run().await;
        });
        Self { sender: tx }
    }

    /// Push a new entry
    pub fn push(&self, entry: LogEntry) {
        let _ = self.sender.try_send(LogBufferMessage::Push(entry));
    }

    /// Get all entries
    pub async fn get_all(&self) -> Vec<LogEntry> {
        let (tx, rx) = oneshot::channel();
        let _ = self.sender.send(LogBufferMessage::GetAll(tx)).await;
        rx.await.unwrap_or_default()
    }

    /// Get the most recent N entries
    pub async fn get_recent(&self, count: usize) -> Vec<LogEntry> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .sender
            .send(LogBufferMessage::GetRecent(count, tx))
            .await;
        rx.await.unwrap_or_default()
    }

    /// Get entries filtered by level
    pub async fn get_by_level(&self, level: &str) -> Vec<LogEntry> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .sender
            .send(LogBufferMessage::GetByLevel(level.to_string(), tx))
            .await;
        rx.await.unwrap_or_default()
    }

    /// Clear all entries
    pub fn clear(&self) {
        let _ = self.sender.try_send(LogBufferMessage::Clear);
    }

    /// Get current count
    pub async fn len(&self) -> usize {
        let (tx, rx) = oneshot::channel();
        let _ = self.sender.send(LogBufferMessage::GetCount(tx)).await;
        rx.await.unwrap_or(0)
    }

    /// Check if empty
    pub async fn is_empty(&self) -> bool {
        self.len().await == 0
    }
}

/// Log buffer actor that processes messages
struct LogBufferActor {
    receiver: mpsc::Receiver<LogBufferMessage>,
    entries: VecDeque<LogEntry>,
    capacity: usize,
}

impl LogBufferActor {
    fn new(receiver: mpsc::Receiver<LogBufferMessage>, capacity: usize) -> Self {
        Self {
            receiver,
            entries: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    async fn run(&mut self) {
        while let Some(msg) = self.receiver.recv().await {
            match msg {
                LogBufferMessage::Push(entry) => {
                    if self.entries.len() >= self.capacity {
                        self.entries.pop_front();
                    }
                    self.entries.push_back(entry);
                }
                LogBufferMessage::GetAll(tx) => {
                    let entries: Vec<LogEntry> = self.entries.iter().cloned().collect();
                    let _ = tx.send(entries);
                }
                LogBufferMessage::GetRecent(count, tx) => {
                    let entries: Vec<LogEntry> = self
                        .entries
                        .iter()
                        .rev()
                        .take(count)
                        .cloned()
                        .collect::<Vec<_>>()
                        .into_iter()
                        .rev()
                        .collect();
                    let _ = tx.send(entries);
                }
                LogBufferMessage::GetByLevel(level, tx) => {
                    let entries: Vec<LogEntry> = self
                        .entries
                        .iter()
                        .filter(|e| e.level == level)
                        .cloned()
                        .collect();
                    let _ = tx.send(entries);
                }
                LogBufferMessage::Clear => {
                    self.entries.clear();
                }
                LogBufferMessage::GetCount(tx) => {
                    let _ = tx.send(self.entries.len());
                }
            }
        }
    }
}

/// Thread-safe wrapper for the log buffer (Arc for sharing)
pub type SharedLogBuffer = Arc<LogBuffer>;

/// Create a shared log buffer with specified capacity
pub fn create_shared_log_buffer(capacity: usize) -> SharedLogBuffer {
    Arc::new(LogBuffer::new(capacity))
}

/// Tracing Layer that captures logs into the ring buffer
pub struct RingBufferLayer {
    buffer: SharedLogBuffer,
    min_level: Level,
}

impl RingBufferLayer {
    /// Create a new RingBufferLayer with specified buffer and minimum level
    pub fn new(buffer: SharedLogBuffer, min_level: Level) -> Self {
        Self { buffer, min_level }
    }

    /// Create with default INFO level
    pub fn with_info_level(buffer: SharedLogBuffer) -> Self {
        Self::new(buffer, Level::INFO)
    }

    /// Get the underlying buffer for reading logs
    pub fn buffer(&self) -> &SharedLogBuffer {
        &self.buffer
    }
}

impl<S> Layer<S> for RingBufferLayer
where
    S: Subscriber,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let metadata = event.metadata();
        let level = *metadata.level();

        // Filter by minimum level
        if level > self.min_level {
            return;
        }

        // Extract message from event
        let mut message = String::new();
        let mut fields = Vec::new();

        event.record(&mut MessageVisitor {
            message: &mut message,
            fields: &mut fields,
        });

        let fields_str = if fields.is_empty() {
            None
        } else {
            Some(fields.join(", "))
        };

        let entry = LogEntry::new(level, metadata.target(), message, fields_str);

        // Send to actor (non-blocking)
        self.buffer.push(entry);
    }
}

/// Visitor to extract message and fields from tracing events
struct MessageVisitor<'a> {
    message: &'a mut String,
    fields: &'a mut Vec<String>,
}

impl tracing::field::Visit for MessageVisitor<'_> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            *self.message = format!("{:?}", value);
        } else {
            self.fields.push(format!("{}={:?}", field.name(), value));
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            *self.message = value.to_string();
        } else {
            self.fields.push(format!("{}={}", field.name(), value));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_ring_buffer_push_and_rotation() {
        let buffer = LogBuffer::new(3);

        buffer.push(LogEntry::new(Level::INFO, "test", "msg1".to_string(), None));
        buffer.push(LogEntry::new(Level::INFO, "test", "msg2".to_string(), None));
        buffer.push(LogEntry::new(Level::INFO, "test", "msg3".to_string(), None));

        // Give actor time to process
        tokio::time::sleep(Duration::from_millis(50)).await;

        assert_eq!(buffer.len().await, 3);

        // This should push out msg1
        buffer.push(LogEntry::new(Level::INFO, "test", "msg4".to_string(), None));
        tokio::time::sleep(Duration::from_millis(50)).await;

        assert_eq!(buffer.len().await, 3);
        let entries = buffer.get_all().await;
        assert_eq!(entries[0].message, "msg2");
        assert_eq!(entries[2].message, "msg4");
    }

    #[tokio::test]
    async fn test_get_recent() {
        let buffer = LogBuffer::new(10);
        for i in 1..=5 {
            buffer.push(LogEntry::new(
                Level::INFO,
                "test",
                format!("msg{}", i),
                None,
            ));
        }

        tokio::time::sleep(Duration::from_millis(50)).await;

        let recent = buffer.get_recent(2).await;
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].message, "msg4");
        assert_eq!(recent[1].message, "msg5");
    }

    #[tokio::test]
    async fn test_get_by_level() {
        let buffer = LogBuffer::new(10);
        buffer.push(LogEntry::new(
            Level::INFO,
            "test",
            "info1".to_string(),
            None,
        ));
        buffer.push(LogEntry::new(
            Level::ERROR,
            "test",
            "error1".to_string(),
            None,
        ));
        buffer.push(LogEntry::new(
            Level::INFO,
            "test",
            "info2".to_string(),
            None,
        ));

        tokio::time::sleep(Duration::from_millis(50)).await;

        let errors = buffer.get_by_level("ERROR").await;
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message, "error1");
    }

    #[tokio::test]
    async fn test_shared_buffer() {
        let buffer = create_shared_log_buffer(100);

        buffer.push(LogEntry::new(
            Level::INFO,
            "test",
            "test message".to_string(),
            None,
        ));

        tokio::time::sleep(Duration::from_millis(50)).await;

        assert_eq!(buffer.len().await, 1);
    }

    #[tokio::test]
    async fn test_clear() {
        let buffer = LogBuffer::new(10);
        buffer.push(LogEntry::new(Level::INFO, "test", "msg1".to_string(), None));
        buffer.push(LogEntry::new(Level::INFO, "test", "msg2".to_string(), None));

        tokio::time::sleep(Duration::from_millis(50)).await;
        assert_eq!(buffer.len().await, 2);

        buffer.clear();
        tokio::time::sleep(Duration::from_millis(50)).await;

        assert_eq!(buffer.len().await, 0);
        assert!(buffer.is_empty().await);
    }
}
