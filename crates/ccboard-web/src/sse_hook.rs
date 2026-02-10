//! SSE (Server-Sent Events) hook for live updates

use leptos::prelude::*;
use leptos::web_sys::{ErrorEvent, EventSource, MessageEvent};
use serde::Deserialize;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

/// SSE event types matching backend DataEvent
#[derive(Debug, Clone, PartialEq)]
pub enum SseEvent {
    /// Stats cache was updated
    StatsUpdated,
    /// A new session was created
    SessionCreated { id: String },
    /// An existing session was updated
    SessionUpdated { id: String },
    /// Configuration changed
    ConfigChanged { scope: String },
    /// Analytics data was updated
    AnalyticsUpdated,
    /// Initial load completed
    LoadCompleted,
    /// Watcher encountered an error
    WatcherError { message: String },
}

/// JSON payload for session events
#[derive(Debug, Deserialize)]
struct SessionEventData {
    id: String,
}

/// JSON payload for config events
#[derive(Debug, Deserialize)]
struct ConfigEventData {
    scope: String,
}

/// JSON payload for error events
#[derive(Debug, Deserialize)]
struct ErrorEventData {
    message: String,
}

/// Parse SSE event data into SseEvent enum
fn parse_sse_event(event_type: &str, data: &str) -> Option<SseEvent> {
    match event_type {
        "stats_updated" => Some(SseEvent::StatsUpdated),
        "analytics_updated" => Some(SseEvent::AnalyticsUpdated),
        "load_completed" => Some(SseEvent::LoadCompleted),
        "session_created" => serde_json::from_str::<SessionEventData>(data)
            .ok()
            .map(|payload| SseEvent::SessionCreated { id: payload.id }),
        "session_updated" => serde_json::from_str::<SessionEventData>(data)
            .ok()
            .map(|payload| SseEvent::SessionUpdated { id: payload.id }),
        "config_changed" => serde_json::from_str::<ConfigEventData>(data)
            .ok()
            .map(|payload| SseEvent::ConfigChanged {
                scope: payload.scope,
            }),
        "watcher_error" => serde_json::from_str::<ErrorEventData>(data)
            .ok()
            .map(|payload| SseEvent::WatcherError {
                message: payload.message,
            }),
        _ => None,
    }
}

/// Leptos hook for SSE subscription
///
/// Returns a signal that updates whenever an SSE event is received.
/// Automatically handles connection, reconnection, and cleanup.
///
/// # Example
///
/// ```rust,ignore
/// let sse_event = use_sse();
///
/// Effect::new(move |_| {
///     if let Some(SseEvent::StatsUpdated) = sse_event.get() {
///         // Trigger stats refetch
///         stats_resource.refetch();
///     }
/// });
/// ```
pub fn use_sse() -> ReadSignal<Option<SseEvent>> {
    let (event, set_event) = signal(None::<SseEvent>);
    let (_connection_status, set_connection_status) = signal(ConnectionStatus::Connecting);

    Effect::new(move |_| {
        // Create EventSource
        let event_source = match EventSource::new("/api/events") {
            Ok(es) => es,
            Err(e) => {
                leptos::logging::error!("Failed to create EventSource: {:?}", e);
                set_connection_status.set(ConnectionStatus::Error);
                return;
            }
        };

        set_connection_status.set(ConnectionStatus::Connected);

        // Clone EventSource for error handler closure
        let es_error = event_source.clone();

        // Handle open event
        let on_open = Closure::wrap(Box::new(move |_: web_sys::Event| {
            leptos::logging::log!("SSE connection opened");
            set_connection_status.set(ConnectionStatus::Connected);
        }) as Box<dyn FnMut(_)>);

        event_source.set_onopen(Some(on_open.as_ref().unchecked_ref()));
        on_open.forget();

        // Handle error event with reconnection
        let on_error = Closure::wrap(Box::new(move |e: ErrorEvent| {
            leptos::logging::warn!("SSE connection error: {:?}", e.message());
            set_connection_status.set(ConnectionStatus::Reconnecting);

            // EventSource automatically reconnects, but we track status
            // Check if connection is actually closed
            if es_error.ready_state() == EventSource::CLOSED {
                leptos::logging::error!("SSE connection closed permanently");
                set_connection_status.set(ConnectionStatus::Error);
            }
        }) as Box<dyn FnMut(_)>);

        event_source.set_onerror(Some(on_error.as_ref().unchecked_ref()));
        on_error.forget();

        // Register listeners for each event type
        let event_types = vec![
            "stats_updated",
            "session_created",
            "session_updated",
            "config_changed",
            "analytics_updated",
            "load_completed",
            "watcher_error",
        ];

        for event_type in event_types {
            let event_type_owned = event_type.to_string();
            let callback = Closure::wrap(Box::new(move |event: MessageEvent| {
                let data = event.data().as_string().unwrap_or_default();

                if let Some(parsed_event) = parse_sse_event(&event_type_owned, &data) {
                    leptos::logging::log!("SSE event received: {:?}", parsed_event);
                    set_event.set(Some(parsed_event));
                }
            }) as Box<dyn FnMut(_)>);

            event_source
                .add_event_listener_with_callback(event_type, callback.as_ref().unchecked_ref())
                .unwrap_or_else(|e| {
                    leptos::logging::error!("Failed to add listener for {}: {:?}", event_type, e);
                });

            callback.forget();
        }

        // Cleanup function (runs when effect is disposed)
        // Note: In Leptos, Effects don't have explicit cleanup, but EventSource
        // will be dropped when component unmounts
    });

    event
}

/// Connection status for SSE
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConnectionStatus {
    /// Attempting initial connection
    Connecting,
    /// Successfully connected
    Connected,
    /// Connection lost, attempting reconnect
    Reconnecting,
    /// Permanent error
    Error,
}

/// Hook to get SSE connection status
pub fn use_sse_status() -> ReadSignal<ConnectionStatus> {
    let (status, _) = signal(ConnectionStatus::Connecting);
    status
}
