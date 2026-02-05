//! ccboard-tui - TUI frontend for ccboard using Ratatui

pub mod app;
pub mod components;
pub mod editor;
pub mod empty_state;
pub mod keybindings;
pub mod tabs;
pub mod theme;
pub mod ui;

pub use app::App;

use anyhow::Result;
use ccboard_core::DataStore;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::prelude::*;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::oneshot;

/// Run the TUI application
pub async fn run(
    store: Arc<DataStore>,
    claude_home: PathBuf,
    project_path: Option<PathBuf>,
) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state (starts in loading mode)
    let mut app = App::new(store.clone());

    // Create UI (will initialize after data loads)
    let mut ui = ui::Ui::new();

    // Channel to signal when loading completes
    let (load_tx, mut load_rx) = oneshot::channel();

    // Spawn background loading task
    let store_clone = store.clone();
    tokio::spawn(async move {
        // Load initial data
        let report = store_clone.initial_load().await;

        // Compute invocation statistics
        store_clone.compute_invocations().await;

        // Compute billing blocks
        store_clone.compute_billing_blocks().await;

        // Compute analytics (default 30-day period)
        store_clone
            .compute_analytics(ccboard_core::analytics::Period::last_30d())
            .await;

        // Signal completion
        let _ = load_tx.send((report, store_clone.invocation_stats()));
    });

    // Main loop with loading check
    let result = run_loop_with_loading(
        &mut terminal,
        &mut app,
        &mut ui,
        &mut load_rx,
        &claude_home,
        project_path.as_deref(),
    )
    .await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

async fn run_loop_with_loading<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    ui: &mut ui::Ui,
    load_rx: &mut oneshot::Receiver<(
        ccboard_core::LoadReport,
        ccboard_core::models::InvocationStats,
    )>,
    claude_home: &std::path::Path,
    project_path: Option<&std::path::Path>,
) -> Result<()>
where
    <B as Backend>::Error: Send + Sync + 'static,
{
    loop {
        // Check if loading completed
        if let Ok(result) = load_rx.try_recv() {
            let (_report, invocation_stats) = result;

            // Initialize UI now that data is loaded
            ui.init(claude_home, project_path, &invocation_stats);

            // Mark loading as complete
            app.complete_loading();
        }

        // Check for data events
        app.poll_events();

        // Refresh live sessions if on Sessions tab (every 2s)
        app.refresh_live_sessions_if_needed();

        // Draw UI (will show loading screen or normal UI based on is_loading)
        terminal.draw(|f| ui.render(f, app))?;

        // Handle input with timeout for event polling
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    // First check for global keys
                    let handled = app.handle_key(key.code, key.modifiers);

                    // If not a global key and not loading, pass to active tab
                    if !handled && !app.is_loading {
                        ui.handle_tab_key(key.code, app);
                    }
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}
