# Phase I: UI/UX Enhancements - Plan d'implémentation

**Date**: 2026-02-04
**Durée estimée**: 4-6h
**Objectif**: Améliorer la lisibilité des graphiques + ajouter monitoring sessions live

---

## Overview

### Demandes utilisateur
1. **Échelle Y pour graphiques** → Contexte quantitatif manquant
2. **Suivi des rules** → À clarifier (guidelines tracking?)
3. **Sessions Claude live** → Monitoring processus actifs

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
2. ✅ **~~Implémenter I.3~~** (Sessions live) - COMPLETED (~90%, auto-refresh deferred)
3. **Clarifier I.2** (rules tracking) avec utilisateur - BLOCKED (user input needed)
4. **Polish I.1 Option B** (Analytics échelle) - 1h optionnel
5. **Polish I.3** (auto-refresh 2s) - 1-2h optionnel

**Remaining estimated**: 1-3h (dépend de I.2 scope + polish souhaité)
