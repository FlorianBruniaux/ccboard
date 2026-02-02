# Plan: ccboard — Unified Claude Code Management Dashboard

## État Actuel (2026-02-02)

**Version**: 0.2.0-alpha
**Branch**: `feat/phase-11-tokens-invocations`
**Status**: 🔄 **IN DEVELOPMENT** — Phase 11 en cours (token tracking + invocation counters)

### Métriques Vérifiées

| Métrique | Valeur | Statut |
|----------|--------|--------|
| **LOC totales** | ~11,000+ lignes | ✅ |
| **Crates** | 4 (ccboard, core, tui, web) | ✅ |
| **Tests** | 86 (67 core + 19 tui) | ✅ Corrigé (était "88") |
| **Clippy warnings** | 0 | ✅ |
| **TUI tabs** | 8 complets | ✅ |
| **Parsers (core)** | 7 (stats, settings, session_index, mcp_config, hooks, rules, task) | ✅ |
| **Parsers (TUI only)** | 1 (frontmatter agents - non partageable avec web) | ⚠️ Dette technique |
| **Initial load** | <2s (1000+ sessions) | ✅ |

### Phases Complétées

| Phase | Description | LOC | Date | Status |
|-------|-------------|-----|------|--------|
| **Phase 0** | Architecture & Planning | - | 2026-01-29 | ✅ |
| **Phase 1-5** | Core Implementation | ~8K | 2026-01-30 | ✅ |
| **Phase 6** | File Opening & MCP UI | +587 | 2026-02-02 | ✅ |
| **Phase 7** | MCP Tab Dédié | +643 | 2026-02-02 | ✅ |
| **Phase 8** | Marketplace Plugin | +120 | 2026-02-02 | ✅ |
| **Phase 9.1** | TUI Polish (Theme + UX) | +514 | 2026-02-02 | ✅ |
| **Phase 9.2** | Command Palette | +469 | 2026-02-02 | ✅ |
| **Phase 9.3** | Breadcrumbs + Icons | +282 | 2026-02-02 | ✅ |
| **Phase 9.4** | PgUp/PgDn + Components | +317 | 2026-02-02 | ✅ |
| **File Watcher** | Live Data Updates | +80 | 2026-02-02 | ✅ |
| **Phase 9.5** | UX Fixes & Improvements | +50 | 2026-02-02 | ✅ |
| **Phase 11** | Token Tracking + Invocations | TBD | 2026-02-02 | 🔄 EN COURS |

---

## Inventaire Features (Audit Code-Level)

### A. Ce qui EXISTE vraiment

| Catégorie | Détail | Vérifié |
|-----------|--------|---------|
| **4 crates** | ccboard (CLI), ccboard-core (data), ccboard-tui (8 tabs), ccboard-web (stub) | ✅ |
| **7 parsers (core)** | stats, settings, session_index, mcp_config, hooks, rules, task | ✅ |
| **1 parser (TUI only)** | frontmatter agents/commands/skills dans `agents.rs`, PAS dans core | ✅ |
| **8 tabs TUI** | Dashboard, Sessions, Config, Hooks, Agents, Costs, History, MCP | ✅ |
| **DataStore** | DashMap + RwLock + Moka cache + EventBus | ✅ |
| **File Watcher** | notify + debounce, events broadcast | ✅ |
| **Web API** | 4 routes: `/`, `/api/stats`, `/api/sessions`, `/api/health` | ✅ |
| **86 tests** | 67 core + 19 TUI (0 rendering) + 0 web | ✅ |

### B. Dead Code / Dette Technique

| Item | Statut | Impact |
|------|--------|--------|
| **session_content_cache** | `#[allow(dead_code)]` jamais utilisé | Bloque on-demand loading |
| **SSE routes** | `sse.rs` existe, zero route `/api/events` wired | Web live updates non fonctionnel |
| **CircuitBreaker** | Type défini, zero logique | Code mort |
| **TaskParser** | Parser OK, zero UI/store connection | Tasks invisibles |
| **Frontmatter parser** | Dans TUI pas core | Web ne peut pas servir agents |
| **Tokens per session** | Champ existe, toujours 0 | ❌ CRITIQUE - feature non implémentée |
| **invocation_count** | Hardcodé à 0 partout | ❌ CRITIQUE - feature non implémentée |
| **Global search** | TODO dans app.rs | Feature promise non livrée |
| **Leptos frontend** | Zero code, string "Coming soon" | Web mode non fonctionnel |

---

## Paysage Concurrentiel (2026-02-02)

### A. Concurrents DIRECTS : Rust TUI (Même Stack)

| Tool | Stars | Stack | Features Clés | Menace |
|------|-------|-------|---------------|--------|
| **agtrace** (lanegrid) | Nouveau (jan 2026) | **Rust, Ratatui, Tokio** | Context window viz, execution timeline, SQLite indexing, multi-provider, MCP integration, git worktree | **🔴 CRITIQUE** - même stack, plus innovant |
| **Claudelytics** (nwiizo) | ? | **Rust** | **9 tabs** TUI, watch mode, burn rate, peco fuzzy, CSV export, projections | **🔴 HAUTE** - plus de tabs que nous |

### B. Concurrents DIRECTS : Cost/Usage Trackers

| Tool | Stars | Stack | Features Clés | Menace |
|------|-------|-------|---------------|--------|
| **ccusage** | **10.3K** | TS/Node | Daily/monthly/session, `--live` burn rate, **MCP server**, 5h blocks, duplicate detection | 🔴 Leader incontesté |
| **Claude-Code-Usage-Monitor** | ~500 | Python/Rich | ML predictions, P90, multi-level alerts, plan detection | 🟡 Predictif unique |
| **VS Code Usage Tracker** | ? | TS Extension | Real-time tokens, burn rate, visual indicators | 🟢 IDE-only |

### C. Concurrents ADJACENTS

| Tool | Stack | Type | Notes |
|------|-------|------|-------|
| **Opcode** | Tauri+React | Desktop GUI wrapper | Interactif, pas monitoring. Checkpoints, custom agents, AGPL |
| **Crystal** | Electron | Desktop parallel sessions | Git worktree isolation, diff viewer, competitive exploration |
| **claudekit** | ? | Framework | 20+ agents, error blocking, checkpoints |
| **ccstatusline** | Rust | Statusline | 900 stars, 62 modules |
| **CCometixLine** | Rust | Statusline | 1.6K stars, git integration |

### D. Plateformes Multi-Provider (Enterprise Adjacent)

| Tool | Stars | Focus |
|------|-------|-------|
| **LiteLLM** | 10K+ | 100+ providers, budget limits, DB logging |
| **Helicone** | 5K+ | Agent tracing, prompt versioning, free tier |
| **Portkey** | 8K+ | AI Gateway, 200+ models, 300B tokens |

### E. MCP Ecosystem

| Tool | Type | Notes |
|------|------|-------|
| **MCP Inspector** (anthropic) | Web UI | Official, debugging |
| **MCP Registry** (anthropic) | Go backend | Discovery, preview |
| **mcptools** | CLI | Homebrew + cargo, production-ready |
| **mcp-debugger** | MCP Server | Step-through debugging |

---

## Matrice Features Complète

| Feature | ccboard | agtrace | Claudelytics | ccusage | Opcode |
|---------|---------|---------|-------------|---------|--------|
| **TUI dashboard multi-tab** | **8 tabs** | ✅ | **9 tabs** | ❌ | ❌ |
| **Rust single binary** | ✅ | ✅ | ✅ | ❌ | ❌ (Tauri) |
| **Config merge 3-level** | **✅ UNIQUE** | ❌ | ❌ | ❌ | ❌ |
| **Hooks viewer** | **✅ UNIQUE** | ❌ | ❌ | ❌ | ❌ |
| **MCP server status TUI** | **✅ UNIQUE** | ❌ | ❌ | ❌ | ❌ |
| **Agents/commands/skills browser** | ✅ | ❌ | ❌ | ❌ | ✅ (custom) |
| **Per-session tokens** | **❌ (0)** | ✅ | ✅ | ✅ | ? |
| **Live burn rate** | **❌** | ✅ | ✅ | ✅ | ✅ |
| **Context window viz** | ❌ | **✅ UNIQUE** | ❌ | ❌ | ❌ |
| **Execution timeline** | ❌ | **✅ UNIQUE** | ❌ | ❌ | ❌ |
| **SQLite indexing** | ❌ | ✅ | ❌ | ❌ | ❌ |
| **Multi-provider** | ❌ | ✅ | ❌ | ❌ | ❌ |
| **5h billing blocks** | ❌ | ? | ✅ | ✅ | ❌ |
| **ML predictions** | ❌ | ❌ | ❌ | ✅ (monitor) | ❌ |
| **Git worktree support** | ❌ | ✅ | ❌ | ❌ | ❌ |
| **MCP server integration** | ❌ | ✅ | ❌ | ✅ | ❌ |
| **File watcher EventBus** | **✅ UNIQUE** | ❌ | ❌ | ❌ | ❌ |
| **Dual TUI+Web** | **✅ UNIQUE** | ❌ | ❌ | ❌ | ❌ |
| **Conversation replay** | ❌ | ❌ | ❌ | ❌ | ✅ (interactif) |
| **Checkpoints/restore** | ❌ | ❌ | ❌ | ❌ | ✅ |
| **CSV/JSON export** | ❌ | ❌ | ✅ | ✅ | ❌ |
| **Watch mode realtime** | ❌ | ✅ | ✅ | ✅ | ❌ |

---

## Gap Analysis

### A. Avantages RÉELLEMENT Exclusifs

| Avantage | Concurrence la plus proche |
|----------|---------------------------|
| **Config merge viewer 3 niveaux** | ✅ Personne ne fait ça |
| **Hooks viewer** | ✅ Personne ne visualise les hooks |
| **MCP server status detection en TUI** | MCP Inspector = web only |
| **Dual TUI+Web single binary** | agtrace = TUI only, ccusage = CLI only |
| **File watcher → EventBus → multi-frontend** | Architecture unique |
| **Agents/commands/skills browser** | Opcode = création (pas browsing read-only) |

### B. Ex-Avantages (Perdus Face à la Concurrence)

| Ex-avantage | Qui l'a aussi |
|-------------|---------------|
| ~~Seul dashboard TUI multi-tab~~ | agtrace (TUI), Claudelytics (9 tabs!) |
| ~~Seul outil Rust~~ | agtrace, Claudelytics, CCometixLine, ccstatusline |
| ~~Seul monitoring Claude Code~~ | 15+ outils maintenant |

### C. Table Stakes Manquantes (BLOQUANT)

| Feature manquante | Nb d'outils qui l'ont | Urgence |
|-------------------|----------------------|---------|
| **Per-session token count** | 8+ outils | 🔴 CRITIQUE - sans ça on est pas crédible |
| **Live burn rate / watch mode** | 8+ outils | 🔴 CRITIQUE - standard du marché |
| **5h billing block tracking** | 6+ outils | 🟡 IMPORTANT - quota system Claude |
| **Model-specific cost breakdown** | 7+ outils | 🟡 IMPORTANT - basic expectation |
| **Export (CSV/JSON)** | 4+ outils | 🟢 NICE - analytics workflow |

### D. Opportunités de Différenciation (0-1 Outils)

| Opportunité | Outils existants | Impact potentiel |
|-------------|-----------------|------------------|
| **Distributed team sync** | 0 | TRÈS HAUT - plus gros gap du marché |
| **Browser collaborative dashboard** | 0 | TRÈS HAUT - on a déjà l'archi web |
| **Auto budget enforcement** | 1 (LiteLLM) | MOYEN |
| **Cross-ecosystem comparison** | 0 | MOYEN |
| **Anomaly detection** | 0 | MOYEN |

### E. Priorités RÉVISÉES

```
P0-BLOQUANT : Per-session tokens + Live burn rate
  → Sans ça, ccboard n'est PAS compétitif face à agtrace/Claudelytics/ccusage
  → 8+ outils le font déjà, c'est TABLE STAKES

P0-BLOQUANT : Invocation counters
  → Notre seul vrai différenciateur (agents/commands/skills browser)
  → est inutile sans comptage

P1 : 5h billing blocks + model cost breakdown
  → Standard du marché, 6-7 outils le font

P2 : Export + watch mode
  → Workflow analytics, attendu par les users

P3 : Conversation replay TUI
  → Killer feature, aucun TUI ne le fait (Opcode = GUI only)
  → Gros différenciateur si bien fait

LONG TERM : MCP server mode ccboard, team sync, web UI
```

---

## Positionnement Stratégique

### Tagline vs Concurrents

```
ccusage       = "combien je dépense"          (single-concern: costs)
agtrace       = "comment mon agent marche"    (single-concern: observability)
Claudelytics  = "ccusage en Rust"             (single-concern: costs+TUI)
Opcode        = "Claude Code with a GUI"      (wrapper interactif)

ccboard       = "tout ~/.claude dans un dashboard"
                (multi-concern: config+hooks+agents+mcp+costs+sessions)
```

### Moat (Fossé Défensif)

1. **Breadth**: Seul outil qui couvre config/hooks/agents/MCP/costs/sessions ensemble
2. **Dual frontend**: TUI + Web + API du même binary
3. **Architecture**: FileWatcher → EventBus → multi-consumer (scalable)
4. **Config expertise**: 3-level merge viewer = unique value pour debugging

### Risque : "Mile Wide, Inch Deep"

- **agtrace** fait 1 chose (observability) mais en profondeur
- **ccusage** fait 1 chose (costs) mais est le standard
- **ccboard** fait 8 choses mais superficiellement sur les P0 (tokens = 0, burn rate = absent)

**Action requise**: Combler les P0 (tokens, burn rate) pour ne pas être disqualifié, PUIS doubler sur nos différenciateurs (config, hooks, agents avec invocations).

---

## Décisions Stratégiques

| Question | Options | Recommandation |
|----------|---------|----------------|
| **Scope** | A. Claude Code only / B. Ecosystem (Code+Desktop+API) / C. Multi-provider | **A** pour maintenant, B plus tard |
| **Feature focus** | A. Deep monitoring / B. Broad dashboard / C. Les deux | **C** - combler P0 depth + garder breadth |
| **Web** | A. Drop / B. TUI-first + API JSON / C. TUI + Web full | **B** - API JSON fonctionne déjà, web defer |
| **MCP mode** | A. Non / B. Resources only / C. Full | **B** rapidement, C plus tard |
| **Positionnement** | A. "Swiss Army Knife" / B. "Config expert" / C. "Full observability" | **A** - "The complete Claude Code dashboard" |

---

## Roadmap

### Phase 11 (P0-BLOQUANT) : Tokens + Invocations + Burn Rate

**Status**: 🔄 EN COURS (2026-02-02)
**Durée estimée**: 2-3 jours
**Objectif**: Combler les table stakes critiques vs agtrace/Claudelytics/ccusage

#### 1. Token Tracking Alternatif (1 jour)

**Problème identifié**:
- Claude Code JSONL : champ `usage` est `null` dans tous les messages
- stats-cache.json : agrégats globaux uniquement, pas de tokens par session

**Solution**: Parser la structure JSONL réelle pour extraire tokens depuis tool results

**Tâches**:
- [ ] Analyser format JSONL pour trouver sources alternatives de tokens
- [ ] Implémenter parser de tokens depuis tool_results ou summary events
- [ ] Ajouter cache des tokens extraits (ne pas re-parser à chaque load)
- [ ] Update SessionMetadata avec tokens réels
- [ ] Tests avec fixtures JSONL réels

**Validation**:
```bash
ccboard
# Sessions tab → colonne tokens affiche valeurs > 0
```

#### 2. Invocation Counters (1-2 jours)

**Objectif**: Compter combien de fois chaque agent/command/skill a été invoqué

**Détection patterns**:
```rust
// Agents: via Task tool
if message.contains("Task tool") && message.contains("subagent_type") {
    extract_agent_name();
}

// Commands: via pattern /command
if message.starts_with('/') {
    extract_command_name();
}

// Skills: via Skill tool
if message.contains("Skill tool") {
    extract_skill_name();
}
```

**Tâches**:
- [ ] Créer InvocationStats structure dans models
- [ ] Implémenter session streaming pour détecter patterns
- [ ] Parser agent invocations (Task tool calls)
- [ ] Parser command invocations (/command pattern)
- [ ] Parser skill invocations (Skill tool)
- [ ] Cache résultats (recompute only on new sessions)
- [ ] Update AgentsTab pour afficher counters
- [ ] Ajouter tri par usage (most used first)
- [ ] Tests unitaires pour detection patterns

**Validation**:
```bash
ccboard
# Onglet Agents → voir "× 23" à côté de chaque command
# Agents triés par usage décroissant
```

#### 3. Live Burn Rate (0.5 jour)

**Objectif**: Mode watch avec calcul burn rate en temps réel

**Tâches**:
- [ ] Implémenter tracking de session active via file watcher
- [ ] Calculer tokens/minute sur fenêtre glissante
- [ ] Afficher burn rate dans Dashboard
- [ ] Ajouter projection coût/heure

**Validation**:
```bash
ccboard
# Dashboard → voir "Burn rate: 1,234 tokens/min" avec session active
```

#### 4. Performance Optimization (0.5 jour)

**Challenge**: Parsing 1000+ sessions peut être lent

**Solutions**:
- Incremental computation (compute only for new/modified sessions)
- Background processing (tokio spawn)
- Progress indicator dans TUI
- Cache persistent (save to ~/.claude/ccboard-cache.json)

**Tâches**:
- [ ] Implémenter incremental computation
- [ ] Add progress bar during initial compute
- [ ] Cache results to disk
- [ ] Background refresh on session changes

---

### Phase 12 (P1) : 5h Blocks + Export + MCP Server

**Durée estimée**: 3-4 jours
**Objectif**: Standard du marché + meta-différenciateur

#### 1. 5h Billing Block Tracking (1 jour)

**Objectif**: Tracker usage dans fenêtres de facturation Claude (5h blocks)

**Tâches**:
- [ ] Détecter blocks de 5h depuis timestamps sessions
- [ ] Calculer usage par block
- [ ] Alert quand proche limite block
- [ ] Afficher dans Costs tab

#### 2. Export CSV/JSON (1 jour)

**Objectif**: Analytics workflow pour users

**Formats**:
- Sessions export (CSV/JSON)
- Costs breakdown (CSV/JSON)
- Agents usage (CSV/JSON)

**Tâches**:
- [ ] Implémenter serializers
- [ ] Add export commands
- [ ] Tests de format output

#### 3. ccboard as MCP Server (2 jours)

**Objectif**: Exposer ccboard data via MCP protocol (resources only)

**Resources**:
- `ccboard://sessions` → Liste sessions
- `ccboard://stats` → Statistiques
- `ccboard://agents` → Agents avec invocations

**Tâches**:
- [ ] MCP server implementation
- [ ] Resource handlers
- [ ] Documentation
- [ ] Tests integration

---

### Phase 13 (P2) : Conversation Replay + Open Source

**Durée estimée**: 4-5 jours
**Objectif**: Killer feature unique + release publique

#### 1. Conversation Replay TUI (3 jours)

**Objectif**: Visualiser déroulement conversation message par message (UNIQUE en TUI)

**Features**:
- Navigation temporelle (message précédent/suivant)
- Affichage tool calls + results
- Syntax highlighting code blocks
- Search dans conversation

**Tâches**:
- [ ] Parser full JSONL pour replay
- [ ] UI conversation viewer
- [ ] Navigation keybindings
- [ ] Syntax highlighting
- [ ] Tests rendering

#### 2. Open Source Release (2 jours)

**Tâches**:
- [ ] Screenshots & GIF démo (avec tokens/invocations visibles)
- [ ] LICENSE file (MIT OR Apache-2.0)
- [ ] CONTRIBUTING.md + CODE_OF_CONDUCT.md
- [ ] GitHub Issues/PR templates
- [ ] CI/CD pipeline (matrix build Linux/macOS/Windows)
- [ ] Publish crates.io
- [ ] Annonces (r/rust, Twitter/X, HN)

---

### Phase 14+ : Web UI + Team Sync (Long-Term)

**Différé** - Focus sur TUI + API JSON d'abord

#### Ideas Backlog

**Tier 1 : Fix Dead Code + Wire Existing**

| Idée | Effort | Impact |
|------|--------|--------|
| Wire TaskParser au store + UI | Faible | Moyen |
| Wire SSE au router web | Très faible | Moyen |
| Activer session_content_cache (dead code) | Faible | Haut (débloque features) |
| Déplacer frontmatter parser dans core | Faible | Moyen (débloque web) |
| Wire global search aux tabs | Faible | Moyen |

**Tier 2 : Features P0/P1** (Couverts par Phases 11-12)

**Tier 3 : Différenciateurs**

| Idée | Effort | Impact |
|------|--------|--------|
| `ccboard doctor` diagnostic | Moyen | HAUT |
| Git commit ↔ session attribution | Haut | HAUT (unique) |
| Session bookmarks | Moyen | MOYEN |

**Tier 4 : Long-term / Speculative**

| Idée | Effort | Impact |
|------|--------|--------|
| **Distributed team sync** | Très haut | TRÈS HAUT (0 competitors) |
| **Web collaborative dashboard** | Très haut | TRÈS HAUT (on a l'archi) |
| Claude Desktop parser (SQLite) | Haut | MOYEN |
| Anthropic API billing réel | Haut | HAUT |
| Plugin system | Très haut | Long-term |
| Multi-machine sync | Très haut | Niche |
| Error pattern detection | Très haut | Incertain |

**Données ~/.claude Inexploitées**

| Path | Contenu | Priorité |
|------|---------|----------|
| `~/.claude/todos/` | Task lists | Wire TaskParser |
| `~/.claude/.credentials.json` | Auth status | Afficher dans Dashboard |
| `~/.claude/statsig/` | Feature flags | Quelles features actives |
| `~/.claude/memory/` | Memories | Si existe, afficher |
| `projects/*/context.json` | Metadata projet | Enrichir sessions |

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

**Sources de données**:
- `~/.claude/stats-cache.json` - Statistics (StatsParser)
- `~/.claude/settings.json` - Global settings (SettingsParser + 3-level merge)
- `.claude/settings.json` - Project settings
- `.claude/settings.local.json` - Local settings (highest priority)
- `~/.claude/claude_desktop_config.json` - MCP config
- `~/.claude/projects/<path>/<id>.jsonl` - Sessions (streaming parser)
- `.claude/agents/*.md` - Agents (frontmatter parser - TUI only)
- `.claude/commands/*.md` - Commands
- `.claude/skills/*/SKILL.md` - Skills
- `.claude/hooks/bash/*.sh` - Hooks

**DataStore**:
- `DashMap<String, SessionMetadata>` - Sessions (per-key locking)
- `parking_lot::RwLock<StatsCache>` - Stats (low contention)
- `parking_lot::RwLock<MergedConfig>` - Settings
- `Moka Cache` - Session content (LRU, on-demand)
- `tokio::broadcast` - EventBus (live updates)

**Performance**:
- Initial load <2s (1000+ sessions)
- Metadata-only scan (lazy full parse)
- File watcher with 500ms debounce
- Cache hit 99.9%

### TUI (ccboard-tui)

**Framework**: Ratatui 0.30 + Crossterm 0.28

**Components**:
- 8 tabs avec navigation complète
- Command palette (fuzzy matching)
- Breadcrumbs trail
- Shared UI components (ListPane, DetailPane, SearchBar)
- Theme system (StatusColor enum)
- Empty states builder pattern

**Keybindings**:
- `q` quit | `Tab`/`Shift+Tab` nav tabs | `1-8` jump tabs
- `j/k` or `↑/↓` nav lists | `h/l` or `←/→` nav columns
- `Enter` detail | `Esc` back/close | `/` search
- `e` edit file | `o` reveal in file manager | `r` refresh
- `:` command palette | `PgUp/PgDn` page nav

### Web (ccboard-web)

**Backend**: Axum 0.8 + Askama templates

**Routes**:
- `GET /` - Dashboard
- `GET /sessions` - Sessions browser
- `GET /config` - Config viewer
- `GET /hooks`, `/agents`, `/costs`, `/history`, `/mcp`
- `GET /api/stats` - JSON API
- `GET /api/events` - SSE live updates (backend ready, non wired)

**Frontend**: Leptos (0% implémenté - différé)

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

## Validation Stratégie (Next Actions)

- [ ] Tester agtrace et Claudelytics pour évaluer leur qualité réelle
- [ ] Vérifier si ccusage MCP server couvre le même scope
- [ ] Décider si conversation replay TUI justifie l'investissement
- [ ] Évaluer effort réel du per-session token parsing (analyser format JSONL)

---

## Contacts & Liens

- **Repo**: https://github.com/FlorianBruniaux/ccboard (à créer)
- **Crates.io**: https://crates.io/crates/ccboard (à publier)
- **License**: MIT OR Apache-2.0
- **Author**: Florian Bruniaux (@FlorianBruniaux)
