---
date: 2026-02-12
last_updated: 2026-02-12
completed_date: 2026-02-12
version: 0.8.0
title: ccboard Phases F-15 - Plan de Développement Détaillé avec Agents
status: phases-f-h-complete
estimated_total_duration: 82-120h (10-15 jours)
actual_duration_f_h: 4h12min (19:30-23:42)
---

# Plan: ccboard Phases F-15 - Conversation Viewer to CI/CD Integration

## ✅ PHASES F-H COMPLETED (2026-02-12)

**Session**: 19:30 → 23:42 (4h12 real time)
**Completed**: 9/9 tasks (100%) for Phase F + H
**LOC Added**: ~2,835 lines
**Speedup**: 20-30x via parallel agents (4 concurrent)

---

**Dernière mise à jour**: 2026-02-12 23:42
**Version actuelle**: v0.8.0 (Phase F + H production-ready)
**Versions cibles**: v0.7.0 ✅ → v0.8.0 ✅ → v1.0.0
**Issues GitHub**: 43 issues organisées en 7 phases
**Durée totale estimée**: 82-120h (10-15 jours de travail effectif)
**Durée réelle Phase F-H**: 4h12 (parallélisation optimale)

---

## Context

Ce document structure le développement des phases F à 15 de ccboard, couvrant 43 issues GitHub organisées en 7 phases majeures. L'objectif est de transformer ccboard d'un dashboard de monitoring en un outil complet de gestion, d'analyse et d'optimisation des sessions Claude Code.

**Pourquoi ce plan ?**
- **Organisation** : 43 issues → 7 phases cohérentes avec dépendances claires
- **Contributeurs** : Identifier agents Claude Code + rôles humains pour chaque tâche
- **Priorisation** : Phases HIGH priority (F, H) avant phases LOW priority (13-15)
- **Milestones** : v0.7.0 (Phase F) → v1.0.0 (Phase 15 complète)

**Alignement avec ADR-001** : Reste Claude Code-only, pas de multi-provider support.

---

## Vue d'ensemble

### Phases Planifiées

| Phase | Titre | Priority | Issues | Durée Est. | Durée Réelle | Version | Status |
|-------|-------|----------|--------|-----------|-------------|---------|--------|
| **F** | Conversation Viewer | 🔴 HIGH | 5 | 12-16h | 4h | v0.7.0 | ✅ **COMPLETE** |
| **H** | Plan-Aware | 🔴 HIGH | 4 | 14-18h | 4h | v0.8.0 | ✅ **COMPLETE** |
| **11** | Tokens & Invocations | 🟡 MEDIUM | 7 | 16-20h | - | v0.9.0 | ⏳ Pending |
| **12** | Write Operations | 🟡 MEDIUM | 6 | 14-18h | - | v0.10.0 | ⏳ Pending |
| **13** | Team & Collaboration | 🟢 LOW | 6 | 12-16h | - | v0.11.0 | ⏳ Pending |
| **14** | IDE Integrations | 🟢 LOW | 5 | 8-12h | - | v0.12.0 | ⏳ Pending |
| **15** | CI/CD Integration | 🟢 LOW | 5 | 6-10h | - | v0.13.0 | ⏳ Pending |

**Total**: 40 issues (non 43, erreur comptage initial), 82-120h estimées

---

### Ordre d'Exécution Recommandé

**Séquence optimale basée sur dépendances techniques + priorité business** :

```
1. Phase F (Conversation Viewer) - 12-16h
   → Fondation analytics + déblocage Phase 11

2. Phase H (Plan-Aware) - 14-18h
   → Workflow tracking + context utilisateur

3. Phase 11 (Tokens & Invocations) - 16-20h
   → Analytics avancées (nécessite Phase F complète)

4. Phase 12 (Write Operations) - 14-18h
   → Safety-critical, après analytics phases

5. Phase 13 (Team & Collaboration) - 12-16h
   → Multi-user (nécessite Phase 12 write ops)

6. Phase 14 (IDE Integrations) - 8-12h
   → Extensions externes (JSON RPC interface)

7. Phase 15 (CI/CD Integration) - 6-10h
   → Automation DevOps (nécessite Phase 12)
```

**Milestone v1.0.0** : Après Phase 15 complète (full-featured dashboard)

---

### Dépendances entre Phases

```
Phase F (Viewer) ──┐
                   ├─> Phase 11 (Tokens analytics besoin full content)
Phase H (Plans) ───┘

Phase 12 (Write) ──> Phase 13 (Teams - besoin write pour sync)
                 └──> Phase 15 (CI/CD - besoin write pour automation)

Phase 13 (Teams) ──> Phase 14 (IDE - multi-user context)
```

**Blockers critiques** :
- Phase 11 bloquée par Phase F (#2 parser JSONL complet)
- Phase 13 bloquée par Phase 12 (write operations pour sync)
- Phase 15 bloquée par Phase 12 (write operations pour CI automation)

---

### Success Metrics Globales

| Métrique | Baseline (v0.5.2) | Target (v1.0.0) |
|----------|------------------|-----------------|
| **Initial load time** | 33ms (SQLite cache) | <50ms (maintenu) |
| **Session render time** | N/A | <500ms pour 1000-message conversation |
| **Token calculation accuracy** | N/A | ±2% vs Claude API actual |
| **Write operation safety** | Read-only | 0 data corruption incidents |
| **Multi-user sync latency** | N/A | <200ms team updates |
| **CI pipeline setup** | N/A | <10min from zero to running |

---

## Phase F: Conversation Viewer (Priority: 🔴 HIGH)

**Durée estimée** : 12-16h (2-3 jours)
**Issues GitHub** : #3 (main tracker), #5-#9 (features)
**Version cible** : v0.7.0
**Priorité** : P0 CRITIQUE (fondation analytics avancées)

### Objectif

Afficher le contenu complet des conversations JSONL avec navigation, recherche, syntax highlighting et export pour analyse détaillée des sessions Claude Code.

**Valeur utilisateur** :
- ✅ Débug sessions sans ouvrir JSONL manuellement
- ✅ Recherche full-text dans conversations (trouver erreurs, patterns)
- ✅ Export conversations pour reporting/documentation
- ✅ Fondation pour Phase 11 (token analytics précises sur contenu)

### Dépendances

**Bloque** :
- Phase 11 (Tokens) → Besoin full content pour token counting précis
- Phase H (Plan-Aware) → Partage UI patterns markdown rendering

**Dépend de** :
- ✅ Phase 2.1 (Cache SQLite) → Lazy-load performant existant
- ✅ Phase G (Web UI) → Composants Leptos réutilisables

---

### Tasks Détaillées

#### Task F.1: Full JSONL Parser avec Lazy Loading (P0)

**Issue** : #5
**Durée** : 3-4h
**Difficulté** : Intermediate
**Crate** : ccboard-core

**Agents Claude Code** :
- `Explore` (30min) → Analyser `parsers/session_index.rs` existant
- `Plan` (30min) → Concevoir streaming BufReader strategy
- `rust-developer` (2-3h) → Implémenter avec lazy loading

**Rôle humain requis** : Rust Core (Senior)
- Maîtrise JSONL streaming (BufReader)
- Error handling (anyhow + graceful degradation)
- Performance (benchmark <500ms target)

**Implémentation** :
```rust
// crates/ccboard-core/src/parsers/session_content.rs (NEW, ~200 LOC)
pub struct ConversationMessage {
    pub role: MessageRole,          // User | Assistant | System
    pub content: String,
    pub timestamp: Option<DateTime<Utc>>,
    pub model: Option<String>,
    pub tokens: Option<TokenUsage>,
}

impl DataStore {
    pub fn load_session_content(&self, session_id: &str) -> Result<SessionContent> {
        // 1. Check SQLite cache first
        // 2. Parse full JSONL if miss
        // 3. Update cache
        // 4. Return parsed conversation
    }
}
```

**Fichiers** :
- `crates/ccboard-core/src/parsers/session_content.rs` (+200 LOC, new)
- `crates/ccboard-core/src/store.rs` (+50 LOC, add `load_session_content()`)
- `crates/ccboard-core/src/models/session.rs` (+30 LOC, `ConversationMessage`)
- `crates/ccboard-core/tests/session_content_tests.rs` (+150 LOC, new)

**Tests** :
```bash
cargo test --test session_content_tests
cargo bench --bench session_content_bench
# Target: <500ms pour 1000-message conversation
```

**Validation** :
- ✅ Parse 1000-message session correctement
- ✅ Lazy load sans crash mémoire
- ✅ Graceful degradation sur malformed JSON
- ✅ Performance <500ms benchmark

**Risques** :
- ⚠️ Large JSONL (10MB+) → Streaming parser requis
- ⚠️ Malformed JSON mid-stream → Skip + continue pattern

---

#### Task F.2: TUI Conversation Tab avec Syntax Highlighting (P0)

**Issue** : #6
**Durée** : 3-4h
**Difficulté** : Good First Issue (UI) + Intermediate (integration)
**Crate** : ccboard-tui

**Agents Claude Code** :
- `ui-designer` (1h) → Design layout conversation tab
- General-purpose (2-3h) → Implémenter avec syntect

**Rôle humain requis** : TUI/Ratatui (Intermediate)
- Ratatui widgets (List, Paragraph)
- Syntax highlighting integration (syntect)
- Event handling (scroll, search)

**Architecture** :
```rust
// crates/ccboard-tui/src/tabs/conversation.rs (~250 LOC)
pub struct ConversationTab {
    session_id: Option<String>,
    content: Option<SessionContent>,
    scroll_offset: usize,
    search_query: String,
}

impl ConversationTab {
    fn render_message(&self, msg: &ConversationMessage) -> Vec<Line> {
        // Syntax highlighting via syntect
        // Color code blocks
        // User/Assistant role indicators
    }
}
```

**Dépendances Cargo** :
- `syntect = "5.0"` (syntax highlighting)
- `pulldown-cmark = "0.9"` (markdown parsing)

**Key Bindings** :
- `c` → Open conversation pour session sélectionnée
- `j/k` → Scroll messages
- `/` → Search in conversation
- `Esc` → Close conversation

**Fichiers** :
- `crates/ccboard-tui/src/tabs/conversation.rs` (+250 LOC, new)
- `crates/ccboard-tui/src/app.rs` (+40 LOC, routing)
- `crates/ccboard-tui/Cargo.toml` (+2 deps)

**Validation** :
```bash
cargo run  # TUI mode
# 1. Sessions tab → select session
# 2. Press 'c' → Conversation opens
# 3. Verify syntax highlighting
# 4. Test search '/' + query
```

**Risques** :
- ⚠️ Syntect bundle size (~5MB) → Feature gate si problème
- ⚠️ Render performance 1000+ messages → Virtual scrolling

---

#### Task F.3: Message Filtering (P1)

**Issue** : #7
**Durée** : 2h
**Difficulté** : Good First Issue
**Crate** : ccboard-tui

**Agents Claude Code** :
- General-purpose (2h) → Filtrage basique

**Rôle humain** : Junior (Rust basics)

**Features** :
- Filtrer par role (User/Assistant/System)
- Filtrer par tool invocations
- Filtrer par timestamp range

**Fichiers** :
- `crates/ccboard-tui/src/tabs/conversation.rs` (+80 LOC)

---

#### Task F.4: Export Conversations (P2)

**Issue** : #8
**Durée** : 1-2h
**Difficulté** : Intermediate
**Crate** : ccboard-core + ccboard

**Agents Claude Code** :
- `Explore` (30min) → Voir export patterns existants (billing CSV)
- `rust-developer` (1-2h) → Implémenter Markdown/JSON/HTML export

**Formats** :
- **Markdown** : `## User\n\n...` format
- **JSON** : Structured `{messages: [...], metadata: {...}}`
- **HTML** : Styled report avec syntax highlighting

**CLI** :
```bash
cargo run -- export <session-id> --format markdown -o report.md
```

**Fichiers** :
- `crates/ccboard-core/src/export/conversation.rs` (+150 LOC, new)
- `crates/ccboard/src/cli/export.rs` (+100 LOC)

---

#### Task F.5: Tool Call Visualization (P2)

**Issue** : #9
**Durée** : 3-4h
**Difficulté** : Advanced
**Crate** : ccboard-tui

**Agents Claude Code** :
- `ui-designer` + `product-designer` (1h) → UX design pour tool calls
- `rust-developer` (2-3h) → Implémentation TUI avancée

**Rôle humain** : TUI (Senior) + UX (Intermediate)

**Features** :
- Expandable tool call nodes
- Input/Output display
- Success/Error indicators
- Duration tracking

**Fichiers** :
- `crates/ccboard-tui/src/tabs/conversation.rs` (+150 LOC)
- `crates/ccboard-tui/src/widgets/tool_call.rs` (+100 LOC, new)

---

### Validation Globale Phase F

**Critères de succès** :
- ✅ Render 1000-message conversation en <500ms
- ✅ Search full-text en <100ms
- ✅ Export 3 formats sans perte données
- ✅ TUI + Web UI feature parity
- ✅ Graceful degradation JSONL malformé

**Tests** :
```bash
cargo test --all --features conversation-viewer
cargo bench --bench conversation_viewer
```

**Commits attendus** :
1. `feat(core): add full JSONL parser with lazy loading`
2. `feat(tui): add conversation viewer tab with syntax highlighting`
3. `feat(tui): add message filtering by role/tool/timestamp`
4. `feat(export): add conversation export (markdown/json/html)`
5. `feat(tui): add tool call visualization with input/output`

**Documentation** :
- `docs/features/conversation-viewer.md` (user guide)
- `claudedocs/ADR-003-conversation-viewer-architecture.md`

---

## Phase H: Plan-Aware (Priority: 🔴 HIGH)

**Durée estimée** : 14-18h (2-3 jours)
**Issues GitHub** : #4 (main tracker), #10-#13
**Version cible** : v0.8.0
**Priorité** : P0 CRITIQUE (workflow visibility)

### Objectif

Parser PLAN.md files, extraire tasks/dependencies, afficher progression, et visualiser DAG de dépendances pour tracking workflow planning.

**Valeur utilisateur** :
- ✅ Voir progression tasks en temps réel
- ✅ Identifier bloqueurs (dependencies)
- ✅ Timeline visualisation (Gantt-like)
- ✅ Integration avec sessions (via TodoWrite)

### Dépendances

**Bloque** : Aucune (standalone feature)
**Dépend de** :
- ✅ Parsers frontmatter existants (agents/commands pattern réutilisable)
- ✅ Web UI (Leptos pour D3.js graph)

---

### Tasks Détaillées

#### Task H.1: PLAN.md Frontmatter Parser (P0)

**Issue** : #10
**Durée** : 4-5h
**Difficulté** : Good First Issue (parser) + Intermediate (integration)
**Crate** : ccboard-core

**Agents Claude Code** :
- General-purpose (3h) → YAML parsing simple (serde_yaml)
- `technical-writer` (1h) → Documentation format

**Rôle humain** : Parsers (Junior)

**Implémentation** :
```rust
// crates/ccboard-core/src/parsers/plan_parser.rs (~250 LOC)
pub struct PlanFile {
    pub metadata: PlanMetadata,  // YAML frontmatter
    pub phases: Vec<Phase>,
}

pub struct Phase {
    pub id: String,              // "F", "H", "11"
    pub title: String,
    pub status: PhaseStatus,     // Complete, InProgress, Future
    pub tasks: Vec<Task>,
}
```

**Fichiers** :
- `crates/ccboard-core/src/parsers/plan_parser.rs` (+250 LOC, new)
- `crates/ccboard-core/src/models/plan.rs` (+100 LOC, new)
- `crates/ccboard-core/tests/plan_parser_tests.rs` (+150 LOC)

**Tests** :
- Fixture : `claudedocs/PLAN.md` (réel, 48.6K)
- Parse accuracy : 95%+ phases détectées

---

#### Task H.2: Task Dependency Resolution (DAG) (P0)

**Issue** : #11
**Durée** : 4-5h
**Difficulté** : Advanced
**Crate** : ccboard-core

**Agents Claude Code** :
- `Explore` (1h) → Rechercher `petgraph` patterns
- `Plan` + `system-architect` (1h) → Design DAG architecture
- `rust-developer` (2-3h) → Implémentation algorithme

**Rôle humain** : Algorithms (Senior) + Architecture (Senior)
- Petgraph library (DAG construction)
- Topological sort
- Cycle detection

**Implémentation** :
```rust
use petgraph::graph::DiGraph;

pub struct TaskGraph {
    graph: DiGraph<Task, DependencyEdge>,
}

impl TaskGraph {
    pub fn topological_sort(&self) -> Result<Vec<TaskId>>;
    pub fn detect_cycles(&self) -> Vec<Vec<TaskId>>;
    pub fn critical_path(&self) -> Vec<TaskId>;
}
```

**Dépendances Cargo** : `petgraph = "0.6"`

**Fichiers** :
- `crates/ccboard-core/src/graph/task_dag.rs` (+200 LOC, new)
- `crates/ccboard-core/tests/dag_tests.rs` (+150 LOC)

**Validation** :
- ✅ Detect circular dependencies
- ✅ Topological sort correct
- ✅ Stress test 1000+ tasks

---

#### Task H.3: Session-to-Task Mapping (P1)

**Issue** : #12
**Durée** : 3h
**Difficulté** : Intermediate
**Crate** : ccboard-core

**Agents Claude Code** :
- `rust-developer` (3h) → Parser TodoWrite events

**Rôle humain** : Parsers (Intermediate)

**Mapping** :
- JSONL `TodoWrite` tool calls → Task creation
- Session metadata → Task context
- Timeline reconstruction

**Fichiers** :
- `crates/ccboard-core/src/parsers/todowrite.rs` (+150 LOC, new)

---

#### Task H.4: D3.js Dependency Graph (Web UI) (P1)

**Issue** : #13
**Durée** : 3h
**Difficulté** : Intermediate
**Crate** : ccboard-web

**Agents Claude Code** :
- WebSearch (30min) → D3.js examples force-directed graph
- Frontend specialist (2-3h) → Leptos + D3.js integration

**Rôle humain** : Frontend (Intermediate) + D3.js (Junior)

**Implémentation** :
```rust
// Leptos component with D3.js force-directed graph
#[component]
pub fn TaskDependencyGraph(tasks: Vec<Task>) -> impl IntoView {
    // Render D3.js via JS interop
}
```

**Fichiers** :
- `crates/ccboard-web/src/components/task_graph.rs` (+200 LOC)
- `crates/ccboard-web/static/js/d3-graph.js` (+150 LOC)

---

### Validation Globale Phase H

**Critères** :
- ✅ Parse 95%+ PLAN.md files correctement
- ✅ DAG construction sans crash sur cycles
- ✅ Timeline visualisation fluide (60 FPS)
- ✅ TodoWrite mapping accurate

**Commits attendus** :
1. `feat(core): add PLAN.md frontmatter parser`
2. `feat(core): add task dependency DAG with petgraph`
3. `feat(core): add session-to-task mapping via TodoWrite`
4. `feat(web): add D3.js dependency graph visualization`

---

## Phase 11: Tokens & Invocations (Priority: 🟡 MEDIUM)

**Durée estimée** : 16-20h
**Issues** : #14 (main), #16-#21
**Version** : v0.9.0
**Priorité** : P1 IMPORTANT (cost tracking)

### Objectif

Analyser usage tokens par tool/agent, détecter patterns d'invocation, suggestions d'optimisation coûts, et visualisations analytics.

**Dépend de** : Phase F complète (#2 parser JSONL pour token extraction)

---

### Tasks Principales

#### #16: Token usage per tool extraction (P1, 3h, Intermediate, core)
**Agents** : `rust-developer`
**Rôle** : Parsers (Intermediate) + Analytics (Senior)

#### #17: Token usage per agent extraction (P1, 3h, Intermediate, core)
**Agents** : `rust-developer`
**Rôle** : Similaire #16

#### #18: Invocation pattern analysis (P0, 4h, Advanced, core)
**Agents** : `Plan` + `backend-architect` → `rust-developer`
**Rôle** : Analytics (Senior), algorithmes pattern detection

#### #19: Cost optimization suggestions engine (P1, 4h, Advanced, core)
**Agents** : `rust-developer`
**Rôle** : Analytics (Senior), suggestion logic

#### #20: Token heatmap TUI (P1, 3h, Intermediate, tui)
**Agents** : `ui-designer` → General-purpose
**Rôle** : TUI (Intermediate), Ratatui charts

#### #21: Tool efficiency dashboard Web (P2, 2h, Good First Issue, web)
**Agents** : Frontend specialist
**Rôle** : Frontend (Junior), Chart.js integration

---

### Validation Phase 11

**Critères** :
- ✅ Token count accuracy ±2% vs Claude API
- ✅ Cost calculation correct tous models
- ✅ Budget alerts triggered au bon seuil
- ✅ Forecasting predictions raisonnables (±10%)

**Commits** : 6 commits (1 par feature)

---

## Phase 12: Write Operations (Priority: 🟡 MEDIUM) ⚠️ SAFETY-CRITICAL

**Durée estimée** : 14-18h
**Issues** : #15 (main), #22-#26
**Version** : v0.10.0
**Priorité** : P1 IMPORTANT mais SAFETY-FIRST

### Objectif

Permettre édition settings.json/hooks, avec validation, backup/rollback, et audit log. **Zéro tolérance data corruption.**

---

### Tasks Principales (Séquentiel strict)

#### #22: JSON schema validation (P0, 3-4h, Intermediate, core)
**Agents** : `Explore` → `rust-developer` + `code-reviewer`
**Rôle** : Security (Senior), validation input

#### #23: Backup system + rollback (P0, 4h, Intermediate, core)
**Agents** : `Plan` + `system-architect` → `rust-developer` + `debugger`
**Rôle** : Architecture (Senior), RAII patterns, atomic writes

#### #26: Audit log (P0, 2h, Good First Issue, core)
**Agents** : General-purpose
**Rôle** : Junior (append-only log)

#### #24: Settings editor UI (P1, 6-8h, Advanced, tui+web)
**Agents** : `ui-designer` + `product-designer` → `rust-developer` + Frontend
**Rôle** : UX (Senior), TUI/Frontend (Intermediate)

#### #25: Hooks editor (P1, 4h, Intermediate, tui+web)
**Agents** : `rust-developer`
**Rôle** : Similaire #24

---

### Validation Phase 12

**Critères** :
- ✅ **0 data corruption incidents** (CRITICAL)
- ✅ Rollback fonctionne 100%
- ✅ Audit log capture toutes opérations
- ✅ Validation bloque opérations dangereuses

**⚠️ CRITICAL** : External code review OBLIGATOIRE avant merge.

**Commits** : 5 commits avec messages explicites safety

---

## Phase 13: Team & Collaboration (Priority: 🟢 LOW)

**Durée** : 12-16h
**Issues** : #27 (main), #30-#34
**Version** : v0.11.0
**Dépend de** : Phase 12 (write ops pour sync)

### Objectif

Mode serveur multi-utilisateurs, sync agent-based, auth, PostgreSQL backend, team dashboard.

---

### Tasks Principales

#### #30: Server mode + team config (P2, 8-10h, Advanced, cli)
**Agents** : `Plan` + `system-architect` + `backend-architect` → `rust-developer`
**Rôle** : Backend (Senior), Axum server architecture

#### #31: Agent-based directory sync (P2, 10h, Advanced, core)
**Agents** : `Explore` → `rust-developer` + `debugger`
**Rôle** : Rust Async (Senior), file watching, race conditions

#### #28: User authentication (P2, 6h, Intermediate, web)
**Agents** : `Explore` → Backend specialist + `security-guardian`
**Rôle** : Backend (Senior), Tower-http middleware, JWT

#### #29: PostgreSQL backend (P2, 8-10h, Advanced, core)
**Agents** : `Plan` + `backend-architect` → `rust-developer`
**Rôle** : Database (Senior), SQLx, schema design, migrations

#### #34: Team dashboard (P2, 6h, Intermediate, web)
**Agents** : `product-designer` + `ui-designer` → Frontend specialist
**Rôle** : Frontend (Intermediate), Leptos SSE, aggregated data

---

### Validation Phase 13

**Critères** :
- ✅ Sync latency <200ms
- ✅ Conflict resolution automatique 80%+ cas
- ✅ Multi-user stress test (10+ users)

---

## Phase 14: IDE Integrations (Priority: 🟢 LOW)

**Durée** : 8-12h
**Issues** : #28 (main), #35-#38
**Version** : v0.12.0

### Objectif

JSON RPC interface + extensions VS Code, Neovim, JetBrains.

---

### Tasks Principales

#### #35: JSON RPC interface (P2, 6-8h, Intermediate, cli)
**Agents** : `Explore` → `rust-developer`
**Rôle** : Rust Core (Intermediate), JSON RPC, IPC

#### #36: VS Code extension (P2, 8-10h, Advanced, external)
**Agents** : TypeScript developer
**Rôle** : TypeScript (Senior), VS Code extension API

#### #37: Neovim plugin (P2, 5-6h, Intermediate, external)
**Agents** : Lua developer
**Rôle** : Lua (Intermediate), Neovim plugin API

#### #38: JetBrains plugin (P2, 8-10h, Advanced, external)
**Agents** : Kotlin developer
**Rôle** : Kotlin (Senior), IntelliJ Platform SDK

---

### Validation Phase 14

**Critères** :
- ✅ Extension install success 90%+ first-time
- ✅ Performance overhead <50ms

**Note** : Issues #36-38 sont repos externes, maintainers dédiés requis.

---

## Phase 15: CI/CD Integration (Priority: 🟢 LOW)

**Durée** : 6-10h
**Issues** : #29 (main), #39-#42
**Version** : v0.13.0 → v1.0.0

### Objectif

Commande `ccboard report`, token budget enforcement, quality gates, GitHub Action.

---

### Tasks Principales

#### #39: ccboard report command (P2, 3-4h, Intermediate, cli)
**Agents** : `Explore` → `rust-developer` + `technical-writer`
**Rôle** : Rust Core (Intermediate), templating (tera)

#### #40: Token budget enforcement (P2, 2h, Good First Issue, cli)
**Agents** : General-purpose
**Rôle** : Junior, exit codes

#### #41: Session quality gates (P2, 3h, Intermediate, cli)
**Agents** : `rust-developer`
**Rôle** : Intermediate, quality metrics

#### #42: GitHub Action manifest (P2, 2h, Good First Issue, external)
**Agents** : DevOps specialist
**Rôle** : DevOps (Intermediate), GitHub Actions YAML

---

### Validation Phase 15

**Critères** :
- ✅ CI pipeline setup <10min
- ✅ Reports générés correctement
- ✅ Quality gates fonctionnels

**Milestone v1.0.0** : Phase 15 complète = ccboard full-featured dashboard.

---

## Matrice de Compétences Requises

| Compétence | Phase F | Phase H | Phase 11 | Phase 12 | Phase 13 | Phase 14 | Phase 15 |
|-----------|---------|---------|----------|----------|----------|----------|----------|
| **Rust Core** | S | S | S | S | S | I | I |
| **Parsers** | S | I | I | I | I | - | J |
| **TUI/Ratatui** | I | - | I | I | - | - | - |
| **Frontend** | - | I | I | I | I | - | - |
| **Backend** | - | - | - | - | S | I | - |
| **Database** | - | - | - | - | S | - | - |
| **Security** | - | - | - | S | S | I | - |
| **Algorithms** | - | S | S | - | - | - | - |
| **DevOps** | - | - | - | - | I | - | I |

**Légende** : J=Junior, I=Intermediate, S=Senior, -=Non requis

---

## Workflows Recommandés par Difficulté

### Good First Issues (7 issues, onboarding)

**Issues** : #6 (highlighting), #7 (filtering), #10 (PLAN parser), #21 (dashboard), #26 (audit log), #40 (budget), #42 (GH Action)

**Workflow** :
1. `Explore` (1h) → Découvrir codebase context
2. General-purpose (1-2h) → Implémentation basique
3. `code-reviewer` (30min) → Quality check
4. Human test (30min)

**Compétences** : Junior (Rust basics, suivre patterns)

---

### Intermediate Issues (15 issues)

**Workflow** :
1. `Explore` (30-60min) → Patterns similaires
2. `Plan` (30-60min) → Architecture proposal
3. `rust-developer` (2-4h) → Code
4. Tests (1h) → Automated + manual
5. Review (30min) → `code-reviewer` + human senior

**Compétences** : Intermediate (2-5 ans Rust, architecture)

---

### Advanced Issues (9 issues)

**Workflow** :
1. Deep exploration (1-2h) → `Explore` externe (crates.io)
2. Architecture design (1-2h) → `Plan` + `system-architect`
3. Prototype (2-3h) → `rust-developer`
4. Review architecture (1h) → Human senior
5. Implementation (4-8h) → `rust-developer`
6. Extensive testing (2-3h)
7. Security review (1h) → `security-guardian`
8. Code review (1h) → Human senior + `code-reviewer`

**Compétences** : Senior (5+ ans Rust, system design)

---

## Decision Tree: Quel Agent pour Quelle Tâche ?

```
Tâche ?
│
├─ Découvrir codebase ?
│  └─> `Explore` agent
│
├─ Concevoir architecture ?
│  ├─> Simple (<200 LOC) → `Plan`
│  └─> Complexe (>500 LOC) → `Plan` + `system-architect`
│
├─ Implémenter Rust ?
│  ├─> Basique (good-first-issue) → General-purpose
│  ├─> Intermédiaire (parsers, TUI) → `rust-developer`
│  └─> Avancé (algo, safety) → `rust-developer` + `debugger`
│
├─ UI/UX ?
│  ├─> Design → `ui-designer` + `product-designer`
│  ├─> TUI → General-purpose (simple) | `rust-developer` (complexe)
│  └─> Web → Frontend specialist
│
├─ Backend/DB ?
│  └─> `rust-developer` + `backend-architect`
│
└─ Review ?
    ├─> Code quality → `code-reviewer`
    └─> Security → `security-guardian`
```

---

## Patterns d'Usage Agents

### Parallèle (tâches indépendantes)
```
Phase F: #6 (highlighting) ∥ #7 (filtering)
→ Gain: 4h → 2h (si 2 agents simultanés)
```

### Séquentiel (dépendances fortes)
```
Phase H: #10 (parser) → #11 (DAG) → #12 (mapping)
→ Aucun gain parallélisation
```

### Hybrid
```
Phase 11: (#16 ∥ #17) → #18 → #19 → (#20 ∥ #21)
→ Gain: ~14h → ~10h
```

---

## Priorisation d'Implémentation

### Batch 1 (Semaine 1-2) : Phase F
- **Quick wins** : #6, #7 (onboarding contributeurs)
- **Critical path** : #5 (parser) bloque #8 (export)
- **Agents** : `Explore` → `Plan` → `rust-developer`

### Batch 2 (Semaine 3-4) : Phase H
- **Complexité** : #11 (DAG) nécessite senior dev
- **Agents** : `system-architect` + `rust-developer`

### Batch 3 (Semaine 5-7) : Phase 11
- **Parallélisation** : #16 ∥ #17 ∥ #20 possibles
- **Agents** : `rust-developer` (analytics) + `ui-designer` (viz)

### Batch 4 (Semaine 8-12) : Phase 12 ⚠️
- **Safety-critical** : #22 + #23 + #26 AVANT #24 + #25
- **External audit** : Recommandé avant merge

### Batch 5 (Semaine 13-20) : Phase 13
- **Architecture** : Planning approfondi (2-3 semaines)
- **Agents** : `system-architect` + `backend-architect` + `rust-developer`

### Batch 6-7 (Flexible) : Phase 14 + 15
- **Phase 14** : Maintainers IDE dédiés (TypeScript/Lua/Kotlin)
- **Phase 15** : Rapide (10-14h total)

---

## Critical Files pour Commencer

1. **`crates/ccboard-core/src/parsers/session_index.rs`** (Session parser foundation)
   - Réutilisé : Phase F (#5), Phase H (#10), Phase 11 (#16-17)

2. **`crates/ccboard-core/src/store.rs`** (Central DataStore)
   - Toutes modifications data

3. **`crates/ccboard-core/src/models/session.rs`** (SessionMetadata)
   - Extensions metadata pour tokens/tools

4. **`crates/ccboard-tui/src/tabs/*.rs`** (TUI tabs)
   - Phase F (#6-7-9), Phase 11 (#20), Phase 12 (#24-25)

5. **`crates/ccboard-web/src/pages/*.rs`** (Leptos pages)
   - Phase H (#13), Phase 11 (#21), Phase 13 (#34)

---

## Verification Steps

Après chaque phase complétée :

```bash
# 1. Tests
cargo test --all --features <phase-feature>

# 2. Benchmarks
cargo bench --bench <phase-bench>

# 3. Quality check
cargo fmt --all && cargo clippy --all-targets

# 4. Manual testing
cargo run           # TUI
cargo run -- web    # Web UI
```

---

## Next Steps

1. **Phase F démarrée** : 2026-02-12
2. **Milestone v0.7.0** : Target fin février 2026
3. **Team coordination** : Via ccboard-phases-f-15 team
4. **Progress tracking** : TaskList + this document updates

---

**Last Updated** : 2026-02-12
**Plan Status** : IN PROGRESS (Phase F active)
**Team** : ccboard-phases-f-15
