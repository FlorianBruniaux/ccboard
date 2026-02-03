# Session Resume: Phase C.2 - History Tab Export CSV/JSON

**Date de création**: 2026-02-03
**Tâche**: Phase C, Task C.2 - History Tab export CSV/JSON
**Status**: Planification complétée, prêt pour implémentation
**Durée estimée**: 2-3h

---

## 📍 Contexte Actuel

### État du Projet
- **Phase C.1**: ⏳ MCP Tab enhancements (pas commencé)
- **Phase C.2**: 📋 **PLANIFIÉ** - History Tab export CSV/JSON
- **Phase C.3**: ✅ **COMPLÉTÉ** - Billing blocks CSV export (2026-02-03)
- **Phase C.4**: ⏳ Sessions Tab live refresh (pas commencé)

### Derniers Commits
```bash
4e38ea5 docs: Update PLAN.md and CHANGELOG.md for Phase C.3 completion
5dba9c7 feat(costs): Add billing blocks CSV export functionality
233890d docs: Save session state and create resume prompt
```

### Infrastructure Disponible (Phase C.3)
✅ Module `ccboard-core/src/export.rs` existe avec :
- `export_billing_blocks_to_csv()` - Pattern de référence
- BufWriter pour performance
- Création auto des répertoires parents
- Gestion d'erreurs avec `anyhow::Context`
- Tests unitaires (5 tests)

---

## 🎯 Objectif Phase C.2

**Ajouter export CSV/JSON des sessions filtrées dans l'onglet History**

### Fonctionnalités à Implémenter
1. **Export CSV** : Sessions → fichier CSV (colonnes : Date, Time, Project, Session ID, Messages, Tokens, Models, Duration)
2. **Export JSON** : Sessions → fichier JSON (full SessionMetadata serialization)
3. **UI dans History tab** :
   - Key binding `x` → Dialog de sélection format
   - `1` → Export CSV
   - `2` → Export JSON
   - Messages succès/erreur
4. **Tests** : 5 unit tests (CSV empty/data, JSON empty/data, dirs)

---

## 📋 Plan d'Implémentation Détaillé

### Étape 1: Extend Export Module (1h)

**Fichier**: `crates/ccboard-core/src/export.rs`

```rust
/// Export sessions to CSV format
///
/// CSV columns: Date, Time, Project, Session ID, Messages, Tokens, Models, Duration (min)
pub fn export_sessions_to_csv(
    sessions: &[SessionMetadata],
    path: &Path,
) -> Result<()> {
    // 1. Create parent directories if needed
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    // 2. Create file with BufWriter
    let file = File::create(path)
        .with_context(|| format!("Failed to create CSV file: {}", path.display()))?;
    let mut writer = BufWriter::new(file);

    // 3. Write header
    writeln!(writer, "Date,Time,Project,Session ID,Messages,Tokens,Models,Duration (min)")
        .context("Failed to write CSV header")?;

    // 4. Write data rows
    for session in sessions {
        let date = session.first_timestamp
            .map(|ts| ts.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| "N/A".to_string());

        let time = session.first_timestamp
            .map(|ts| ts.format("%H:%M:%S").to_string())
            .unwrap_or_else(|| "N/A".to_string());

        let models = session.models_used.join(";");

        let duration = if let (Some(first), Some(last)) =
            (&session.first_timestamp, &session.last_timestamp) {
            let diff = last.signed_duration_since(*first);
            diff.num_minutes()
        } else {
            0
        };

        writeln!(
            writer,
            "\"{}\",\"{}\",\"{}\",\"{}\",{},{},\"{}\",{}",
            date,
            time,
            session.project_path,
            session.session_id,
            session.message_count,
            session.total_tokens,
            models,
            duration
        )
        .with_context(|| format!("Failed to write row for session {}", session.session_id))?;
    }

    writer.flush().context("Failed to flush CSV writer")?;
    Ok(())
}

/// Export sessions to JSON format
///
/// Pretty-printed JSON array of session metadata
pub fn export_sessions_to_json(
    sessions: &[SessionMetadata],
    path: &Path,
) -> Result<()> {
    // 1. Create parent directories
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    // 2. Serialize to JSON (pretty print)
    let json = serde_json::to_string_pretty(sessions)
        .context("Failed to serialize sessions to JSON")?;

    // 3. Write to file
    std::fs::write(path, json)
        .with_context(|| format!("Failed to write JSON file: {}", path.display()))?;

    Ok(())
}

// Tests (5 tests)
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use ccboard_core::models::SessionMetadata;
    use chrono::{TimeZone, Utc};

    #[test]
    fn test_export_sessions_csv_empty() { /* ... */ }

    #[test]
    fn test_export_sessions_csv_with_data() { /* ... */ }

    #[test]
    fn test_export_sessions_json_empty() { /* ... */ }

    #[test]
    fn test_export_sessions_json_with_data() { /* ... */ }

    #[test]
    fn test_export_sessions_creates_dirs() { /* ... */ }
}
```

**Mise à jour** : `crates/ccboard-core/src/lib.rs`
```rust
pub use export::{
    export_billing_blocks_to_csv,
    export_sessions_to_csv,    // NEW
    export_sessions_to_json,   // NEW
};
```

---

### Étape 2: History Tab UI (0.5h)

**Fichier**: `crates/ccboard-tui/src/tabs/history.rs`

**Ajouter au struct HistoryTab** :
```rust
pub struct HistoryTab {
    // ... existing fields ...

    /// Show export dialog
    show_export_dialog: bool,
    /// Export success/error message
    export_message: Option<String>,
}
```

**Ajouter key bindings** (dans `handle_key()`) :
```rust
KeyCode::Char('x') | KeyCode::Char('X') => {
    self.show_export_dialog = true;
}

KeyCode::Char('1') if self.show_export_dialog => {
    self.export_csv();
    self.show_export_dialog = false;
}

KeyCode::Char('2') if self.show_export_dialog => {
    self.export_json();
    self.show_export_dialog = false;
}

KeyCode::Esc if self.show_export_dialog => {
    self.show_export_dialog = false;
}
```

**Ajouter méthodes d'export** :
```rust
fn export_csv(&mut self) {
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let filename = format!("sessions_export_{}.csv", timestamp);
    let export_path = dirs::home_dir()
        .unwrap()
        .join(".claude/exports")
        .join(filename);

    match ccboard_core::export_sessions_to_csv(&self.filtered_sessions, &export_path) {
        Ok(_) => {
            self.export_message = Some(format!(
                "✓ Exported {} sessions to {}",
                self.filtered_sessions.len(),
                export_path.display()
            ));
        }
        Err(e) => {
            self.export_message = Some(format!("✗ Export failed: {}", e));
        }
    }
}

fn export_json(&mut self) {
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let filename = format!("sessions_export_{}.json", timestamp);
    let export_path = dirs::home_dir()
        .unwrap()
        .join(".claude/exports")
        .join(filename);

    match ccboard_core::export_sessions_to_json(&self.filtered_sessions, &export_path) {
        Ok(_) => {
            self.export_message = Some(format!(
                "✓ Exported {} sessions to {}",
                self.filtered_sessions.len(),
                export_path.display()
            ));
        }
        Err(e) => {
            self.export_message = Some(format!("✗ Export failed: {}", e));
        }
    }
}
```

**Ajouter render pour dialog** (dans `render()`) :
```rust
// Render export dialog
if self.show_export_dialog {
    let dialog_lines = vec![
        Line::from(Span::styled("Export Sessions", Style::default().add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from("Select format:"),
        Line::from("  1. CSV (Date, Time, Project, Messages, Tokens)"),
        Line::from("  2. JSON (Full session metadata)"),
        Line::from(""),
        Line::from(Span::styled("ESC to cancel", Style::default().fg(Color::DarkGray))),
    ];

    let dialog = Paragraph::new(dialog_lines)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title("Export"));

    // Center dialog (30x10)
    let dialog_area = centered_rect(30, 10, area);
    frame.render_widget(Clear, dialog_area);
    frame.render_widget(dialog, dialog_area);
}

// Render export message
if let Some(msg) = &self.export_message {
    let message_widget = Paragraph::new(msg.as_str())
        .style(if msg.starts_with('✓') {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::Red)
        });
    // Render at bottom of screen
}
```

---

### Étape 3: Tests & Documentation (0.5h)

**Tests unitaires** :
```bash
cargo test -p ccboard-core export::tests::test_export_sessions
```

**Help modal** : Ajouter dans `crates/ccboard-tui/src/components/help_modal.rs`
```rust
// History tab section
"x           : Export filtered sessions (CSV/JSON)"
```

**Tests manuels** :
```bash
# 1. CSV export
cargo run
# Tab to History (tab 7)
# Search: /project
# Press 'x', then '1'
# Verify: ~/.claude/exports/sessions_export_YYYYMMDD_HHMMSS.csv

# 2. JSON export
# Press 'x', then '2'
# Verify: ~/.claude/exports/sessions_export_YYYYMMDD_HHMMSS.json

# 3. Empty filter (all sessions)
# Clear search: 'c'
# Export: 'x', '1'
# Should export 3600+ sessions
```

---

## 🔧 Commandes de Validation

### Quick Start (Full Validation)
```bash
cargo test -p ccboard-core export && cargo clippy --all-targets && cargo run
```

### Tests Unitaires Détaillés
```bash
cargo test -p ccboard-core export::tests::test_export_sessions -- --nocapture
```

### Vérification Manuelle
```bash
# Export test
cargo run
# History tab → 'x' → '1' (CSV) ou '2' (JSON)

# Check CSV
cat ~/.claude/exports/sessions_export_*.csv | head -5

# Check JSON
cat ~/.claude/exports/sessions_export_*.json | jq '.[0]'
```

---

## 📊 Format de Sortie Attendu

### CSV
```csv
Date,Time,Project,Session ID,Messages,Tokens,Models,Duration (min)
"2026-02-03","14:30:45","/Users/x/project","abc123",25,15000,"sonnet;opus",45
"2026-02-03","10:15:20","/Users/x/other","def456",10,5000,"sonnet",15
```

### JSON
```json
[
  {
    "file_path": "/Users/x/.claude/projects/abc123.jsonl",
    "project_path": "/Users/x/project",
    "session_id": "abc123",
    "first_timestamp": "2026-02-03T14:30:45Z",
    "last_timestamp": "2026-02-03T15:15:30Z",
    "message_count": 25,
    "total_tokens": 15000,
    "models_used": ["sonnet", "opus"]
  }
]
```

---

## ✅ Critères de Succès

C.2 est complète quand :

1. ✅ `export_sessions_to_csv()` implémenté dans `export.rs`
2. ✅ `export_sessions_to_json()` implémenté dans `export.rs`
3. ✅ 5 tests unitaires passent (CSV empty/data, JSON empty/data, dirs)
4. ✅ Key binding `x` dans History tab
5. ✅ Dialog de sélection format (1=CSV, 2=JSON)
6. ✅ Messages succès/erreur affichés
7. ✅ Fichiers exportés dans `~/.claude/exports/`
8. ✅ Help text mis à jour
9. ✅ `cargo fmt && cargo clippy && cargo test --all` → 0 warnings

---

## 📁 Fichiers à Modifier

### Nouveaux/Modifiés
```
crates/ccboard-core/src/export.rs                (+120 LOC, 2 functions + 5 tests)
crates/ccboard-core/src/lib.rs                   (+2 LOC, re-exports)
crates/ccboard-tui/src/tabs/history.rs           (+80 LOC, export logic + UI)
crates/ccboard-tui/src/components/help_modal.rs  (+1 LOC, help text)
```

**Total estimé** : ~200 LOC

---

## 🚫 Hors Scope (Phase E)

- Export avec sélection de colonnes custom
- Export vers autres formats (Excel, Markdown)
- Export depuis Sessions tab (tâche C.4)
- Sélection du chemin d'export custom
- Batch export de tous les projets

---

## 📝 Notes Importantes

1. **Réutiliser les patterns de C.3** : BufWriter, error contexts, directory creation
2. **SessionMetadata est Serializable** : `#[derive(Serialize)]` déjà présent
3. **filtered_sessions déjà computed** : Pas de processing supplémentaire
4. **Export path pattern** : `~/.claude/exports/sessions_export_YYYYMMDD_HHMMSS.{csv,json}`
5. **Timestamp dans filename** : Évite l'écrasement, facilite l'identification

---

## 🔗 Références

- **Plan détaillé** : `TASK_C2_PLAN.md`
- **Phase C.3 (référence)** : Commit `5dba9c7` - Billing blocks CSV export
- **Module export existant** : `crates/ccboard-core/src/export.rs`
- **History tab** : `crates/ccboard-tui/src/tabs/history.rs`
- **SessionMetadata** : `crates/ccboard-core/src/models/session_metadata.rs`

---

## 🎯 Prompt de Reprise

**Copier-coller ce prompt dans une nouvelle session Claude Code** :

```
Je reprends le travail sur ccboard - Phase C.2 (History Tab Export CSV/JSON).

Contexte :
- Phase C.3 (Billing blocks CSV export) COMPLÉTÉE le 2026-02-03 (commit 5dba9c7)
- Infrastructure d'export déjà en place dans crates/ccboard-core/src/export.rs
- Plan détaillé disponible dans RESUME_C2.md et TASK_C2_PLAN.md

Objectif Phase C.2 :
Ajouter export CSV/JSON des sessions filtrées dans l'onglet History du TUI.

Actions à faire :
1. Ajouter export_sessions_to_csv() et export_sessions_to_json() dans export.rs
2. Ajouter key binding 'x' dans History tab avec dialog de sélection format
3. Ajouter 5 tests unitaires
4. Mettre à jour help modal

Estimation : 2-3h

Lis RESUME_C2.md pour le plan d'implémentation détaillé, puis implémente Phase C.2.
```

---

**Fichier de reprise créé** : 2026-02-03
**Status** : ✅ Prêt pour implémentation
**Durée estimée** : 2-3h
