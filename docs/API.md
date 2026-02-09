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

### GET `/api/health`

Health check endpoint returning server status and basic metrics.

**Response** (200 OK):
```json
{
  "status": "healthy",
  "sessions": 1234,
  "stats_loaded": true
}
```

**Fields**:
- `status` (string): `"healthy"` if all systems operational, `"degraded"` if issues detected
- `sessions` (integer): Total number of sessions loaded in memory
- `stats_loaded` (boolean): Whether stats-cache.json was successfully loaded

**Use Case**: Monitor backend health, check if data is loaded before making other API calls

**Example**:
```bash
curl http://localhost:8080/api/health | jq
```

---

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

### GET `/api/sessions/recent`

Returns the N most recent sessions across all projects (lightweight endpoint for dashboards).

**Query Parameters**:
- `limit` (integer, optional): Number of sessions to return (default: 5, max: 100)

**Response** (200 OK):
```json
{
  "sessions": [
    {
      "id": "ea23759a-...",
      "date": "2026-02-09T10:30:00Z",
      "project": "-Users-john-code-myapp",
      "model": "claude-sonnet-4-5",
      "messages": 42,
      "tokens": 12345,
      "cost": 1.23,
      "preview": "How do I implement authentication?"
    }
  ],
  "total": 1234
}
```

**Fields**:
- `sessions` (array): Recent sessions sorted by date descending
- `total` (integer): Total number of sessions across all projects

**Example**:
```bash
curl "http://localhost:8080/api/sessions/recent?limit=10" | jq
```

---

### GET `/api/sessions/live`

Returns active Claude Code processes with real-time CPU and RAM monitoring.

**Response** (200 OK):
```json
{
  "sessions": [
    {
      "pid": 12345,
      "startTime": "2026-02-09T10:30:00Z",
      "workingDirectory": "/Users/john/code/myapp",
      "command": "claude",
      "cpuPercent": 15.3,
      "memoryMb": 512,
      "tokens": 1234,
      "sessionId": "ea23759a-...",
      "sessionName": "myapp-session"
    }
  ],
  "total": 1
}
```

**Fields**:
- `pid` (integer): Process ID
- `startTime` (ISO 8601): Process start time
- `cpuPercent` (float): Current CPU usage percentage
- `memoryMb` (integer): Current memory usage in MB
- `tokens` (integer): Tokens consumed in current session
- `sessionId` (string): Associated session UUID (if available)

**Use Case**: Live monitoring dashboard with CPU/RAM badges (bonus feature Sprint 1)

**Example**:
```bash
curl http://localhost:8080/api/sessions/live | jq
```

---

### GET `/api/sessions`

Returns session metadata with pagination, filtering, and sorting.

**Query Parameters**:
- `page` (integer, optional): Page number (default: 0)
- `limit` (integer, optional): Page size (default: 50, max: 100)
- `search` (string, optional): Search in session ID, project path, or first message
- `project` (string, optional): Filter by project path (partial match)
- `model` (string, optional): Filter by model name (partial match)
- `since` (string, optional): Filter by time range (e.g., `7d`, `30d`, `1h`)
- `sort` (string, optional): Sort field (`date`, `tokens`, `cost`) (default: `date`)
- `order` (string, optional): Sort order (`asc`, `desc`) (default: `desc`)

**Response** (200 OK):
```json
{
  "sessions": [
    {
      "id": "ea23759a-1234-5678-90ab-cdef01234567",
      "date": "2026-02-09T10:30:00Z",
      "project": "-Users-john-code-myapp",
      "model": "claude-sonnet-4-5",
      "messages": 42,
      "tokens": 12345,
      "input_tokens": 5000,
      "output_tokens": 7000,
      "cache_creation_tokens": 300,
      "cache_read_tokens": 45,
      "cost": 1.23,
      "status": "completed",
      "first_timestamp": "2026-02-09T10:00:00Z",
      "duration_seconds": 1800,
      "preview": "How do I implement authentication?"
    }
  ],
  "total": 1234,
  "page": 0,
  "page_size": 50
}
```

**Response Fields**:
- `sessions` (array): Array of session objects
- `total` (integer): Total number of sessions matching filters (before pagination)
- `page` (integer): Current page number
- `page_size` (integer): Number of sessions per page

**Session Object Fields**:
- `id` (string): Session UUID
- `date` (ISO 8601): Last message timestamp
- `project` (string): Project path
- `model` (string): Primary model used (first in list)
- `messages` (integer): Number of messages in session
- `tokens` (integer): Total tokens (input + output + cache)
- `input_tokens` (integer): Input tokens consumed
- `output_tokens` (integer): Output tokens generated
- `cache_creation_tokens` (integer): Tokens written to cache
- `cache_read_tokens` (integer): Tokens read from cache
- `cost` (float): Estimated cost in USD
- `first_timestamp` (ISO 8601): Session start time
- `duration_seconds` (integer): Session duration
- `preview` (string): First user message (truncated)

**Error Codes**:
- `500 Internal Server Error`: Failed to load sessions from SQLite cache

**Examples**:
```bash
# Get recent sessions for specific project
curl "http://localhost:8080/api/sessions?project=myapp&limit=10" | jq

# Search sessions containing "auth"
curl "http://localhost:8080/api/sessions?search=auth" | jq

# Get sessions from last 7 days, sorted by cost
curl "http://localhost:8080/api/sessions?since=7d&sort=cost&order=desc" | jq

# Filter by model and paginate
curl "http://localhost:8080/api/sessions?model=opus&page=1&limit=20" | jq
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

### GET `/api/hooks`

Returns all configured hooks from merged settings (global + project + local) with script content.

**Response** (200 OK):
```json
{
  "hooks": [
    {
      "name": "pre-commit",
      "event": "pre-commit",
      "command": ".claude/hooks/bash/pre-commit.sh",
      "description": "Run cargo fmt and clippy before commit",
      "async": false,
      "timeout": 30,
      "cwd": null,
      "matcher": null,
      "scriptPath": ".claude/hooks/bash/pre-commit.sh",
      "scriptContent": "#!/bin/bash\n# Description: Run cargo fmt and clippy..."
    }
  ],
  "total": 1
}
```

**Fields**:
- `name` (string): Hook identifier (event name or event-group-index)
- `event` (string): Event that triggers the hook (e.g., `pre-commit`, `post-session`)
- `command` (string): Command or script path
- `description` (string): Extracted from `# Description:` comment in script, or command itself
- `async` (boolean): Whether hook runs asynchronously
- `timeout` (integer): Timeout in seconds
- `scriptContent` (string): Full script content if command is a `.sh` file

**Use Case**: Hooks tab in TUI/Web, syntax highlighting for bash scripts

**Example**:
```bash
curl http://localhost:8080/api/hooks | jq
```

---

### GET `/api/mcp`

Returns MCP server configuration from `claude_desktop_config.json`.

**Response** (200 OK):
```json
{
  "servers": [
    {
      "name": "filesystem",
      "command": "npx -y @modelcontextprotocol/server-filesystem /Users/john",
      "serverType": "stdio",
      "url": null,
      "args": ["/Users/john"],
      "env": {},
      "hasEnv": false
    },
    {
      "name": "brave-search",
      "command": null,
      "serverType": "http",
      "url": "http://localhost:3100/sse",
      "args": [],
      "env": {"BRAVE_API_KEY": "..."},
      "hasEnv": true
    }
  ],
  "total": 2
}
```

**Fields**:
- `name` (string): Server identifier
- `command` (string): Command for stdio servers
- `serverType` (string): `"stdio"` or `"http"`
- `url` (string): URL for HTTP servers
- `args` (array): Command arguments
- `env` (object): Environment variables
- `hasEnv` (boolean): Whether server has environment variables

**Use Case**: MCP tab in TUI/Web, server status monitoring

**Example**:
```bash
curl http://localhost:8080/api/mcp | jq
```

---

### GET `/api/agents`

Returns agents from `~/.claude/agents/` with frontmatter metadata.

**Response** (200 OK):
```json
{
  "items": [
    {
      "name": "backend-architect",
      "frontmatter": {
        "title": "Backend Architect",
        "version": "1.0.0",
        "description": "Expert backend developer"
      },
      "body": "# Backend Architect\n\nExpert senior en architecture backend...",
      "path": "/Users/john/.claude/agents/backend-architect.md"
    }
  ],
  "total": 1
}
```

**Fields**:
- `name` (string): Agent filename (without `.md`)
- `frontmatter` (object): YAML metadata between `---` markers
- `body` (string): Markdown content after frontmatter
- `path` (string): Full file path

**Use Case**: Agents/Capabilities tab in TUI/Web

**Example**:
```bash
curl http://localhost:8080/api/agents | jq
```

---

### GET `/api/commands`

Returns commands from `~/.claude/commands/` with frontmatter metadata.

**Response**: Same format as `/api/agents`

**Example**:
```bash
curl http://localhost:8080/api/commands | jq
```

---

### GET `/api/skills`

Returns skills from `~/.claude/skills/*/SKILL.md` with frontmatter metadata.

**Response** (200 OK):
```json
{
  "items": [
    {
      "name": "ccboard",
      "frontmatter": {
        "name": "ccboard",
        "invoke": "ccboard",
        "version": "0.5.0"
      },
      "body": "# ccboard Skill\n\nComprehensive TUI/Web dashboard...",
      "path": "/Users/john/.claude/skills/ccboard/SKILL.md"
    }
  ],
  "total": 1
}
```

**Fields**: Same as `/api/agents` and `/api/commands`

**Note**: Skills are scanned from subdirectories, looking for `SKILL.md` files

**Example**:
```bash
curl http://localhost:8080/api/skills | jq
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
