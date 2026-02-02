//! Event bus for ccboard using tokio::broadcast
//!
//! Provides a publish-subscribe mechanism for data updates.

use tokio::sync::broadcast;

/// Events emitted by the data layer
#[derive(Debug, Clone)]
pub enum DataEvent {
    /// Stats cache was updated
    StatsUpdated,
    /// A new session was created
    SessionCreated(String),
    /// An existing session was updated
    SessionUpdated(String),
    /// Configuration changed
    ConfigChanged(ConfigScope),
    /// Initial load completed
    LoadCompleted,
    /// Watcher encountered an error
    WatcherError(String),
}

/// Scope of configuration change
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigScope {
    Global,
    Project(String),
    Local(String),
    Mcp,
}

/// Event bus for broadcasting data events
///
/// Uses tokio::broadcast for multi-consumer support.
/// TUI subscribes for redraw triggers, Web uses for SSE push.
pub struct EventBus {
    sender: broadcast::Sender<DataEvent>,
}

impl EventBus {
    /// Create a new event bus with specified channel capacity
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    /// Create with default capacity (256 events)
    pub fn default_capacity() -> Self {
        Self::new(256)
    }

    /// Publish an event to all subscribers
    pub fn publish(&self, event: DataEvent) {
        // Ignore send errors (no subscribers)
        let _ = self.sender.send(event);
    }

    /// Subscribe to receive events
    pub fn subscribe(&self) -> broadcast::Receiver<DataEvent> {
        self.sender.subscribe()
    }

    /// Get current number of active subscribers
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::default_capacity()
    }
}

impl Clone for EventBus {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_event_bus_publish_subscribe() {
        let bus = EventBus::default_capacity();
        let mut rx = bus.subscribe();

        bus.publish(DataEvent::StatsUpdated);
        bus.publish(DataEvent::SessionCreated("test-session".to_string()));

        let event1 = rx.recv().await.unwrap();
        assert!(matches!(event1, DataEvent::StatsUpdated));

        let event2 = rx.recv().await.unwrap();
        assert!(matches!(event2, DataEvent::SessionCreated(id) if id == "test-session"));
    }

    #[tokio::test]
    async fn test_event_bus_multiple_subscribers() {
        let bus = EventBus::default_capacity();
        let mut rx1 = bus.subscribe();
        let mut rx2 = bus.subscribe();

        assert_eq!(bus.subscriber_count(), 2);

        bus.publish(DataEvent::LoadCompleted);

        let e1 = rx1.recv().await.unwrap();
        let e2 = rx2.recv().await.unwrap();

        assert!(matches!(e1, DataEvent::LoadCompleted));
        assert!(matches!(e2, DataEvent::LoadCompleted));
    }

    #[test]
    fn test_event_bus_no_subscribers_ok() {
        let bus = EventBus::default_capacity();
        // Should not panic even with no subscribers
        bus.publish(DataEvent::StatsUpdated);
    }
}
