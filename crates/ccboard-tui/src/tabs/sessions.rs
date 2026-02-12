//! Sessions tab - Project tree + session list + detail view

use crate::components::highlight_matches;
use ccboard_core::models::{SessionLine, SessionMetadata};
use ccboard_core::parsers::SessionContentParser;
use chrono::{DateTime, Duration, Utc};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, List, ListItem, ListState, Paragraph, Scrollbar, ScrollbarOrientation,
        ScrollbarState,
    },
    Frame,
};
use serde_json;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;

/// Date filter for session list
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateFilter {
    All,
    Last24h,
    Last7d,
    Last30d,
}

/// Sort mode for session list
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionSortMode {
    DateDesc,     // newest first (default)
    DateAsc,      // oldest first
    TokensDesc,   // most tokens first
    TokensAsc,    // least tokens first
    DurationDesc, // longest first
    MessagesDesc, // most messages first
}

impl SessionSortMode {
    /// Get the next sort mode in the cycle
    fn next(&self) -> Self {
        match self {
            SessionSortMode::DateDesc => SessionSortMode::DateAsc,
            SessionSortMode::DateAsc => SessionSortMode::TokensDesc,
            SessionSortMode::TokensDesc => SessionSortMode::TokensAsc,
            SessionSortMode::TokensAsc => SessionSortMode::DurationDesc,
            SessionSortMode::DurationDesc => SessionSortMode::MessagesDesc,
            SessionSortMode::MessagesDesc => SessionSortMode::DateDesc,
        }
    }

    /// Get display string for UI
    fn display(&self) -> &str {
        match self {
            SessionSortMode::DateDesc => "Date ↓",
            SessionSortMode::DateAsc => "Date ↑",
            SessionSortMode::TokensDesc => "Tokens ↓",
            SessionSortMode::TokensAsc => "Tokens ↑",
            SessionSortMode::DurationDesc => "Duration ↓",
            SessionSortMode::MessagesDesc => "Messages ↓",
        }
    }
}

impl DateFilter {
    /// Get the next filter in the cycle
    fn next(&self) -> Self {
        match self {
            DateFilter::All => DateFilter::Last24h,
            DateFilter::Last24h => DateFilter::Last7d,
            DateFilter::Last7d => DateFilter::Last30d,
            DateFilter::Last30d => DateFilter::All,
        }
    }

    /// Get the cutoff datetime for filtering
    fn cutoff(&self) -> Option<DateTime<Utc>> {
        let now = Utc::now();
        match self {
            DateFilter::All => None,
            DateFilter::Last24h => Some(now - Duration::hours(24)),
            DateFilter::Last7d => Some(now - Duration::days(7)),
            DateFilter::Last30d => Some(now - Duration::days(30)),
        }
    }

    /// Check if a session matches this filter
    fn matches(&self, session: &SessionMetadata) -> bool {
        match self.cutoff() {
            None => true,
            Some(cutoff) => session
                .first_timestamp
                .map(|ts| ts >= cutoff)
                .unwrap_or(false),
        }
    }

    /// Get display string for UI
    fn display(&self) -> &str {
        match self {
            DateFilter::All => "All",
            DateFilter::Last24h => "24h",
            DateFilter::Last7d => "7d",
            DateFilter::Last30d => "30d",
        }
    }
}

/// Sessions tab state
pub struct SessionsTab {
    /// Project tree state (selected project index)
    project_state: ListState,
    /// Session list state (selected session index)
    session_state: ListState,
    /// Live sessions list state (for scrolling)
    live_sessions_state: ListState,
    /// Current focus: Live Sessions (0), Projects (1), or Sessions (2)
    focus: usize,
    /// Cached project list (sorted)
    projects: Vec<String>,
    /// Search filter
    search_filter: String,
    /// Search mode active
    search_active: bool,
    /// Global search mode (all projects) vs local (current project)
    search_global: bool,
    /// Date filter for sessions
    date_filter: DateFilter,
    /// Sort mode for sessions
    sort_mode: SessionSortMode,
    /// Show detail popup for historical sessions
    show_detail: bool,
    /// Show detail popup for live sessions
    show_live_detail: bool,
    /// Show replay popup for selected session
    show_replay: bool,
    /// Replay messages cache (loaded on demand)
    replay_messages: Vec<SessionLine>,
    /// Replay scroll state
    replay_scroll: ListState,
    /// Expanded tool results (set of message indices)
    replay_expanded: HashSet<usize>,
    /// Error message to display
    error_message: Option<String>,
    /// Last refresh timestamp
    last_refresh: Instant,
    /// Refresh notification message
    refresh_message: Option<String>,
    /// Notification timestamp for auto-clear timing
    notification_time: Option<Instant>,
    /// Previous session count (to detect changes)
    prev_session_count: usize,
    /// Vim-style: waiting for second 'g' press
    pending_gg: bool,
}

impl Default for SessionsTab {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionsTab {
    pub fn new() -> Self {
        let mut project_state = ListState::default();
        project_state.select(Some(0));
        let mut session_state = ListState::default();
        session_state.select(Some(0));
        let mut live_sessions_state = ListState::default();
        live_sessions_state.select(Some(0));
        let mut replay_scroll = ListState::default();
        replay_scroll.select(Some(0));

        Self {
            project_state,
            session_state,
            live_sessions_state,
            focus: 1, // Start with Projects focused (1), not Live Sessions (0)
            projects: Vec::new(),
            search_filter: String::new(),
            search_active: false,
            search_global: false,
            date_filter: DateFilter::All,
            sort_mode: SessionSortMode::DateDesc,
            show_detail: false,
            show_live_detail: false,
            show_replay: false,
            replay_messages: Vec::new(),
            replay_scroll,
            replay_expanded: HashSet::new(),
            error_message: None,
            last_refresh: Instant::now(),
            refresh_message: None,
            notification_time: None,
            prev_session_count: 0,
            pending_gg: false,
        }
    }

    /// Handle key input for this tab
    pub fn handle_key(
        &mut self,
        key: crossterm::event::KeyCode,
        _sessions_by_project: &HashMap<String, Vec<Arc<SessionMetadata>>>,
    ) {
        use crossterm::event::KeyCode;

        // Search mode has priority
        if self.search_active {
            match key {
                KeyCode::Char(c) => {
                    self.search_filter.push(c);
                }
                KeyCode::Backspace => {
                    self.search_filter.pop();
                }
                KeyCode::Esc => {
                    self.search_active = false;
                    self.search_filter.clear();
                }
                KeyCode::Enter => {
                    self.search_active = false;
                }
                _ => {}
            }
            return;
        }

        match key {
            KeyCode::Tab => {
                // Cycle focus: Live Sessions (0) → Projects (1) → Sessions (2) → Live Sessions
                self.focus = (self.focus + 1) % 3;
            }
            KeyCode::Left | KeyCode::Char('h') => {
                // Move focus left: Live Sessions ← Projects ← Sessions
                if self.focus == 0 {
                    self.focus = 2;
                } else {
                    self.focus -= 1;
                }
            }
            KeyCode::Right | KeyCode::Char('l') => {
                // Move focus right: Live Sessions → Projects → Sessions
                self.focus = (self.focus + 1) % 3;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.show_replay {
                    self.move_replay_selection(-1);
                } else {
                    match self.focus {
                        0 => self.move_live_sessions_selection(-1),
                        1 => {
                            self.move_project_selection(-1);
                            // Reset session selection when project changes
                            self.session_state.select(Some(0));
                        }
                        2 => self.move_session_selection(-1),
                        _ => {}
                    }
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.show_replay {
                    self.move_replay_selection(1);
                } else {
                    match self.focus {
                        0 => self.move_live_sessions_selection(1),
                        1 => {
                            self.move_project_selection(1);
                            self.session_state.select(Some(0));
                        }
                        2 => self.move_session_selection(1),
                        _ => {}
                    }
                }
            }
            KeyCode::Enter => {
                if self.show_replay {
                    // Toggle expanded state for selected message
                    if let Some(idx) = self.replay_scroll.selected() {
                        if self.replay_expanded.contains(&idx) {
                            self.replay_expanded.remove(&idx);
                        } else {
                            self.replay_expanded.insert(idx);
                        }
                    }
                } else if self.focus == 0 {
                    // Toggle live session detail
                    self.show_live_detail = !self.show_live_detail;
                } else if self.focus == 2 {
                    // Toggle historical session detail
                    self.show_detail = !self.show_detail;
                }
            }
            KeyCode::Char('e') => {
                // Open selected session file in editor
                if self.focus == 2 {
                    if let Some(session) = self.get_selected_session(_sessions_by_project) {
                        if let Err(e) = crate::editor::open_in_editor(&session.file_path) {
                            self.error_message = Some(format!("Failed to open editor: {}", e));
                        }
                    }
                }
            }
            KeyCode::Char('o') => {
                // Reveal session file in file manager
                if self.focus == 2 {
                    if let Some(session) = self.get_selected_session(_sessions_by_project) {
                        if let Err(e) = crate::editor::reveal_in_file_manager(&session.file_path) {
                            self.error_message =
                                Some(format!("Failed to open file manager: {}", e));
                        }
                    }
                }
            }
            KeyCode::Char('r') => {
                // Resume session in Claude CLI
                if self.focus == 2 {
                    if let Some(session) = self.get_selected_session(_sessions_by_project) {
                        if let Err(e) = crate::editor::resume_claude_session(&session.id) {
                            self.error_message = Some(format!("Failed to resume session: {}", e));
                        } else {
                            self.refresh_message = Some("Session resumed".to_string());
                            self.notification_time = Some(Instant::now());
                        }
                    }
                }
            }
            KeyCode::Char('v') => {
                // Toggle replay viewer for selected session (works from Sessions pane)
                if self.focus == 2 && !self.show_replay {
                    if let Some(session) = self.get_selected_session(_sessions_by_project) {
                        // Load session content asynchronously (blocking for now, will improve later)
                        match tokio::runtime::Runtime::new() {
                            Ok(rt) => {
                                let path = session.file_path.clone();
                                match rt.block_on(SessionContentParser::parse_session_lines(&path))
                                {
                                    Ok(mut messages) => {
                                        // Filter to only user/assistant/tool messages
                                        messages = SessionContentParser::filter_messages(messages);
                                        self.replay_messages = messages;
                                        self.replay_scroll.select(Some(0));
                                        self.replay_expanded.clear();
                                        self.show_replay = true;
                                    }
                                    Err(e) => {
                                        self.error_message =
                                            Some(format!("Failed to load session: {}", e));
                                    }
                                }
                            }
                            Err(e) => {
                                self.error_message =
                                    Some(format!("Failed to create runtime: {}", e));
                            }
                        }
                    }
                }
            }
            KeyCode::Char('y') => {
                // Copy session ID to clipboard (works from Sessions pane)
                if self.focus == 2 {
                    if let Some(session) = self.get_selected_session(_sessions_by_project) {
                        match arboard::Clipboard::new() {
                            Ok(mut clipboard) => {
                                if let Err(e) = clipboard.set_text(&session.id) {
                                    self.error_message = Some(format!("Failed to copy: {}", e));
                                } else {
                                    self.refresh_message = Some(format!(
                                        "✓ Copied: {}",
                                        &session.id[..8.min(session.id.len())]
                                    ));
                                    self.notification_time = Some(Instant::now());
                                }
                            }
                            Err(e) => {
                                self.error_message = Some(format!("Clipboard unavailable: {}", e));
                            }
                        }
                    }
                }
            }
            KeyCode::Char('d') => {
                // Cycle date filter (works from any pane)
                self.date_filter = self.date_filter.next();
                self.refresh_message = Some(format!("Date filter: {}", self.date_filter.display()));
                self.notification_time = Some(Instant::now());
                // Reset selection to top when filter changes
                self.session_state.select(Some(0));
            }
            KeyCode::Char('s') => {
                // Cycle sort mode (works from Sessions pane)
                if self.focus == 2 {
                    self.sort_mode = self.sort_mode.next();
                    self.refresh_message = Some(format!("Sort: {}", self.sort_mode.display()));
                    self.notification_time = Some(Instant::now());
                    // Reset selection to top when sort changes
                    self.session_state.select(Some(0));
                }
            }
            KeyCode::Esc => {
                if self.error_message.is_some() {
                    self.error_message = None;
                } else if self.show_replay {
                    self.show_replay = false;
                    self.replay_messages.clear();
                    self.replay_expanded.clear();
                } else if self.show_live_detail {
                    self.show_live_detail = false;
                } else {
                    self.show_detail = false;
                }
            }
            KeyCode::Char('/') => {
                self.search_active = !self.search_active;
                if !self.search_active {
                    self.search_filter.clear();
                    self.search_global = false;
                } else {
                    // Set global search mode based on current focus
                    // focus==1 (Projects) or focus==0 (Live) → global search
                    // focus==2 (Sessions) → local search within selected project
                    self.search_global = self.focus != 2;
                }
            }
            KeyCode::PageUp => {
                // Jump up by 10 items
                if self.focus == 0 {
                    self.move_project_selection(-10);
                    self.session_state.select(Some(0));
                } else {
                    self.move_session_selection(-10);
                }
            }
            KeyCode::PageDown => {
                // Jump down by 10 items
                if self.focus == 0 {
                    self.move_project_selection(10);
                    self.session_state.select(Some(0));
                } else {
                    self.move_session_selection(10);
                }
            }
            KeyCode::Char('g') => {
                // Vim-style: 'gg' to go to top
                if self.pending_gg {
                    // Second 'g' - go to top
                    if self.focus == 0 {
                        self.project_state.select(Some(0));
                        self.session_state.select(Some(0));
                    } else {
                        self.session_state.select(Some(0));
                    }
                    self.pending_gg = false;
                } else {
                    // First 'g' - wait for second
                    self.pending_gg = true;
                }
            }
            KeyCode::Char('G') => {
                // Vim-style: 'G' to go to bottom
                if self.focus == 0 {
                    if !self.projects.is_empty() {
                        self.project_state.select(Some(self.projects.len() - 1));
                        self.session_state.select(Some(0));
                    }
                } else {
                    // Will be clamped in render_sessions
                    self.session_state.select(Some(usize::MAX));
                }
                self.pending_gg = false;
            }
            KeyCode::Home => {
                // Go to first item
                if self.focus == 0 {
                    self.project_state.select(Some(0));
                    self.session_state.select(Some(0));
                } else {
                    self.session_state.select(Some(0));
                }
                self.pending_gg = false;
            }
            KeyCode::End => {
                // Go to last item
                if self.focus == 0 {
                    if !self.projects.is_empty() {
                        self.project_state.select(Some(self.projects.len() - 1));
                        self.session_state.select(Some(0));
                    }
                } else {
                    self.session_state.select(Some(usize::MAX));
                }
                self.pending_gg = false;
            }
            _ => {
                // Reset pending_gg on any other key
                self.pending_gg = false;
            }
        }
    }

    fn move_project_selection(&mut self, delta: i32) {
        if self.projects.is_empty() {
            return;
        }
        let current = self.project_state.selected().unwrap_or(0) as i32;
        let new_idx = (current + delta).clamp(0, self.projects.len() as i32 - 1) as usize;
        self.project_state.select(Some(new_idx));
    }

    fn move_session_selection(&mut self, delta: i32) {
        let current = self.session_state.selected().unwrap_or(0) as i32;
        let new_idx = (current + delta).max(0) as usize;
        self.session_state.select(Some(new_idx));
    }

    fn move_live_sessions_selection(&mut self, delta: i32) {
        let current = self.live_sessions_state.selected().unwrap_or(0) as i32;
        let new_idx = (current + delta).max(0) as usize;
        self.live_sessions_state.select(Some(new_idx));
    }

    fn move_replay_selection(&mut self, delta: i32) {
        if self.replay_messages.is_empty() {
            return;
        }
        let current = self.replay_scroll.selected().unwrap_or(0) as i32;
        let new_idx = (current + delta).clamp(0, self.replay_messages.len() as i32 - 1) as usize;
        self.replay_scroll.select(Some(new_idx));
    }

    fn get_selected_session<'a>(
        &self,
        sessions_by_project: &'a HashMap<String, Vec<Arc<SessionMetadata>>>,
    ) -> Option<&'a Arc<SessionMetadata>> {
        let project_idx = self.project_state.selected()?;
        let project = self.projects.get(project_idx)?;
        let sessions = sessions_by_project.get(project)?;
        let session_idx = self.session_state.selected()?;
        sessions.get(session_idx)
    }

    fn get_selected_live_session<'a>(
        &self,
        live_sessions: &'a [ccboard_core::LiveSession],
    ) -> Option<&'a ccboard_core::LiveSession> {
        let live_idx = self.live_sessions_state.selected()?;
        live_sessions.get(live_idx)
    }

    fn find_session_by_id<'a>(
        &self,
        sessions_by_project: &'a HashMap<String, Vec<Arc<SessionMetadata>>>,
        session_id: &str,
    ) -> Option<&'a Arc<SessionMetadata>> {
        // Search through all projects for matching session ID
        for sessions in sessions_by_project.values() {
            for session in sessions {
                if session.id == session_id {
                    return Some(session);
                }
            }
        }
        None
    }

    /// Render the sessions tab
    pub fn render(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        sessions_by_project: &HashMap<String, Vec<Arc<SessionMetadata>>>,
        live_sessions: &[ccboard_core::LiveSession],
        _scheme: ccboard_core::models::config::ColorScheme,
    ) {
        // Update project cache
        self.projects = sessions_by_project.keys().cloned().collect();
        self.projects.sort();

        // Layout: [search bar (always visible), live sessions (if any), content]
        let live_height = if live_sessions.is_empty() {
            0
        } else {
            // Fixed height: 7 sessions visible (14 lines) + 2 for borders = 16 lines
            16
        };

        let mut constraints = vec![Constraint::Length(3)]; // search bar
        if live_height > 0 {
            constraints.push(Constraint::Length(live_height)); // live sessions
        }
        constraints.push(Constraint::Min(0)); // content

        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);

        // Render search bar (always visible)
        let search_title = if self.search_active {
            " Search (Esc to cancel) "
        } else {
            " Search (Press / to focus) "
        };
        let search_placeholder = if self.search_filter.is_empty() {
            "Search projects, messages, models..."
        } else {
            ""
        };
        let search_border_color = if self.search_active {
            Color::Cyan
        } else {
            Color::DarkGray
        };

        let search_block = Block::default()
            .title(search_title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(search_border_color));

        let search_text = if self.search_filter.is_empty() {
            format!("/ {}", search_placeholder)
        } else {
            format!("/ {}", self.search_filter)
        };

        let search_input = Paragraph::new(search_text).block(search_block).style(
            if self.search_filter.is_empty() {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default().fg(Color::White)
            },
        );

        frame.render_widget(search_input, main_chunks[0]);

        // Render live sessions if any
        let content_chunk_idx = if live_height > 0 {
            self.render_live_sessions(frame, main_chunks[1], live_sessions, self.focus == 0);
            2
        } else {
            1
        };

        let content_area = main_chunks[content_chunk_idx];

        // If replay is open, show it full width
        if self.show_replay {
            self.render_replay_popup(frame, content_area);
            return;
        }

        // If live session detail is open, show it full width
        if self.show_live_detail {
            // Try to get the selected live session and its metadata
            if let Some(live_session) = self.get_selected_live_session(live_sessions) {
                if let Some(session_id) = &live_session.session_id {
                    if let Some(session_meta) =
                        self.find_session_by_id(sessions_by_project, session_id)
                    {
                        self.render_live_detail(
                            frame,
                            content_area,
                            live_session,
                            Some(session_meta),
                        );
                    } else {
                        self.render_live_detail(frame, content_area, live_session, None);
                    }
                } else {
                    self.render_live_detail(frame, content_area, live_session, None);
                }
            }
            return;
        }

        // Main layout: projects tree | session list | detail (if open)
        let constraints = if self.show_detail {
            vec![
                Constraint::Percentage(25),
                Constraint::Percentage(35),
                Constraint::Percentage(40),
            ]
        } else {
            vec![Constraint::Percentage(30), Constraint::Percentage(70)]
        };

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(content_area);

        // Render projects tree
        self.render_projects(frame, chunks[0], sessions_by_project);

        // Get sessions for selected project or all projects (global search)
        let selected_project = self
            .project_state
            .selected()
            .and_then(|i| self.projects.get(i))
            .cloned();

        // Collect sessions based on search mode
        let all_sessions_vec: Vec<Arc<SessionMetadata>>;
        let all_sessions: &[Arc<SessionMetadata>] = if self.search_global && self.search_active {
            // Global search: collect all sessions from all projects
            all_sessions_vec = sessions_by_project
                .values()
                .flat_map(|v| v.iter().map(Arc::clone))
                .collect();
            &all_sessions_vec
        } else {
            // Local search: sessions from selected project only
            all_sessions_vec = selected_project
                .as_ref()
                .and_then(|p| sessions_by_project.get(p))
                .map(|v| v.to_vec())
                .unwrap_or_default();
            &all_sessions_vec
        };

        // Filter sessions based on search and date filter (Arc clone is cheap: 8 bytes)
        let mut sessions: Vec<Arc<SessionMetadata>> = all_sessions
            .iter()
            .filter(|s| {
                // Apply date filter
                if !self.date_filter.matches(s) {
                    return false;
                }

                // Apply search filter if active
                if self.search_filter.is_empty() {
                    return true;
                }

                let query_lower = self.search_filter.to_lowercase();
                s.id.to_lowercase().contains(&query_lower)
                    || s.project_path.to_lowercase().contains(&query_lower)
                    || s.first_user_message
                        .as_ref()
                        .is_some_and(|m| m.to_lowercase().contains(&query_lower))
                    || s.models_used
                        .iter()
                        .any(|m: &String| m.to_lowercase().contains(&query_lower))
            })
            .map(Arc::clone)
            .collect();

        // Apply sort mode
        match self.sort_mode {
            SessionSortMode::DateDesc => {
                sessions.sort_by(|a, b| b.last_timestamp.cmp(&a.last_timestamp));
            }
            SessionSortMode::DateAsc => {
                sessions.sort_by(|a, b| a.last_timestamp.cmp(&b.last_timestamp));
            }
            SessionSortMode::TokensDesc => {
                sessions.sort_by(|a, b| b.total_tokens.cmp(&a.total_tokens));
            }
            SessionSortMode::TokensAsc => {
                sessions.sort_by(|a, b| a.total_tokens.cmp(&b.total_tokens));
            }
            SessionSortMode::DurationDesc => {
                sessions.sort_by(|a, b| {
                    let a_dur = a
                        .last_timestamp
                        .zip(a.first_timestamp)
                        .map(|(end, start)| (end - start).num_seconds())
                        .unwrap_or(0);
                    let b_dur = b
                        .last_timestamp
                        .zip(b.first_timestamp)
                        .map(|(end, start)| (end - start).num_seconds())
                        .unwrap_or(0);
                    b_dur.cmp(&a_dur)
                });
            }
            SessionSortMode::MessagesDesc => {
                sessions.sort_by(|a, b| b.message_count.cmp(&a.message_count));
            }
        }

        // Clamp session selection
        if let Some(sel) = self.session_state.selected() {
            if sel >= sessions.len() && !sessions.is_empty() {
                self.session_state.select(Some(sessions.len() - 1));
            }
        }

        // Render session list
        self.render_sessions(frame, chunks[1], &sessions);

        // Render detail popup if open
        if self.show_detail && chunks.len() > 2 {
            let selected_session = self.session_state.selected().and_then(|i| sessions.get(i));
            self.render_detail(frame, chunks[2], selected_session);
        }

        // Render keyboard hints at bottom
        self.render_keyboard_hints(frame, area);

        // Render error popup if present
        if self.error_message.is_some() {
            self.render_error_popup(frame, area);
        }

        // Render refresh notification if present
        self.render_refresh_notification(frame, area);
    }

    fn render_live_sessions(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        live_sessions: &[ccboard_core::LiveSession],
        is_focused: bool,
    ) {
        use chrono::Local;

        // Clamp selection to valid range
        if let Some(sel) = self.live_sessions_state.selected() {
            if sel >= live_sessions.len() && !live_sessions.is_empty() {
                self.live_sessions_state
                    .select(Some(live_sessions.len() - 1));
            }
        }

        let border_color = if is_focused {
            Color::Cyan
        } else {
            Color::Green
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(Span::styled(
                format!(" ⚡ Live Sessions ({}) ", live_sessions.len()),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if live_sessions.is_empty() {
            return;
        }

        let items: Vec<ListItem> = live_sessions
            .iter()
            .map(|s| {
                let now = Local::now();
                let duration = now.signed_duration_since(s.start_time);
                let hours = duration.num_hours();
                let minutes = duration.num_minutes() % 60;

                let duration_str = if hours > 0 {
                    format!("{}h{}m", hours, minutes)
                } else {
                    format!("{}m", minutes)
                };

                let cwd_display = s
                    .working_directory
                    .as_ref()
                    .and_then(|p| std::path::Path::new(p).file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");

                // Line 1: Process info + Session ID
                let session_id_display = s
                    .session_id
                    .as_ref()
                    .map(|id| format!(" [{}]", &id[..8.min(id.len())]))
                    .unwrap_or_default();

                let line1 = format!(
                    "🟢 PID {} • {} • {} ago{}",
                    s.pid, cwd_display, duration_str, session_id_display
                );

                // Line 2: Session name (if available)
                let session_name_line = s.session_name.as_ref().map(|name| {
                    Line::from(format!("   ├─ 📝 {}", name)).style(Style::default().fg(Color::Cyan))
                });

                // Line 3: Performance metrics (shortened to fit)
                let tokens_str = s.tokens.map_or("?".to_string(), |t| Self::format_short(t));
                let metrics_line = format!(
                    "   ├─ CPU:{:>4.1}% RAM:{:>4}MB Tok:{:>5}",
                    s.cpu_percent, s.memory_mb, tokens_str
                );

                // Build lines vec conditionally
                let mut lines = vec![Line::from(line1).style(Style::default().fg(Color::Green))];
                if let Some(name_line) = session_name_line {
                    lines.push(name_line);
                }
                lines.push(Line::from(metrics_line).style(Style::default().fg(Color::DarkGray)));

                ListItem::new(lines)
            })
            .collect();

        let list = List::new(items).highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

        frame.render_stateful_widget(list, inner, &mut self.live_sessions_state);

        // Render scrollbar
        if live_sessions.len() > 10 {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"));

            let mut scrollbar_state = ScrollbarState::new(live_sessions.len())
                .position(self.live_sessions_state.selected().unwrap_or(0));

            frame.render_stateful_widget(scrollbar, inner, &mut scrollbar_state);
        }
    }

    /// Format large numbers with K/M/B suffixes
    fn format_short(n: u64) -> String {
        if n >= 1_000_000_000 {
            format!("{}B", n / 1_000_000_000)
        } else if n >= 1_000_000 {
            format!("{}M", n / 1_000_000)
        } else if n >= 1_000 {
            format!("{}K", n / 1_000)
        } else {
            n.to_string()
        }
    }

    fn render_projects(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        sessions_by_project: &HashMap<String, Vec<Arc<SessionMetadata>>>,
    ) {
        let is_focused = self.focus == 1; // Projects focus
        let border_color = if is_focused {
            Color::Cyan
        } else {
            Color::DarkGray
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(Span::styled(
                format!(" Projects ({}) ", self.projects.len()),
                Style::default().fg(Color::White).bold(),
            ));

        let items: Vec<ListItem> = self
            .projects
            .iter()
            .enumerate()
            .map(|(i, path)| {
                let is_selected = self.project_state.selected() == Some(i);
                let display = Self::format_project_path(path);
                let session_count = sessions_by_project.get(path).map(|v| v.len()).unwrap_or(0);

                let style = if is_selected && is_focused {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else if is_selected {
                    Style::default().fg(Color::White)
                } else {
                    Style::default().fg(Color::DarkGray)
                };

                let count_style = if is_selected {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::DarkGray)
                };

                ListItem::new(Line::from(vec![
                    Span::styled(format!(" {} ", if is_selected { "▶" } else { " " }), style),
                    Span::styled(display, style),
                    Span::styled(format!(" ({})", session_count), count_style),
                ]))
            })
            .collect();

        let list = List::new(items).block(block).highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

        frame.render_stateful_widget(list, area, &mut self.project_state);
    }

    fn render_sessions(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        sessions: &[Arc<SessionMetadata>],
    ) {
        let is_focused = self.focus == 2; // Sessions focus
        let border_color = if is_focused {
            Color::Cyan
        } else {
            Color::DarkGray
        };

        let time_ago = self.format_time_ago();
        const MAX_DISPLAY: usize = 500;
        let total_count = sessions.len();
        let display_count = total_count.min(MAX_DISPLAY);

        // Build title with optional date filter, sort mode, and search mode indicators
        let mut title_parts = vec!["Sessions".to_string()];

        // Add search mode indicator
        if self.search_global && self.search_active {
            title_parts.push("(global)".to_string());
        }

        // Add date filter if not "All"
        if self.date_filter != DateFilter::All {
            title_parts.push(format!("({})", self.date_filter.display()));
        }

        // Add sort mode if not default
        if self.sort_mode != SessionSortMode::DateDesc {
            title_parts.push(format!("[{}]", self.sort_mode.display()));
        }

        let title_prefix = title_parts.join(" ");

        let title_text = if self.search_filter.is_empty() && total_count > MAX_DISPLAY {
            format!(
                " {} (showing {} / {}) • {} ",
                title_prefix, display_count, total_count, time_ago
            )
        } else if self.search_filter.is_empty() {
            format!(" {} ({}) • {} ", title_prefix, total_count, time_ago)
        } else if total_count > MAX_DISPLAY {
            format!(
                " {} (showing {} / {} results) • {} ",
                title_prefix, display_count, total_count, time_ago
            )
        } else {
            format!(
                " {} ({} results) • {} ",
                title_prefix, total_count, time_ago
            )
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(Span::styled(
                title_text,
                Style::default().fg(Color::White).bold(),
            ));

        if sessions.is_empty() {
            let empty_msg = if self.search_filter.is_empty() {
                // No sessions in selected project
                vec![
                    Line::from(""),
                    Line::from(Span::styled(
                        "📂 No sessions found",
                        Style::default().fg(Color::Yellow),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        "No Claude Code sessions in this project",
                        Style::default().fg(Color::DarkGray),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        "💡 Start a session:",
                        Style::default().fg(Color::Cyan),
                    )),
                    Line::from(Span::styled(
                        "   cd <project-dir> && claude",
                        Style::default().fg(Color::Green),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        "📁 Sessions stored in: ~/.claude/projects/",
                        Style::default().fg(Color::DarkGray),
                    )),
                ]
            } else {
                // Search returned no results
                vec![
                    Line::from(""),
                    Line::from(Span::styled(
                        "🔍 No matching sessions",
                        Style::default().fg(Color::Yellow),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        format!("No sessions match: \"{}\"", self.search_filter),
                        Style::default().fg(Color::DarkGray),
                    )),
                    Line::from(""),
                    Line::from(Span::styled("💡 Try:", Style::default().fg(Color::Cyan))),
                    Line::from(Span::styled(
                        "   • Shorter query or different project",
                        Style::default().fg(Color::DarkGray),
                    )),
                    Line::from(Span::styled(
                        "   • Clear filter (Esc)",
                        Style::default().fg(Color::DarkGray),
                    )),
                ]
            };

            let empty = Paragraph::new(empty_msg)
                .block(block)
                .alignment(ratatui::layout::Alignment::Left);
            frame.render_widget(empty, area);
            return;
        }

        // Limit display to first MAX_DISPLAY items for performance
        let displayed_sessions = &sessions[..display_count];

        let items: Vec<ListItem> = displayed_sessions
            .iter()
            .enumerate()
            .map(|(i, session)| {
                let is_selected = self.session_state.selected() == Some(i);

                // Format timestamp
                let date_str = session
                    .last_timestamp
                    .map(|ts| ts.format("%m/%d %H:%M").to_string())
                    .unwrap_or_else(|| "unknown".to_string());

                // Format preview
                let preview = session
                    .first_user_message
                    .as_ref()
                    .map(|m| {
                        let truncated: String = m.chars().take(40).collect();
                        if m.len() > 40 {
                            format!("{}...", truncated)
                        } else {
                            truncated
                        }
                    })
                    .unwrap_or_else(|| "No preview".to_string());

                let style = if is_selected && is_focused {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else if is_selected {
                    Style::default().fg(Color::White)
                } else {
                    Style::default().fg(Color::Gray)
                };

                let tokens_str = Self::format_tokens(session.total_tokens);
                let msgs_str = format!("{}msg", session.message_count);

                // Build preview spans with optional highlighting
                let mut preview_spans = vec![Span::styled(
                    format!(" {} ", if is_selected { "▶" } else { " " }),
                    style,
                )];

                // Add project prefix if global search is active
                if self.search_global && self.search_active {
                    let project_short = Self::format_project_path(&session.project_path);
                    preview_spans.push(Span::styled(
                        format!("[{}] ", project_short),
                        Style::default().fg(Color::Blue),
                    ));
                }

                preview_spans.extend(vec![
                    Span::styled(format!("{} ", date_str), Style::default().fg(Color::Yellow)),
                    Span::styled(
                        format!("{:>6} ", tokens_str),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::styled(
                        format!("{:>5} ", msgs_str),
                        Style::default().fg(Color::Green),
                    ),
                ]);

                // Add branch if available
                if let Some(ref branch) = session.branch {
                    let branch_display = if branch.len() > 12 {
                        format!("{}… ", &branch[..11])
                    } else {
                        format!("{} ", branch)
                    };
                    preview_spans.push(Span::styled(
                        branch_display,
                        Style::default().fg(Color::Magenta),
                    ));
                }

                // Add highlighted preview if searching, otherwise plain preview
                if !self.search_filter.is_empty() {
                    let highlighted = highlight_matches(&preview, &self.search_filter);
                    preview_spans.extend(highlighted);
                } else {
                    preview_spans.push(Span::styled(preview, style));
                }

                ListItem::new(Line::from(preview_spans))
            })
            .collect();

        let list = List::new(items).block(block);

        frame.render_stateful_widget(list, area, &mut self.session_state);

        // Scrollbar
        if sessions.len() > (area.height as usize - 2) {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None);
            let mut scrollbar_state = ScrollbarState::new(sessions.len())
                .position(self.session_state.selected().unwrap_or(0));
            frame.render_stateful_widget(
                scrollbar,
                area.inner(ratatui::layout::Margin {
                    vertical: 1,
                    horizontal: 0,
                }),
                &mut scrollbar_state,
            );
        }
    }

    fn render_detail(&self, frame: &mut Frame, area: Rect, session: Option<&Arc<SessionMetadata>>) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(Span::styled(
                " Session Detail ",
                Style::default().fg(Color::White).bold(),
            ));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let Some(session) = session else {
            let empty =
                Paragraph::new("No session selected").style(Style::default().fg(Color::DarkGray));
            frame.render_widget(empty, inner);
            return;
        };

        let mut lines = vec![
            Line::from(vec![
                Span::styled("ID: ", Style::default().fg(Color::DarkGray)),
                Span::styled(&session.id, Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("File: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    session.file_path.display().to_string(),
                    Style::default().fg(Color::Yellow),
                ),
            ]),
            Line::from(vec![
                Span::styled("Project: ", Style::default().fg(Color::DarkGray)),
                Span::styled(&session.project_path, Style::default().fg(Color::Yellow)),
            ]),
        ];

        // Add branch if available
        if let Some(ref branch) = session.branch {
            lines.push(Line::from(vec![
                Span::styled("Branch: ", Style::default().fg(Color::DarkGray)),
                Span::styled(branch, Style::default().fg(Color::Magenta)),
            ]));
        }

        lines.extend(vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("Started: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    session
                        .first_timestamp
                        .map(|ts| ts.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_else(|| "unknown".to_string()),
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(vec![
                Span::styled("Ended: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    session
                        .last_timestamp
                        .map(|ts| ts.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_else(|| "unknown".to_string()),
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(vec![
                Span::styled("Duration: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    session.duration_display(),
                    Style::default().fg(Color::Green),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Messages: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    session.message_count.to_string(),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
            Line::from(vec![
                Span::styled("Tokens: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    Self::format_tokens(session.total_tokens),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
        ]);

        // Token breakdown (if available)
        if session.input_tokens > 0 || session.output_tokens > 0 {
            lines.push(Line::from(vec![
                Span::styled("  ├─ Input: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    Self::format_tokens(session.input_tokens),
                    Style::default().fg(Color::Green),
                ),
            ]));
            lines.push(Line::from(vec![
                Span::styled("  ├─ Output: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    Self::format_tokens(session.output_tokens),
                    Style::default().fg(Color::Yellow),
                ),
            ]));
            if session.cache_creation_tokens > 0 {
                lines.push(Line::from(vec![
                    Span::styled("  ├─ Cache Write: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        Self::format_tokens(session.cache_creation_tokens),
                        Style::default().fg(Color::Magenta),
                    ),
                ]));
            }
            if session.cache_read_tokens > 0 {
                lines.push(Line::from(vec![
                    Span::styled("  └─ Cache Read: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        Self::format_tokens(session.cache_read_tokens),
                        Style::default().fg(Color::Blue),
                    ),
                ]));
            }
        }

        lines.extend(vec![
            Line::from(vec![
                Span::styled("File Size: ", Style::default().fg(Color::DarkGray)),
                Span::styled(session.size_display(), Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("Subagents: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    if session.has_subagents { "Yes" } else { "No" },
                    if session.has_subagents {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    },
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Models: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    if session.models_used.is_empty() {
                        "unknown".to_string()
                    } else {
                        session.models_used.join(", ")
                    },
                    Style::default().fg(Color::Magenta),
                ),
            ]),
        ]);

        // Tool usage (top 5 if available)
        if !session.tool_usage.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Tool Usage (top 5):",
                Style::default().fg(Color::DarkGray),
            )));

            let mut tool_usage_vec: Vec<_> = session.tool_usage.iter().collect();
            tool_usage_vec.sort_by(|a, b| b.1.cmp(a.1)); // sort by count descending

            for (tool, count) in tool_usage_vec.iter().take(5) {
                lines.push(Line::from(vec![
                    Span::styled("  • ", Style::default().fg(Color::DarkGray)),
                    Span::styled(tool.as_str(), Style::default().fg(Color::Cyan)),
                    Span::styled(format!(" ({})", count), Style::default().fg(Color::Yellow)),
                ]));
            }
        }

        lines.extend(vec![
            Line::from(""),
            Line::from(Span::styled(
                "First message:",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(Span::styled(
                session
                    .first_user_message
                    .as_deref()
                    .unwrap_or("No preview available"),
                Style::default().fg(Color::White),
            )),
        ]);

        let detail = Paragraph::new(lines).wrap(ratatui::widgets::Wrap { trim: true });
        frame.render_widget(detail, inner);
    }

    fn format_project_path(path: &str) -> String {
        // Shorten path for display
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() <= 3 {
            path.to_string()
        } else {
            format!(".../{}", parts[parts.len() - 2..].join("/"))
        }
    }

    fn format_tokens(tokens: u64) -> String {
        if tokens >= 1_000_000 {
            format!("{:.1}M", tokens as f64 / 1_000_000.0)
        } else if tokens >= 1_000 {
            format!("{:.1}K", tokens as f64 / 1_000.0)
        } else {
            tokens.to_string()
        }
    }

    fn render_live_detail(
        &self,
        frame: &mut Frame,
        area: Rect,
        live_session: &ccboard_core::LiveSession,
        session_meta: Option<&Arc<SessionMetadata>>,
    ) {
        use chrono::Local;

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green))
            .title(Span::styled(
                " 🟢 Live Session Detail ",
                Style::default().fg(Color::Green).bold(),
            ));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let now = Local::now();
        let duration = now.signed_duration_since(live_session.start_time);
        let hours = duration.num_hours();
        let minutes = duration.num_minutes() % 60;
        let duration_str = if hours > 0 {
            format!("{}h {}m", hours, minutes)
        } else {
            format!("{} min", minutes)
        };

        let mut lines = vec![
            Line::from(Span::styled(
                "PROCESS INFORMATION",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("PID: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    live_session.pid.to_string(),
                    Style::default().fg(Color::Cyan).bold(),
                ),
            ]),
            Line::from(vec![
                Span::styled("Running: ", Style::default().fg(Color::DarkGray)),
                Span::styled(duration_str, Style::default().fg(Color::Green)),
            ]),
            Line::from(vec![
                Span::styled("CPU: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{:.1}%", live_session.cpu_percent),
                    Style::default().fg(if live_session.cpu_percent > 50.0 {
                        Color::Red
                    } else if live_session.cpu_percent > 20.0 {
                        Color::Yellow
                    } else {
                        Color::Green
                    }),
                ),
            ]),
            Line::from(vec![
                Span::styled("Memory: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{} MB", live_session.memory_mb),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
            Line::from(vec![
                Span::styled("Working Dir: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    live_session
                        .working_directory
                        .as_ref()
                        .map(|s| s.as_str())
                        .unwrap_or("unknown"),
                    Style::default().fg(Color::Yellow),
                ),
            ]),
            Line::from(""),
        ];

        // Add session metadata if available
        if let Some(session) = session_meta {
            let mut session_lines = vec![
                Line::from(Span::styled(
                    "SESSION INFORMATION",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Session ID: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        &session.id[..8.min(session.id.len())],
                        Style::default().fg(Color::White),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Project: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(&session.project_path, Style::default().fg(Color::Yellow)),
                ]),
            ];

            // Add branch if available
            if let Some(ref branch) = session.branch {
                session_lines.push(Line::from(vec![
                    Span::styled("Branch: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(branch, Style::default().fg(Color::Magenta)),
                ]));
            }

            session_lines.extend(vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled("Messages: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        session.message_count.to_string(),
                        Style::default().fg(Color::Cyan),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Tokens: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        Self::format_tokens(live_session.tokens.unwrap_or(session.total_tokens)),
                        Style::default().fg(Color::Cyan),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Models: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        if session.models_used.is_empty() {
                            "unknown".to_string()
                        } else {
                            session.models_used.join(", ")
                        },
                        Style::default().fg(Color::Magenta),
                    ),
                ]),
                Line::from(""),
                Line::from(Span::styled(
                    "First message:",
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(Span::styled(
                    session
                        .first_user_message
                        .as_ref()
                        .map(|s| s.as_str())
                        .unwrap_or("(no message)"),
                    Style::default().fg(Color::White),
                )),
            ]);

            lines.extend(session_lines);
        } else if let Some(session_name) = &live_session.session_name {
            lines.extend(vec![
                Line::from(Span::styled(
                    "SESSION INFORMATION",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Name: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(session_name, Style::default().fg(Color::White)),
                ]),
                Line::from(vec![
                    Span::styled("Tokens: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        live_session
                            .tokens
                            .map(|t| Self::format_short(t))
                            .unwrap_or_else(|| "?".to_string()),
                        Style::default().fg(Color::Cyan),
                    ),
                ]),
                Line::from(""),
                Line::from(Span::styled(
                    "(Full session details not available yet)",
                    Style::default().fg(Color::DarkGray).italic(),
                )),
            ]);
        } else {
            lines.extend(vec![Line::from(Span::styled(
                "Session metadata not available",
                Style::default().fg(Color::DarkGray).italic(),
            ))]);
        }

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Press Enter to close",
            Style::default().fg(Color::DarkGray),
        )));

        let paragraph = Paragraph::new(lines).wrap(ratatui::widgets::Wrap { trim: false });
        frame.render_widget(paragraph, inner);
    }

    fn render_replay_popup(&mut self, frame: &mut Frame, area: Rect) {
        let total_messages = self.replay_messages.len();
        let selected_idx = self.replay_scroll.selected().unwrap_or(0);

        let title = format!(
            " 🎬 Session Replay • Message {}/{} ",
            if total_messages > 0 {
                selected_idx + 1
            } else {
                0
            },
            total_messages
        );

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Magenta))
            .title(Span::styled(
                title,
                Style::default().fg(Color::Magenta).bold(),
            ));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if self.replay_messages.is_empty() {
            let empty = Paragraph::new("No messages in this session")
                .style(Style::default().fg(Color::DarkGray));
            frame.render_widget(empty, inner);
            return;
        }

        // Build message list items
        let items: Vec<ListItem> = self
            .replay_messages
            .iter()
            .enumerate()
            .flat_map(|(idx, msg)| {
                let is_selected = Some(idx) == self.replay_scroll.selected();
                let is_expanded = self.replay_expanded.contains(&idx);

                let mut lines = Vec::new();

                // Timestamp + type indicator
                let timestamp_str = msg
                    .timestamp
                    .map(|ts| ts.format("%H:%M:%S").to_string())
                    .unwrap_or_else(|| "??:??:??".to_string());

                let (type_icon, type_color) = match msg.line_type.as_str() {
                    "user" => ("👤", Color::Cyan),
                    "assistant" => ("🤖", Color::Green),
                    "tool_use" => ("🔧", Color::Yellow),
                    "tool_result" => ("📊", Color::Blue),
                    _ => ("•", Color::DarkGray),
                };

                // Header line: timestamp + icon + type
                let header_style = if is_selected {
                    Style::default().fg(type_color).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(type_color)
                };

                lines.push(Line::from(vec![
                    Span::styled(
                        if is_selected { "▶ " } else { "  " },
                        Style::default().fg(Color::White),
                    ),
                    Span::styled(format!("[{}] ", timestamp_str), header_style),
                    Span::styled(format!("{} ", type_icon), header_style),
                    Span::styled(msg.line_type.clone(), header_style),
                ]));

                // Message content
                if let Some(message) = &msg.message {
                    if let Some(content) = &message.content {
                        let content_text = Self::extract_message_content(content);
                        let preview = if content_text.len() > 150 {
                            format!("{}...", &content_text[..147])
                        } else {
                            content_text
                        };

                        lines.push(Line::from(vec![Span::styled(
                            format!("  {}", preview),
                            Style::default().fg(Color::White),
                        )]));
                    }

                    // Tool calls (if present and expanded)
                    if let Some(tool_calls) = &message.tool_calls {
                        if !tool_calls.is_empty() {
                            lines.push(Line::from(vec![Span::styled(
                                format!(
                                    "  {} {} tool call(s) {}",
                                    if is_expanded { "▼" } else { "▶" },
                                    tool_calls.len(),
                                    if is_expanded { "" } else { "[Enter to expand]" }
                                ),
                                Style::default().fg(Color::Yellow),
                            )]));

                            if is_expanded {
                                for (i, tool_call) in tool_calls.iter().enumerate() {
                                    let tool_name = tool_call
                                        .get("name")
                                        .and_then(|n| n.as_str())
                                        .unwrap_or("unknown");
                                    lines.push(Line::from(vec![Span::styled(
                                        format!("    {}. {}", i + 1, tool_name),
                                        Style::default().fg(Color::Yellow),
                                    )]));
                                }
                            }
                        }
                    }

                    // Tool results (if present and expanded)
                    if let Some(tool_results) = &message.tool_results {
                        if !tool_results.is_empty() {
                            lines.push(Line::from(vec![Span::styled(
                                format!(
                                    "  {} {} result(s) {}",
                                    if is_expanded { "▼" } else { "▶" },
                                    tool_results.len(),
                                    if is_expanded { "" } else { "[Enter to expand]" }
                                ),
                                Style::default().fg(Color::Blue),
                            )]));

                            if is_expanded {
                                for (i, result) in tool_results.iter().enumerate() {
                                    let output = result
                                        .get("output")
                                        .and_then(|o| o.as_str())
                                        .unwrap_or("(no output)");
                                    let preview = if output.len() > 100 {
                                        format!("{}...", &output[..97])
                                    } else {
                                        output.to_string()
                                    };
                                    lines.push(Line::from(vec![Span::styled(
                                        format!("    {}. {}", i + 1, preview),
                                        Style::default().fg(Color::DarkGray),
                                    )]));
                                }
                            }
                        }
                    }

                    // Token usage
                    if let Some(usage) = &message.usage {
                        let total = usage.total();
                        if total > 0 {
                            lines.push(Line::from(vec![Span::styled(
                                format!(
                                    "  📊 {} tokens (in:{} out:{} cache_r:{} cache_w:{})",
                                    total,
                                    usage.input_tokens,
                                    usage.output_tokens,
                                    usage.cache_read_tokens,
                                    usage.cache_write_tokens
                                ),
                                Style::default().fg(Color::DarkGray),
                            )]));
                        }
                    }
                } else if msg.line_type == "tool_use" || msg.line_type == "tool_result" {
                    // Tool events without message wrapper (fallback)
                    lines.push(Line::from(vec![Span::styled(
                        "  (Tool event details not available)",
                        Style::default().fg(Color::DarkGray).italic(),
                    )]));
                }

                // Separator line
                lines.push(Line::from(""));

                vec![ListItem::new(lines)]
            })
            .collect();

        let list = List::new(items).highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

        frame.render_stateful_widget(list, inner, &mut self.replay_scroll);

        // Scrollbar
        if self.replay_messages.len() > (inner.height as usize) {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"));

            let mut scrollbar_state = ScrollbarState::new(self.replay_messages.len())
                .position(self.replay_scroll.selected().unwrap_or(0));

            frame.render_stateful_widget(scrollbar, inner, &mut scrollbar_state);
        }

        // Keyboard hints overlay (bottom of popup)
        let hints_area = Rect {
            x: area.x + 1,
            y: area.y + area.height.saturating_sub(2),
            width: area.width.saturating_sub(2),
            height: 1,
        };

        let hints = Line::from(vec![
            Span::raw(" ["),
            Span::styled(
                "j/k ↑↓",
                Style::default().fg(Color::Black).bg(Color::Magenta).bold(),
            ),
            Span::raw("] "),
            Span::styled("scroll", Style::default().fg(Color::White)),
            Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
            Span::raw("["),
            Span::styled(
                "Enter",
                Style::default().fg(Color::Black).bg(Color::Magenta).bold(),
            ),
            Span::raw("] "),
            Span::styled("expand/collapse", Style::default().fg(Color::White)),
            Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
            Span::raw("["),
            Span::styled(
                "Esc",
                Style::default().fg(Color::Black).bg(Color::Magenta).bold(),
            ),
            Span::raw("] "),
            Span::styled("close", Style::default().fg(Color::White)),
        ]);

        let hints_widget = Paragraph::new(hints)
            .style(Style::default().bg(Color::DarkGray))
            .alignment(ratatui::layout::Alignment::Center);

        frame.render_widget(hints_widget, hints_area);
    }

    /// Extract text content from message content field
    ///
    /// Content can be either a String or an array of content blocks.
    /// This handles both formats gracefully.
    fn extract_message_content(content: &serde_json::Value) -> String {
        match content {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Array(blocks) => {
                let mut parts = Vec::new();
                for block in blocks {
                    if let Some(text) = block.get("text").and_then(|t| t.as_str()) {
                        parts.push(text.to_string());
                    } else if let Some(block_type) = block.get("type").and_then(|t| t.as_str()) {
                        // Handle non-text blocks (like tool_use)
                        parts.push(format!("[{}]", block_type));
                    }
                }
                parts.join(" ")
            }
            _ => "(unsupported content format)".to_string(),
        }
    }

    fn render_error_popup(&self, frame: &mut Frame, area: Rect) {
        // Center popup (40% width, 30% height)
        let popup_width = (area.width as f32 * 0.4).max(40.0) as u16;
        let popup_height = (area.height as f32 * 0.3).max(8.0) as u16;
        let popup_x = (area.width.saturating_sub(popup_width)) / 2;
        let popup_y = (area.height.saturating_sub(popup_height)) / 2;

        let popup_area = Rect {
            x: area.x + popup_x,
            y: area.y + popup_y,
            width: popup_width,
            height: popup_height,
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red))
            .title(Span::styled(
                " Error ",
                Style::default().fg(Color::Red).bold(),
            ));

        let inner = block.inner(popup_area);
        frame.render_widget(block, popup_area);

        let error_text = self.error_message.as_deref().unwrap_or("Unknown error");

        let lines = vec![
            Line::from(Span::styled(error_text, Style::default().fg(Color::White))),
            Line::from(""),
            Line::from(Span::styled(
                "Press Esc to close",
                Style::default().fg(Color::DarkGray),
            )),
        ];

        let paragraph = Paragraph::new(lines).wrap(ratatui::widgets::Wrap { trim: true });
        frame.render_widget(paragraph, inner);
    }

    /// Mark data as refreshed (called when new data is loaded)
    pub fn mark_refreshed(&mut self, current_session_count: usize) {
        let prev_count = self.prev_session_count;
        self.prev_session_count = current_session_count;
        self.last_refresh = Instant::now();

        // Show notification if session count changed
        if prev_count > 0 && current_session_count != prev_count {
            let diff = current_session_count as i32 - prev_count as i32;
            if diff > 0 {
                self.refresh_message = Some(format!("✓ {} new session(s) detected", diff));
            } else {
                self.refresh_message = Some(format!("✓ {} session(s) removed", -diff));
            }
            self.notification_time = Some(Instant::now());
        } else if prev_count == 0 || current_session_count == prev_count {
            self.refresh_message = Some("✓ Data refreshed".to_string());
            self.notification_time = Some(Instant::now());
        }
    }

    /// Get the currently selected session ID (for conversation viewer)
    /// Returns the session ID string needed by DataStore.load_session_content()
    pub fn selected_session_id(
        &self,
        sessions_by_project: &HashMap<String, Vec<Arc<SessionMetadata>>>,
    ) -> Option<String> {
        let selected_session = self.get_selected_session(sessions_by_project)?;
        Some(selected_session.id.to_string())
    }

    /// Format time since last refresh
    fn format_time_ago(&self) -> String {
        let elapsed = self.last_refresh.elapsed();
        let secs = elapsed.as_secs();

        if secs < 5 {
            "just now".to_string()
        } else if secs < 60 {
            format!("{}s ago", secs)
        } else if secs < 3600 {
            let mins = secs / 60;
            format!("{}m ago", mins)
        } else {
            let hours = secs / 3600;
            format!("{}h ago", hours)
        }
    }

    /// Render refresh notification overlay (bottom banner)
    fn render_keyboard_hints(&self, frame: &mut Frame, area: Rect) {
        // Calculate position at bottom of area
        let hint_area = Rect {
            x: area.x,
            y: area.y + area.height.saturating_sub(1),
            width: area.width,
            height: 1,
        };

        let hints = match self.focus {
            0 => {
                // Live Sessions
                vec![
                    Span::raw(" ["),
                    Span::styled(
                        "Tab",
                        Style::default().fg(Color::Black).bg(Color::Cyan).bold(),
                    ),
                    Span::raw("] "),
                    Span::styled("cycle focus", Style::default().fg(Color::White)),
                    Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
                    Span::raw("["),
                    Span::styled(
                        "↑↓ j/k",
                        Style::default().fg(Color::Black).bg(Color::Cyan).bold(),
                    ),
                    Span::raw("] "),
                    Span::styled("navigate", Style::default().fg(Color::White)),
                    Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
                    Span::raw("["),
                    Span::styled(
                        "Enter",
                        Style::default().fg(Color::Black).bg(Color::Cyan).bold(),
                    ),
                    Span::raw("] "),
                    Span::styled("detail", Style::default().fg(Color::White)),
                ]
            }
            1 => {
                // Projects
                vec![
                    Span::raw(" ["),
                    Span::styled(
                        "Tab",
                        Style::default().fg(Color::Black).bg(Color::Cyan).bold(),
                    ),
                    Span::raw("] "),
                    Span::styled("cycle focus", Style::default().fg(Color::White)),
                    Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
                    Span::raw("["),
                    Span::styled(
                        "↑↓ j/k",
                        Style::default().fg(Color::Black).bg(Color::Cyan).bold(),
                    ),
                    Span::raw("] "),
                    Span::styled("navigate", Style::default().fg(Color::White)),
                    Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
                    Span::raw("["),
                    Span::styled(
                        "h/l",
                        Style::default().fg(Color::Black).bg(Color::Cyan).bold(),
                    ),
                    Span::raw("] "),
                    Span::styled("switch pane", Style::default().fg(Color::White)),
                ]
            }
            2 => {
                // Sessions
                vec![
                    Span::raw(" ["),
                    Span::styled(
                        "Tab",
                        Style::default().fg(Color::Black).bg(Color::Cyan).bold(),
                    ),
                    Span::raw("] "),
                    Span::styled("cycle focus", Style::default().fg(Color::White)),
                    Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
                    Span::raw("["),
                    Span::styled(
                        "↑↓ j/k",
                        Style::default().fg(Color::Black).bg(Color::Cyan).bold(),
                    ),
                    Span::raw("] "),
                    Span::styled("navigate", Style::default().fg(Color::White)),
                    Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
                    Span::raw("["),
                    Span::styled(
                        "Enter",
                        Style::default().fg(Color::Black).bg(Color::Cyan).bold(),
                    ),
                    Span::raw("] "),
                    Span::styled("detail", Style::default().fg(Color::White)),
                    Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
                    Span::raw("["),
                    Span::styled(
                        "v",
                        Style::default().fg(Color::Black).bg(Color::Cyan).bold(),
                    ),
                    Span::raw("] "),
                    Span::styled("replay", Style::default().fg(Color::White)),
                    Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
                    Span::raw("["),
                    Span::styled(
                        "e",
                        Style::default().fg(Color::Black).bg(Color::Cyan).bold(),
                    ),
                    Span::raw("] "),
                    Span::styled("edit", Style::default().fg(Color::White)),
                    Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
                    Span::raw("["),
                    Span::styled(
                        "r",
                        Style::default().fg(Color::Black).bg(Color::Cyan).bold(),
                    ),
                    Span::raw("] "),
                    Span::styled("resume", Style::default().fg(Color::White)),
                    Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
                    Span::raw("["),
                    Span::styled(
                        "d",
                        Style::default().fg(Color::Black).bg(Color::Cyan).bold(),
                    ),
                    Span::raw("] "),
                    Span::styled("date filter", Style::default().fg(Color::White)),
                ]
            }
            _ => vec![],
        };

        let hint_line = Line::from(hints);
        let hint_paragraph = Paragraph::new(hint_line)
            .style(Style::default().bg(Color::DarkGray))
            .alignment(ratatui::layout::Alignment::Center);

        frame.render_widget(hint_paragraph, hint_area);
    }

    fn render_refresh_notification(&mut self, frame: &mut Frame, area: Rect) {
        // Check if notification should be cleared (2.5 seconds elapsed)
        if let Some(notif_time) = self.notification_time {
            if notif_time.elapsed().as_secs_f32() > 2.5 {
                self.refresh_message = None;
                self.notification_time = None;
                return;
            }
        }

        let Some(msg) = &self.refresh_message else {
            return;
        };

        // Bottom notification (60% width, 3 lines height)
        let msg_width = (area.width as f32 * 0.6) as u16;
        let msg_height = 3;
        let msg_x = (area.width.saturating_sub(msg_width)) / 2;
        let msg_y = area.height.saturating_sub(msg_height + 2);

        let msg_area = Rect {
            x: area.x + msg_x,
            y: area.y + msg_y,
            width: msg_width,
            height: msg_height,
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green));

        let inner = block.inner(msg_area);
        frame.render_widget(block, msg_area);

        let paragraph = Paragraph::new(msg.as_str())
            .style(Style::default().fg(Color::Green))
            .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(paragraph, inner);
    }
}
