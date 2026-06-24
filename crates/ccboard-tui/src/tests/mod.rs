//! TUI tab render smoke tests
//!
//! Each test creates a TestBackend terminal (120x40), constructs the tab with `new()`,
//! calls `render()` with empty/None data, and asserts no panic + buffer is non-empty.

#[cfg(test)]
mod tab_smoke {
    use ccboard_core::models::config::ColorScheme;
    use ratatui::{backend::TestBackend, Terminal};

    fn make_terminal() -> Terminal<TestBackend> {
        Terminal::new(TestBackend::new(120, 40)).expect("terminal")
    }

    // ─── Dashboard ────────────────────────────────────────────────────────────

    #[test]
    fn dashboard_renders_empty() {
        use crate::tabs::dashboard::DashboardTab;
        let tab = DashboardTab::new();
        let mut terminal = make_terminal();
        terminal
            .draw(|frame| {
                tab.render(
                    frame,
                    frame.area(),
                    None,
                    None,
                    None,
                    ColorScheme::default(),
                );
            })
            .expect("draw");
        let buf = terminal.backend().buffer().clone();
        assert!(!buf.content().iter().all(|c| c.symbol() == " "));
    }

    // ─── Sessions ─────────────────────────────────────────────────────────────

    #[test]
    fn sessions_renders_empty() {
        use crate::tabs::sessions::SessionsTab;
        use ccboard_core::store::{DataStore, DataStoreConfig};
        use std::collections::HashMap;
        use std::path::PathBuf;

        let store = DataStore::new(
            PathBuf::from("/tmp/ccboard-test-nonexistent"),
            None,
            DataStoreConfig::default(),
        );
        let mut tab = SessionsTab::new();
        let mut terminal = make_terminal();
        terminal
            .draw(|frame| {
                tab.render(
                    frame,
                    frame.area(),
                    &HashMap::new(),
                    &[],
                    ColorScheme::default(),
                    &store,
                );
            })
            .expect("draw");
        let buf = terminal.backend().buffer().clone();
        assert!(!buf.content().iter().all(|c| c.symbol() == " "));
    }

    // ─── Config ───────────────────────────────────────────────────────────────

    #[test]
    fn config_renders_empty() {
        use crate::tabs::config::ConfigTab;
        use ccboard_core::models::{MergedConfig, Settings};
        use ccboard_core::parsers::Rules;

        let mut tab = ConfigTab::new();
        let config = MergedConfig {
            global: None,
            project: None,
            local: None,
            merged: Settings::default(),
        };
        let rules = Rules::default();
        let mut terminal = make_terminal();
        terminal
            .draw(|frame| {
                tab.render(
                    frame,
                    frame.area(),
                    &config,
                    None,
                    &rules,
                    ColorScheme::default(),
                );
            })
            .expect("draw");
        let buf = terminal.backend().buffer().clone();
        assert!(!buf.content().iter().all(|c| c.symbol() == " "));
    }

    // ─── Hooks ────────────────────────────────────────────────────────────────

    #[test]
    fn hooks_renders_empty() {
        use crate::tabs::hooks::HooksTab;
        use ccboard_core::models::Settings;

        let mut tab = HooksTab::new();
        let settings = Settings::default();
        let mut terminal = make_terminal();
        terminal
            .draw(|frame| {
                tab.render(frame, frame.area(), &settings, ColorScheme::default());
            })
            .expect("draw");
        let buf = terminal.backend().buffer().clone();
        assert!(!buf.content().iter().all(|c| c.symbol() == " "));
    }

    // ─── Agents ───────────────────────────────────────────────────────────────

    #[test]
    fn agents_renders_empty() {
        use crate::tabs::agents::AgentsTab;

        let mut tab = AgentsTab::new();
        let mut terminal = make_terminal();
        terminal
            .draw(|frame| {
                tab.render(frame, frame.area(), ColorScheme::default());
            })
            .expect("draw");
        let buf = terminal.backend().buffer().clone();
        assert!(!buf.content().iter().all(|c| c.symbol() == " "));
    }

    // ─── Costs ────────────────────────────────────────────────────────────────

    #[test]
    fn costs_renders_empty() {
        use crate::tabs::costs::CostsTab;

        let mut tab = CostsTab::new();
        let mut terminal = make_terminal();
        terminal
            .draw(|frame| {
                tab.render(
                    frame,
                    frame.area(),
                    None,
                    None,
                    ColorScheme::default(),
                    None,
                );
            })
            .expect("draw");
        let buf = terminal.backend().buffer().clone();
        assert!(!buf.content().iter().all(|c| c.symbol() == " "));
    }

    // ─── History ──────────────────────────────────────────────────────────────

    #[test]
    fn history_renders_empty() {
        use crate::tabs::history::HistoryTab;

        let mut tab = HistoryTab::new();
        let mut terminal = make_terminal();
        terminal
            .draw(|frame| {
                tab.render(frame, frame.area(), &[], None, ColorScheme::default());
            })
            .expect("draw");
        let buf = terminal.backend().buffer().clone();
        assert!(!buf.content().iter().all(|c| c.symbol() == " "));
    }

    // ─── MCP ──────────────────────────────────────────────────────────────────

    #[test]
    fn mcp_renders_empty() {
        use crate::tabs::mcp::McpTab;

        let mut tab = McpTab::new();
        let mut terminal = make_terminal();
        terminal
            .draw(|frame| {
                tab.render(frame, frame.area(), None, &[], ColorScheme::default());
            })
            .expect("draw");
        let buf = terminal.backend().buffer().clone();
        assert!(!buf.content().iter().all(|c| c.symbol() == " "));
    }

    // ─── Analytics ────────────────────────────────────────────────────────────

    #[test]
    fn analytics_renders_empty() {
        use crate::tabs::analytics::AnalyticsTab;

        let tab = AnalyticsTab::new();
        let mut terminal = make_terminal();
        terminal
            .draw(|frame| {
                tab.render(frame, frame.area(), None, None, ColorScheme::default());
            })
            .expect("draw");
        let buf = terminal.backend().buffer().clone();
        assert!(!buf.content().iter().all(|c| c.symbol() == " "));
    }

    // ─── Search ───────────────────────────────────────────────────────────────

    #[test]
    fn search_renders_empty() {
        use crate::tabs::search::{render_search_tab, SearchTab};

        let tab = SearchTab::new();
        let mut terminal = make_terminal();
        terminal
            .draw(|frame| {
                render_search_tab(&tab, frame, frame.area(), ColorScheme::default());
            })
            .expect("draw");
        let buf = terminal.backend().buffer().clone();
        assert!(!buf.content().iter().all(|c| c.symbol() == " "));
    }

    // ─── Activity ─────────────────────────────────────────────────────────────

    #[test]
    fn activity_renders_empty() {
        use crate::tabs::activity::ActivityTab;
        use ccboard_core::store::{DataStore, DataStoreConfig};
        use std::path::PathBuf;

        let store = DataStore::new(
            PathBuf::from("/tmp/ccboard-test-nonexistent"),
            None,
            DataStoreConfig::default(),
        );
        let mut tab = ActivityTab::new();
        let mut terminal = make_terminal();
        terminal
            .draw(|frame| {
                tab.render(frame, frame.area(), &[], &store, ColorScheme::default());
            })
            .expect("draw");
        let buf = terminal.backend().buffer().clone();
        assert!(!buf.content().iter().all(|c| c.symbol() == " "));
    }

    // ─── Plugins ──────────────────────────────────────────────────────────────

    #[test]
    fn plugins_renders_empty() {
        use crate::tabs::plugins::PluginsTab;
        use ccboard_core::store::{DataStore, DataStoreConfig};
        use std::path::PathBuf;
        use std::sync::Arc;

        let store = Arc::new(DataStore::new(
            PathBuf::from("/tmp/ccboard-test-nonexistent"),
            None,
            DataStoreConfig::default(),
        ));
        let mut tab = PluginsTab::new();
        let mut terminal = make_terminal();
        terminal
            .draw(|frame| {
                tab.render(frame, frame.area(), &store, ColorScheme::default());
            })
            .expect("draw");
        let buf = terminal.backend().buffer().clone();
        assert!(!buf.content().iter().all(|c| c.symbol() == " "));
    }
}
