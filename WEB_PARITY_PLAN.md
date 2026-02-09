# Plan de Parité TUI/Web - ccboard Frontend

**Objectif** : Atteindre 100% de parité fonctionnelle entre TUI et Web

## État Actuel (2/7 complétées)

✅ **Task #1** : Config page avec layout 4 colonnes + syntax highlighting
- Fichiers modifiés : `crates/ccboard-web/src/pages/config.rs`, `static/style.css`
- Commit : `0fed2cd`

✅ **Task #2** : Dashboard KPIs manquants (Messages, Cache Hit, MCP Servers)
- Fichiers modifiés : `router.rs`, `api.rs`, `pages/dashboard.rs`
- Commit : `db7267e`

## Tâches Restantes (5/7)

---

### Task #3 : Hooks Page

**Référence TUI** : `/Users/florianbruniaux/Desktop/ccboard/hooks.png`

**Layout** :
- Colonne gauche : Liste des hooks (nom + trigger)
- Colonne droite : Détail du hook sélectionné
  - Nom
  - Description
  - Trigger
  - Script bash avec syntax highlighting

**Fichiers à créer** :
1. `crates/ccboard-web/src/pages/hooks.rs`
   - Component `Hooks()` avec split view
   - Component `HooksList()` (liste cliquable)
   - Component `HookDetail()` (détail + script)

2. Backend API endpoint :
   - Modifier `router.rs` : ajouter `GET /api/hooks`
   - Handler retourne : `Vec<{ name, description, trigger, script_path, script_content }>`

**Données source** : `DataStore::settings().merged.hooks` (structure `HookGroup`)

**CSS à ajouter** :
- `.hooks-page` : flex container
- `.hooks-list` : sidebar 300px
- `.hook-detail` : flex-1
- `.hook-script` : pre avec syntax highlighting bash

**Estimation** : 1h30

---

### Task #4 : MCP Page

**Référence TUI** : `/Users/florianbruniaux/Desktop/ccboard/MCP.png`

**Layout** :
- Colonne gauche : Liste des MCP servers (nom + status)
- Colonne droite : Détails du serveur sélectionné
  - Status (Up/Down)
  - Command
  - Arguments
  - Environment variables
  - Config File path
  - Actions (aucune pour MVP read-only)

**Fichiers à créer** :
1. `crates/ccboard-web/src/pages/mcp.rs`
   - Component `Mcp()` avec split view
   - Component `McpServersList()` (liste avec badges status)
   - Component `McpServerDetail()` (détails structurés)

2. Backend API endpoint :
   - Modifier `router.rs` : ajouter `GET /api/mcp`
   - Handler retourne : `McpConfig` du store

**Données source** : `DataStore::mcp_config()` (structure `McpConfig`)

**CSS à ajouter** :
- `.mcp-page` : flex container
- `.mcp-servers-list` : sidebar 250px
- `.mcp-server-detail` : flex-1
- `.mcp-status-badge` : green (up) / red (down)

**Estimation** : 1h

---

### Task #5 : Costs Pages (4 onglets)

**Référence TUI** :
- `/Users/florianbruniaux/Desktop/ccboard/Costs - Overview.png`
- `/Users/florianbruniaux/Desktop/ccboard/Costs - By model.png`
- `/Users/florianbruniaux/Desktop/ccboard/Costs - Daily.png`
- `/Users/florianbruniaux/Desktop/ccboard/Costs - Billing block.png`

**Layout** : Page avec 4 tabs

**Tab 1 : Overview**
- Total Estimated Cost (grand chiffre)
- Token Breakdown (barre horizontale : Input / Output / Cache)
- Model Cost Distribution (table : Model | Cost | Input | Output)

**Tab 2 : By Model**
- Table détaillée par modèle avec colonnes :
  - Model name
  - Input cost
  - Output cost
  - Cache cost
  - Total cost
  - Sorting par coût décroissant

**Tab 3 : Daily**
- Bar chart des 14 derniers jours
- Axe Y : cost ($)
- Axe X : dates
- Valeurs affichées au-dessus des barres

**Tab 4 : Billing Blocks**
- Table avec colonnes :
  - Date range (UTC)
  - Block (UTC)
  - Tokens
  - Sessions
  - Cost

**Fichiers à créer** :
1. `crates/ccboard-web/src/pages/costs.rs`
   - Component `Costs()` avec tabs
   - Component `CostsOverview()`
   - Component `CostsByModel()`
   - Component `CostsDaily()`
   - Component `CostsBillingBlocks()`

2. Backend API endpoint :
   - Modifier `router.rs` : ajouter `GET /api/costs/detailed`
   - Handler calcule et retourne toutes les données nécessaires

**Données source** :
- `StatsCache::model_usage` pour costs par modèle
- `BillingBlockManager` pour billing blocks
- Analytics pour daily costs

**CSS à ajouter** :
- `.costs-tabs` : navigation tabs
- `.costs-overview` : grid layout
- `.costs-table` : table responsive

**Estimation** : 2h30

---

### Task #6 : Agents/Commands/Skills Pages

**Référence TUI** :
- `/Users/florianbruniaux/Desktop/ccboard/Agents - Agents.png`
- `/Users/florianbruniaux/Desktop/ccboard/Agents - Commands.png`
- `/Users/florianbruniaux/Desktop/ccboard/Agents - skills.png`

**Layout** : Page avec 3 tabs (ou 3 pages séparées)

**Chaque tab** :
- Colonne gauche : Liste des agents/commands/skills (nom)
- Colonne droite : Détail
  - Nom
  - Description
  - Metadata (frontmatter YAML)

**Fichiers à créer** :
1. `crates/ccboard-web/src/pages/agents.rs`
   - Component `Agents()` avec tabs/split view
   - Component `AgentsList()` / `CommandsList()` / `SkillsList()`
   - Component `AgentDetail()` / `CommandDetail()` / `SkillDetail()`

2. Backend API endpoints :
   - `GET /api/agents` : liste agents
   - `GET /api/commands` : liste commands
   - `GET /api/skills` : liste skills

3. Parser frontmatter :
   - Utiliser parser existant dans `ccboard-core`
   - Extraire YAML + markdown body

**Données source** :
- `.claude/agents/*.md`
- `.claude/commands/*.md`
- `.claude/skills/*.md`

**CSS à ajouter** :
- `.agents-page` : flex container
- `.agents-list` : sidebar 280px
- `.agent-detail` : flex-1 avec sections

**Estimation** : 2h

---

### Task #7 : Analytics Sub-Views

**Référence TUI** :
- `/Users/florianbruniaux/Desktop/ccboard/Analytics - overview.png` (existe déjà)
- `/Users/florianbruniaux/Desktop/ccboard/Analytics - trends.png`
- `/Users/florianbruniaux/Desktop/ccboard/Analytics - patterns.png`
- `/Users/florianbruniaux/Desktop/ccboard/Analytics - Insight.png`

**Layout** : Page Analytics avec 4 tabs

**Tab 1 : Overview** (existe déjà)
- Token Usage Forecast
- Budget Status
- Top Projects by Cost

**Tab 2 : Trends**
- Scatter plot avec trend line
- Tokens (ligne pleine)
- Sessions x100 (ligne pointillée)
- Forecast 10% conf (ligne prédictive)
- Axes temporels

**Tab 3 : Patterns**
- Usage patterns analysis
- Peak hours heatmap
- Weekly activity distribution

**Tab 4 : Insights**
- AI-generated insights (texte)
- Top insights list :
  - "Peak hours: 9AM-12PM (40% of sessions)"
  - "Consider batch work: sessions longer in mornings"
  - etc.

**Fichiers à modifier** :
1. `crates/ccboard-web/src/pages/analytics.rs`
   - Ajouter tabs navigation
   - Créer components pour chaque sub-view
   - Component `AnalyticsTrends()`
   - Component `AnalyticsPatterns()`
   - Component `AnalyticsInsights()`

2. Backend analytics :
   - Vérifier que `AnalyticsData` contient toutes les données
   - Ajouter calculs manquants si nécessaire

**Données source** :
- `AnalyticsData::trends` pour Trends
- `StatsCache::hour_counts` pour Patterns
- Calculs custom pour Insights

**CSS à ajouter** :
- `.analytics-tabs` : navigation
- `.trends-chart` : scatter plot container
- `.patterns-heatmap` : grid layout
- `.insights-list` : bullet points avec icônes

**Estimation** : 2h

---

## Plan d'Exécution

**Ordre recommandé** (par priorité d'impact UX) :

1. **Task #3 : Hooks** (1h30)
   - Fonctionnalité complète manquante
   - Layout simple

2. **Task #4 : MCP** (1h)
   - Fonctionnalité complète manquante
   - Layout simple

3. **Task #5 : Costs** (2h30)
   - Forte valeur ajoutée
   - Données déjà disponibles

4. **Task #6 : Agents** (2h)
   - Fonctionnalités complètes manquantes
   - Parsing à implémenter

5. **Task #7 : Analytics** (2h)
   - Enrichissement d'une page existante
   - Moins critique

**Temps total estimé** : 9h

---

## Checklist de Validation

### Pour chaque tâche :

**Backend** :
- [ ] Endpoint API créé et testé avec curl
- [ ] Handler retourne les bonnes données
- [ ] Pas d'erreurs de compilation Rust
- [ ] cargo clippy passe sans warnings

**Frontend** :
- [ ] Page Leptos créée et routée
- [ ] Composants fonctionnels
- [ ] CSS responsive
- [ ] trunk build passe sans erreurs
- [ ] Test manuel dans le navigateur

**Intégration** :
- [ ] Frontend fetch API correctement
- [ ] Données affichées comme attendu
- [ ] Layout correspond au TUI
- [ ] Aucune régression sur pages existantes

### Test final global :

- [ ] Toutes les pages accessibles depuis la nav
- [ ] Aucune console error dans le navigateur
- [ ] Build production passe (`trunk build --release`)
- [ ] Comparaison visuelle TUI vs Web satisfaisante

---

## Commandes Utiles

### Build & Test

```bash
# Backend
cargo build --all
cargo clippy --all-targets
cargo test --all

# Frontend
cd crates/ccboard-web
trunk serve --port 3333

# Full stack
# Terminal 1: Backend
cargo run --bin ccboard -- web --port 8080

# Terminal 2: Frontend
trunk serve --port 3333
```

### Test API Endpoints

```bash
# Stats
curl http://localhost:8080/api/stats | jq '.total_sessions'

# Config
curl http://localhost:8080/api/config/merged | jq '.merged'

# Hooks (après implémentation)
curl http://localhost:8080/api/hooks | jq '.'

# MCP (après implémentation)
curl http://localhost:8080/api/mcp | jq '.servers | keys'
```

---

## Notes de Développement

### Principes à suivre :

1. **Commiter après chaque tâche complétée**
   - Format : `feat(web): [description courte] (Task #X)`
   - Inclure Co-Authored-By: Claude Sonnet 4.5

2. **TaskUpdate à chaque étape**
   - Marquer `in_progress` au début
   - Marquer `completed` à la fin

3. **Tester manuellement avant de commit**
   - Lancer le frontend
   - Naviguer vers la page
   - Vérifier que tout fonctionne

4. **Pas de duplication de code**
   - Réutiliser composants existants (StatsCard, etc.)
   - Factoriser les patterns répétés

5. **CSS cohérent**
   - Utiliser variables CSS existantes
   - Suivre le style guide (dark theme)
   - Layout responsive

---

## Fichiers Récapitulatifs

**Commits effectués** :
- `0fed2cd` : Task #1 Config page
- `db7267e` : Task #2 Dashboard KPIs

**Branche** : `feat/web-w1-leptos-spa`

**Tasks** : Gérées via TaskCreate/TaskUpdate dans cette session

---

## Après Complétion

### Phase finale :

1. **Test complet**
   - Parcourir toutes les pages
   - Vérifier feature parity avec TUI
   - Screenshots comparatifs

2. **Documentation**
   - Mettre à jour CLAUDE.md
   - Marquer Phase G comme ✅ 100% complète
   - Documenter nouvelles pages

3. **PR & Merge**
   - Rebase sur main si nécessaire
   - Créer PR avec description détaillée
   - Merge dans main

4. **Release**
   - Bump version dans Cargo.toml
   - Tag release
   - Update changelog
