# Guide de Test - Phase 6 (File Opening + MCP UI)

**Date** : 2026-02-02
**Status** : Build ✅ | Clippy ✅ | Ready to test

## Préparation

### 1. Setup Variables d'Environnement

```bash
# Définir votre éditeur préféré (priorité: VISUAL > EDITOR)
export VISUAL=vim       # ou nvim, nano, code, etc.
# OU
export EDITOR=vim

# Vérifier
echo $VISUAL
echo $EDITOR
```

### 2. Données de Test Nécessaires

Assurez-vous d'avoir :
- ✅ `~/.claude/` avec sessions réelles
- ✅ `~/.claude/claude_desktop_config.json` avec MCP servers configurés
- ✅ Agents dans `.claude/agents/*.md`
- ✅ Hooks dans `.claude/hooks/bash/*.sh` (si disponibles)
- ✅ Configuration en cascade (global/project/local)

### 3. Lancer ccboard

```bash
cd /Users/florianbruniaux/Sites/perso/ccboard
./target/release/ccboard

# Ou avec projet spécifique
./target/release/ccboard --project ~/Sites/aristote
```

---

## Tests par Feature

### ✅ Task 9 : Dashboard - MCP Card

**Feature** : 5ème stat card montrant nombre de MCP servers

**Steps** :
1. Lancer ccboard → Tab 1 (Dashboard)
2. Vérifier : 5 cards au lieu de 4
3. 5ème card = "◉ MCP" avec count de servers
4. Couleur : **Vert** si >0 servers, **Gris** si 0

**Validation** :
- [ ] 5 cards visibles (Tokens | Sessions | Messages | Cache | MCP)
- [ ] Count MCP correspond à `claude_desktop_config.json`
- [ ] Couleur correcte (vert/gris)

**Screenshots** : Prendre capture dashboard avec 5 cards

---

### ✅ Task 1 : Agents - Editor Opening

**Feature** : Press `e` pour ouvrir agent file dans $EDITOR

**Steps** :
1. Tab 5 (Agents) → Sub-tab "◉ Agents"
2. Sélectionner un agent avec ↑/↓
3. Press `e`
4. **Attendu** : Terminal exit → Editor s'ouvre avec fichier .md
5. Quitter editor (`:q` pour vim) → Retour TUI proprement

**Validation** :
- [ ] Editor s'ouvre (vim/nvim/nano selon $VISUAL/$EDITOR)
- [ ] Fichier correct chargé (agent .md file)
- [ ] Après quit editor → TUI se restaure sans glitch
- [ ] Aucun crash si fichier manquant → popup erreur

**Test Edge Cases** :
```bash
# Test fallback si pas d'env var
unset VISUAL
unset EDITOR
./target/release/ccboard
# Press 'e' → devrait ouvrir 'nano' (macOS/Linux) ou 'notepad.exe' (Windows)
```

---

### ✅ Task 2 : Sessions - Editor Opening

**Feature** : Press `e` pour ouvrir session .jsonl dans $EDITOR

**Steps** :
1. Tab 2 (Sessions)
2. Sélectionner un projet (← →)
3. Sélectionner une session (↑ ↓)
4. Press `Enter` → Detail popup s'ouvre
5. **Vérifier** : File path affiché dans detail
6. Press `e` (dans popup ou hors popup)
7. **Attendu** : Editor s'ouvre avec .jsonl file

**Validation** :
- [ ] File path visible dans detail popup
- [ ] Editor s'ouvre avec session JSONL
- [ ] Gros fichiers (>100MB) gérés sans lag
- [ ] Retour TUI après quit editor

**Test Performance** :
- Ouvrir une session avec >1000 messages → vérifier pas de lag

---

### ✅ Task 3 : History - Editor Opening

**Feature** : Press `e` pour ouvrir session .jsonl depuis History tab

**Steps** :
1. Tab 7 (History)
2. (Optionnel) Search avec `/` + query
3. Sélectionner une session (↑ ↓)
4. Press `Enter` → Detail popup
5. **Vérifier** : File path affiché
6. Press `e`
7. **Attendu** : Editor s'ouvre

**Validation** :
- [ ] Identique à Sessions tab (même behavior)
- [ ] Search ne casse pas l'opening
- [ ] File path correctement résolu

---

### ✅ Task 5 : Hooks - Editor Opening

**Feature** : Press `e` pour ouvrir hook .sh file dans $EDITOR

**Steps** :
1. Tab 4 (Hooks)
2. Sélectionner un event (← → pour colonne gauche)
3. Sélectionner un hook (↑ ↓ dans colonne droite)
4. **Vérifier** : File path affiché dans detail
5. Press `e`
6. **Attendu** : Editor s'ouvre avec .sh script

**Validation** :
- [ ] File path visible dans hook details
- [ ] Editor s'ouvre avec bash script correct
- [ ] Syntax highlighting (si editor le supporte)
- [ ] Pas d'erreur si hook inline (pas de file_path) → popup erreur

**Test Edge Case** :
- Hook sans file_path (inline dans settings.json) → devrait afficher popup "No file path available"

---

### ✅ Task 6 : File Manager Reveal

**Feature** : Press `o` pour reveal file dans file manager

**Platforms** :
- macOS : `open -R` → Finder avec fichier sélectionné
- Linux : `xdg-open` → File manager par défaut
- Windows : `explorer /select,` → Explorer avec sélection

**Steps** :
1. N'importe quel tab avec file (Agents/Sessions/History/Hooks/Config)
2. Sélectionner un item avec file_path
3. Press `o`
4. **Attendu** : File manager s'ouvre (non-blocking)
5. TUI reste responsive (pas de freeze)

**Validation** :
- [ ] File manager s'ouvre
- [ ] Fichier sélectionné/highlighté (macOS/Windows)
- [ ] TUI reste responsive (pas de `.wait()`)
- [ ] Pas de crash si file manquant

**Cross-Platform Test** :
- Test sur macOS uniquement pour ce projet (ajouter Linux/Windows si available)

---

### ✅ Task 4 : Config - File Editing par Colonne

**Feature** : Press `e` pour éditer config file selon colonne focus

**Mapping Colonnes** :
- **Colonne 0 (Global)** : `~/.claude/settings.json`
- **Colonne 1 (Project)** : `.claude/settings.json`
- **Colonne 2 (Local)** : `.claude/settings.local.json`
- **Colonne 3 (Merged)** : Pas de fichier (read-only) → aucun effet

**Steps** :
1. Tab 3 (Config)
2. Press `←` ou `→` pour changer focus colonne
3. **Vérifier** : Colonne active = highlighted border (Cyan vs DarkGray)
4. Press `e`
5. **Attendu** :
   - Colonnes 0-2 : Editor s'ouvre avec JSON config
   - Colonne 3 : Aucun effet OU popup "Merged is read-only"

**Validation** :
- [ ] Focus colonne visible (border color)
- [ ] Global (col 0) → ouvre `~/.claude/settings.json`
- [ ] Project (col 1) → ouvre `.claude/settings.json`
- [ ] Local (col 2) → ouvre `.claude/settings.local.json`
- [ ] Merged (col 3) → pas d'action ou erreur explicite
- [ ] Fichiers créés si manquants OU erreur claire

**Test Edge Cases** :
```bash
# Test fichier manquant
rm ~/.claude/settings.local.json
# Press 'e' sur colonne Local → devrait afficher erreur "File does not exist"
```

---

### ✅ Task 7 : Enhanced MCP Section

**Feature** : Section MCP multi-ligne dans Config tab (colonne Merged)

**Format Attendu** :
```
─── MCP Servers ───
  ● playwright (configured)
    npx -y @modelcontextprotocol/server-playwright
    Env: 2 vars

  ● sequential-thinking (configured)
    npx -y @modelcontextprotocol/server-sequential-thinking
    Env: (none)
```

**Steps** :
1. Tab 3 (Config) → Colonne Merged (focus col 3 avec `→`)
2. Scroll vers le bas (↓) jusqu'à section "MCP Servers"
3. **Vérifier** :
   - Bullet vert `●` pour chaque server
   - 3 lignes par server (nom, commande, env)
   - Commande affichée jusqu'à 60 chars
   - Env count : "Env: N vars" ou "Env: (none)"

**Validation** :
- [ ] Multi-ligne format (3 lignes par server)
- [ ] Bullets verts ● pour tous servers
- [ ] Command correctement affiché (truncated si >60 chars)
- [ ] Env count affiché ("2 vars", "none", etc.)
- [ ] Section vide si aucun server → "(No MCP servers configured)"

---

### ✅ Task 8 : MCP Detail Modal

**Feature** : Press `m` pour voir détails complets MCP server

**Steps** :
1. Tab 3 (Config) → Colonne Merged
2. Scroll vers MCP section
3. Focus sur un server name (↑ ↓)
4. Press `m`
5. **Attendu** : Modal overlay s'ouvre (70% width/height)

**Modal Content** :
```
┌─ MCP Server Details ─────────────────────────────┐
│                                                   │
│ ● playwright (configured)                        │
│   Command:                                        │
│   npx -y @modelcontextprotocol/server-playwright│
│                                                   │
│   Environment Variables:                          │
│   NODE_ENV=production                            │
│   DEBUG=pw:*                                     │
│                                                   │
│   Config: ~/.claude/claude_desktop_config.json   │
│                                                   │
│ [Esc: close | e: edit config]                   │
└───────────────────────────────────────────────────┘
```

**Steps dans Modal** :
6. **Vérifier** : Toutes les infos affichées (full command, all env vars)
7. Press `e` → Editor s'ouvre avec `claude_desktop_config.json`
8. Quit editor → Modal auto-close → Retour Config tab
9. Press `Esc` → Modal se ferme sans édition

**Validation** :
- [ ] Modal s'ouvre avec `m` key
- [ ] Full command affiché (pas de troncature)
- [ ] All env vars affichés (key=value)
- [ ] Config file path affiché
- [ ] `e` key → ouvre JSON config
- [ ] Esc → ferme modal
- [ ] Pas de crash si pas de MCP servers

**Test Edge Cases** :
- Aucun MCP server configuré → devrait afficher "No MCP servers configured"
- MCP server sans env vars → "Environment Variables: (none)"

---

## Tests d'Intégration

### Workflow Complet : Edit Agent + Edit Config + View MCP

1. Tab 5 (Agents) → Select agent → `e` → Edit → Save → Quit
2. Tab 3 (Config) → Col 1 → `e` → Edit settings.json → Save → Quit
3. Tab 3 (Config) → Col 3 → Scroll MCP → `m` → View modal → `e` → Edit JSON → Quit
4. Tab 1 (Dashboard) → Verify MCP count updated (si ajout server)
5. Press `F5` → Refresh → Verify changes reflected

**Validation** :
- [ ] Aucun crash durant workflow complet
- [ ] TUI se restaure proprement après chaque editor exit
- [ ] Changes visibles après refresh (F5)

### Performance Test

```bash
# Terminal de petite taille (stress test layout)
resize -s 24 80
./target/release/ccboard

# Tester toutes les features avec petit terminal
# Vérifier : pas de crash, layout lisible, modals visibles
```

---

## Checklist Finale

### Build & Quality
- [x] `cargo build --release` passe
- [x] `cargo clippy --all-targets` passe (0 warnings)
- [ ] `cargo test --all` passe
- [ ] `cargo fmt --all --check` passe

### Features Phase 6
- [ ] Task 9 : Dashboard MCP card
- [ ] Task 1 : Agents editor opening
- [ ] Task 2 : Sessions editor opening
- [ ] Task 3 : History editor opening
- [ ] Task 5 : Hooks editor opening
- [ ] Task 6 : File manager reveal ('o' key)
- [ ] Task 4 : Config file editing par colonne
- [ ] Task 7 : Enhanced MCP section
- [ ] Task 8 : MCP detail modal

### Cross-Cutting
- [ ] Error popups s'affichent correctement (Esc to close)
- [ ] Terminal state restauré après editor (alternate screen, raw mode)
- [ ] $VISUAL > $EDITOR > fallback fonctionne
- [ ] Keybindings cohérents ('e' edit, 'o' reveal, 'm' modal, Esc close)

---

## Rapport de Bugs

Si bugs trouvés, documenter ici :

### Bug Template

**Feature** : [Task number + nom]
**Steps to Reproduce** :
1. ...
2. ...

**Expected** : ...
**Actual** : ...
**Error Message** : (si applicable)
**Severity** : Critical / Major / Minor

---

## Commandes Rapides

```bash
# Build
cargo build --release --all

# Run
./target/release/ccboard
./target/release/ccboard --project ~/path/to/project

# Tests
cargo test --all
cargo clippy --all-targets
cargo fmt --all

# Debug logging
RUST_LOG=ccboard=debug ./target/release/ccboard
```

---

## Notes de Test

### Environnements Testés
- [ ] macOS (primary)
- [ ] Linux (optional)
- [ ] Windows (optional)

### Editors Testés
- [ ] vim/nvim
- [ ] nano
- [ ] VS Code (`code --wait`)
- [ ] Fallback (pas d'env var)

### Terminal Sizes
- [ ] Standard (80x24)
- [ ] Large (120x40)
- [ ] Small (stress test)

---

**Status** : ✅ Ready to test
**Estimated Test Time** : 30-45 minutes (complet)
**Priority** : Tester Task 9 (Dashboard) + Task 8 (Modal) en premier (fonctionnalités les plus visibles)
