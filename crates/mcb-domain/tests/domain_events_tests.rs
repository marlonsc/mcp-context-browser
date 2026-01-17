//! Unit tests for domain events

use async_trait::async_trait;
use mcb_domain::events::{DomainEvent, EventPublisher};
use std::sync::Mutex;

// Mock event publisher for testing
struct MockEventPublisher {
    published_events: Mutex<Vec<DomainEvent>>,
    subscriber_count: usize,
}

impl MockEventPublisher {
    fn new() -> Self {
        Self {
            published_events: Mutex::new(Vec::new()),
            subscriber_count: 1,
        }
    }

    fn with_no_subscribers() -> Self {
        Self {
            published_events: Mutex::new(Vec::new()),
            subscriber_count: 0,
        }
    }

    fn get_published_events(&self) -> Vec<DomainEvent> {
        self.published_events.lock().unwrap().clone()
    }
}

#[async_trait]
impl EventPublisher for MockEventPublisher {
    async fn publish(&self, event: DomainEvent) -> mcb_domain::Result<()> {
        self.published_events.lock().unwrap().push(event);
        Ok(())
    }

    fn has_subscribers(&self) -> bool {
        self.subscriber_count > 0
    }
}

#[test]
fn test_domain_event_creation() {
    let event = DomainEvent::IndexRebuild {
        collection: Some("test-collection".to_string()),
    };

    // Test that event can be created and debugged
    let debug_str = format!("{:?}", event);
    assert!(debug_str.contains("IndexRebuild"));
    assert!(debug_str.contains("test-collection"));
}

#[test]
fn test_domain_event_variants() {
    // Test each event variant
    let index_rebuild = DomainEvent::IndexRebuild {
        collection: Some("my-collection".to_string()),
    };

    let sync_completed = DomainEvent::SyncCompleted {
        path: "/path/to/code".to_string(),
        files_changed: 42,
    };

    let cache_invalidate = DomainEvent::CacheInvalidate {
        namespace: Some("embeddings".to_string()),
    };

    let snapshot_created = DomainEvent::SnapshotCreated {
        root_path: "/code".to_string(),
        file_count: 100,
    };

    let file_changes = DomainEvent::FileChangesDetected {
        root_path: "/code".to_string(),
        added: 5,
        modified: 10,
        removed: 2,
    };

    // Just verify they can be created
    assert!(matches!(index_rebuild, DomainEvent::IndexRebuild { .. }));
    assert!(matches!(sync_completed, DomainEvent::SyncCompleted { .. }));
    assert!(matches!(
        cache_invalidate,
        DomainEvent::CacheInvalidate { .. }
    ));
    assert!(matches!(
        snapshot_created,
        DomainEvent::SnapshotCreated { .. }
    ));
    assert!(matches!(
        file_changes,
        DomainEvent::FileChangesDetected { .. }
    ));
}

#[test]
fn test_domain_event_clone() {
    let event1 = DomainEvent::SyncCompleted {
        path: "/code".to_string(),
        files_changed: 10,
    };

    let event2 = event1.clone();

    assert_eq!(event1, event2);
}

#[test]
fn test_event_publisher_creation() {
    let publisher = MockEventPublisher::new();
    let events = publisher.get_published_events();
    assert!(events.is_empty());
}

#[test]
fn test_has_subscribers() {
    let publisher_with_subs = MockEventPublisher::new();
    assert!(publisher_with_subs.has_subscribers());

    let publisher_no_subs = MockEventPublisher::with_no_subscribers();
    assert!(!publisher_no_subs.has_subscribers());
}

#[tokio::test]
async fn test_publish_single_event() {
    let publisher = MockEventPublisher::new();

    let event = DomainEvent::IndexRebuild {
        collection: Some("test".to_string()),
    };

    let result = publisher.publish(event).await;
    assert!(result.is_ok());

    let published_events = publisher.get_published_events();
    assert_eq!(published_events.len(), 1);

    assert!(matches!(
        &published_events[0],
        DomainEvent::IndexRebuild { collection } if collection == &Some("test".to_string())
    ));
}

#[tokio::test]
async fn test_publish_multiple_events() {
    let publisher = MockEventPublisher::new();

    let events = vec![
        DomainEvent::IndexRebuild {
            collection: Some("coll-1".to_string()),
        },
        DomainEvent::SyncCompleted {
            path: "/path".to_string(),
            files_changed: 5,
        },
        DomainEvent::CacheInvalidate { namespace: None },
    ];

    for event in events {
        publisher.publish(event).await.unwrap();
    }

    let published_events = publisher.get_published_events();
    assert_eq!(published_events.len(), 3);
}

#[test]
fn test_event_publisher_trait_object() {
    // Test that we can use EventPublisher as a trait object
    let publisher: Box<dyn EventPublisher> = Box::new(MockEventPublisher::new());
    assert!(publisher.has_subscribers());
}

#[tokio::test]
async fn test_event_serialization() {
    // Events should be serializable (for transport/logging)
    let event = DomainEvent::FileChangesDetected {
        root_path: "/code".to_string(),
        added: 1,
        modified: 2,
        removed: 3,
    };

    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("FileChangesDetected"));
    assert!(json.contains("/code"));

    let deserialized: DomainEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event, deserialized);
}
