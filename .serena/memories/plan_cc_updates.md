# Plan : Mise à jour ccboard - Nouvelles features Claude Code (mai 2026)

Source : claude-code-guide MCP digest 30j, versions v2.1.118 → v2.1.152.
Créé le 2026-05-27. Toutes les tâches sont indépendantes sauf si noté.

---

## Phase 1 : Hook Events (PRIORITÉ HAUTE)

### P1-A : Compléter HookType enum
**Fichier** : `crates/ccboard-core/src/parsers/hooks.rs:36`

Ajouter les variantes manquantes à `pub enum HookType` :
- `PostToolUse` (depuis v2.1.118+, très courant dans les settings.json)
- `PreToolUse`
- `Stop`
- `SubagentStop`
- `MessageDisplay` (v2.1.152 : nouveau, transforme/cache le texte assistant)
- `PreCompact`
- `PostCompact`
- `Notification`

Mettre à jour `HookType::from_filename()` pour reconnaître ces noms de fichiers.
Mettre à jour le pattern match dans l'onglet Hooks TUI si exhaustif.

### P1-B : Nouveaux champs sur le struct Hook
**Fichier** : `crates/ccboard-core/src/parsers/hooks.rs:61`

Ajouter à `pub struct Hook` :
```rust
pub terminal_sequence: Option<String>,   // v2.1.141 : output field
pub continue_on_block: bool,             // v2.1.120 : PostToolUse flag
pub hook_specific_output: Option<serde_json::Value>, // updatedToolOutput etc.
```

Ces champs se lisent dans le JSON stdin/output des hooks, pas depuis le nom de fichier.
Parsing : lire le contenu du script pour détecter les patterns JSON connus.

### P1-C : Affichage dans l'onglet Hooks TUI
**Fichier** : `crates/ccboard-tui/src/tabs/` (hooks tab)

Dans la vue détail d'un hook :
- Afficher le nouveau type (MessageDisplay, PostToolUse, etc.)
- Badge "continueOnBlock" si activé
- Section "Terminal Sequence" si présente

---

## Phase 2 : Settings : Auto Mode & Worktree (PRIORITÉ HAUTE)

### P2-A : AutoModeConfig struct
**Fichier** : `crates/ccboard-core/src/models/config.rs`

Ajouter :
```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AutoModeConfig {
    pub allow: Option<Vec<serde_json::Value>>,
    pub soft_deny: Option<Vec<serde_json::Value>>,
    pub hard_deny: Option<Vec<serde_json::Value>>,  // v2.1.136 : blocage inconditionnel
    pub environment: Option<Vec<serde_json::Value>>,
}
```

Ajouter à `Settings` :
```rust
#[serde(rename = "autoMode")]
pub auto_mode: Option<AutoModeConfig>,
```

### P2-B : WorktreeConfig struct
**Fichier** : `crates/ccboard-core/src/models/config.rs`

Ajouter :
```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorktreeConfig {
    pub base_ref: Option<String>,     // v2.1.133 : "fresh" | "head"
    pub bg_isolation: Option<String>, // v2.1.143 : "none" | default
}
```

Ajouter à `Settings` :
```rust
pub worktree: Option<WorktreeConfig>,
```

### P2-C : Champ Enterprise
**Fichier** : `crates/ccboard-core/src/models/config.rs`

Ajouter à `Settings` :
```rust
#[serde(rename = "allowAllClaudeAiMcps")]
pub allow_all_claude_ai_mcps: Option<bool>,  // v2.1.149 Enterprise
```

### P2-D : Affichage dans l'onglet Config TUI
Ajouter une section "Auto Mode" dans l'onglet Config affichant :
- allow rules count / soft_deny count / hard_deny count
- worktree.baseRef et bgIsolation

---

## Phase 3 : MCP : alwaysLoad (PRIORITÉ MOYENNE)

### P3-A : Champ alwaysLoad sur McpServer
**Fichier** : `crates/ccboard-core/src/parsers/mcp_config.rs:21`

Ajouter à `pub struct McpServer` :
```rust
#[serde(rename = "alwaysLoad", default)]
pub always_load: bool,  // v2.1.121 : charge le serveur même sans slash-command
```

### P3-B : Affichage dans l'onglet MCP TUI
Badge "always" ou icône distincte pour les serveurs avec `alwaysLoad: true`.

---

## Phase 4 : Skills/Agents Frontmatter (PRIORITÉ MOYENNE)

### P4-A : Compléter AgentEntry avec les nouveaux champs frontmatter
**Fichier** : `crates/ccboard-tui/src/tabs/agents.rs:19`

Ajouter à `pub struct AgentEntry` :
```rust
pub allowed_tools: Vec<String>,      // existant mais peut manquer
pub disallowed_tools: Vec<String>,   // v2.1.152 : nouveau champ frontmatter
pub context_mode: Option<String>,    // v2.1.150 : "fork" pour subagent isolé
pub effort: Option<String>,          // v2.1.146 : low/medium/high/etc.
pub skills: Vec<String>,             // v2.1.142 : skills héritées par les agents
pub claude_effort_var: bool,         // v2.1.120 : utilise ${CLAUDE_EFFORT}
```

### P4-B : Parser YAML frontmatter pour ces champs
**Fichier** : `crates/ccboard-tui/src/tabs/agents.rs` (fonction scan)

Lire ces champs depuis le bloc YAML entre `---` des fichiers .md.
Pattern à suivre : logique existante pour `description` et `allowed_tools`.

### P4-C : Affichage dans l'onglet Agents TUI
Vue détail d'un agent/skill :
- Liste "disallowed-tools" en rouge si présente
- Badge "fork" si context: fork
- Badge "effort: medium" etc.
- Liste des skills héritées

---

## Phase 5 : Env Vars (PRIORITÉ BASSE)

### P5-A : Nouvelles env vars à afficher dans l'onglet Config
**Fichier** : `crates/ccboard-tui/src/tabs/` (config tab)

Ajouter à la section "Variables d'environnement connues" :
- `CLAUDE_CODE_SESSION_ID` (v2.1.132) : ID de session dans l'env bash des hooks
- `CLAUDE_CODE_DISABLE_ALTERNATE_SCREEN` (v2.1.132) : désactive le screen alternatif
- `CLAUDE_CODE_PACKAGE_MANAGER_AUTO_UPDATE` (v2.1.129) : auto-update Homebrew/WinGet
- `ANTHROPIC_WORKSPACE_ID` (v2.1.141) : ID workspace enterprise

Afficher "définie ✓" ou "non définie" pour chacune selon `settings.env`.

---

## Ordre d'exécution recommandé

1. **P1-A** : HookType (30 min, zero dépendance)
2. **P2-A + P2-B + P2-C** : Settings structs (45 min, atomique)
3. **P3-A** : MCP alwaysLoad (15 min, trivial)
4. **P4-A + P4-B** : AgentEntry frontmatter (1h, parser YAML)
5. **P1-B + P1-C** : Hook fields + TUI (45 min, dépend de P1-A)
6. **P2-D** : Config TUI display (30 min, dépend de P2-A/B/C)
7. **P4-C** : Agents TUI display (30 min, dépend de P4-A/B)
8. **P3-B** : MCP TUI badge (15 min, dépend de P3-A)
9. **P5-A** : Env vars display (20 min, indépendant)

**Estimation totale** : ~5h de dev, 9 commits logiques.

---

## Notes techniques

- Tous les structs Settings/McpServer utilisent `serde_json::Value` dans le champ `extra`
  pour les champs inconnus. Avant d'ajouter un champ typé, vérifier qu'il n'est pas déjà
  capturé dans `extra` (ce serait une régression silencieuse).
- Le parser hooks.rs lit les fichiers `.sh` depuis `.claude/hooks/bash/`. Les nouveaux
  types de hooks sont définis dans `settings.json` (section `hooks`), pas uniquement
  par nom de fichier. Vérifier la logique de détection duale.
- `worktree.baseRef: "fresh"` est le nouveau défaut depuis v2.1.133 : breaking change
  (avant c'était `head`). À documenter dans l'UI si on affiche la valeur.
