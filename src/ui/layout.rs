use crate::app::Application;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    prelude::{CrosstermBackend, Terminal},
};
use std::{io::Result, sync::Arc, sync::Mutex, sync::RwLock};

use super::{input::InputBar, main::MainArea, top::TopArea};
use crate::ui::state::LayoutState;

pub struct LayoutController {
    app: Arc<Application>,
    state: Arc<RwLock<LayoutState>>,
    input_bar: InputBar,
    main_area: MainArea,
    top_area: TopArea,
}

impl LayoutController {
    pub fn new(app: Arc<Application>) -> Self {
        let state = Arc::new(RwLock::new(LayoutState::new(Arc::clone(&app))));
        LayoutController {
            input_bar: InputBar::new(Arc::clone(&app), Arc::clone(&state)),
            main_area: MainArea::new(Arc::clone(&app), Arc::clone(&state)),
            top_area: TopArea::new(Arc::clone(&app), Arc::clone(&state)),
            app,
            state,
        }
    }

    pub fn draw(
        &mut self,
        terminal: Arc<Mutex<Terminal<CrosstermBackend<std::io::Stdout>>>>,
    ) -> Result<bool> {
        let mut event_result = None;
        if event::poll(std::time::Duration::from_millis(16))? {
            event_result = Some(event::read()?);
            if let Some(Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: event::KeyModifiers::CONTROL,
                ..
            })) = event_result
            {
                // Handle Control+C here
                return Ok(false);
            }
        }

        let input_result = event_result.clone();
        let mut main_input_result = event_result.clone();
        if self.input_bar.active {
            main_input_result = None;
        }

        terminal.lock().unwrap().draw(|frame| {
            let areas = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(4),
                    Constraint::Length(3),
                    Constraint::Min(1),
                ])
                .split(frame.size());

            self.top_area.render(frame, areas[0], None);
            self.input_bar.render(frame, areas[1], input_result);
            self.main_area.render(frame, areas[2], main_input_result)
        })?;

        Ok(true)
    }
}
