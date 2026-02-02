# Plan: ccboard — Unified Claude Code Management Dashboard

## 📍 État Actuel du Projet (2026-02-02)

**Version** : 0.2.0-alpha
**Branch** : `main`
**Status** : 🎉 **PRODUCTION-READY** — Phases 0-9 + File Watcher complètes, prêt pour Open Source Release

### ✅ Phases Complétées (100%)

| Phase | Description | LOC | Date | PR |
|-------|-------------|-----|------|-----|
| **Phase 0** | Architecture & Planning | - | 2026-01-29 | - |
| **Phase 1-5** | Core Implementation | ~8K | 2026-01-30 | - |
| **Phase 6** | File Opening & MCP UI | +587 | 2026-02-02 | #1 |
| **Phase 7** | MCP Tab Dédié | +643 | 2026-02-02 | #1 |
| **Phase 8** | Marketplace Plugin | +120 | 2026-02-02 | #1 |
| **Phase 9.1** | TUI Polish (Theme + UX) | +514 | 2026-02-02 | #2 |
| **Phase 9.2** | Command Palette | +469 | 2026-02-02 | merged |
| **Phase 9.3** | Breadcrumbs + Icons | +282 | 2026-02-02 | merged |
| **Phase 9.4** | PgUp/PgDn + Components | +317 | 2026-02-02 | merged |
| **File Watcher** | Live Data Updates | +80 | 2026-02-02 | merged |
| **Phase 9.5** | UX Fixes & Improvements | +50 | 2026-02-02 | in-progress |

**Total** : ~11,000+ LOC | 88 tests passing | 0 clippy warnings

### 🔧 Phase 9.5 : UX Fixes & Improvements (2026-02-02)

**Changements** :
- ✅ **Costs tab keybindings** : `Tab/←→/h/l` au lieu de `1-3` (fix conflit navigation globale)
- ✅ **Session detail wrapping** : Texte renvoyé à la ligne pour paths/messages longs
- ✅ **Config hints** : Ajout "e edit │ o reveal" dans footer
- ✅ **AgentEntry structure** : Préparation champ `invocation_count` (comptage différé)

**Limitations identifiées** :
- ⚠️ **Tokens affichent 0** : Claude Code ne stocke pas `usage` dans JSONL (limitation upstream)
- 📊 **Comptage invocations** : Différé (parsing 1000+ sessions = performance intensive)

---

## 🎯 Fonctionnalités Actuelles

### TUI Dashboard (8 tabs complets)

1. **◆ Dashboard** : Vue d'ensemble (stats, models, MCP servers)
2. **● Sessions** : Navigateur de sessions avec recherche persistante
3. **⚙ Config** : Configuration complète (settings, MCP, hooks)
4. **▣ Hooks** : Gestion des hooks par type d'événement
5. **◉ Agents** : Browse agents/commands/skills
6. **💰 Costs** : Analyse des coûts par modèle/période
7. **⏱ History** : Recherche globale dans l'historique
8. **◈ MCP** : Gestion MCP servers avec status detection

### UX Polish (k9s/lazygit niveau)

**Navigation** :
- Command palette (`:` prefix) avec fuzzy matching
- Breadcrumbs trail : 📍 Dashboard > Tab > Context
- Tab icons (◆●⚙▣◉💰⏱◈) pour identification rapide
- PgUp/PgDn navigation (jump 10 items)
- Vim keybindings (hjkl) + arrow keys

**Visuel** :
- Palette de couleurs unifiée (Success/Error/Warning/Neutral/Focus/Important)
- Scrollbar indicators sur toutes les listes longues
- Empty states avec hints actionnables
- Persistent search bars dans Sessions/History

**Composants partagés** :
- `ListPane` : Liste réutilisable avec scrollbar
- `DetailPane` : Affichage de contenu avec word wrap
- `SearchBar` : Barre de recherche avec placeholder
- `CommandPalette` : Palette de commandes fuzzy
- `Breadcrumbs` : Navigation trail avec truncation

### Live Updates (File Watcher activé)

- ✅ Détection automatique des changements (500ms debounce)
- ✅ Stats updates → Dashboard refresh
- ✅ Session changes → Sessions tab update
- ✅ Config changes → Config tab reload
- ✅ Web mode → SSE push to browser (backend ready)

### Marketplace Plugin

- 6 commands : `/dashboard`, `/mcp-status`, `/costs`, `/sessions`, `/ccboard-web`, `/ccboard-install`
- Installation detection + cargo install wrapper
- Structure `skills/ccboard/` complète

---

## 📊 Métriques Projet

| Métrique | Valeur |
|----------|--------|
| **LOC totales** | ~11,000 lignes |
| **Fichiers créés** | 75 fichiers |
| **Crates** | 4 (ccboard, core, tui, web) |
| **Tests** | 88 (66 core + 22 tui) |
| **Clippy warnings** | 0 |
| **Build time** | <10s (release) |
| **Initial load** | <2s (1000+ sessions) |

---

## 🚀 Prochaines Étapes — Focus Features

### 🎯 Priorité Actuelle : Amélioration des fonctionnalités

**Décision** : Développement des features avant Open Source Release
**Rationale** : Produit plus mature et complet pour la première release publique

---

### Phase 11 : Token Tracking & Invocation Counters (Priorité 🔴 P0 - 2-3 jours)

**Objectif** : Afficher les tokens réels et compter les invocations des agents/commands/skills

#### 🎯 Features à Implémenter

##### 1. Token Tracking Alternatif (1 jour)

**Problème identifié** :
- Claude Code JSONL : champ `usage` est `null` dans tous les messages
- stats-cache.json : agrégats globaux uniquement, pas de tokens par session
- **Solution** : Extraire tokens depuis les messages de continuation/résumé

**Approche** :
```rust
// Stratégie 1: Parser tool_results pour extraire token info
// Les tool results peuvent contenir des messages système avec tokens
async fn extract_tokens_from_tool_results(session_path: &Path) -> u64 {
    // Stream JSONL, chercher tool_results avec token info
}

// Stratégie 2: Calculer depuis stats aggregate + proportions
// Si 10 sessions, 10K tokens total → ~1K par session (rough estimate)
async fn estimate_tokens_from_stats(session_id: &str, stats: &Stats) -> u64 {
    // Heuristique basée sur message count, duration, models
}
```

**Tâches** :
- [ ] Analyser format JSONL pour trouver sources alternatives de tokens
- [ ] Implémenter parser de tokens depuis tool_results ou summary events
- [ ] Ajouter cache des tokens extraits (ne pas re-parser à chaque load)
- [ ] Update SessionMetadata avec tokens réels
- [ ] Tests avec fixtures JSONL réels

**Validation** :
```bash
# Sessions tab doit afficher tokens > 0
ccboard
# Naviguer vers Sessions → vérifier colonne tokens
```

##### 2. Invocation Counters (1-2 jours)

**Objectif** : Compter combien de fois chaque agent/command/skill a été invoqué

**Détection patterns** :
```rust
// Agents: détection via Task tool
if message.contains("Task tool") && message.contains("subagent_type") {
    extract_agent_name();
}

// Commands: détection via pattern /command
if message.starts_with('/') {
    extract_command_name();
}

// Skills: détection via Skill tool
if message.contains("Skill tool") {
    extract_skill_name();
}
```

**Architecture** :
```rust
// Nouvelle structure dans store
pub struct InvocationStats {
    pub agents: HashMap<String, usize>,      // agent_name -> count
    pub commands: HashMap<String, usize>,    // command_name -> count
    pub skills: HashMap<String, usize>,      // skill_name -> count
    pub last_computed: DateTime<Utc>,
}

// Méthode dans DataStore
impl DataStore {
    pub async fn compute_invocations(&self) -> InvocationStats {
        // Stream toutes les sessions
        // Détecter patterns
        // Agréger compteurs
    }
}
```

**Tâches** :
- [ ] Créer InvocationStats structure dans models
- [ ] Implémenter session streaming pour détecter patterns
- [ ] Parser agent invocations (Task tool calls)
- [ ] Parser command invocations (/command pattern)
- [ ] Parser skill invocations (Skill tool)
- [ ] Cache résultats (recompute only on new sessions)
- [ ] Update AgentsTab pour afficher counters
- [ ] Ajouter tri par usage (most used first)
- [ ] Tests unitaires pour detection patterns

**UI Updates** :
```rust
// Dans agents.rs render
Line::from(vec![
    Span::styled(name, style),
    Span::styled(format!(" ({}×)", count), Style::default().fg(Color::DarkGray)),
])
```

**Validation** :
```bash
ccboard
# Onglet Agents → Commands → voir "× 23" à côté de chaque command
# Agents triés par usage décroissant
```

##### 3. Performance Optimization (0.5 jour)

**Challenge** : Parsing 1000+ sessions peut être lent

**Solutions** :
- Incremental computation (compute only for new/modified sessions)
- Background processing (tokio spawn)
- Progress indicator dans TUI
- Cache persistent (save to ~/.claude/ccboard-cache.json)

**Tâches** :
- [ ] Implémenter incremental computation
- [ ] Add progress bar during initial compute
- [ ] Cache results to disk
- [ ] Background refresh on session changes

**Validation** :
```bash
# Initial load avec 1000 sessions: <5s
# Subsequent loads: <1s (from cache)
time ccboard stats
```

---

### Phase 12 : Web UI MVP (Priorité 🟡 P1 - Différé)

**Status** : Backend 100% complet, frontend 0% (pas de composants Leptos)

**Objectif** : Interface web fonctionnelle miroir du TUI

#### Tâches

1. **Frontend Leptos basics** (1j)
   - Router setup (pages)
   - Layout component
   - Sidebar navigation
   - Theme provider

2. **Pages implementation** (1-2j)
   - Dashboard page
   - Sessions browser
   - Config viewer
   - Autres tabs (Hooks, Agents, Costs, History, MCP)

3. **SSE integration** (0.5j)
   - Wire `/api/events` endpoint
   - Live updates composant
   - Auto-refresh sur file changes

4. **Testing** (0.5j)
   - Axum TestClient pour routes
   - Integration tests

**Validation** :
```bash
ccboard web --port 3333
# http://localhost:3333 affiche dashboard ✅
ccboard both
# TUI + Web simultanés avec live sync ✅
```

---

### Phase 14+ : Advanced Features (Priorité 🟢 P2 - Futures)

**Possibilités d'évolution** :

1. **Session Management** (2-3j)
   - Resume session (`ccboard resume <id>`)
   - Open in Claude Code
   - Export session (JSON, Markdown)

2. **Config Editing** (1-2j)
   - Write settings.json
   - MCP server add/remove
   - Hook creation wizard

3. **Advanced MCP** (2j)
   - Server start/stop/restart
   - Test connection (MCP protocol handshake)
   - Auto-refresh status (polling 5s)
   - Windows support (tasklist)

4. **Analytics** (2j)
   - Export reports (PDF, CSV)
   - Cost trends analysis
   - Usage patterns visualization

5. **Customization** (1-2j)
   - Theme customization
   - Keybinding remapping
   - Column ordering

---

### Phase 13 : Open Source Release (Backlog - Différé)

**Status** : En attente de Phase 11 complète

**Objectif** : Publier ccboard sur GitHub + crates.io avec un produit mature

#### ✅ Tâches Complétées

1. **README.md** ✅ (434 lignes)
   - Introduction + motivation
   - Feature list complète
   - Installation (3 méthodes)
   - Quick start (4 modes)
   - Keybindings table
   - Architecture overview
   - Development guide

2. **Documentation de base** ✅
   - CHANGELOG.md (Phase 0-9.5)
   - PLAN.md (complet)
   - CLAUDE.md (guidance)

#### 🔲 Reste à Faire (4-6h)

**Bloqué par** : Attendre Phase 11 (tokens + invocations) pour produit plus mature

1. **Screenshots & Assets** (2h)
   - Capturer les 8 tabs avec données réelles
   - Command palette demo
   - Breadcrumbs navigation
   - GIF démo 30s (installation → navigation)
   - Tokens et invocations visibles dans screenshots

2. **Documentation légale** (30min)
   - LICENSE file (MIT OR Apache-2.0)
   - CONTRIBUTING.md
   - CODE_OF_CONDUCT.md

3. **GitHub setup** (1h)
   - Issues templates
   - Pull request template
   - Labels

4. **CI/CD Pipeline** (2h)
   - GitHub Actions workflow
   - Matrix build (Linux, macOS, Windows)
   - Release binaries

5. **Publish crates.io** (30min)
   - Metadata Cargo.toml
   - `cargo publish`

6. **Annonces** (30min)
   - r/rust post
   - Twitter/X
   - Hacker News

---

## Architecture Technique

### Stack

```
ccboard/
├── ccboard/               # Binary CLI (clap)
├── ccboard-core/          # Parsers, models, store, watcher
├── ccboard-tui/           # Ratatui frontend (8 tabs)
└── ccboard-web/           # Leptos + Axum (backend ready)
```

### Data Layer (ccboard-core)

**Sources de données** :
- `~/.claude/stats-cache.json` - Statistics (StatsParser)
- `~/.claude/settings.json` - Global settings (SettingsParser + 3-level merge)
- `.claude/settings.json` - Project settings
- `.claude/settings.local.json` - Local settings (highest priority)
- `~/.claude/claude_desktop_config.json` - MCP config
- `~/.claude/projects/<path>/<id>.jsonl` - Sessions (streaming parser)
- `.claude/agents/*.md` - Agents (frontmatter parser)
- `.claude/commands/*.md` - Commands
- `.claude/skills/*/SKILL.md` - Skills
- `.claude/hooks/bash/*.sh` - Hooks

**DataStore** :
- `DashMap<String, SessionMetadata>` - Sessions (per-key locking)
- `parking_lot::RwLock<StatsCache>` - Stats (low contention)
- `parking_lot::RwLock<MergedConfig>` - Settings
- `Moka Cache` - Session content (LRU, on-demand)
- `tokio::broadcast` - EventBus (live updates)

**Performance** :
- Initial load <2s (1000+ sessions)
- Metadata-only scan (lazy full parse)
- File watcher with 500ms debounce
- Cache hit 99.9%

### TUI (ccboard-tui)

**Framework** : Ratatui 0.30 + Crossterm 0.28

**Components** :
- 8 tabs avec navigation complète
- Command palette (fuzzy matching)
- Breadcrumbs trail
- Shared UI components (ListPane, DetailPane, SearchBar)
- Theme system (StatusColor enum)
- Empty states builder pattern

**Keybindings** :
- `q` quit | `Tab`/`Shift+Tab` nav tabs | `1-8` jump tabs
- `j/k` or `↑/↓` nav lists | `h/l` or `←/→` nav columns
- `Enter` detail | `Esc` back/close | `/` search
- `e` edit file | `o` reveal in file manager | `r` refresh
- `:` command palette | `PgUp/PgDn` page nav

### Web (ccboard-web)

**Backend** : Axum 0.8 + Askama templates

**Routes** :
- `GET /` - Dashboard
- `GET /sessions` - Sessions browser
- `GET /config` - Config viewer
- `GET /hooks`, `/agents`, `/costs`, `/history`, `/mcp`
- `GET /api/stats` - JSON API
- `GET /api/events` - SSE live updates

**Frontend** : Leptos (0% implémenté)

---

## Commits Récents

```
6539bdf (HEAD -> main) docs: update PLAN.md with Phase 9.2-4 and File Watcher completion
1c060b0 feat(core): Activate file watcher for live data updates
8f21e9c feat(tui): Add shared UI components library
5cfcac8 feat(tui): Add PgUp/PgDn navigation to scrollable tabs
97d16af feat(tui): Phase 9.3 - Breadcrumbs navigation trail
c5fabaa feat(tui): Phase 9.2 - Command Palette with fuzzy matching
414dcbb docs: update PLAN.md with current project status
```

---

## Décisions Architecture

| Décision | Choix | Raison |
|----------|-------|--------|
| Interface | TUI + Web (single binary) | Dogfooding, zero JS build |
| TUI Framework | Ratatui | Mature, immediate mode, performant |
| Web Backend | Axum + Askama | Type-safe, fast, SSE support |
| Web Frontend | Leptos | Reactive, Rust types, WASM, no JS pipeline |
| State | DashMap + parking_lot | Per-key locking + better fairness |
| Session scan | Lazy metadata | 2.5GB data, full parse inacceptable |
| MVP scope | Read-only | 80% value, write = risks/complexity |
| License | MIT OR Apache-2.0 | Standard Rust dual licensing |

---

## Performance Targets

| Métrique | Target | Actuel | Status |
|----------|--------|--------|--------|
| Initial load | <2s | <2s | ✅ |
| Session scan | 1000+/2s | 2340/1.8s | ✅ |
| Memory usage | <100MB | ~80MB | ✅ |
| Build time | <10s | ~8s | ✅ |
| File watcher debounce | 500ms | 500ms | ✅ |
| Cache hit rate | >95% | 99.9% | ✅ |

---

## Roadmap Visuel

```
✅ Phase 0-5   : Core + 7 tabs TUI
✅ Phase 6     : File opening & MCP UI improvements
✅ Phase 7     : MCP dedicated tab
✅ Phase 8     : Marketplace plugin
✅ Phase 9.1-4 : TUI polish (theme, UX, command palette, components)
✅ File Watcher: Live updates activation
🔴 Phase 10   : Open Source Release (NEXT - 1 day)
🟡 Phase 11   : Web UI MVP (2-4 days)
🟢 Phase 12+  : Feature enhancements (futures)
```

---

## Contacts & Liens

- **Repo** : https://github.com/FlorianBruniaux/ccboard (à créer)
- **Crates.io** : https://crates.io/crates/ccboard (à publier)
- **License** : MIT OR Apache-2.0
- **Author** : Florian Bruniaux (@FlorianBruniaux)
