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

Returns global Claude Code statistics aggregated from `~/.claude/stats-cache.json`, enriched with analytics (forecast, daily activity, model breakdown).

**Response** (200 OK):
```json
{
  "version": 2,
  "lastComputedDate": "2026-02-10",
  "firstSessionDate": "2025-12-10T09:45:00.350Z",
  "totalSessions": 1757,
  "totalMessages": 512937,
  "thisMonthCost": 11205.38,
  "avgSessionCost": 3.06,
  "cacheHitRatio": 0.999,
  "mcpServersCount": 3,
  "mostUsedModel": "claude-opus-4-6",
  "totalSpeculationTimeSavedMs": 0,
  "longestSession": {
    "sessionId": "d78b55ae-...",
    "messageCount": 10827,
    "date": null
  },
  "modelUsage": {
    "claude-opus-4-6": {
      "inputTokens": 5000,
      "outputTokens": 7000,
      "cacheCreationInputTokens": 300,
      "cacheReadInputTokens": 45,
      "costUsd": 123.45,
      "contextWindow": 0,
      "maxOutputTokens": 0,
      "webSearchRequests": 0
    }
  },
  "hourCounts": { "0": 6, "1": 2, "10": 133 },
  "dailyActivity": [
    {
      "date": "2026-02-10",
      "sessionCount": 42,
      "messageCount": 12345,
      "toolCallCount": 3456
    }
  ],
  "dailyModelTokens": [
    {
      "date": "2026-02-10",
      "tokensByModel": { "claude-opus-4-6": 1332082 }
    }
  ],
  "dailyTokens30d": [66938374, 45000000],
  "forecastTokens30d": [33071759, 40000000],
  "forecastConfidence": 0.14,
  "forecastCost30d": 9921.53,
  "projectsByCost": [
    {
      "project": "/Users/john/code/myapp",
      "cost": 4986.04,
      "percentage": 44.5
    }
  ]
}
```

**Top-Level Fields**:
- `version` (integer): Stats cache format version
- `lastComputedDate` (string): Date stats were last computed (YYYY-MM-DD)
- `firstSessionDate` (ISO 8601): Earliest session timestamp
- `totalSessions` (integer): Total number of sessions in stats cache
- `totalMessages` (integer): Total messages across all sessions
- `thisMonthCost` (float): Total cost in USD for the current month
- `avgSessionCost` (float): Average cost per session in USD
- `cacheHitRatio` (float): Cache read/creation ratio (0-1)
- `mcpServersCount` (integer): Number of configured MCP servers
- `mostUsedModel` (string|null): Most frequently used model name
- `totalSpeculationTimeSavedMs` (integer): Speculative execution time saved in ms

**Nested Objects**:
- `longestSession` (object): Session with most messages (`sessionId`, `messageCount`, `date`)
- `modelUsage` (object): Per-model token breakdown keyed by model ID, each containing `inputTokens`, `outputTokens`, `cacheCreationInputTokens`, `cacheReadInputTokens`, `costUsd`, `contextWindow`, `maxOutputTokens`, `webSearchRequests`
- `hourCounts` (object): Message count per hour (0-23), keyed as strings

**Arrays**:
- `dailyActivity` (array): Daily aggregates with `date`, `sessionCount`, `messageCount`, `toolCallCount`
- `dailyModelTokens` (array): Daily token usage per model with `date` and `tokensByModel` map
- `dailyTokens30d` (array): Daily total tokens for last 30 days (integers)
- `forecastTokens30d` (array): Predicted daily tokens for next 30 days (integers)
- `forecastConfidence` (float): Forecast confidence score (0-1)
- `forecastCost30d` (float): Predicted cost for next 30 days in USD
- `projectsByCost` (array): Top 5 projects by cost with `project`, `cost`, `percentage`

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
      "first_timestamp": "2026-02-09T10:00:00Z",
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
      "duration_seconds": null,
      "preview": "How do I implement authentication?"
    }
  ],
  "total": 1234
}
```

**Fields**:
- `sessions` (array): Recent sessions sorted by date descending (same object format as `/api/sessions`)
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
      "startTime": "2026-02-09T10:30:00+01:00",
      "workingDirectory": "/Users/john/code/myapp",
      "command": "claude",
      "cpuPercent": 15.3,
      "memoryMb": 512,
      "tokens": 1234,
      "sessionId": "ea23759a-...",
      "sessionName": null
    }
  ],
  "total": 3
}
```

**Fields**:
- `pid` (integer): Process ID
- `startTime` (ISO 8601): Process start time (with timezone offset)
- `workingDirectory` (string): Current working directory of the process
- `command` (string): Process command name (e.g., `"claude"`)
- `cpuPercent` (float): Current CPU usage percentage
- `memoryMb` (integer): Current memory usage in MB
- `tokens` (integer): Tokens consumed in current session
- `sessionId` (string|null): Associated session UUID (matched from JSONL files)
- `sessionName` (string|null): Optional session name (if set by user)
- `total` (integer): Total number of active Claude processes

**Use Case**: Live monitoring dashboard with CPU/RAM badges

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
- `duration_seconds` (integer|null): Session duration (null if not computed)
- `preview` (string): First user message (truncated to ~200 chars)

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
      "name": "UserPromptSubmit",
      "event": "UserPromptSubmit",
      "command": "current_model=$(jq -r ...) ...",
      "description": "current_model=$(jq -r ...) ...",
      "async": false,
      "timeout": null,
      "cwd": null,
      "matcher": null,
      "scriptPath": null,
      "scriptContent": null
    },
    {
      "name": "PreToolUse-0-0",
      "event": "PreToolUse",
      "command": "case \"$TOOL_INPUT\" in ...",
      "description": "case \"$TOOL_INPUT\" in ...",
      "async": false,
      "timeout": null,
      "cwd": null,
      "matcher": "Bash",
      "scriptPath": null,
      "scriptContent": null
    }
  ],
  "total": 5
}
```

**Fields**:
- `name` (string): Hook identifier (event name or `event-group-index` for multiple hooks per event)
- `event` (string): Event trigger (`UserPromptSubmit`, `PreToolUse`, `Custom`, etc.)
- `command` (string): Inline command or script path
- `description` (string): Extracted from `# Description:` comment in script, or command itself
- `async` (boolean): Whether hook runs asynchronously
- `timeout` (integer|null): Timeout in seconds (null if not set)
- `cwd` (string|null): Working directory override
- `matcher` (string|null): Tool matcher pattern (e.g., `"Bash"` for PreToolUse hooks)
- `scriptPath` (string|null): Path to external script file (if command references a `.sh` file)
- `scriptContent` (string|null): Full script content (loaded when scriptPath is set)
- `total` (integer): Total number of hooks

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
- `frontmatter` (object): YAML metadata between `---` markers (empty `{}` if no frontmatter)
- `body` (string): Markdown content after frontmatter
- `path` (string): Full file path
- `total` (integer): Total number of items

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

### GET `/api/plugins`

Returns plugin usage analytics aggregated across the last 10 000 sessions — invocation counts, token consumption, and dead-code detection for skills and commands.

**Response** (200 OK):
```json
{
  "analytics": {
    "skill_usage": [
      { "name": "ccboard", "invocations": 42, "tokens": 12345, "last_used": "2026-03-28T14:00:00Z" }
    ],
    "command_usage": [
      { "name": "commit", "invocations": 15, "tokens": 4500, "last_used": "2026-03-29T10:00:00Z" }
    ],
    "dead_skills": ["unused-skill"],
    "dead_commands": []
  },
  "generated_at": "2026-03-30T09:00:00Z"
}
```

**Use Case**: Plugins tab — usage analytics, dead-code detection, sort by usage/cost/name

**Example**:
```bash
curl http://localhost:8080/api/plugins | jq
```

---

### GET `/api/analytics/suggestions`

Returns actionable cost-optimization suggestions based on dead plugins and high-cost tools.

**Response** (200 OK):
```json
{
  "suggestions": [
    {
      "type": "dead_plugin",
      "plugin": "unused-skill",
      "message": "Skill 'unused-skill' has 0 invocations. Consider removing it.",
      "potential_saving_usd": null
    },
    {
      "type": "high_cost_tool",
      "tool": "Bash",
      "tokens": 782000,
      "pct_of_total": 34.2,
      "message": "Bash accounts for 34% of total tokens. Consider batching commands.",
      "potential_saving_usd": 12.50
    }
  ],
  "generated_at": "2026-03-30T09:00:00Z"
}
```

**Use Case**: Analytics > Discover sub-view

**Example**:
```bash
curl http://localhost:8080/api/analytics/suggestions | jq
```

---

### GET `/api/quota`

Returns current budget and quota status. Returns an error object if no budget is configured.

**Response** (200 OK):
```json
{
  "current_cost": 38.42,
  "budget_limit": 50.0,
  "usage_pct": 76.8,
  "projected_monthly_cost": 61.5,
  "projected_overage": 11.5,
  "alert_level": "warning"
}
```

**Fields**:
- `current_cost` (float): Month-to-date cost in USD
- `budget_limit` (float): Configured monthly budget in USD
- `usage_pct` (float): Percentage of budget consumed (0–100+)
- `projected_monthly_cost` (float): Forecasted end-of-month cost
- `projected_overage` (float): Forecasted overage vs budget (0 if under budget)
- `alert_level` (string): `"safe"` / `"warning"` / `"critical"` / `"exceeded"`

**Error Response** (when no budget configured):
```json
{ "error": "No budget configured or stats not loaded" }
```

**Configuration**: Set `budget.monthlyBudgetUsd` in `~/.claude/settings.json` to enable.

**Example**:
```bash
curl http://localhost:8080/api/quota | jq
```

---

### GET `/api/search`

Full-text search across all session content using SQLite FTS5.

**Query Parameters**:
- `q` (string, required): Search query (minimum 2 characters)
- `limit` (integer, optional): Maximum results to return (default: 50)

**Response** (200 OK):
```json
{
  "results": [
    {
      "session_id": "ea23759a-...",
      "path": "/Users/john/.claude/projects/-Users-john-code-myapp/ea23759a.jsonl",
      "project": "-Users-john-code-myapp",
      "first_user_message": "How do I implement authentication?",
      "snippet": "...implement <b>authentication</b> with JWT tokens...",
      "rank": 0.95
    }
  ],
  "total": 3,
  "query": "authentication"
}
```

**Fields**:
- `results` (array): Search results ranked by BM25 relevance
- `snippet` (string): Highlighted excerpt with `<b>` tags around matches
- `rank` (float): Relevance score (higher = more relevant)
- `total` (integer): Number of results returned

**Example**:
```bash
curl "http://localhost:8080/api/search?q=authentication&limit=10" | jq
```

---

### GET `/api/activity/violations`

Returns cross-session security violations feed (credential access, destructive commands).

**Query Parameters**:
- `min_severity` (string, optional): Minimum severity to return — `"Info"` (default, all), `"Warning"`, `"Critical"`
- `limit` (integer, optional): Maximum number of violations to return (default: 100)

**Response** (200 OK):
```json
{
  "violations": [
    {
      "session_id": "ea23759a-...",
      "timestamp": "2026-03-28T14:32:00Z",
      "severity": "Warning",
      "category": "CredentialAccess",
      "detail": "Read ~/.aws/credentials",
      "action_hint": "Verify no secrets were exposed; rotate credentials if in doubt"
    }
  ],
  "total": 12,
  "displayed": 12,
  "critical_count": 2,
  "warning_count": 7,
  "info_count": 3
}
```

**Severity levels**: `"Info"` → `"Warning"` → `"Critical"`

**Example**:
```bash
curl "http://localhost:8080/api/activity/violations?min_severity=Warning" | jq
```

---

### GET `/api/activity/{session_id}`

On-demand security analysis of a single session's tool calls. Results are cached in SQLite after the first call.

**Path Parameters**:
- `session_id` (string): Session UUID

**Response** (200 OK):
```json
{
  "session_id": "ea23759a-...",
  "file_accesses": [
    { "path": "~/.aws/credentials", "operation": "Read" }
  ],
  "bash_commands": [
    { "command": "rm -rf /tmp/foo", "risk_level": "Low" }
  ],
  "network_calls": [],
  "alerts": [
    {
      "severity": "Warning",
      "category": "CredentialAccess",
      "detail": "Read ~/.aws/credentials",
      "action_hint": "Verify no secrets were exposed"
    }
  ]
}
```

**Example**:
```bash
curl http://localhost:8080/api/activity/ea23759a-1234-5678-90ab-cdef01234567 | jq
```

---

### GET `/api/task-graph`

Returns the task dependency graph parsed from a project `PLAN.md` file. Searches `claudedocs/PLAN.md`, `.claude/PLAN.md`, and `~/.claude/claudedocs/PLAN.md` in order.

**Response** (200 OK, plan found):
```json
{
  "found": true,
  "plan_path": "/Users/john/code/myapp/claudedocs/PLAN.md",
  "phases": [...],
  "graph": {
    "nodes": [...],
    "edges": [...]
  }
}
```

**Response** (200 OK, no plan found):
```json
{ "found": false, "plan_path": null }
```

**Example**:
```bash
curl http://localhost:8080/api/task-graph | jq
```

---

### GET `/api/claude-mem/summaries`

Returns session summaries stored by the claude-mem integration (if enabled).

**Response** (200 OK):
```json
{
  "enabled": true,
  "summaries": [
    {
      "id": 1,
      "memory_session_id": "abc123",
      "project": "/Users/john/code/myapp",
      "request": "Implement user authentication",
      "completed": "Added JWT middleware and login/logout routes",
      "next_steps": "Add refresh token rotation",
      "files_edited": ["src/auth.rs", "src/routes.rs"],
      "created_at": "2026-03-28T14:00:00Z"
    }
  ],
  "total": 1
}
```

**Example**:
```bash
curl http://localhost:8080/api/claude-mem/summaries | jq
```

---

### POST `/api/claude-mem/toggle`

Enable or disable the claude-mem integration at runtime.

**Request Body**:
```json
{ "enabled": true }
```

**Response** (200 OK):
```json
{ "enabled": true }
```

**Example**:
```bash
curl -X POST http://localhost:8080/api/claude-mem/toggle \
  -H "Content-Type: application/json" \
  -d '{"enabled": true}' | jq
```

---

### GET `/api/insights`

Returns insights from `~/.ccboard/insights.db` — the cross-session knowledge base populated by the session-stop hook and `/ccboard-remember` skill.

**Query Parameters**:

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `project` | string | — | Filter by project path (exact match) |
| `type` | string | — | Filter by insight type: `progress`, `decision`, `blocked`, `pattern`, `fix`, `context` |
| `limit` | integer | 50 | Maximum number of results |
| `archived` | integer | 0 | Include archived insights (`1`) or not (`0`) |

**Response** (200 OK):
```json
{
  "insights": [
    {
      "id": 1,
      "session_id": "abc123",
      "project": "/Users/you/Sites/myproject",
      "type": "progress",
      "content": "Implemented Brain tab with filter bar and detail pane",
      "reasoning": null,
      "archived": false,
      "created_at": "2026-03-30T06:46:52Z"
    }
  ],
  "total": 1
}
```

**Use Case**: Brain tab — cross-session knowledge base

**Example**:
```bash
curl "http://localhost:8080/api/insights?type=blocked&limit=10" | jq
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
- **Multi-user**: Not currently supported (local-only design)

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

Planned endpoints:
- `GET /api/conversation/:session_id` — Full JSONL session content for the conversation viewer

---

**Last Updated**: 2026-03-30
**API Version**: v0.21.0
**Backend**: Axum + Tokio
**Frontend**: Leptos + WASM
