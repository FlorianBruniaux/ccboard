# ccboard Web API Documentation

This document describes the REST API and Server-Sent Events (SSE) interface provided by the ccboard web backend (Axum).

---

## Quick Start

**Start the backend**:
```bash
cargo run -- web --port 8080
```

**Base URL**: `http://localhost:8080`

**CORS**: Configured for local development (allows `http://localhost:3333`)

---

## Endpoints

### GET `/api/stats`

Returns global Claude Code statistics aggregated from `~/.claude/stats-cache.json`.

**Response** (200 OK):
```json
{
  "total_sessions": 1234,
  "total_tokens": 45678900,
  "total_cost": 123.45,
  "projects": 8,
  "active_agents": 5,
  "total_commands": 42
}
```

**Fields**:
- `total_sessions` (integer): Total number of Claude Code sessions
- `total_tokens` (integer): Total tokens consumed across all sessions
- `total_cost` (float): Total cost in USD (calculated from token usage)
- `projects` (integer): Number of projects in `~/.claude/projects/`
- `active_agents` (integer): Number of active agents across all projects
- `total_commands` (integer): Number of commands executed

**Error Codes**:
- `500 Internal Server Error`: Stats cache failed to load

**Example**:
```bash
curl http://localhost:8080/api/stats | jq
```

---

### GET `/api/sessions?project=<path>`

Returns session metadata for a specific project.

**Query Parameters**:
- `project` (string, required): Project path (e.g., `-Users-john-code-myapp`)

**Response** (200 OK):
```json
[
  {
    "id": "ea23759a-1234-5678-90ab-cdef01234567",
    "timestamp": "2026-02-09T10:30:00Z",
    "message_count": 42,
    "models": ["claude-sonnet-4-5"],
    "tokens": 12345,
    "cost": 1.23
  },
  {
    "id": "fb34860b-2345-6789-01bc-def012345678",
    "timestamp": "2026-02-08T14:20:00Z",
    "message_count": 18,
    "models": ["claude-sonnet-4-5", "claude-haiku-4-5"],
    "tokens": 5678,
    "cost": 0.56
  }
]
```

**Fields**:
- `id` (string): Session UUID
- `timestamp` (ISO 8601): Session start time
- `message_count` (integer): Number of messages in session
- `models` (array of strings): Claude models used in session
- `tokens` (integer): Total tokens consumed in session
- `cost` (float): Session cost in USD

**Error Codes**:
- `400 Bad Request`: Missing or invalid `project` parameter
- `404 Not Found`: Project not found in `~/.claude/projects/`
- `500 Internal Server Error`: Failed to load sessions from SQLite cache

**Example**:
```bash
curl "http://localhost:8080/api/sessions?project=-Users-john-code-myapp" | jq
```

---

### GET `/api/config/merged`

Returns merged configuration from global, project, and local settings.

**Response** (200 OK):
```json
{
  "global": {
    "model": "claude-sonnet-4-5",
    "temperature": 0.7
  },
  "project": {
    "temperature": 0.5
  },
  "local": {
    "max_tokens": 4096
  },
  "merged": {
    "model": "claude-sonnet-4-5",
    "temperature": 0.5,
    "max_tokens": 4096
  }
}
```

**Merge Priority**: `local` > `project` > `global` > defaults

**Fields**:
- `global` (object): Settings from `~/.claude/settings.json`
- `project` (object): Settings from `.claude/settings.json`
- `local` (object): Settings from `.claude/settings.local.json`
- `merged` (object): Final merged configuration (highest priority wins)

**Error Codes**:
- `500 Internal Server Error`: Failed to parse configuration files

**Example**:
```bash
curl http://localhost:8080/api/config/merged | jq
```

---

### GET `/api/events` (Server-Sent Events)

Live update stream for real-time monitoring. Pushes events when `~/.claude` files change.

**Event Types**:
- `stats_updated`: Global stats changed (e.g., `stats-cache.json` modified)
- `session_created`: New session detected (e.g., new `.jsonl` file)
- `session_updated`: Session file modified (e.g., message added)
- `config_changed`: Configuration file changed (e.g., `settings.json` modified)

**Response** (SSE stream):
```
event: stats_updated
data: {"total_sessions": 1235, "total_tokens": 45680000}

event: session_created
data: {"id": "fb34860b-...", "project": "-Users-john-code-myapp"}

event: session_updated
data: {"id": "ea23759a-...", "message_count": 43}

event: config_changed
data: {"scope": "global", "key": "model", "value": "claude-opus-4-6"}
```

**Usage (JavaScript)**:
```javascript
const events = new EventSource('http://localhost:8080/api/events');

events.addEventListener('stats_updated', (e) => {
  const data = JSON.parse(e.data);
  console.log('Stats updated:', data);
});

events.addEventListener('session_created', (e) => {
  const data = JSON.parse(e.data);
  console.log('New session:', data.id);
});

events.onerror = (err) => {
  console.error('SSE error:', err);
  events.close();
};
```

**Connection Management**:
- Server sends keepalive every 30 seconds (`: keepalive` comment)
- Client should reconnect on error (automatic in most SSE libraries)
- Server closes connection after 5 minutes idle (no events)

**Error Codes**:
- `500 Internal Server Error`: EventBus subscription failed

---

## CORS Configuration

The API is configured for local development with CORS enabled:

**Allowed Origins**:
- `http://localhost:3333` (Leptos frontend via Trunk)
- `http://127.0.0.1:3333` (alternative localhost)

**Allowed Methods**:
- `GET`, `POST`, `OPTIONS`

**Allowed Headers**:
- `Content-Type`, `Authorization`

**Production Note**: Update CORS origins in `ccboard-web/src/router.rs` before deploying to production.

---

## Error Handling

All endpoints return JSON errors with this format:

```json
{
  "error": "Human-readable error message",
  "code": "ERROR_CODE",
  "details": "Optional additional context"
}
```

**Common Error Codes**:
- `STATS_LOAD_FAILED`: Failed to load stats-cache.json
- `SESSIONS_LOAD_FAILED`: Failed to query SQLite cache
- `CONFIG_PARSE_FAILED`: Failed to parse settings.json
- `PROJECT_NOT_FOUND`: Specified project does not exist
- `INVALID_PARAMETER`: Missing or invalid query parameter

---

## Performance Considerations

### Caching

- **Stats**: Cached in memory, invalidated on `stats-cache.json` change
- **Sessions**: Stored in SQLite cache (89x faster than JSONL parsing)
- **Config**: Cached in memory, invalidated on `settings.json` change

### Rate Limiting

Currently **no rate limiting** (local development only). Production deployment should add:
- Token bucket rate limiting (e.g., 100 requests/minute per IP)
- Request size limits (max 1MB for POST bodies)

### Scalability

- **Single-instance**: Designed for local `~/.claude` monitoring (1 user)
- **Multi-user**: See Phase 13 roadmap (PostgreSQL + team server mode)

---

## Client SDKs

### JavaScript/TypeScript

```typescript
// stats.ts
export async function getStats() {
  const response = await fetch('http://localhost:8080/api/stats');
  if (!response.ok) throw new Error('Failed to fetch stats');
  return response.json();
}

// sessions.ts
export async function getSessions(project: string) {
  const url = `http://localhost:8080/api/sessions?project=${encodeURIComponent(project)}`;
  const response = await fetch(url);
  if (!response.ok) throw new Error('Failed to fetch sessions');
  return response.json();
}

// events.ts
export function subscribeToEvents(handlers: {
  onStatsUpdated?: (data: any) => void;
  onSessionCreated?: (data: any) => void;
}) {
  const events = new EventSource('http://localhost:8080/api/events');

  if (handlers.onStatsUpdated) {
    events.addEventListener('stats_updated', (e) => {
      handlers.onStatsUpdated(JSON.parse(e.data));
    });
  }

  if (handlers.onSessionCreated) {
    events.addEventListener('session_created', (e) => {
      handlers.onSessionCreated(JSON.parse(e.data));
    });
  }

  return events;
}
```

### Rust

```rust
// Using reqwest for HTTP client
use reqwest::Client;
use serde_json::Value;

async fn get_stats(client: &Client) -> Result<Value, reqwest::Error> {
    let response = client
        .get("http://localhost:8080/api/stats")
        .send()
        .await?;

    response.json().await
}

async fn get_sessions(client: &Client, project: &str) -> Result<Vec<Value>, reqwest::Error> {
    let url = format!("http://localhost:8080/api/sessions?project={}", project);
    let response = client.get(&url).send().await?;

    response.json().await
}
```

---

## Testing

### Manual Testing

```bash
# Start backend
cargo run -- web --port 8080

# Test stats endpoint
curl http://localhost:8080/api/stats | jq

# Test sessions endpoint
curl "http://localhost:8080/api/sessions?project=-Users-john-code-myapp" | jq

# Test config endpoint
curl http://localhost:8080/api/config/merged | jq

# Test SSE (leave running)
curl -N http://localhost:8080/api/events
```

### Automated Testing

```bash
# Integration tests (requires real ~/.claude data)
cargo test --test api_integration

# Load testing with wrk
wrk -t4 -c100 -d30s http://localhost:8080/api/stats
```

---

## Troubleshooting

### "Failed to load stats"

**Cause**: `~/.claude/stats-cache.json` missing or corrupted

**Solution**: Run Claude Code once to generate stats cache

---

### "Connection refused"

**Cause**: Backend not running or wrong port

**Solution**: Verify backend is running with `lsof -i :8080`

---

### SSE connection drops

**Cause**: Server closes connection after 5 minutes idle

**Solution**: Client should automatically reconnect (built into EventSource)

---

### CORS errors in browser

**Cause**: Frontend running on non-allowed origin

**Solution**: Add origin to CORS config in `ccboard-web/src/router.rs`

---

## Future API Additions

See `claudedocs/ROADMAP.md` for planned endpoints:
- `POST /api/config/update` (Phase 12: Write operations)
- `GET /api/conversation/:session_id` (Phase F: Conversation viewer)
- `GET /api/plan/:project` (Phase H: Plan-aware)
- `GET /api/tokens/breakdown` (Phase 11: Token analytics)

---

**Last Updated**: 2026-02-09
**API Version**: v0.5.0
**Backend**: Axum + Tokio
**Frontend**: Leptos + WASM
