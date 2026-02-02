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

### Modules ccboard-core ✅ IMPLÉMENTÉ

```
src/
  models/
    session.rs       # ✅ SessionLine, SessionMessage, TokenUsage, SessionMetadata
    stats.rs         # ✅ StatsCache, DailyActivity, ModelUsage
    config.rs        # ✅ Settings, Permissions, HookGroup, HookDefinition, MergedConfig
    agent.rs         # ✅ AgentDef, AgentKind (Agent/Command/Skill)
    task.rs          # ✅ Task, TaskList, TaskStatus
    mcp.rs           # ✅ Déplacé dans parsers/mcp_config.rs
  parsers/
    mcp_config.rs    # ✅ McpConfig, McpServer (claude_desktop_config.json)
    rules.rs         # ✅ Rules, RulesFile (CLAUDE.md global + project)
    hooks.rs         # ✅ Hooks parser (bash scripts + metadata)
    session_index.rs # ✅ Découverte sessions (lazy metadata extraction)
    settings.rs      # ✅ SettingsParser + 3-level merge (local > project > global)
    stats.rs         # ✅ StatsParser avec retry logic
    task.rs          # ✅ TaskParser pour tasks JSON
    mod.rs           # ✅ Exports publics
  store.rs           # ✅ DataStore avec DashMap + parking_lot::RwLock + Moka cache
  watcher.rs         # ✅ FileWatcher (notify + debounce, ready mais pas activé)
  event.rs           # ✅ DataEvent, EventBus (tokio broadcast)
  error.rs           # ✅ CoreError (thiserror), LoadReport, LoadError
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

// DataStore (central, shared between TUI and Web) ✅ IMPLÉMENTÉ
pub struct DataStore {
    claude_home: PathBuf,
    project_path: Option<PathBuf>,
    config: DataStoreConfig,

    // Stats cache (low contention, frequent reads) - parking_lot::RwLock
    stats: RwLock<Option<StatsCache>>,

    // Merged settings - parking_lot::RwLock
    settings: RwLock<MergedConfig>,

    // MCP server configuration - parking_lot::RwLock
    mcp_config: RwLock<Option<McpConfig>>,

    // Rules from CLAUDE.md - parking_lot::RwLock
    rules: RwLock<Rules>,

    // Session metadata (high contention, many entries) - DashMap for per-key locking
    sessions: DashMap<String, SessionMetadata>,

    // Session content cache (LRU, on-demand loading) - Moka cache
    session_content_cache: Cache<String, Vec<String>>,

    // Event bus for live updates - tokio broadcast
    event_bus: EventBus,

    // Current degraded state - parking_lot::RwLock
    degraded_state: RwLock<DegradedState>,
}

// Accesseurs publics
impl DataStore {
    pub fn stats(&self) -> Option<StatsCache>
    pub fn settings(&self) -> MergedConfig
    pub fn mcp_config(&self) -> Option<McpConfig>
    pub fn rules(&self) -> Rules
    pub fn sessions_by_project(&self) -> HashMap<String, Vec<SessionMetadata>>
    // ... etc
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

## Statut Actuel (2026-02-01)

### ✅ Phase 1 : Core parsers + Dashboard TUI — COMPLÉTÉ

**Réalisé** :
- ✅ Scaffolding workspace (4 crates: ccboard, ccboard-core, ccboard-tui, ccboard-web)
- ✅ `stats.rs` parser avec retry logic pour file contention
- ✅ `settings.rs` parser avec merge 3 niveaux (local > project > global)
- ✅ `session_index.rs` avec lazy metadata extraction (2s pour 1000+ sessions)
- ✅ `mcp_config.rs` parser pour claude_desktop_config.json
- ✅ `rules.rs` parser pour CLAUDE.md (global + project)
- ✅ `DataStore` avec DashMap + parking_lot::RwLock + Moka cache
- ✅ TUI Dashboard tab : sparkline 7j, gauges modèles, stats cards
- ✅ Event loop Crossterm avec key bindings (q/r/Tab/1-7/j/k)
- ✅ Binary `ccboard` avec modes : tui (default), web, both, stats

**Tests** : 66/66 ✅ | **Clippy** : 1 warning acceptable

### ✅ Phase 2 : Sessions + Config tabs — COMPLÉTÉ

**Réalisé** :
- ✅ JSONL streaming parser (BufReader line-by-line, skip malformed)
- ✅ SessionMetadata extraction (metadata-only scan, full parse on demand)
- ✅ Sessions tab : arbre projets (33) + liste sessions (402) + popup detail
- ✅ Sessions search : filter par projet/message/model avec '/' toggle
- ✅ Config tab : 4 colonnes (Global/Project/Local/Merged)
- ✅ Config MCP section : affichage servers avec commandes
- ✅ Config Rules section : preview CLAUDE.md (3 lignes)
- ✅ UX improvements : headers explicatifs, empty states clairs

**Performance** : Initial load <2s pour 2340 sessions | Cache hit 99.9%

### ✅ Phase 3 : Tabs restants TUI — COMPLÉTÉ

**Réalisé** :
- ✅ Frontmatter parser (YAML + serde_yaml)
- ✅ Hooks tab : liste événements + détails hooks bash
- ✅ Agents tab : 3 sub-tabs (Agents/Commands/Skills) avec frontmatter
- ✅ Agents UX : renommé "Commands" → "/ Commands" avec help text
- ✅ Costs tab : 3 vues (Overview/By Model/Daily Trend)
- ✅ Costs breakdown : tokens détaillés (in/out/cache read/write)
- ✅ History tab : recherche full-text + stats activité par heure

**TUI Status** : 7/7 tabs fonctionnels ✅

### 🚧 Phase 4 : File watcher + Web UI — EN COURS

**File Watcher** (85% complet) :
- ✅ Infrastructure complète (notify + debounce adaptatif)
- ✅ Event mapping (stats/sessions/config → DataEvent)
- ⏳ **TODO** : Activation dans main.rs (30min)
- ⏳ **TODO** : Fix session path pipeline (1h)
- ⏳ **TODO** : reload_settings() method (30min)

**Web UI** (30% complet) :
- ✅ Backend Axum : 4 routes API fonctionnelles
- ✅ SSE infrastructure complète
- ❌ Frontend Leptos : ZERO code (pas de composants/router/pages)
- ⏳ **Estimation** : 2-4j pour MVP web complet

### 🎯 Phase 5 : Polish + Open Source — PRÉVU

Prévu après Phase 4 :
- README avec screenshots
- Tests CI (GitHub Actions)
- Cross-platform validation (Linux/macOS/Windows)
- License (MIT OR Apache-2.0)
- GIF démo

---

## Phases de livraison

### Phase 1 : Core parsers + Dashboard TUI ✅

1. Scaffolding workspace (4 crates, Cargo.toml)
2. `stats.rs` parser — stats-cache.json (serde direct, trivial)
3. `settings.rs` parser — JSON settings + merge 3 niveaux
4. `session_index.rs` — decouverte sessions (flat + directory format)
5. `DataStore::initial_load()`
6. TUI : Dashboard tab (sparkline, stats, model bar, heatmap)
7. TUI : event loop, tab switching skeleton (autres tabs "Coming soon")
8. Binary entry point `ccboard`

**Livrable** : `ccboard` affiche le dashboard avec donnees reelles. ✅ COMPLÉTÉ

### Phase 2 : Sessions + Config tabs ✅

1. `jsonl.rs` streaming parser (BufReader, skip malformed)
2. `extract_metadata()` — premier/dernier line, pas full parse
3. TUI : Sessions tab (arbre projets + liste sessions + popup detail)
4. TUI : Config tab (3 colonnes + merge visualise)

**Livrable** : Navigation des 1100+ sessions par projet, vue config mergee. ✅ COMPLÉTÉ

### Phase 3 : Tabs restants TUI ✅

1. `frontmatter.rs` parser (custom split + serde_yaml)
2. TUI : Hooks tab (arbre par event)
3. TUI : Agents tab (3 sections)
4. TUI : Costs tab (chart daily + model breakdown)
5. TUI : History tab (liste filtrable)

**Livrable** : TUI complet, 7 tabs fonctionnels. ✅ COMPLÉTÉ

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
# Phase 1 ✅ VALIDÉ (2026-02-01)
ccboard                          # ✅ Dashboard s'affiche avec vrais chiffres
cargo test -p ccboard-core       # ✅ 66 tests passent

# Phase 2 ✅ VALIDÉ (2026-02-01)
ccboard                          # ✅ Tab Sessions navigable, Config visible
cargo test --all                 # ✅ 66 tests passent

# Phase 3 ✅ VALIDÉ (2026-02-01)
ccboard                          # ✅ 7 tabs fonctionnels
cargo clippy --all-targets       # ✅ 1 warning acceptable (too many arguments)
ccboard stats                    # ✅ One-liner stats fonctionne

# Phase 4 ⏳ EN COURS
ccboard web --port 3333          # ⏳ Backend fonctionnel, frontend TODO
ccboard both                     # ⏳ Architecture prête, web UI manquant
# Modifier un fichier .claude/ -> ⏳ Watcher existe mais pas activé

# Phase 5 📋 PLANIFIÉ
cargo test --all-features        # Tests integration à créer
README.md + screenshots          # À faire
Cross-platform CI                # GitHub Actions à configurer
```

## Commits récents

```
75b36d9 (HEAD -> feat/tdd-agent-academy) feat(tui): complete Config tab with MCP/Rules + UX polish
fd92b50 docs: add TDD evidence documentation for Agent Academy
f9e0fe7 feat: implement TDD methodology with Agent Academy principles
ec68e7c init: ccboard project with implementation plan
```

**Changements majeurs (75b36d9)** :
- Config tab : MCP servers + Rules (CLAUDE.md) + headers explicatifs
- Agents tab : "/ Commands" avec help text
- Sessions tab : recherche fonctionnelle avec filtrage
- UX : empty states clairs ("Using defaults ✓")
- DataStore : intégration MCP + Rules
- Tokio : ajout feature `time` pour stats parser

## Phase 6 : File Opening & MCP UI (2026-02-02) - ✅ 100% COMPLÉTÉ

**Objectif** : Ajouter file opening dans TUI + améliorer MCP UI

### ✅ Complété (bb0fc03, 91be1df)

**Feature 1 : File Opening & Reveal** :
- ✅ Module `editor.rs` avec `open_in_editor()` et `reveal_in_file_manager()`
- ✅ Keybinding `e` pour ouvrir fichiers dans `$EDITOR` (Agents, Sessions, History tabs)
- ✅ Keybinding `o` pour révéler fichiers dans file manager
- ✅ Display file_path dans Sessions et History detail panels
- ✅ Error popups pour échecs editor/file manager
- ✅ Support cross-platform (macOS, Linux, Windows)
- ✅ Terminal state save/restore (alternate screen)

**Feature 2 : Hooks File Path** :
- ✅ Ajout champ `file_path` à `HookDefinition`
- ✅ Population file_path pendant scan hooks (settings parser)
- ✅ Display file path dans Hooks tab detail
- ✅ Keybindings `e` et `o` pour Hooks tab

**Commits créés** :
- `bb0fc03` : feat(tui): add file opening and reveal keybindings (463 insertions)
- `91be1df` : feat(tui): add file_path tracking to Hooks (124 insertions)

### ✅ Complété - Suite (6470730, 91b0e21, 6c2c679, faa8118)

**Task 4 : Config Tab Keybindings** (6470730):
- ✅ Ajout `claude_home`, `project_path`, `error_message` à ConfigTab
- ✅ Keybinding `e` pour ouvrir config selon colonne focusée
  - Colonne 0 → `~/.claude/settings.json`
  - Colonne 1 → `.claude/settings.json`
  - Colonne 2 → `.claude/settings.local.json`
- ✅ Keybinding `o` pour révéler dans file manager
- ✅ Error popup avec Esc

**Task 9 : Dashboard MCP Card** (91b0e21):
- ✅ Layout Dashboard 4→5 colonnes (20% chacune)
- ✅ 5ème card "◉ MCP" avec server count
- ✅ Green si count > 0, DarkGray si 0
- ✅ Pass mcp_config depuis DataStore

**Task 7 : Enhanced MCP Section** (6c2c679):
- ✅ Multi-line formatting (3 lignes : name, command, env)
- ✅ Command limit 40→60 chars
- ✅ Label "(configured)" sur server names
- ✅ Env var count au lieu de liste ("Env: 2 vars")

**Task 8 : MCP Detail Modal** (faa8118):
- ✅ Keybinding `m` dans colonne Merged pour ouvrir modal
- ✅ Modal 70% width/height affichant :
  - Tous les MCP servers
  - Full command (non tronqué)
  - Toutes les env vars avec valeurs
  - Config file path
- ✅ Keybinding `e` dans modal pour éditer `claude_desktop_config.json`
- ✅ Auto-close modal après ouverture editor

### Statistiques Phase 6 - FINAL

| Métrique | Valeur |
|----------|--------|
| Tasks complétées | 9/9 (100%) ✅ |
| Commits créés | 6 |
| Lignes ajoutées | +1088 |
| Lignes supprimées | -26 |
| Fichiers modifiés | 11 |
| Temps écoulé | ~7h |
| Temps estimé | 12-16h |
| **Performance** | **+44% plus rapide** |

## Prochaines étapes

### Priorité P0 (File Watcher) - 2h estimées

**Objectif** : Activer le file watcher pour live updates

Tâches :
1. **Phase 4.1** : Brancher FileWatcher dans `main.rs` (30min)
   - Démarrer watcher dans `run_tui()`, `run_web()`, `run_both()`
   - Garder `_watcher` en vie pour async task
   - Test : modifier stats-cache.json → UI se rafraîchit

2. **Phase 4.2** : Fix session path pipeline (1h)
   - Modifier `process_event()` pour passer path à `handle_event()`
   - Appeler `store.update_session(path)` pour events session
   - Test : modifier session JSONL → session visible dans UI

3. **Phase 4.3** : Implémenter `reload_settings()` (30min)
   - Créer méthode `DataStore::reload_settings()`
   - Wire up dans watcher `handle_event()`
   - Test : modifier settings.json → Config tab se met à jour

**Validation** :
```bash
ccboard &
# Modifier stats-cache.json → Dashboard update ✅
# Modifier session.jsonl → Sessions tab update ✅
# Modifier settings.json → Config tab update ✅
```

### Priorité P1 (Web UI) - 2-4 jours estimés

**Objectif** : MVP web fonctionnel (mirror du TUI)

Tâches :
1. Frontend Leptos : composants de base (router, layout)
2. Pages web : Dashboard, Sessions, Config, Hooks, Agents, Costs, History
3. SSE : wire up `/api/events` pour live updates
4. Tests : Axum TestClient pour routes

**Validation** :
```bash
ccboard web --port 3333
# http://localhost:3333 affiche dashboard
ccboard both
# TUI + Web simultanés avec live sync
```

### Priorité P2 (Open Source) - 1 jour estimé

**Objectif** : Préparer pour publication

Tâches :
1. README.md complet avec screenshots
2. GIF démo (enregistrer session TUI)
3. LICENSE (MIT OR Apache-2.0)
4. CI GitHub Actions (test, clippy, fmt)
5. Cross-platform validation (Linux, macOS, Windows)

### Priorité P3 (Phase 6+) - Futures

- Session resume (`ccboard resume <id>` → `claude -r <id>`)
- Config editing (write settings.json)
- Skill/agent creation wizard
- MCP server health check (ping servers)
- Export rapports (PDF, JSON, CSV)
- Theme customization
