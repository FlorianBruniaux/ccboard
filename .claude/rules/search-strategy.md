# Search Strategy

**Purpose**: Optimized code navigation for Rust workspace (ccboard) using semantic search and call graph analysis

**Context**: ccboard is a complex Rust workspace (4 crates, ~15K LOC) with distributed patterns across parsers, TUI/Web frontends, and concurrent data structures.

---

## Decision Tree

```
Need to find code?
├─ Exact pattern known (regex, precise name) → Grep tool (native Claude)
├─ Intent-based ("session parsing", "SQLite cache") → grepai search
├─ Known symbol (struct/function/trait) → grepai search → Read targeted
└─ Call graph (who calls X?) → grepai trace callers/callees
```

**Key difference from TypeScript**: No Serena MCP for Rust - use `grepai search` + `Read` for targeted code extraction.

---

## Workflow Optimized (Token Efficiency)

**1. Broad search**: `grepai search "intent description" --limit 5` (~800 tokens)
   - Example: `grepai search "session JSONL parser"`
   - Returns: `session_index.rs`, `stats.rs` snippets

**2. Targeted read**: `Read file_path` (~500-1000 tokens per file)
   - Only read files identified by grepai
   - Focus on specific functions/structs

**3. Additional context**: `grepai trace callers "function_name"` (~800 tokens)
   - Example: `grepai trace callers "reload_stats"`
   - Shows TUI tabs, Web routes, file watcher usage

**4. Dependency analysis**: `grepai trace callees "function_name"` (~800 tokens)
   - Example: `grepai trace callees "DataStore::initial_load"`
   - Shows all parsers called during initialization

### Workflow Comparison

| Workflow                   | Tokens | Status              |
| -------------------------- | ------ | ------------------- |
| **Grep brute + Read**      | ~15K   | High noise          |
| **grepai + Read (opt)**    | ~4K    | 🎯 Recommended      |
| **grepai only**            | ~2-3K  | Fast but may miss context |

**Real reduction tested**: 73% vs traditional (measured on MethodeAristote TypeScript codebase)

---

## Tool Selection Matrix

| Tool             | ✅ Use when                                                                              | ❌ Do NOT use                                   |
| ---------------- | --------------------------------------------------------------------------------------- | ----------------------------------------------- |
| **Grep (native)** | Exact regex pattern, file search by name, performance critical (~20ms)                 | Intent-based search, complex navigation         |
| **grepai**       | Intent search, call graph, feature discovery, distributed patterns, architectural analysis | Exact pattern known (too slow ~2s), simple reads |
| **Read**         | After grepai identifies targets, full file context needed, understanding implementation | Broad exploration (use grepai first)            |

---

## Main Commands

### Grep (native)

```bash
Grep: pattern="impl.*Parser" output_mode="files_with_matches"
Grep: pattern="anyhow::Result" in path="crates/ccboard-core/src/"
Grep: pattern="parking_lot::RwLock" output_mode="content" -C 3
```

### grepai

```bash
# Semantic search
grepai search "session parsing logic"
grepai search "SQLite cache implementation"
grepai search "parking_lot RwLock usage"

# Call graph analysis
grepai trace callers "reload_stats"
grepai trace callees "DataStore::initial_load"
grepai trace graph "parse_session" --depth 2

# Index management
grepai status
grepai index  # Re-index after major changes
```

### Read (targeted)

```bash
# After grepai identifies targets
Read: file_path="crates/ccboard-core/src/parsers/session_index.rs"
Read: file_path="crates/ccboard-core/src/store.rs" offset=50 limit=100
```

---

## ccboard Patterns (Rust Workspace)

### Parser Discovery

**Goal**: Find all parsers and understand parsing strategy

**Workflow**:
1. `grepai search "JSONL streaming parser"` → Identifies `session_index.rs`, `stats.rs`
2. `Read crates/ccboard-core/src/parsers/session_index.rs` → Understand metadata extraction
3. `grepai trace callers "parse_session_metadata"` → See who calls it (DataStore)

**Expected results**:
- `stats.rs` (JSON parser with retry logic)
- `session_index.rs` (JSONL streaming, lazy loading)
- `frontmatter.rs` (YAML split + serde_yaml)

### DataStore Architecture

**Goal**: Understand DataStore concurrency model and update flow

**Workflow**:
1. `grepai search "DataStore reload stats"` → Finds `store.rs`
2. `Read crates/ccboard-core/src/store.rs` → See Arc + RwLock patterns
3. `grepai trace callers "reload_stats"` → TUI tabs + Web routes + file watcher
4. `grepai trace callees "reload_stats"` → Stats parser dependencies

**Expected results**:
- **Callers**: `tui/tabs/dashboard.rs`, `web/routes/stats.rs`, `core/watcher.rs`
- **Callees**: `parsers/stats.rs`, error handling chains

### Concurrency Patterns

**Goal**: Find all parking_lot::RwLock usage and Arc patterns

**Workflow**:
1. `grepai search "parking_lot RwLock"` → Identifies all concurrent data structures
2. `Read` targeted files → Understand locking strategy
3. `grepai search "Arc<DataStore>"` → See shared ownership patterns

**Expected results**:
- `store.rs` (RwLock for stats/settings)
- `tui/app.rs` (Arc<DataStore> shared)
- `web/routes.rs` (Arc<DataStore> in Axum state)

### Error Handling Chains

**Goal**: Trace anyhow context propagation

**Workflow**:
1. `grepai search "anyhow context propagation"` → Find error handling patterns
2. `Grep pattern="\.context\(" output_mode="content" -C 2` → See all context calls
3. `grepai trace callees "initial_load"` → Follow error chain

**Expected results**:
- Parsers: `.context("Failed to parse X")?` patterns
- DataStore: `.context("Failed to load stats")?` chains
- Graceful degradation: `LoadReport` population

---

## Token Optimization (Metrics)

### Impact Measured on ccboard

| Operation                  | Traditional Workflow | Optimized Workflow | Reduction |
| -------------------------- | -------------------- | ------------------ | --------- |
| **Code exploration**       | ~15K tokens          | ~3.5K tokens       | 77%       |
| **Dependency tracing**     | ~10K tokens          | ~2K tokens         | 80%       |
| **Parser discovery**       | ~5K tokens           | ~1K tokens         | 80%       |
| **Architecture analysis**  | ~8K tokens           | ~2.5K tokens       | 69%       |

**Estimated long session (30 min)**:
- Traditional: Context window at 80% (risk /compact)
- Optimized: Context window at 50% (comfortable margin)

### Token-Saving Rules

1. **Always**: `grepai search` BEFORE `Read` (avoid reading full files blindly)
2. **Never**: Read entire file if grepai can extract snippet context
3. **Prefer**: grepai search focused on 1-2 keywords (not long phrases)
4. **Limit**: `grepai trace graph --depth 2` (not depth >3 except complex debugging)
5. **Combine**: Use Grep for exact patterns, grepai for intent (don't mix tools inefficiently)

---

## Troubleshooting

| Problem                       | Solution                                                                  |
| ----------------------------- | ------------------------------------------------------------------------- |
| "connection refused" Ollama   | Relaunch `.claude/scripts/grepai-start.sh` or `brew services start ollama` |
| Empty/stale index             | `grepai index` to re-index completely                                     |
| Non-relevant results          | Check embedding model: `ollama list` (must have nomic-embed-text)         |
| grepai not found              | Install: `curl -sSL https://raw.githubusercontent.com/yoanbernabeu/grepai/main/install.sh \| sh` |
| Index not updating            | Ensure `grepai watch` is running (check with `ps aux \| grep grepai`)    |

---

## Initial Setup

**Helper script**: `bash .claude/scripts/grepai-start.sh`

**Manual**:

```bash
# 1. Install grepai
curl -sSL https://raw.githubusercontent.com/yoanbernabeu/grepai/main/install.sh | sh

# 2. Initialize project (in ccboard root)
cd /Users/florianbruniaux/Sites/perso/ccboard
grepai init  # Choose: ollama, nomic-embed-text, gob

# 3. Index the project
grepai index

# 4. Verify status
grepai status
```

**Expected index size for ccboard**:
- Files indexed: ~80-100 (Rust sources)
- Chunks: ~500-800
- Symbols: ~400-600

---

## MCP Integration

**Configuration**: See `.mcp.json` at project root

**Usage in Claude Code**:
- `grepai_search(query="intent description", limit=5)`
- `grepai_trace_callers(symbol="function_name")`
- `grepai_trace_callees(symbol="function_name")`
- `grepai_trace_graph(symbol="function_name", depth=2)`
- `grepai_index_status()` - Check health before using tools

**When to use MCP vs CLI**:
- **MCP tools**: When working in Claude Code interactive session
- **CLI**: When testing setup, debugging index, manual exploration

---

## ccboard-Specific Use Cases

### Understanding Architecture
- `grepai search "DataStore Arc RwLock"` → Find concurrency patterns
- `grepai trace callers "initial_load"` → See all entry points (TUI, Web)

### Tracing Error Propagation
- `grepai search "anyhow context"` → Find error handling sites
- `Grep pattern="\.context\(" -C 3` → See context messages

### Discovering Parsers
- `grepai search "parser JSONL"` → Find session parser
- `grepai search "parser frontmatter"` → Find agents/commands/skills parser

### Modifying Shared State
- `grepai trace callers "reload_stats"` → Who triggers reloads?
- `grepai trace callees "update_session"` → What gets updated downstream?

### Performance Analysis
- `grepai search "SQLite cache"` → Find cache implementation
- `grepai search "lazy loading"` → Find on-demand loading patterns

---

**Auto-loaded**: This file is referenced from `CLAUDE.md` for quick reference during development.
