use crate::app::Application;
use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::{CrosstermBackend, Terminal};
use std::{
    io::{stdout, Result},
    sync::Arc,
    sync::Mutex,
};
pub mod base;
pub mod input;
pub mod layout;
pub mod main;
pub mod state;
pub mod top;
pub mod types;

pub fn run_ui(app: Application) -> Result<()> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let terminal = Arc::new(Mutex::new(Terminal::new(CrosstermBackend::new(stdout()))?));
    terminal.lock().unwrap().clear()?;

    let mut layout_controller = layout::LayoutController::new(Arc::new(app));

    loop {
        if !layout_controller.draw(Arc::clone(&terminal))? {
            break;
        }
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
