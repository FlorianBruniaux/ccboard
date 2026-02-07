//! Server-Sent Events for live updates

use axum::response::sse::{Event, KeepAlive, Sse};
use ccboard_core::{DataEvent, EventBus};
use futures::stream::Stream;
use std::convert::Infallible;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;

/// Create an SSE stream from the event bus
/// Takes EventBus by value (cheap clone, Arc internally)
pub fn create_sse_stream(
    event_bus: EventBus,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = event_bus.subscribe();
    let stream = BroadcastStream::new(rx);

    let sse_stream = stream.filter_map(|result: Result<DataEvent, _>| {
        result.ok().map(|event: DataEvent| {
            let (event_type, data) = match event {
                DataEvent::StatsUpdated => ("stats_updated", "{}".to_string()),
                DataEvent::SessionCreated(id) => {
                    ("session_created", format!(r#"{{"id":"{}"}}"#, id))
                }
                DataEvent::SessionUpdated(id) => {
                    ("session_updated", format!(r#"{{"id":"{}"}}"#, id))
                }
                DataEvent::ConfigChanged(scope) => {
                    ("config_changed", format!(r#"{{"scope":"{:?}"}}"#, scope))
                }
                DataEvent::AnalyticsUpdated => ("analytics_updated", "{}".to_string()),
                DataEvent::LoadCompleted => ("load_completed", "{}".to_string()),
                DataEvent::WatcherError(msg) => {
                    ("watcher_error", format!(r#"{{"message":"{}"}}"#, msg))
                }
            };

            Ok(Event::default().event(event_type).data(data))
        })
    });

    Sse::new(sse_stream).keep_alive(KeepAlive::default())
}
