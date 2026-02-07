# SSE Live Updates & Toast Notifications - Developer Guide

This guide explains how to use the SSE (Server-Sent Events) system and toast notifications in ccboard-web.

## Overview

The SSE system provides real-time updates from the backend to the frontend without polling. When data changes on the server (stats updated, new session, etc.), the frontend is immediately notified and can react accordingly.

## Using SSE in Your Component

### 1. Import the Hook

```rust
use crate::sse_hook::{use_sse, SseEvent};
use crate::components::use_toast;
```

### 2. Setup SSE Subscription

```rust
#[component]
pub fn MyComponent() -> impl IntoView {
    // Get SSE event signal
    let sse_event = use_sse();

    // Get toast context
    let toast = use_toast();

    // Create a resource you want to refetch on events
    let (version, set_version) = signal(0u32);
    let data = LocalResource::new(move || {
        let _ = version.get(); // Track version
        async move { fetch_data().await }
    });

    // React to SSE events
    Effect::new(move |_| {
        if let Some(event) = sse_event.get() {
            match event {
                SseEvent::StatsUpdated => {
                    set_version.update(|v| *v += 1);
                    toast.info("Data updated".to_string());
                }
                SseEvent::SessionCreated { id } => {
                    set_version.update(|v| *v += 1);
                    toast.success(format!("New session: {}", id));
                }
                _ => {}
            }
        }
    });

    // ... rest of component
}
```

## Available SSE Events

```rust
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
```

## Using Toast Notifications

### Get Toast Context

```rust
let toast = use_toast();
```

### Show Toasts

```rust
// Info toast (blue, 3s auto-dismiss)
toast.info("Operation completed".to_string());

// Success toast (green, 3s auto-dismiss)
toast.success("Data saved successfully".to_string());

// Warning toast (yellow, 3s auto-dismiss)
toast.warning("High token usage detected".to_string());

// Error toast (red, 5s auto-dismiss)
toast.error("Failed to load data".to_string());
```

### Toast Features

- **Auto-dismiss**: Toasts automatically disappear after a timeout (3s for info/success/warning, 5s for errors)
- **Manual dismiss**: Users can close any toast by clicking the `×` button
- **Stack**: Multiple toasts stack vertically in bottom-right corner
- **Responsive**: Full-width on mobile devices
- **Animations**: Smooth slide-in animation for each toast

## Best Practices

### 1. Don't Show Redundant Toasts

If you're refetching data silently (user doesn't need to know), don't show a toast:

```rust
SseEvent::StatsUpdated => {
    set_version.update(|v| *v += 1);
    // No toast - silent background update
}
```

### 2. Use Appropriate Toast Types

- **Info** (blue): General updates, informational messages
- **Success** (green): Successful operations, confirmations
- **Warning** (yellow): Cautionary messages, approaching limits
- **Error** (red): Failures, errors that need attention

### 3. Keep Messages Short

Toasts have limited space. Keep messages concise:

```rust
// ✅ Good
toast.info("Stats updated".to_string());

// ❌ Bad (too long)
toast.info("The statistics cache has been successfully updated with the latest data from the backend server".to_string());
```

### 4. Avoid Toast Spam

If an event fires very frequently, consider debouncing or only showing toasts for important events:

```rust
SseEvent::SessionUpdated { .. } => {
    set_version.update(|v| *v += 1);
    // Only toast for creation, not every update
}
```

## Architecture Details

### SSE Connection

The `use_sse()` hook:
- Automatically connects to `/api/events` on component mount
- Handles reconnection if connection is lost
- Parses incoming events into `SseEvent` enum
- Updates a signal whenever an event is received
- Cleans up connection when component unmounts

### Toast State Management

The toast system uses Leptos context:
- `ToastProvider` wraps the app root (in `app.rs`)
- `ToastContext` manages global toast state
- `use_toast()` hook provides access from any component
- Toasts are stored in a reactive `RwSignal<Vec<Toast>>`
- Auto-dismiss is implemented via `set_timeout()`

## Testing

### Manual Testing

1. Start the web server:
   ```bash
   cargo run -- web --port 3333
   ```

2. Open browser DevTools Console to see SSE events:
   ```javascript
   // You'll see logs like:
   // "SSE connection opened"
   // "SSE event received: StatsUpdated"
   ```

3. Trigger events by modifying files in `~/.claude/`:
   - Modify `stats-cache.json` → `StatsUpdated` event
   - Create a new Claude session → `SessionCreated` event

### Debugging SSE

Enable debug logging in browser console:

```rust
leptos::logging::log!("Debug message here");
```

View SSE stream in DevTools Network tab:
- Filter by "events"
- Click on the `/api/events` request
- See EventStream messages in real-time

## Common Patterns

### Pattern 1: Refetch on Event

```rust
let (version, set_version) = signal(0u32);
let data = LocalResource::new(move || {
    let _ = version.get();
    async move { fetch_data().await }
});

Effect::new(move |_| {
    if let Some(SseEvent::StatsUpdated) = sse_event.get() {
        set_version.update(|v| *v += 1);
    }
});
```

### Pattern 2: Optimistic Update + Toast

```rust
Effect::new(move |_| {
    if let Some(SseEvent::SessionCreated { id }) = sse_event.get() {
        // Optimistic: add to list immediately
        sessions.update(|s| s.push(new_session));

        // Then refetch for consistency
        set_version.update(|v| *v += 1);

        // Notify user
        toast.success(format!("New session: {}", id));
    }
});
```

### Pattern 3: Conditional Toasts

```rust
Effect::new(move |_| {
    if let Some(event) = sse_event.get() {
        match event {
            SseEvent::WatcherError { message } => {
                // Always show errors
                toast.error(format!("Error: {}", message));
            }
            SseEvent::StatsUpdated => {
                // Only show if user is on dashboard
                if current_page.get() == "/dashboard" {
                    toast.info("Stats updated".to_string());
                }
            }
            _ => {}
        }
    }
});
```

## Troubleshooting

### SSE Not Connecting

1. Check server is running and `/api/events` endpoint is accessible
2. Check browser console for connection errors
3. Verify CORS headers allow SSE (should be OK for same-origin)

### Toasts Not Appearing

1. Verify `ToastProvider` wraps your app in `app.rs`
2. Check `use_toast()` is called inside a component (not outside)
3. Check CSS is loaded (`style.css` includes toast styles)

### Multiple Toasts Overlapping

This is expected behavior - toasts stack vertically. If it's too much:
- Reduce auto-dismiss time
- Only show toasts for important events
- Add debouncing for frequent events

## Examples

See implementation in:
- `src/pages/dashboard.rs` - Stats updates with toasts
- `src/pages/sessions.rs` - Session creation with success toasts
- `src/pages/analytics.rs` - Analytics refresh with info toasts
