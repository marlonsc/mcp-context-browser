//! Logging tests
//!
//! Tests migrated from src/infrastructure/logging.rs

use mcp_context_browser::infrastructure::logging::{create_shared_log_buffer, LogBuffer, LogEntry};
use std::time::Duration;
use tracing::Level;

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
