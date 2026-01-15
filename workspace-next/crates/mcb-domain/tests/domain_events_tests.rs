//! Unit tests for domain events

#[cfg(test)]
mod tests {
    use mcb_domain::{DomainEvent, EventPublisher};
    use async_trait::async_trait;
    use std::sync::Mutex;

    // Mock event for testing
    #[derive(Debug, Clone, PartialEq)]
    struct TestEvent {
        id: String,
        message: String,
        timestamp: i64,
    }

    impl DomainEvent for TestEvent {
        fn event_type(&self) -> &str {
            "test_event"
        }

        fn event_id(&self) -> &str {
            &self.id
        }

        fn aggregate_id(&self) -> &str {
            "test-aggregate"
        }

        fn occurred_at(&self) -> i64 {
            self.timestamp
        }
    }

    // Mock event publisher for testing
    struct MockEventPublisher {
        published_events: Mutex<Vec<Box<dyn DomainEvent>>>,
    }

    impl MockEventPublisher {
        fn new() -> Self {
            Self {
                published_events: Mutex::new(Vec::new()),
            }
        }

        fn get_published_events(&self) -> Vec<Box<dyn DomainEvent>> {
            self.published_events.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl EventPublisher for MockEventPublisher {
        async fn publish(&self, event: Box<dyn DomainEvent>) -> mcb_domain::Result<()> {
            self.published_events.lock().unwrap().push(event);
            Ok(())
        }

        async fn publish_batch(&self, events: Vec<Box<dyn DomainEvent>>) -> mcb_domain::Result<()> {
            let mut published = self.published_events.lock().unwrap();
            for event in events {
                published.push(event);
            }
            Ok(())
        }
    }

    #[test]
    fn test_domain_event_creation() {
        let event = TestEvent {
            id: "event-123".to_string(),
            message: "Test event message".to_string(),
            timestamp: 1640995200,
        };

        assert_eq!(event.event_type(), "test_event");
        assert_eq!(event.event_id(), "event-123");
        assert_eq!(event.aggregate_id(), "test-aggregate");
        assert_eq!(event.occurred_at(), 1640995200);
    }

    #[test]
    fn test_domain_event_debug() {
        let event = TestEvent {
            id: "debug-event".to_string(),
            message: "Debug message".to_string(),
            timestamp: 1641081600,
        };

        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("TestEvent"));
        assert!(debug_str.contains("debug-event"));
        assert!(debug_str.contains("Debug message"));
    }

    #[test]
    fn test_domain_event_clone() {
        let event1 = TestEvent {
            id: "clone-test".to_string(),
            message: "Original message".to_string(),
            timestamp: 1641168000,
        };

        let event2 = event1.clone();

        assert_eq!(event1.id, event2.id);
        assert_eq!(event1.message, event2.message);
        assert_eq!(event1.timestamp, event2.timestamp);
    }

    #[test]
    fn test_event_publisher_creation() {
        let publisher = MockEventPublisher::new();
        let events = publisher.get_published_events();
        assert!(events.is_empty());
    }

    #[tokio::test]
    async fn test_publish_single_event() {
        let publisher = MockEventPublisher::new();

        let event = TestEvent {
            id: "published-event".to_string(),
            message: "Event to publish".to_string(),
            timestamp: 1641254400,
        };

        let result = publisher.publish(Box::new(event)).await;
        assert!(result.is_ok());

        let published_events = publisher.get_published_events();
        assert_eq!(published_events.len(), 1);

        // Note: We can't easily downcast the trait object to test the exact content,
        // but we can verify it was published
    }

    #[tokio::test]
    async fn test_publish_batch_events() {
        let publisher = MockEventPublisher::new();

        let events = vec![
            Box::new(TestEvent {
                id: "batch-event-1".to_string(),
                message: "First batch event".to_string(),
                timestamp: 1641340800,
            }) as Box<dyn DomainEvent>,
            Box::new(TestEvent {
                id: "batch-event-2".to_string(),
                message: "Second batch event".to_string(),
                timestamp: 1641427200,
            }) as Box<dyn DomainEvent>,
        ];

        let result = publisher.publish_batch(events).await;
        assert!(result.is_ok());

        let published_events = publisher.get_published_events();
        assert_eq!(published_events.len(), 2);
    }

    #[test]
    fn test_event_publisher_trait_object() {
        // Test that we can use EventPublisher as a trait object
        let publisher: Box<dyn EventPublisher> = Box::new(MockEventPublisher::new());
        // Just test that the trait object can be created
        assert!(true);
    }

    #[tokio::test]
    async fn test_event_publisher_error_handling() {
        // Since our mock always returns Ok, we can't easily test error cases
        // In a real implementation, we would test error conditions
        let publisher = MockEventPublisher::new();

        let event = TestEvent {
            id: "error-test".to_string(),
            message: "Testing error handling".to_string(),
            timestamp: 1641513600,
        };

        // Our mock always succeeds, so this should work
        let result = publisher.publish(Box::new(event)).await;
        assert!(result.is_ok());
    }
}