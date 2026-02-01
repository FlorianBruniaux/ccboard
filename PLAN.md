# Plan: ccboard — Unified Claude Code Management Dashboard

## Decisions prises

| Question | Choix |
|----------|-------|
| Interface | TUI (Ratatui) + Web UI (Axum + htmx) depuis un seul binaire |
| Audience | Dogfood perso, open-source rapidement |
| Stack | Rust |
| MVP scope | Full dashboard (read-only) |
| Nom | `ccboard` (Claude Code Board) |

## Architecture

```
ccboard/
  Cargo.toml                    # workspace root
  crates/
    ccboard-core/               # parsers, models, store, watcher
    ccboard-tui/                # Ratatui frontend (7 tabs)
    ccboard-web/                # Axum + Askama + htmx
    ccboard-cli/                # binary entry point (clap)
```

**Principe** : Single binary, deux frontends. `ccboard` (TUI par defaut), `ccboard web`, `ccboard both`.

### Data Layer partagee (ccboard-core)

Sources de donnees Claude Code a lire :

| Type | Format | Chemin | Parser |
|------|--------|--------|--------|
| Stats | JSON | `~/.claude/stats-cache.json` | serde_json direct |
| Settings global | JSON | `~/.claude/settings.json` | serde_json + merge |
| Settings project | JSON | `.claude/settings.json` | serde_json + merge |
| Settings local | JSON | `.claude/settings.local.json` | serde_json + merge |
| MCP config | JSON | `~/.claude/claude_desktop_config.json` | serde_json |
| Sessions | JSONL | `~/.claude/projects/<path>/<id>.jsonl` | streaming BufReader |
| Tasks | JSON | `~/.claude/tasks/<list-id>/<task-id>.json` | serde_json |
| Agents | MD + YAML frontmatter | `.claude/agents/*.md` | custom split + serde_yaml |
| Commands | MD + YAML frontmatter | `.claude/commands/*.md` | custom split + serde_yaml |
| Skills | MD + YAML frontmatter | `.claude/skills/*/SKILL.md` | custom split + serde_yaml |
| Hooks | Shell scripts | `.claude/hooks/bash/*.sh` | lecture + metadata settings |
| History | JSON | `~/.claude/statsCache` (hourCounts) | dans stats-cache |
| CLAUDE.md | Markdown | `~/.claude/CLAUDE.md` + `./CLAUDE.md` | texte brut |

**Config merge priority** : local > project > global > defaults

### Modules ccboard-core

```
src/
  models/
    session.rs       # SessionLine, SessionMessage, TokenUsage, SessionMetadata
    stats.rs         # StatsCache, DailyActivity, ModelUsage
    config.rs        # Settings, Permissions, HookGroup, HookDefinition, MergedConfig
    agent.rs         # AgentDef, AgentKind (Agent/Command/Skill)
    task.rs          # Task, TaskList
    mcp.rs           # McpConfig, McpServer
  parsers/
    jsonl.rs         # streaming JSONL (BufReader line-by-line, skip malformed)
    frontmatter.rs   # YAML between --- delimiters + serde_yaml
    settings.rs      # JSON parse + 3-level merge logic
    session_index.rs # decouverte sessions (flat .jsonl + directory format)
    stats.rs         # stats-cache.json direct parse
  store.rs           # DataStore avec Arc<RwLock<T>> par domaine
  watcher.rs         # notify crate, emet DataEvent (StatsChanged, SessionCreated, etc.)
  discovery.rs       # scan ~/.claude + project dirs
  error.rs           # thiserror types
```

### Structs cles

```rust
// Session (JSONL lines)
pub struct SessionLine {
    pub session_id: String,
    pub line_type: String,          // "user", "assistant", "file-history-snapshot"
    pub timestamp: DateTime<Utc>,
    pub cwd: Option<String>,
    pub git_branch: Option<String>,
    pub message: Option<SessionMessage>,
}

// Metadata extraite (premier + dernier line, pas full parse)
pub struct SessionMetadata {
    pub id: String,
    pub project_path: String,
    pub first_timestamp: DateTime<Utc>,
    pub last_timestamp: DateTime<Utc>,
    pub message_count: usize,
    pub models_used: Vec<String>,
    pub file_size_bytes: u64,
    pub first_user_message: Option<String>,  // 200 chars preview
    pub has_subagents: bool,
}

// Settings (JSON)
pub struct Settings {
    pub permissions: Option<Permissions>,
    pub hooks: Option<HashMap<String, Vec<HookGroup>>>,
    pub model: Option<String>,
    pub env: Option<HashMap<String, String>>,
    pub enabled_plugins: Option<HashMap<String, bool>>,
}

// Agent/Command/Skill (frontmatter)
pub struct AgentDef {
    pub file_path: String,
    pub name: String,
    pub description: Option<String>,
    pub model: Option<String>,
    pub tools: Option<String>,
    pub body: String,
    pub kind: AgentKind,  // Agent | Command | Skill
}

// DataStore (central, shared between TUI and Web)
pub struct DataStore {
    pub stats: Arc<RwLock<Option<StatsCache>>>,
    pub global_config: Arc<RwLock<Option<Settings>>>,
    pub project_configs: Arc<RwLock<HashMap<String, Settings>>>,
    pub sessions: Arc<RwLock<Vec<SessionMetadata>>>,
    pub agents: Arc<RwLock<Vec<AgentDef>>>,
    pub task_lists: Arc<RwLock<Vec<TaskList>>>,
    pub mcp_config: Arc<RwLock<Option<McpConfig>>>,
}
```

### TUI (ccboard-tui) — 7 tabs

```
  [1:Dashboard] [2:Sessions] [3:Config] [4:Hooks] [5:Agents] [6:Costs] [7:History]
```

| Tab | Contenu | Widgets |
|-----|---------|---------|
| Dashboard | Overview : sparkline 30j, model bar, peak hours, quick stats | Sparkline, Bar, Heatmap |
| Sessions | Arbre projets (gauche) + liste sessions (droite) + detail popup | Tree, List, Popup |
| Config | Vue 3 colonnes : Global / Project / Local + merged result | Table, Diff |
| Hooks | Arbre : EventName > matcher > hooks (command, async, timeout) | Tree |
| Agents | 3 sections : Agents / Commands / Skills avec frontmatter | List + Detail |
| Costs | Chart tokens daily par model + cache ratio + estimation USD | BarChart, Table |
| History | Prompts recents, filtre par projet, recherche texte | List + Search |

**Key bindings** : `Tab`/`Shift+Tab` nav tabs, `j/k` nav listes, `Enter` detail, `/` search, `r` refresh, `q` quit, `1-7` jump tabs.

### Web (ccboard-web) — htmx + Askama

**Choix htmx** (pas Leptos/Dioxus/SPA JS) : zero build pipeline JS, 14KB client, meme binary Rust, rendu serveur.

**Routes** :

```
GET /                          # Dashboard page
GET /sessions                  # Sessions browser
GET /sessions/{project}        # Sessions by project
GET /sessions/{project}/{id}   # Session detail
GET /config                    # Config viewer
GET /hooks                     # Hooks viewer
GET /agents                    # Agents/Commands/Skills
GET /costs                     # Cost dashboard
GET /history                   # Prompt history

GET /api/stats                 # JSON API
GET /api/sessions              # JSON API
GET /api/config/merged         # JSON API
GET /api/events                # SSE live updates
GET /static/*                  # Embedded assets (htmx.min.js, style.css)
```

### Binary (ccboard-cli)

```rust
#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    mode: Option<Mode>,       // tui (default), web, both
    #[arg(long)]
    claude_home: Option<PathBuf>,  // default ~/.claude
    #[arg(long)]
    project: Option<PathBuf>,      // focus on specific project
}
```

## Dependencies

```toml
# core
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1"
thiserror = "2"
notify = "7"
tokio = { version = "1", features = ["sync", "fs"] }
walkdir = "2"
dirs = "6"
tracing = "0.1"

# tui
ratatui = "0.30"
crossterm = "0.28"

# web
axum = "0.8"
askama = "0.13"
askama_axum = "0.5"
rust-embed = "8"
tower-http = { version = "0.6", features = ["cors"] }

# cli
clap = { version = "4", features = ["derive"] }
open = "5"
tracing-subscriber = "0.3"
```

## Phases de livraison

### Phase 1 : Core parsers + Dashboard TUI

1. Scaffolding workspace (4 crates, Cargo.toml)
2. `stats.rs` parser — stats-cache.json (serde direct, trivial)
3. `settings.rs` parser — JSON settings + merge 3 niveaux
4. `session_index.rs` — decouverte sessions (flat + directory format)
5. `DataStore::initial_load()`
6. TUI : Dashboard tab (sparkline, stats, model bar, heatmap)
7. TUI : event loop, tab switching skeleton (autres tabs "Coming soon")
8. Binary entry point `ccboard`

**Livrable** : `ccboard` affiche le dashboard avec donnees reelles.

### Phase 2 : Sessions + Config tabs

1. `jsonl.rs` streaming parser (BufReader, skip malformed)
2. `extract_metadata()` — premier/dernier line, pas full parse
3. TUI : Sessions tab (arbre projets + liste sessions + popup detail)
4. TUI : Config tab (3 colonnes + merge visualise)

**Livrable** : Navigation des 1100+ sessions par projet, vue config mergee.

### Phase 3 : Tabs restants TUI

1. `frontmatter.rs` parser (custom split + serde_yaml)
2. TUI : Hooks tab (arbre par event)
3. TUI : Agents tab (3 sections)
4. TUI : Costs tab (chart daily + model breakdown)
5. TUI : History tab (liste filtrable)

**Livrable** : TUI complet, 7 tabs fonctionnels.

### Phase 4 : File watcher + Web UI

1. `watcher.rs` (notify, debounce 500ms, emet DataEvent)
2. Wire watcher -> TUI refresh
3. Web : Axum router + Askama templates + htmx
4. Web : toutes les pages miroir du TUI
5. Web : SSE endpoint pour live updates
6. Binary : `ccboard web --port 3333` et `ccboard both`

**Livrable** : TUI + Web, auto-refresh sur changements fichiers.

### Phase 5 : Polish + Open Source

1. Estimation couts (pricing Anthropic * token counts)
2. Session full message viewer (pagine)
3. `ccboard stats` — mode one-shot terminal
4. README, LICENSE, screenshots, GIF
5. Tests CI, `cargo publish`
6. Cross-platform (Linux, macOS)

### Phase 6+ (post-MVP)

- Session resume (spawn `claude -r <id>`)
- Config editing (write settings.json)
- Skill/agent creation wizard
- MCP server health check
- Export rapports (PDF, JSON)
- Theme customization

## Decisions de trade-off

| Decision | Choix | Raison |
|----------|-------|--------|
| Web UI | htmx + Askama | Zero JS build, 14KB, meme binaire, MVP rapide |
| Frontmatter | Custom split + serde_yaml | Format trivial, pas besoin de crate dedie |
| MVP scope | **Read-only** | 80% de la valeur = voir les donnees. Write ajoute risques. Phase 6+ |
| Session resume | **Pas dans MVP** | Spawn CLI = surface de securite + complexite. Phase 6+ |
| Shared state | Arc<RwLock<T>> par domaine | Pas un seul giant lock. Reads >> writes |
| Session scanning | Lazy metadata | 2.5GB de sessions. Full parse au startup = inacceptable |

## Performance

- **Session scan** : metadata from first+last line only. Full parse on demand.
- **Parallelisme** : `tokio::spawn` par project directory pour scan initial. Target < 2s.
- **Memoire** : SessionMetadata en store, pas le contenu. Contenu charge a la demande.
- **File watcher** : debounce 500ms pour eviter refresh excessifs.
- **Stats cache** : deja pre-agrege par Claude Code. Parse once, watch changes.

## Testing

| Couche | Strategie |
|--------|-----------|
| Parsers (core) | Fixtures JSON/JSONL/MD reelles (sanitized). Tests unitaires serde. |
| Config merge | 3 fichiers reels -> assert priorite correcte |
| JSONL streaming | Fichier 100MB+ -> test regression perf |
| TUI | Ratatui `TestBackend` headless -> snapshot tests |
| Web | Axum `TestClient` -> assert 200 + content-type |
| Integration | `#[cfg(feature = "integration")]` avec ~/.claude reel |

## Verification post-implementation

```bash
# Phase 1
ccboard                          # Dashboard s'affiche avec vrais chiffres
cargo test -p ccboard-core       # Tous les parsers passent

# Phase 2
ccboard                          # Tab Sessions navigable, Config visible
cargo test --all                 # Tous tests passent

# Phase 3
ccboard                          # 7 tabs fonctionnels
cargo clippy --all-targets       # Zero warnings

# Phase 4
ccboard web --port 3333          # http://localhost:3333 affiche dashboard
ccboard both                     # TUI + Web simultanes
# Modifier un fichier .claude/ -> auto-refresh visible

# Phase 5
cargo test --all-features        # Integration tests inclus
ccboard stats                    # One-liner stats dans terminal
```
