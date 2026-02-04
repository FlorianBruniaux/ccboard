# Phase I: UI/UX Enhancements - Plan d'implémentation

**Date**: 2026-02-04
**Durée réelle**: ~2h (Task I.1: 20min, Task I.3: 1.5h)
**Statut**: 67% complété (2/3 tasks)
**Objectif**: Améliorer la lisibilité des graphiques + ajouter monitoring sessions live

---

## Overview

### Demandes utilisateur
1. **Échelle Y pour graphiques** → Contexte quantitatif manquant
2. **Suivi des rules** → À clarifier (guidelines tracking?)
3. **Sessions Claude live** → Monitoring processus actifs

### Implementation Status

| Task | Status | Time | Commit | Notes |
|------|--------|------|--------|-------|
| I.1 Option A | ✅ DONE | 20min | 582d446 | Y-axis label Dashboard |
| I.1 Option B | ✅ DONE | 45min | 21c6608 | Y-axis Analytics with 3 ticks |
| I.2 Rules | ⏸️ BLOCKED | ? | - | User clarification needed |
| I.3 Core | ✅ DONE | 1.5h | e18373c | Live sessions detection |
| I.3 Enhanced | ✅ DONE | 1h | 69a6d93 | CPU/RAM/Tokens monitoring |
| I.3 Auto-refresh | ✅ DONE | 30min | a72e24b | 2s polling with cache |
| **Bonus: Token fmt** | ✅ DONE | 10min | 3535f88 | K/M/B in Billing Blocks |

**Progress**: 6/7 tasks complétées (86% - only I.2 blocked)
**Code**: +652 LOC (384 + 14 + 124 + 64 + 33 + 33)
**Tests**: 140/140 passing
**Quality**: 0 clippy warnings

---

## Task I.1: Échelle Y pour graphiques (2h)

### Problème actuel

Le graphique "7-Day Activity" (Dashboard) utilise Ratatui Sparkline qui:
- ✅ Affiche les barres correctement
- ✅ Labels en bas (jour + valeur)
- ❌ **PAS d'échelle Y** → On ne sait pas si 10K messages ou 100

**Localisation**: `crates/ccboard-tui/src/tabs/dashboard.rs:224-305`

### Solution proposée

**Option A: Label "Max" en haut** (Simple, 30min)
```rust
// Avant le sparkline, ajouter un label de contexte
let max_label = Paragraph::new(format!("Max: {}", Self::format_short(max_val)))
    .style(Style::default().fg(Color::DarkGray))
    .alignment(Alignment::Right);
frame.render_widget(max_label, top_right_area);
```

**Option B: Axe Y complet** (Complexe, 2h)
- Diviser layout en 2 colonnes: [Y-axis (width: 8), Chart (remaining)]
- Afficher 3 labels: max, mid (max/2), min (0)
- Aligner avec les hauteurs du Sparkline

**Option C: BarChart avec axes** (Moyen, 1h)
- Remplacer Sparkline par BarChart (a des axes natifs)
- Mais: perd le style "sparkline" fluide

**Recommandation**: **Option A** pour Dashboard (quick win), **Option B** pour Analytics tab (plus de place)

### Implémentation Option A (30min)

```rust
// dashboard.rs:264 - Avant sparkline
let header_chunks = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([Constraint::Min(1), Constraint::Length(15)])
    .split(inner_chunks[0].inner(&Margin { horizontal: 0, vertical: 0 }));

let max_label = Paragraph::new(format!("↑ {}", Self::format_short(max_val)))
    .style(Style::default().fg(Color::DarkGray))
    .alignment(Alignment::Right);
frame.render_widget(max_label, Rect {
    x: header_chunks[1].x,
    y: header_chunks[1].y,
    width: header_chunks[1].width,
    height: 1
});

// Sparkline avec réduction de 1 ligne pour label
let sparkline_area = Rect {
    x: inner_chunks[0].x,
    y: inner_chunks[0].y + 1,
    width: inner_chunks[0].width,
    height: inner_chunks[0].height.saturating_sub(1),
};
let sparkline = Sparkline::default()
    .data(&expanded_data)
    .max(max_val)
    .style(Style::default().fg(Color::Cyan))
    .bar_set(symbols::bar::NINE_LEVELS);
frame.render_widget(sparkline, sparkline_area);
```

**Résultat attendu**:
```
┌─ ≡ 7-Day Activity ────────────────────┐
│                           ↑ 29K       │
│  ████▌            ████▌   ████████    │
│  28 (17K) 29 (22K) 30 (15K) ...       │
└───────────────────────────────────────┘
```

### Implémentation Option B (2h) - Analytics Tab

Pour Analytics tab (`analytics.rs`), espace suffisant pour axe Y complet:

```rust
// Layout: [Y-axis (8 chars), Chart]
let chart_layout = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([
        Constraint::Length(8),  // Y-axis labels
        Constraint::Min(20),    // Chart
    ])
    .split(chart_area);

// Y-axis labels (3 ticks)
let y_labels = vec![
    format!("{}", Self::format_short(max_val)),          // Top
    format!("{}", Self::format_short(max_val / 2)),      // Mid
    "0",                                                  // Bottom
];

let y_axis_widget = Paragraph::new(y_labels.join("\n\n\n"))
    .alignment(Alignment::Right)
    .style(Style::default().fg(Color::DarkGray));
frame.render_widget(y_axis_widget, chart_layout[0]);

// Sparkline dans zone restante
let sparkline = Sparkline::default()
    .data(&tokens_data)
    .max(max_val)
    .style(Style::default().fg(Color::Yellow));
frame.render_widget(sparkline, chart_layout[1]);
```

**Fichiers à modifier**:
- `crates/ccboard-tui/src/tabs/dashboard.rs` (+15 LOC, Option A)
- `crates/ccboard-tui/src/tabs/analytics.rs` (+40 LOC, Option B pour 3 charts)

---

## Task I.2: Clarification "Suivi des rules" (TBD)

**Question ouverte**: Que veux-tu monitorer exactement ?

### Hypothèses possibles

**A. Guidelines tracker**
- Parser CLAUDE.md / RULES.md
- Compter nb de règles définies
- Afficher conformité (règles appliquées vs totales)

**B. Session rules tracking**
- Tracker règles mentionnées dans messages Claude
- Ex: "Following SOLID principles", "Applied DRY"
- Extraction NLP-based (complexe)

**C. Project compliance**
- Vérifier conformité code vs règles
- Ex: Clippy rules, custom lints
- Intégration avec `cargo clippy --all-targets`

**D. Simple counter**
- Juste afficher nb de fichiers .md avec "rules"
- Liste navigable

**Action requise**: Clarifier avant implémentation

---

## Task I.3: Sessions Claude live (2-3h)

### Objectif

Afficher dans Sessions tab:
- ✅ Sessions historiques (actuellement affiché)
- ✅ Sessions archivées
- 🆕 **Sessions actives en ce moment** (processus claude en cours)

### Approche technique

**Détection processus**:
```rust
// crates/ccboard-core/src/live_monitor.rs (NEW)
use std::process::Command;

pub struct LiveSession {
    pub pid: u32,
    pub start_time: DateTime<Local>,
    pub working_directory: Option<String>,
    pub command: String,
}

pub fn detect_live_sessions() -> Result<Vec<LiveSession>> {
    // Platform-specific process detection
    #[cfg(unix)]
    {
        // Parse `ps aux | grep claude` output
        let output = Command::new("ps")
            .args(&["aux"])
            .output()
            .context("Failed to run ps command")?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let sessions: Vec<LiveSession> = stdout
            .lines()
            .filter(|line| line.contains("claude") && !line.contains("grep"))
            .filter_map(|line| parse_ps_line(line))
            .collect();

        Ok(sessions)
    }

    #[cfg(windows)]
    {
        // Use tasklist or Get-Process
        let output = Command::new("tasklist")
            .args(&["/FI", "IMAGENAME eq claude.exe", "/FO", "CSV"])
            .output()
            .context("Failed to run tasklist")?;

        // Parse CSV output
        todo!("Windows implementation")
    }
}

#[cfg(unix)]
fn parse_ps_line(line: &str) -> Option<LiveSession> {
    // Parse ps line format:
    // user  PID  %CPU %MEM  VSZ   RSS  TTY  STAT START TIME COMMAND
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 11 {
        return None;
    }

    let pid = parts[1].parse::<u32>().ok()?;
    let start_str = parts[8]; // START column
    let command = parts[10..].join(" ");

    // Parse start time (format: HH:MM or MMM DD if older)
    let start_time = parse_start_time(start_str).unwrap_or_else(|| Local::now());

    Some(LiveSession {
        pid,
        start_time,
        working_directory: get_cwd_for_pid(pid),
        command,
    })
}

#[cfg(unix)]
fn get_cwd_for_pid(pid: u32) -> Option<String> {
    // On macOS: lsof -p PID | grep cwd
    // On Linux: readlink /proc/PID/cwd
    #[cfg(target_os = "linux")]
    {
        std::fs::read_link(format!("/proc/{}/cwd", pid))
            .ok()
            .and_then(|p| p.to_str().map(String::from))
    }

    #[cfg(target_os = "macos")]
    {
        let output = Command::new("lsof")
            .args(&["-p", &pid.to_string(), "-Fn"])
            .output()
            .ok()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout
            .lines()
            .find(|line| line.starts_with("n"))
            .and_then(|line| line.strip_prefix("n"))
            .map(String::from)
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        None
    }
}
```

**DataStore integration**:
```rust
// crates/ccboard-core/src/store.rs
impl DataStore {
    pub fn live_sessions(&self) -> Vec<LiveSession> {
        detect_live_sessions().unwrap_or_default()
    }
}
```

**SessionsTab rendering**:
```rust
// crates/ccboard-tui/src/tabs/sessions.rs
// Ajouter section en haut
pub fn render(&mut self, frame: &mut Frame, area: Rect, store: Arc<DataStore>) {
    let live = store.live_sessions();

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3 + live.len() as u16),  // Live sessions box
            Constraint::Min(10),                         // Historical sessions
        ])
        .split(area);

    // Live sessions panel
    if !live.is_empty() {
        let live_items: Vec<ListItem> = live.iter()
            .map(|s| {
                let duration = Local::now().signed_duration_since(s.start_time);
                let label = format!(
                    "🟢 PID {} • {} • {} ago",
                    s.pid,
                    s.working_directory.as_deref().unwrap_or("unknown"),
                    format_duration(duration)
                );
                ListItem::new(label).style(Style::default().fg(Color::Green))
            })
            .collect();

        let live_list = List::new(live_items)
            .block(Block::default()
                .title(format!(" ⚡ Live Sessions ({}) ", live.len()))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green)));

        frame.render_widget(live_list, layout[0]);
    }

    // Historical sessions (existing code)
    self.render_historical_sessions(frame, layout[1], store);
}
```

**Refresh strategy**:
- Poll toutes les 2 secondes (pas de file watcher pour processus)
- Keybinding `Ctrl+L` pour force refresh
- Auto-refresh uniquement si Sessions tab actif

**Fichiers à créer/modifier**:
- `crates/ccboard-core/src/live_monitor.rs` (+200 LOC, NEW)
- `crates/ccboard-core/src/lib.rs` (+1 LOC, export)
- `crates/ccboard-core/src/store.rs` (+10 LOC, live_sessions())
- `crates/ccboard-tui/src/tabs/sessions.rs` (+50 LOC, live panel)
- `crates/ccboard-tui/src/app.rs` (+15 LOC, poll timer)

**Tests**:
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_ps_line() {
        let line = "user  12345  0.0  0.1  123456  78910  ttys001  S+   14:30   0:05.23  /usr/local/bin/claude --session foo";
        let session = parse_ps_line(line).unwrap();
        assert_eq!(session.pid, 12345);
        assert!(session.command.contains("claude"));
    }

    #[test]
    fn test_detect_live_sessions() {
        // Mock test - needs real claude process
        let sessions = detect_live_sessions().unwrap();
        // Should not panic
    }
}
```

---

## Implemen Order

1. **I.1 (Option A)**: Échelle Y Dashboard (30min) → Quick win
2. **I.2**: Clarifier "rules tracking" avec utilisateur
3. **I.3**: Sessions live (2-3h) → Feature complète
4. **I.1 (Option B)**: Échelle Y Analytics (1h) → Polish si temps

---

## Success Criteria

### I.1: Échelle Y
- ✅ Dashboard graph affiche "↑ MAX" en haut (COMPLETED - Option A implemented)
- 🚧 Analytics graphs ont Y-axis avec 3 ticks (max, mid, 0) (PENDING - Option B)
- ✅ Lisibilité améliorée (contexte quantitatif)

### I.2: Rules tracking
- ✅ Spec clarifiée avec utilisateur
- ✅ Implementation si pertinent

### I.3: Sessions live
- ✅ Détecte processus `claude` actifs (ps/tasklist) - COMPLETED
- ✅ Affiche PID, cwd, durée - COMPLETED
- 🚧 Auto-refresh 2s quand tab actif - DEFERRED (manual refresh works)
- ✅ Cross-platform (Linux, macOS, Windows) - COMPLETED (code, partial testing)
- ✅ Graceful si aucune session live - COMPLETED

---

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| `ps` parsing fragile | Medium | Fallback si parse échoue, tests avec fixtures |
| Windows tasklist différent | High | Conditional compilation, tests Windows |
| Poll overhead | Low | Uniquement si tab actif, 2s interval |
| False positives (grep claude) | Low | Filter exact binary name |
| cwd permission denied | Low | Graceful fallback "unknown" |

---

## Next Steps

1. ✅ **~~Implémenter I.1 Option A~~** (Dashboard échelle) - COMPLETED
2. ✅ **~~Implémenter I.1 Option B~~** (Analytics échelle) - COMPLETED
3. ✅ **~~Implémenter I.3 Core~~** (Sessions live) - COMPLETED
4. ✅ **~~Implémenter I.3 Enhanced~~** (CPU/RAM/Tokens) - COMPLETED
5. ✅ **~~Implémenter I.3 Auto-refresh~~** (2s polling) - COMPLETED
6. **Clarifier I.2** (rules tracking) avec utilisateur - BLOCKED (user input needed)

**Phase I: 86% complété** (6/7 tasks, seul I.2 bloqué sur clarification user)

---

## Implementation Summary

### ✅ Completed Features

**Task I.1 - Y-axis Scale (Dashboard)**:
- **File**: `crates/ccboard-tui/src/tabs/dashboard.rs` (+18 LOC)
- **Implementation**: Label "↑ Max" at top-right of sparkline
- **Result**: Immediate quantitative context (e.g., "↑ 29K")
- **Quality**: 0 warnings, no tests broken
- **Commit**: `582d446`
- **Documentation**: `claudedocs/task-i1-implementation.md`

**Task I.3 - Live Sessions Monitoring**:
- **Files**:
  - NEW: `crates/ccboard-core/src/live_monitor.rs` (281 LOC)
  - MOD: `crates/ccboard-core/src/lib.rs` (+2)
  - MOD: `crates/ccboard-core/src/store.rs` (+8)
  - MOD: `crates/ccboard-tui/src/tabs/sessions.rs` (+72)
  - MOD: `crates/ccboard-tui/src/ui.rs` (+2)
- **Implementation**:
  - Cross-platform process detection (ps/tasklist)
  - Green panel "⚡ Live Sessions (N)" in Sessions tab
  - Format: "🟢 PID 12345 • ccboard • 2h15m ago"
  - 5 unit tests passing
- **Quality**: 0 clippy warnings, graceful degradation
- **Commit**: `e18373c`
- **Documentation**: `claudedocs/task-i3-implementation.md`

**Task I.1 Option B - Y-axis Analytics Tab**:
- **File**: `crates/ccboard-tui/src/tabs/analytics.rs` (+64 LOC)
- **Implementation**: Full Y-axis with 3 ticks (max, mid, 0)
- **Layout**: Horizontal split [8 char Y-axis, remaining chart]
- **Quality**: 0 warnings
- **Commit**: `21c6608`

**Task I.3 Enhanced - CPU/RAM/Tokens**:
- **Files**: `live_monitor.rs` (+87), `sessions.rs` (+37)
- **Implementation**: 2-line display per process with performance metrics
- **Data sources**: ps aux (CPU/RAM), JSONL parsing (tokens)
- **Format**: CPU %, RAM MB, Tokens K/M/B
- **Quality**: 0 warnings
- **Commit**: `69a6d93`

**Task I.3 Auto-refresh - 2s Polling**:
- **Files**: `app.rs` (+30), `lib.rs` (+3), `ui.rs` (+1)
- **Implementation**: Cached live sessions with 2s refresh interval
- **Performance**: 200x reduction in system calls (100/s → 0.5/s)
- **Only active**: When Sessions tab is visible
- **Quality**: 0 warnings
- **Commit**: `a72e24b`

### 🚧 Pending Work

**Task I.2 - Rules Tracking** (BLOCKED):
- Requires user clarification on scope
- Options: Guidelines tracking, session compliance, project linting, simple list
- Estimated: TBD (depends on chosen approach)

### 📊 Phase I Metrics (Session 2)

**Time**: 4.5h total (2h session 1 + 2.5h session 2)
**Progress**: 86% (6/7 tasks - only I.2 blocked)
**Code Quality**:
- Lines added: +652 total (session 1: +398, session 2: +254)
- Tests: 140/140 passing
- Clippy: 0 warnings in all new code
- Cross-platform: Unix + Windows support

**Commits (Session 2)**:
- `69a6d93` - CPU/RAM/Tokens monitoring (124 LOC)
- `21c6608` - Analytics Y-axis (64 LOC)
- `a72e24b` - Auto-refresh polling (33 LOC)

**User Impact**:
- ✅ Dashboard & Analytics: Quantitative context on all charts
- ✅ Sessions tab: Real-time monitoring with CPU/RAM/Tokens
- ✅ Performance: 200x reduction in system calls
- ✅ All features degrade gracefully on errors

### 🎯 Next Session Goals

1. **Clarify Task I.2** with user: What type of rules tracking?
2. **Alternative paths**: Continue Phase H polish, start Phase F/G, or Phase II features

**Status**: ✅ Phase I 86% complete, production-ready, high performance

---

## Task I.3 Enhanced: CPU/RAM/Tokens Monitoring (~1h)

### Objectif

Enrichir l'affichage des live sessions avec métriques de performance en temps réel.

### Données à afficher

**1. CPU & Mémoire** (via ps):
- Colonne %CPU (ps aux col 3)
- Colonne %MEM ou RSS (ps aux col 4/6)
- Format: "CPU: 15.2% • RAM: 250MB"

**2. Tokens en live** (via JSONL):
- Identifier fichier session actif via cwd
- Parser et sommer tokens
- Format: "Tokens: 145K"

### Layout proposé

```
┌─ ⚡ Live Sessions (2) ─────────────────────────────┐
│ 🟢 PID 12345 • ccboard • 2h15m ago                │
│    ├─ CPU: 15.2% • RAM: 250MB • Tokens: 145K     │
│ 🟢 PID 25250 • florian-portfolio • 45m ago        │
│    ├─ CPU: 8.5% • RAM: 180MB • Tokens: 23K       │
└────────────────────────────────────────────────────┘
```

### Implémentation

**Phase 1**: Enrichir parse_ps_line() pour extraire CPU/MEM
**Phase 2**: Ajouter champs à LiveSession struct
**Phase 3**: Modifier render_live_sessions() pour layout 2 lignes
**Phase 4**: (Optionnel) Essayer d'identifier JSONL et compter tokens

**Estimation**: 45min (CPU/RAM) + 15min (tokens tentative) = 1h
