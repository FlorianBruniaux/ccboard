//! Activity tab — tool call audit and security alert viewer
//!
//! Two view modes toggled with Tab:
//!   Sessions mode: session list + per-session analysis detail
//!   Violations mode: consolidated cross-session feed with remediation hints
//!
//! Keybindings:
//!   j / ↓       Navigate down
//!   k / ↑       Navigate up
//!   a / Enter   Analyze selected session (Sessions mode)
//!   r           Scan all unanalyzed sessions (Violations mode) / refresh
//!   Tab         Toggle Sessions ↔ Violations

pub mod violations;

use crate::theme::Palette;
use ccboard_core::models::activity::{ActivitySummary, AlertSeverity};
use ccboard_core::models::config::ColorScheme;
use ccboard_core::models::SessionMetadata;
use ccboard_core::DataStore;
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use tokio::sync::Semaphore;
use violations::ViolationsView;

/// Which right-pane view is active
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ViewMode {
    /// Per-session analysis detail
    Sessions,
    /// Consolidated violations feed
    Violations,
}

/// Activity tab state
pub struct ActivityTab {
    /// Current right-pane view
    view_mode: ViewMode,
    /// Ratatui list selection for the session list
    list_state: ListState,
    /// Session IDs currently being analyzed (prevents duplicate spawns)
    analyzing_ids: HashSet<String>,
    /// Session IDs where analysis failed — shared with async tasks so they can self-register.
    /// Stored separately so the user can retry (press 'a' again clears the failure).
    failed_ids: Arc<Mutex<HashSet<String>>>,
    /// Violations sub-view
    violations: ViolationsView,
    /// Number of sessions currently being scanned via batch scan (r key)
    scanning_count: Arc<std::sync::atomic::AtomicUsize>,
    /// Semaphore limiting parallel batch analyses to 4
    scan_semaphore: Arc<Semaphore>,
}

impl Default for ActivityTab {
    fn default() -> Self {
        Self::new()
    }
}

impl ActivityTab {
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self {
            view_mode: ViewMode::Sessions,
            list_state,
            analyzing_ids: HashSet::new(),
            failed_ids: Arc::new(Mutex::new(HashSet::new())),
            violations: ViolationsView::new(),
            scanning_count: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            scan_semaphore: Arc::new(Semaphore::new(4)),
        }
    }

    /// Refresh violations cache from DataStore (call on DataEvent::AnalyticsUpdated)
    pub fn refresh_violations(&mut self, store: &DataStore) {
        self.violations.cache = store.all_violations();
        self.violations.loaded = true;
    }

    /// Handle keyboard input for this tab
    pub fn handle_key(
        &mut self,
        key: KeyCode,
        sessions: &[Arc<SessionMetadata>],
        store: &Arc<DataStore>,
    ) {
        match key {
            // Toggle between Sessions and Violations views
            KeyCode::Tab => {
                self.view_mode = match self.view_mode {
                    ViewMode::Sessions => ViewMode::Violations,
                    ViewMode::Violations => ViewMode::Sessions,
                };
            }

            KeyCode::Char('j') | KeyCode::Down => match self.view_mode {
                ViewMode::Sessions => {
                    let i = self.list_state.selected().unwrap_or(0);
                    if i + 1 < sessions.len() {
                        self.list_state.select(Some(i + 1));
                    }
                }
                ViewMode::Violations => self.violations.next(),
            },

            KeyCode::Char('k') | KeyCode::Up => match self.view_mode {
                ViewMode::Sessions => {
                    let i = self.list_state.selected().unwrap_or(0);
                    if i > 0 {
                        self.list_state.select(Some(i - 1));
                    }
                }
                ViewMode::Violations => self.violations.prev(),
            },

            // Analyze selected session (Sessions mode)
            KeyCode::Char('a') | KeyCode::Enter => {
                if self.view_mode == ViewMode::Sessions {
                    self.spawn_analysis_for_selected(sessions, store);
                }
            }

            // Scan all unanalyzed sessions (any mode)
            KeyCode::Char('r') => {
                self.spawn_batch_scan(sessions, store);
            }

            _ => {}
        }
    }

    /// Spawn analysis for the currently selected session
    fn spawn_analysis_for_selected(
        &mut self,
        sessions: &[Arc<SessionMetadata>],
        store: &Arc<DataStore>,
    ) {
        if let Some(idx) = self.list_state.selected() {
            if let Some(session) = sessions.get(idx) {
                let id = session.id.as_str().to_string();
                if !self.analyzing_ids.contains(&id) {
                    self.analyzing_ids.insert(id.clone());
                    if let Ok(mut f) = self.failed_ids.lock() {
                        f.remove(&id);
                    }
                    let store_clone = Arc::clone(store);
                    let failed_ids_clone = Arc::clone(&self.failed_ids);
                    tokio::spawn(async move {
                        if store_clone.analyze_session(&id).await.is_err() {
                            if let Ok(mut f) = failed_ids_clone.lock() {
                                f.insert(id);
                            }
                        }
                    });
                }
            }
        }
    }

    /// Spawn background analysis for all sessions not yet in the DashMap.
    /// Concurrency limited to 4 via semaphore.
    fn spawn_batch_scan(&mut self, sessions: &[Arc<SessionMetadata>], store: &Arc<DataStore>) {
        for session in sessions {
            let id = session.id.as_str().to_string();
            // Skip if already analyzed (in-memory) or currently in-flight
            if store.get_session_activity(&id).is_some() || self.analyzing_ids.contains(&id) {
                continue;
            }
            self.analyzing_ids.insert(id.clone());
            if let Ok(mut f) = self.failed_ids.lock() {
                f.remove(&id);
            }

            let store_clone = Arc::clone(store);
            let failed_ids_clone = Arc::clone(&self.failed_ids);
            let sem_clone = Arc::clone(&self.scan_semaphore);
            let counter = Arc::clone(&self.scanning_count);

            tokio::spawn(async move {
                let _permit = sem_clone.acquire().await;
                counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                if store_clone.analyze_session(&id).await.is_err() {
                    if let Ok(mut f) = failed_ids_clone.lock() {
                        f.insert(id);
                    }
                }
                counter.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
            });
        }
    }

    /// Render the full tab
    pub fn render(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        sessions: &[Arc<SessionMetadata>],
        store: &DataStore,
        scheme: ColorScheme,
    ) {
        // Lock failed_ids once per frame — passed to sub-renders to avoid duplicate locks (DRY)
        let failed_snapshot: HashSet<String> = self
            .failed_ids
            .lock()
            .map(|f| f.clone())
            .unwrap_or_default();

        // Clean up in-flight set: completed or failed analyses
        let completed: Vec<String> = self
            .analyzing_ids
            .iter()
            .filter(|id| store.get_session_activity(id).is_some() || failed_snapshot.contains(*id))
            .cloned()
            .collect();
        for id in completed {
            self.analyzing_ids.remove(&id);
        }

        // Refresh violations cache when analyses complete
        if self.view_mode == ViewMode::Violations {
            self.violations.cache = store.all_violations();
            self.violations.loaded = true;
        }

        let p = Palette::new(scheme);

        // Left pane always shows session list; right pane depends on view_mode
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
            .split(area);

        self.render_session_list(frame, chunks[0], sessions, store, &failed_snapshot, &p);

        match self.view_mode {
            ViewMode::Sessions => {
                let selected = self.list_state.selected().and_then(|i| sessions.get(i));
                self.render_detail(frame, chunks[1], selected, store, &p);
            }
            ViewMode::Violations => {
                let scanning = self
                    .scanning_count
                    .load(std::sync::atomic::Ordering::Relaxed);
                self.violations.render(frame, chunks[1], scanning, &p);
            }
        }
    }

    // ─── Left pane ────────────────────────────────────────────────────────────

    fn render_session_list(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        sessions: &[Arc<SessionMetadata>],
        store: &DataStore,
        // failed_snapshot passed in — locked once in render(), not again here (DRY fix)
        failed_snapshot: &HashSet<String>,
        p: &Palette,
    ) {
        let items: Vec<ListItem> = sessions
            .iter()
            .map(|s| {
                let id = s.id.as_str().to_string();
                let is_analyzing = self.analyzing_ids.contains(&id);
                let is_failed = failed_snapshot.contains(&id);
                let activity = store.get_session_activity(&id);

                let label = s
                    .project_path
                    .as_str()
                    .rsplit('/')
                    .next()
                    .unwrap_or("unknown");
                let short_id = &id[..id.len().min(8)];

                let badge = if is_analyzing {
                    Span::styled(" ⟳", Style::default().fg(p.focus))
                } else if is_failed {
                    Span::styled(" ✗", Style::default().fg(p.error))
                } else if let Some(ref summary) = activity {
                    let critical = summary
                        .alerts
                        .iter()
                        .filter(|a| a.severity == AlertSeverity::Critical)
                        .count();
                    let total = summary.alerts.len();
                    if critical > 0 {
                        Span::styled(format!(" ⚠{}", critical), Style::default().fg(p.error))
                    } else if total > 0 {
                        Span::styled(format!(" ●{}", total), Style::default().fg(p.warning))
                    } else {
                        Span::styled(" ✓", Style::default().fg(p.success))
                    }
                } else {
                    Span::styled(" ○", Style::default().fg(p.muted))
                };

                let line = Line::from(vec![
                    Span::styled(format!("{} ", short_id), Style::default().fg(p.muted)),
                    Span::styled(label.to_string(), Style::default().fg(p.fg)),
                    badge,
                ]);
                ListItem::new(line)
            })
            .collect();

        // Mode indicator in the title
        let mode_hint = match self.view_mode {
            ViewMode::Sessions => " [Tab→Violations]",
            ViewMode::Violations => " [Tab→Sessions]",
        };
        let title = format!(" 🔍 Activity ({}) {} ", sessions.len(), mode_hint);

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(p.muted))
                    .title(Span::styled(title, Style::default().fg(p.focus).bold())),
            )
            .highlight_style(
                Style::default()
                    .bg(p.focus)
                    .fg(p.bg)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");

        frame.render_stateful_widget(list, area, &mut self.list_state);
    }

    // ─── Right pane — Session detail ──────────────────────────────────────────

    fn render_detail(
        &self,
        frame: &mut Frame,
        area: Rect,
        session: Option<&Arc<SessionMetadata>>,
        store: &DataStore,
        p: &Palette,
    ) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(p.muted))
            .title(Span::styled(
                " Analysis ",
                Style::default().fg(p.focus).bold(),
            ));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let Some(session) = session else {
            frame.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::styled("Select a session and press ", Style::default().fg(p.muted)),
                    Span::styled("a", Style::default().fg(p.focus).bold()),
                    Span::styled(" to analyze", Style::default().fg(p.muted)),
                ])),
                inner,
            );
            return;
        };

        let id = session.id.as_str().to_string();

        if self.analyzing_ids.contains(&id) {
            frame.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::styled("⟳ Analyzing session… ", Style::default().fg(p.focus)),
                    Span::styled(&id[..id.len().min(16)], Style::default().fg(p.muted)),
                ])),
                inner,
            );
            return;
        }

        let Some(summary) = store.get_session_activity(&id) else {
            frame.render_widget(
                Paragraph::new(vec![
                    Line::from(vec![
                        Span::styled("Session: ", Style::default().fg(p.muted)),
                        Span::styled(&id[..id.len().min(16)], Style::default().fg(p.fg)),
                    ]),
                    Line::from(""),
                    Line::from(vec![
                        Span::styled("Press ", Style::default().fg(p.muted)),
                        Span::styled("a", Style::default().fg(p.focus).bold()),
                        Span::styled(" to analyze  │  ", Style::default().fg(p.muted)),
                        Span::styled("r", Style::default().fg(p.focus).bold()),
                        Span::styled(" to scan all sessions", Style::default().fg(p.muted)),
                    ]),
                ]),
                inner,
            );
            return;
        };

        self.render_summary(frame, inner, &summary, p);
    }

    fn render_summary(
        &self,
        frame: &mut Frame,
        area: Rect,
        summary: &ActivitySummary,
        p: &Palette,
    ) {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(2), Constraint::Min(0)])
            .split(area);

        let critical_count = summary
            .alerts
            .iter()
            .filter(|a| a.severity == AlertSeverity::Critical)
            .count();
        let alert_style = if critical_count > 0 {
            Style::default().fg(p.error).bold()
        } else if !summary.alerts.is_empty() {
            Style::default().fg(p.warning).bold()
        } else {
            Style::default().fg(p.success).bold()
        };

        let stats_line = Line::from(vec![
            Span::styled("Files: ", Style::default().fg(p.muted)),
            Span::styled(
                format!("{} ", summary.file_accesses.len()),
                Style::default().fg(p.fg).bold(),
            ),
            Span::styled("│ Bash: ", Style::default().fg(p.muted)),
            Span::styled(
                format!("{} ", summary.bash_commands.len()),
                Style::default().fg(p.fg).bold(),
            ),
            Span::styled("│ Net: ", Style::default().fg(p.muted)),
            Span::styled(
                format!("{} ", summary.network_calls.len()),
                Style::default().fg(p.fg).bold(),
            ),
            Span::styled("│ Alerts: ", Style::default().fg(p.muted)),
            Span::styled(format!("{}", summary.alerts.len()), alert_style),
        ]);

        let stats_block = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(p.muted));
        let stats_inner = stats_block.inner(rows[0]);
        frame.render_widget(stats_block, rows[0]);
        frame.render_widget(Paragraph::new(stats_line), stats_inner);

        let content = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(rows[1]);

        self.render_alerts(frame, content[0], summary, p);
        self.render_tool_calls(frame, content[1], summary, p);
    }

    fn render_alerts(&self, frame: &mut Frame, area: Rect, summary: &ActivitySummary, p: &Palette) {
        let items: Vec<ListItem> = if summary.alerts.is_empty() {
            vec![ListItem::new(Line::from(vec![Span::styled(
                "✓ No alerts detected",
                Style::default().fg(p.success),
            )]))]
        } else {
            summary
                .alerts
                .iter()
                .map(|alert| {
                    let (icon, sev_style) = match alert.severity {
                        AlertSeverity::Critical => ("🔴", Style::default().fg(p.error)),
                        AlertSeverity::Warning => ("🟡", Style::default().fg(p.warning)),
                        AlertSeverity::Info => ("🔵", Style::default().fg(p.focus)),
                    };
                    let cat = format!("{:?}", alert.category);
                    let detail = if alert.detail.chars().count() > 50 {
                        let t: String = alert.detail.chars().take(47).collect();
                        format!("{}…", t)
                    } else {
                        alert.detail.clone()
                    };
                    ListItem::new(Line::from(vec![
                        Span::raw(format!("{} ", icon)),
                        Span::styled(
                            format!("[{}] ", cat),
                            sev_style.add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(detail, Style::default().fg(p.fg)),
                    ]))
                })
                .collect()
        };

        let title = format!(" Alerts ({}) ", summary.alerts.len());
        frame.render_widget(
            List::new(items).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(p.muted))
                    .title(Span::styled(title, Style::default().fg(p.focus))),
            ),
            area,
        );
    }

    fn render_tool_calls(
        &self,
        frame: &mut Frame,
        area: Rect,
        summary: &ActivitySummary,
        p: &Palette,
    ) {
        let mut items: Vec<ListItem> = Vec::new();

        for fa in summary.file_accesses.iter().take(8) {
            let op = format!("{:?}", fa.operation).to_lowercase();
            let path = fa.path.rsplit('/').next().unwrap_or(&fa.path);
            items.push(ListItem::new(Line::from(vec![
                Span::styled(format!("{:<6} ", op), Style::default().fg(p.focus)),
                Span::styled(path.to_string(), Style::default().fg(p.fg)),
            ])));
        }

        for bc in summary.bash_commands.iter().take(5) {
            let cmd = if bc.command.chars().count() > 45 {
                let t: String = bc.command.chars().take(42).collect();
                format!("{}…", t)
            } else {
                bc.command.clone()
            };
            let cmd_style = if bc.is_destructive {
                Style::default().fg(p.error)
            } else {
                Style::default().fg(p.fg)
            };
            items.push(ListItem::new(Line::from(vec![
                Span::styled("bash   ", Style::default().fg(p.warning)),
                Span::styled(cmd, cmd_style),
            ])));
        }

        for nc in summary.network_calls.iter().take(5) {
            items.push(ListItem::new(Line::from(vec![
                Span::styled("net    ", Style::default().fg(p.muted)),
                Span::styled(nc.domain.clone(), Style::default().fg(p.fg)),
            ])));
        }

        if items.is_empty() {
            items.push(ListItem::new(Line::from(vec![Span::styled(
                "No tool calls recorded",
                Style::default().fg(p.muted),
            )])));
        }

        let total =
            summary.file_accesses.len() + summary.bash_commands.len() + summary.network_calls.len();
        frame.render_widget(
            List::new(items).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(p.muted))
                    .title(Span::styled(
                        format!(" Tool Calls ({}) ", total),
                        Style::default().fg(p.focus),
                    )),
            ),
            area,
        );
    }
}
