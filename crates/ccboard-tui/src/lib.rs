//! ccboard-tui - TUI frontend for ccboard using Ratatui

pub mod app;
pub mod editor;
pub mod tabs;
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

    // Create app state
    let mut app = App::new(store);

    // Create UI and initialize
    let mut ui = ui::Ui::new();
    ui.init(&claude_home, project_path.as_deref());

    // Main loop
    let result = run_loop(&mut terminal, &mut app, &mut ui).await;

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

async fn run_loop<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    ui: &mut ui::Ui,
) -> Result<()>
where
    <B as Backend>::Error: Send + Sync + 'static,
{
    loop {
        // Check for data events
        app.poll_events();

        // Draw UI
        terminal.draw(|f| ui.render(f, app))?;

        // Handle input with timeout for event polling
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    // First check for global keys
                    let handled = app.handle_key(key.code);

                    // If not a global key, pass to active tab
                    if !handled {
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
